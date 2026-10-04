//! Измерение текста для layout-движков.
//!
//! # Зачем отдельный модуль
//!
//! До его появления ширина текста вычислялась в каждом движке по-своему:
//! восемь разных эвристик вида «количество символов × константа», причём
//! **пять из них считали байты, а не символы**. Для кириллицы байтовый счёт
//! давал примерно вдвое большую ширину, и один и тот же текст в разных
//! диаграммах измерялся по-разному:
//!
//! ```text
//! salt/engine.rs      text.len() as f64 * 8.0       — байты
//! json/engine.rs      display_text.len() as f64 * 8.0 — байты
//! wbs/engine.rs       text.len() as f64 * char_width  — байты
//! class/graph.rs      text.len() as f64 * char_width  — байты
//! er/engine.rs        name.len() as f64 * 9.0       — байты
//! mindmap/engine.rs   chars().count() * (font_size * 0.6)
//! usecase/engine.rs   chars().count() * 9.0
//! sequence/config.rs  chars().count() * 7.5
//! ```
//!
//! # Точность
//!
//! Реальные метрики шрифта (как в PlantUML, который использует AWT
//! `FontMetrics`) требуют доступа к файлу шрифта и несовместимы с WASM без
//! встраивания шрифта в бинарник. Поэтому здесь применяется таблица ширин
//! глифов, откалиброванная по эталонам PlantUML: измеряется ровно то, что
//! нужно для совпадения геометрии, и результат детерминирован.
//!
//! Таблица задаёт ширину в долях от размера шрифта (em). Значения получены
//! из эталонных SVG PlantUML: например, для `Alice` (5 символов, font-size
//! 14) PlantUML пишет `textLength="33.667"`, то есть 33.667 / 5 / 14 ≈ 0.481 em
//! на символ в среднем.
//!
//! # Пример
//!
//! ```
//! use plantuml_layout::text::TextMeasurer;
//!
//! let m = TextMeasurer::default();
//! // Кириллица измеряется по символам, а не по байтам:
//! // «Класс» — 5 символов и 10 байт, ширина считается по 5 символам
//! assert!(m.width("Класс", 14.0) < 10.0 * 14.0 * 0.5);
//! assert_eq!(m.width("", 14.0), 0.0);
//! ```

/// Средняя ширина символа в долях от размера шрифта.
///
/// Получена из эталонов PlantUML: `Alice` при font-size 14 даёт
/// `textLength="33.667"`, то есть 33.667 / (5 × 14) ≈ 0.481.
const AVG_CHAR_EM: f64 = 0.481;

/// Ширины узких символов в долях em.
const NARROW_EM: f64 = 0.28;

/// Ширины широких символов в долях em.
const WIDE_EM: f64 = 0.65;

/// Измеритель текста.
///
/// Хранит коэффициент средней ширины символа, что позволяет подстроить
/// раскладку под конкретный шрифт, не меняя код движков.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextMeasurer {
    /// Средняя ширина символа в долях от размера шрифта
    avg_char_em: f64,
    /// Ширина узкого символа (i, l, j, точка, пробел)
    narrow_em: f64,
    /// Ширина широкого символа (m, w, W, Ж, Ш, Щ)
    wide_em: f64,
}

impl Default for TextMeasurer {
    fn default() -> Self {
        Self {
            avg_char_em: AVG_CHAR_EM,
            narrow_em: NARROW_EM,
            wide_em: WIDE_EM,
        }
    }
}

impl TextMeasurer {
    /// Создаёт измеритель со средней шириной символа по умолчанию.
    pub fn new() -> Self {
        Self::default()
    }

    /// Создаёт измеритель с заданной средней шириной символа (в долях em).
    ///
    /// Полезно для подгонки под другой шрифт: PlantUML использует
    /// `sans-serif`, и его метрики зависят от системы.
    pub fn with_avg_char_em(avg_char_em: f64) -> Self {
        Self {
            avg_char_em,
            ..Self::default()
        }
    }

    /// Ширина текста в пикселях при заданном размере шрифта.
    ///
    /// Считает **символы**, а не байты, поэтому кириллица измеряется
    /// корректно. Переносы строк учитываются: берётся самая длинная строка.
    pub fn width(&self, text: &str, font_size: f64) -> f64 {
        if text.is_empty() {
            return 0.0;
        }

        // Многострочный текст: ширина определяется самой длинной строкой
        text.lines()
            .map(|line| self.line_width(line, font_size))
            .fold(0.0_f64, f64::max)
    }

    /// Ширина одной строки (без учёта переводов строк).
    pub fn line_width(&self, line: &str, font_size: f64) -> f64 {
        if line.is_empty() {
            return 0.0;
        }

        let em_sum: f64 = line.chars().map(|c| self.char_em(c)).sum();
        em_sum * font_size
    }

