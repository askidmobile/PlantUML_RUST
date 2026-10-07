//! Сверка нашего измерения текста с эталонным `textLength` из PlantUML.
//!
//! # Зачем
//!
//! Точность измерения текста задаёт всё остальное: ширины коробок,
//! расстояния между участниками, ширину рамок фрагментов. Ошибка в 2% на
//! подписи превращается в десятки пикселей на большой диаграмме.
//!
//! Корпус готовит `scripts/extract-text-corpus.py`: он вытаскивает из
//! эталонных SVG пары «подпись — textLength» вместе с кеглем и начертанием.
//! Этот пример считает те же ширины нашим [`TextMeasurer`] и печатает
//! расхождения — и по отдельным подписям, и суммарно по кеглям.
//!
//! # Запуск
//!
//! ```text
//! python3 scripts/extract-text-corpus.py
//! cargo run -p plantuml-core --example text_calibration
//! ```
//!
//! # Что искать в выводе
//!
//! Важна не общая цифра, а расхождение ПО КЕГЛЯМ И НАЧЕРТАНИЯМ.
//! Систематический сдвиг в одной категории означает, что неверна не
//! таблица глифов, а коэффициент для этой категории.
//!
//! Так была найдена ошибка в коэффициенте полужирного: полный корпус даёт
//! оптимум 1.0987 при стоявшем в коде 1.124, то есть полужирный текст
//! систематически завышался (+3.97% на кегле 11).
//!
//! # Осторожно
//!
//! Исправлять коэффициент ТОЛЬКО по этому отчёту нельзя. Замер показал,
//! что подстановка 1.0987 делает golden-набор хуже: сводная ширина растёт
//! со 112 до 122 px, четыре кейса ухудшаются, ни один не улучшается.
//! Значит расхождения в тех местах частично КОМПЕНСИРУЮТ завышение
//! жирного. Порядок такой: сначала убрать компенсирующие ошибки, потом
//! править коэффициент.

use std::collections::BTreeMap;

use plantuml_layout::text::TextMeasurer;

/// Путь к корпусу, который готовит `scripts/extract-text-corpus.py`.
const CORPUS_PATH: &str = "/tmp/textcorpus.tsv";

/// Сколько худших подписей показать.
const WORST_LIMIT: usize = 15;

struct Row {
    text: String,
    size: f64,
    bold: bool,
    reference: f64,
}

fn load_corpus() -> Result<Vec<Row>, String> {
    let data = std::fs::read_to_string(CORPUS_PATH).map_err(|error| {
        format!(
            "не читается {CORPUS_PATH}: {error}\n\
             Сначала выполните: python3 scripts/extract-text-corpus.py"
        )
    })?;

    let mut rows = Vec::new();
    for line in data.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() != 4 {
            continue;
        }
        rows.push(Row {
            text: parts[0].to_string(),
            size: parts[1].parse().unwrap_or(13.0),
            bold: parts[2] == "1",
            reference: parts[3].parse().unwrap_or(0.0),
        });
    }
    Ok(rows)
}

fn main() {
    let rows = match load_corpus() {
        Ok(rows) => rows,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(1);
        }
    };

    let measurer = TextMeasurer::default();
    let mut worst: Vec<(f64, &Row, f64)> = Vec::new();
    let mut by_kind: BTreeMap<(String, bool), (f64, f64, usize)> = BTreeMap::new();
    let (mut sum_reference, mut sum_ours) = (0.0_f64, 0.0_f64);

    for row in &rows {
        let ours = if row.bold {
            measurer.width_bold(&row.text, row.size)
        } else {
            measurer.width(&row.text, row.size)
        };

        sum_reference += row.reference;
        sum_ours += ours;

        let entry = by_kind
            .entry((format!("{:.0}", row.size), row.bold))
            .or_insert((0.0, 0.0, 0));
        entry.0 += row.reference;
        entry.1 += ours;
        entry.2 += 1;

        let relative = if row.reference > 0.0 {
            (ours - row.reference) / row.reference
        } else {
            0.0
        };
        worst.push((relative, row, ours));
    }

    worst.sort_by(|a, b| {
        b.0.abs()
            .partial_cmp(&a.0.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    println!("=== ХУДШИЕ {WORST_LIMIT} ПО ОТНОСИТЕЛЬНОЙ ОШИБКЕ ===");
    for (relative, row, ours) in worst.iter().take(WORST_LIMIT) {
        let short: String = row.text.chars().take(34).collect();
        let weight = if row.bold { "Ж" } else { " " };
        println!(
            "  {:+7.1}%  кегль {:>4} {}  эталон {:7.2}  наш {:7.2}  «{short}»",
            relative * 100.0,
            row.size,
            weight,
            row.reference,
            ours
        );
    }

    println!("\n=== ПО КЕГЛЮ И НАЧЕРТАНИЮ ===");
    for ((size, bold), (reference, ours, count)) in &by_kind {
        let kind = if *bold {
            "полужирный"
        } else {
            "обычный   "
        };
        println!(
            "  кегль {size:>4} {kind}  пар {count:>3}  эталон {reference:9.2}  наш {ours:9.2}  {:+.2}%",
            (ours - reference) / reference * 100.0
        );
    }

    println!(
        "\nИТОГО: эталон {sum_reference:.2}, наш {sum_ours:.2}, {:+.2}%",
        (sum_ours - sum_reference) / sum_reference * 100.0
    );
}