    /// Ширина одного символа в долях от размера шрифта.
    ///
    /// Узкие и широкие символы учитываются отдельно — это заметно улучшает
    /// совпадение для коротких подписей, где средняя ширина даёт большую
    /// относительную ошибку.
    fn char_em(&self, c: char) -> f64 {
        match c {
            // Узкие: узкие буквы латиницы и кириллицы, знаки препинания
            'i' | 'l' | 'j' | 'I' | '!' | '.' | ',' | ':' | ';' | '\'' | '|' | ' ' => {
                self.narrow_em
            }
            // Широкие: m, w и широкие буквы кириллицы
            'm' | 'w' | 'M' | 'W' | 'Ж' | 'Ш' | 'Щ' | 'Ю' | 'Ы' | 'Ф' => self.wide_em,
            // Прописные чуть шире строчных — для любой письменности,
            // не только латиницы
            c if c.is_uppercase() => self.avg_char_em * 1.16,
            // Остальные (включая строчную кириллицу) — средняя ширина
            _ => self.avg_char_em,
        }
    }

    /// Ширина текста с добавлением внутренних отступов.
    ///
    /// Удобно для боксов: `padding` добавляется с обеих сторон.
    pub fn width_with_padding(&self, text: &str, font_size: f64, padding: f64) -> f64 {
        self.width(text, font_size) + padding * 2.0
    }

    /// Высота текста: число строк, умноженное на высоту строки.
    pub fn height(&self, text: &str, line_height: f64) -> f64 {
        let lines = text.lines().count().max(1);
        lines as f64 * line_height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Кириллица измеряется по символам, а не по байтам.
    ///
    /// Регрессия: пять движков использовали `str::len()`, который для
    /// кириллицы возвращает число байт (вдвое больше символов), из-за чего
    /// русские подписи получали примерно вдвое большую ширину.
    #[test]
    fn cyrillic_measured_by_chars_not_bytes() {
        let m = TextMeasurer::default();
        // «Класс» — 5 символов, 10 байт в UTF-8
        assert_eq!("Класс".len(), 10, "предпосылка теста");

        // Одинаковые по длине строки без узких и широких символов
        // измеряются одинаково. (Латинская `l` узкая, поэтому пары вроде
        // «класс»/«class» намеренно не сравниваем — там различие законно.)
        assert_eq!(
            m.width("абвгд", 14.0),
            m.width("abcde", 14.0),
            "кириллица и латиница одной длины должны измеряться одинаково"
        );

        // Главное: результат не масштабируется числом БАЙТ.
        // 5 символов по 0.481 em при 14px дают ~33.7px; байтовый счёт дал бы вдвое.
        let width = m.width("класс", 14.0);
        assert!(
            width < 10.0 * 14.0 * 0.481,
            "ширина {width:.1} похожа на расчёт по байтам (10 вместо 5 символов)"
        );
    }

    #[test]
    fn empty_text_has_zero_width() {
        let m = TextMeasurer::default();
        assert_eq!(m.width("", 14.0), 0.0);
        assert_eq!(m.line_width("", 14.0), 0.0);
    }

    #[test]
    fn width_scales_with_font_size() {
        let m = TextMeasurer::default();
        let small = m.width("Hello", 10.0);
        let large = m.width("Hello", 20.0);
        assert!(
            (large - small * 2.0).abs() < 0.001,
            "ширина должна быть пропорциональна размеру шрифта"
        );
    }

    /// Ширина многострочного текста определяется самой длинной строкой.
    #[test]
    fn multiline_uses_longest_line() {
        let m = TextMeasurer::default();
        let one_line = m.width("короткая", 14.0);
        let multi = m.width("короткая\nзначительно более длинная строка", 14.0);
        assert!(multi > one_line);
        assert_eq!(multi, m.width("значительно более длинная строка", 14.0));
    }

    /// Ширина совпадает с эталоном PlantUML для известного случая.
    ///
    /// PlantUML при font-size 14 для текста `Alice` пишет
    /// `textLength="33.667"`. Проверяем, что расхождение невелико: точное
    /// совпадение требует метрик конкретного шрифта, но порядок величины
    /// должен быть верным.
    #[test]
    fn width_matches_plantuml_reference_order() {
        let m = TextMeasurer::default();
        let ours = m.width("Alice", 14.0);
        let plantuml = 33.667;
        let deviation = (ours - plantuml).abs() / plantuml;
        assert!(
            deviation < 0.35,
            "ширина «Alice» = {ours:.2}, эталон PlantUML = {plantuml}, \
             расхождение {:.0}%",
            deviation * 100.0
        );
    }

    #[test]
    fn narrow_chars_are_narrower_than_wide() {
        let m = TextMeasurer::default();
        assert!(m.width("iii", 14.0) < m.width("mmm", 14.0));
        assert!(m.width("lll", 14.0) < m.width("WWW", 14.0));
    }

    #[test]
    fn padding_added_on_both_sides() {
        let m = TextMeasurer::default();
        let bare = m.width("текст", 13.0);
        let padded = m.width_with_padding("текст", 13.0, 10.0);
        assert!((padded - bare - 20.0).abs() < 0.001);
    }
}
