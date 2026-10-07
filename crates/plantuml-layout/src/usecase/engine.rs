//! Use Case Diagram Layout Engine
//!
//! Алгоритм layout для диаграмм вариантов использования.
//! PlantUML стиль: актёры слева, система справа с use cases внутри.

use std::collections::HashMap;

use plantuml_ast::common::Direction;
use plantuml_ast::usecase::{UseCaseDiagram, UseCaseRelationType, UseCaseRelationship};
use plantuml_model::{Point, Rect};

use super::config::UseCaseLayoutConfig;

/// Базовая ширина эллипса use case, измеренная по эталону PlantUML.
const USE_CASE_BASE_WIDTH: f64 = 55.1;
/// Прибавка к ширине на каждый символ подписи.
const USE_CASE_CHAR_WIDTH: f64 = 6.48;
/// Базовая высота эллипса use case.
const USE_CASE_BASE_HEIGHT: f64 = 21.4;
/// Прибавка к высоте на каждый символ подписи.
const USE_CASE_CHAR_HEIGHT: f64 = 0.95;
/// Зазор между подписью актёра и первым вариантом использования.
///
/// Измерено по эталону usecase_basic: подпись актёра на y=78.495,
/// верх первого эллипса — 141.8.
/// Шаг между use case в режиме `left to right direction`.
///
/// Измерено по эталону `UseCase: Простой`: эллипсы идут через 72–76 при
/// высотах 35–42, то есть 34.6 промежутка. Общая константа 60 верна для
/// `usecase_basic` и в LTR не подходит.
/// Прибавка к ширине самого широкого use case для рамки системы.
///
/// Измерено по эталону `UseCase: Простой`: рамка 220.2, самый широкий
/// эллипс 188.2.
/// Верх рамки системы в режиме `left to right`.
///
/// Измерено по эталону `UseCase: Простой`: рамка стоит на y=7, тогда как
/// общий `margin` равен 16.
const LTR_PACKAGE_TOP: f64 = 7.0;

/// Отступ от верха рамки до первого use case в режиме `left to right`.
///
/// Эталон: рамка на 7, верх первого эллипса на 42.
const LTR_PACKAGE_TOP_INSET: f64 = 35.0;

/// Отступ от низа последнего use case до низа рамки в режиме `left to right`.
///
/// Эталон: последний эллипс кончается на 377, рамка — на 393.
const LTR_PACKAGE_BOTTOM_INSET: f64 = 16.0;

const USECASE_SYSTEM_PADDING: f64 = 32.0;

const LTR_USECASE_SPACING: f64 = 34.6;

/// Зазор между эллипсами СОСЕДНИХ вариантов в одном ряду.
///
/// Режим сверху-вниз раскладывает варианты рядами; измерено по эталону
/// `UC_noLTR` (PlantUML 1.2024.3): центры `111, 323, 526, 736, 954` при
/// ширинах 176.3, 178.0, 157.4, 192.2, 173.7 — между КРАЯМИ соседних
/// эллипсов ровно 35.
const ROW_USECASE_SPACING: f64 = 35.0;

const ACTOR_LABEL_GAP: f64 = 55.0;

/// Скругление углов рамки системы.
///
/// Эталон `UseCase: Простой`: `rx="2.5" ry="2.5"`.
const SYSTEM_CORNER_RADIUS: f64 = 2.5;

/// Кегль заголовка рамки системы.
///
/// Эталон `UseCase: Простой`: `font-size="14"`, а не 15, как у остальных
/// рамок.
const SYSTEM_TITLE_FONT_SIZE: f64 = 14.0;

/// Базис заголовка от верха рамки системы.
///
/// Эталон `UseCase: Простой`: рамка стоит на `y=7`, базис подписи — на
/// `21.995`, то есть 15 ниже.
const SYSTEM_TITLE_OFFSET: f64 = 15.0;

/// Заливка эллипсов вариантов использования и головы актёра.
///
/// Эталон `UseCase: Простой`: все семь эллипсов (пять вариантов и две
/// головы актёров) залиты `#F1F1F1`, тогда как тема даёт `#E2E2F0` —
/// цвет узлов class-диаграмм.
const USECASE_FILL: &str = "#F1F1F1";

/// Толщина обводки эллипса варианта использования (эталон: 0.5).
const USECASE_STROKE_WIDTH: f64 = 0.5;

use crate::{EdgeType, ElementType, LayoutElement, LayoutResult};

/// Layout engine для use case diagrams
pub struct UseCaseLayoutEngine {
    config: UseCaseLayoutConfig,
}

impl UseCaseLayoutEngine {
    /// Создаёт новый engine с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self {
            config: UseCaseLayoutConfig::default(),
        }
    }

    /// Создаёт engine с заданной конфигурацией
    pub fn with_config(config: UseCaseLayoutConfig) -> Self {
        Self { config }
    }

    /// Выполняет layout диаграммы
    pub fn layout(&self, diagram: &UseCaseDiagram) -> LayoutResult {
        let mut elements = Vec::new();
        let mut element_positions: HashMap<String, Rect> = HashMap::new();

        // `left to right direction` меняет местами АКТЁРОВ и СИСТЕМУ, но
        // НЕ ориентацию списка use case: в эталоне `UseCase: Простой`
        // эллипсы по-прежнему идут в столбец (cy = 62, 136, 208, 280, 356),
        // а различие в том, что рамка системы стоит СПРАВА (x=137.5), а
        // актёры «Покупатель» и «Менеджер» — СЛЕВА (x=6 и 10.6).
        let is_left_to_right = diagram.direction == Direction::LeftToRight;

        // Максимальная ширина имён актёров.
        //
        // Раньше здесь была оценка «символ * 9.0». Для «Пользователь» она
        // давала 108 при эталонных 103.1 — на 4.8% больше, причём ошибка
        // зависела от алфавита: для латиницы 9px на символ завышает сильнее.
        let max_actor_label_width = diagram
            .actors
            .iter()
            .map(|a| self.config.text.width(&a.name, self.config.font_size))
            .fold(0.0f64, f64::max);

        // Ширина блока актёра: фигура занимает `actor_width`, но подпись
        // шире, поэтому блок расширяется до неё — иначе подпись выходит
        // за габарит элемента и обрезается краем холста.
        let actor_total_width = self.config.actor_width.max(max_actor_label_width);

        // Собираем все use cases (из packages и верхнего уровня)
        let mut all_usecases: Vec<(&str, Option<&str>)> = Vec::new();

        for uc in &diagram.use_cases {
            all_usecases.push((&uc.name, uc.alias.as_deref()));
        }

        for pkg in &diagram.packages {
            for uc in &pkg.use_cases {
                all_usecases.push((&uc.name, uc.alias.as_deref()));
            }
        }

        // Наибольшая ширина эллипса среди use case: по ней выравнивается
        // вся группа, как в PlantUML.
        let max_usecase_width = all_usecases
            .iter()
            .map(|(name, _)| self.usecase_natural_size(name).0)
            .fold(0.0_f64, f64::max)
            .max(self.config.usecase_width.min(60.0));

        // Рамка системы рисуется ТОЛЬКО если в диаграмме есть package.
        // В эталоне PlantUML при отсутствии package рамки нет вовсе
        // (usecase_basic: ни одного прямоугольника).
        let has_package = !diagram.packages.is_empty();

        // Слои вариантов использования по графу связей.
        //
        // PlantUML считает раскладку GraphViz-ом: актёры — нулевой ранг,
        // варианты без входящих связей ОТ ДРУГИХ ВАРИАНТОВ — первый,
        // и так далее. Каждый ранг рисуется РЯДОМ.
        //
        // Эталон (PlantUML 1.2024.3, тот же исходник без директивы
        // `left to right direction`): актёры в ряд сверху (головы на
        // `cx = 323` и `845`), рамка `7…1057`, а все пять эллипсов —
        // в ОДНОМ ряду (`cy = 163.6`) с центрами `111, 323, 526, 736,
        // 954`, то есть слева направо группами по актёру и внутри группы
        // в порядке объявления. Прежний код ставил варианты В СТОЛБЕЦ
        // всегда, из-за чего та же диаграмма выходила `240x663` вместо
        // `1063x207`: связи шли наискось через всю фигуру, а подпись
        // второго актёра накладывалась на заголовок рамки.
        //
        // Столбец при этом получается сам собой: в `usecase_basic`
        // (`UC1 --> UC2`) ранг UC1 равен 0, ранг UC2 — 1, и ряды встают
        // друг под друга ровно как в эталоне.
        let mut index_of: HashMap<&str, usize> = HashMap::new();
        for (i, (name, alias)) in all_usecases.iter().enumerate() {
            index_of.insert(name, i);
            if let Some(a) = alias {
                index_of.insert(a, i);
            }
        }
        let mut layers = vec![0usize; all_usecases.len()];
        for _ in 0..all_usecases.len() {
            let mut changed = false;
            for rel in &diagram.relationships {
                if let (Some(&from), Some(&to)) = (
                    index_of.get(rel.from.as_str()),
                    index_of.get(rel.to.as_str()),
                ) {
                    if from != to && layers[from] + 1 > layers[to] {
                        layers[to] = layers[from] + 1;
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        let layer_count = layers.iter().copied().max().unwrap_or(0) + 1;
        let mut rows: Vec<Vec<usize>> = vec![Vec::new(); layer_count];
        for (i, layer) in layers.iter().enumerate() {
            rows[*layer].push(i);
        }

        let usecase_width = |i: usize| self.usecase_natural_size(all_usecases[i].0).0;
        let usecase_height = |i: usize| self.usecase_natural_size(all_usecases[i].0).1;
        // Ширина ряда: эллипсы в ряд через тот же зазор, что и в LTR
        // (эталон даёт 35 между краями соседних эллипсов).
        let row_width = |row: &Vec<usize>| -> f64 {
            row.iter().map(|&i| usecase_width(i)).sum::<f64>()
                + row.len().saturating_sub(1) as f64 * ROW_USECASE_SPACING
        };
        let row_height = |row: &Vec<usize>| -> f64 {
            row.iter().map(|&i| usecase_height(i)).fold(0.0, f64::max)
        };
        let content_width = rows.iter().map(row_width).fold(0.0, f64::max);
        let content_height = rows.iter().map(row_height).sum::<f64>()
            + layer_count.saturating_sub(1) as f64 * self.config.vertical_spacing;

        let (system_width, system_height) = if has_package {
            // Ширина рамки системы.
            //
            // Измерено по эталонам: рамка 220.2 при самом широком эллипсе
            // 188.2 (`UseCase: Простой`) и рамка 1050 при ряде шириной
            // 1018 (`UC_noLTR`) — то есть содержимое плюс 32. А по
            // вертикали: 35 сверху и 16 снизу (в LTR содержимое занимает
            // 42..377 при рамке 7..393, в режиме сверху-вниз — 143.6..183.6
            // при рамке 107..201.2). Прежняя формула добавляла
            // `package_padding * 2 + 40` = 90 и давала 268.2.
            (
                if is_left_to_right {
                    max_usecase_width + USECASE_SYSTEM_PADDING
                } else {
                    content_width + USECASE_SYSTEM_PADDING
                },
                if is_left_to_right {
                    let step = LTR_USECASE_SPACING;
                    let inner: f64 = (0..all_usecases.len()).map(usecase_height).sum::<f64>()
                        + all_usecases.len().saturating_sub(1) as f64 * step;
                    inner + LTR_PACKAGE_TOP_INSET + LTR_PACKAGE_BOTTOM_INSET
                } else {
                    content_height + LTR_PACKAGE_TOP_INSET + LTR_PACKAGE_BOTTOM_INSET
                },
            )
        } else if is_left_to_right {
            // Без package размеры области совпадают с содержимым
            (
                max_usecase_width,
                (0..all_usecases.len()).map(usecase_height).sum::<f64>()
                    + all_usecases.len().saturating_sub(1) as f64 * LTR_USECASE_SPACING,
            )
        } else {
            (content_width, content_height)
        };

        // PlantUML размещает актёров СВЕРХУ, а варианты использования —
        // вертикально ПОД ними (эталон usecase_basic: актёр на y=6..64,
        // первый use case на y=141.8, второй на y=237.06).
        // Раньше актёр стоял слева, а use case — справа, из-за чего
        // диаграмма была широкой и низкой (310x136 против 172x280).
        let layout_width = if is_left_to_right {
            max_usecase_width.max(actor_total_width)
        } else {
            system_width.max(actor_total_width)
        };
        // Верх первого ряда вариантов.
        //
        // В обоих режимах он стоит на `margin + высота актёра + зазор`:
        // эталон `usecase_basic` даёт верх первого эллипса 141.8, эталон
        // `UC_noLTR` — 143.6. В LTR варианты идут в столбец от этой точки,
        // в режиме сверху-вниз — рядами, а рамка (если она есть) сдвинута
        // выше на `LTR_PACKAGE_TOP_INSET`: эталон `UC_noLTR` ставит рамку
        // на 107 при первом эллипсе на 143.6, то есть 35 сверху.
        let first_row_top = self.config.margin + self.config.actor_height + ACTOR_LABEL_GAP;
        let (system_x, system_y) = if is_left_to_right {
            // Актёры слева, система справа от них.
            (
                self.config.margin + actor_total_width + ACTOR_LABEL_GAP - 20.5,
                LTR_PACKAGE_TOP,
            )
        } else {
            (
                self.config.margin + (layout_width - system_width) / 2.0,
                first_row_top
                    - if has_package {
                        LTR_PACKAGE_TOP_INSET
                    } else {
                        0.0
                    },
            )
        };
        // Ось, на которой центрируются варианты использования.
        //
        // Эталон `UseCase: Простой`: центры ВСЕХ пяти эллипсов совпадают
        // (`cx = 247.626`) и равны центру рамки (`137.53 + 220.2/2`).
        // Эталон `usecase_basic`: оба эллипса и актёр стоят на одной оси
        // `82.13`. Прежний код выравнивал эллипсы ЛЕВЫМ краем, поэтому
        // центры расходились на половину разницы ширин (у нас 226.4…242.6
        // вместо 242.57) — «лесенка» вместо столбца.
        let axis_x = system_x + system_width / 2.0;

        if has_package {
            let system_name = diagram.packages[0].name.clone();
            let system_bounds = Rect::new(system_x, system_y, system_width, system_height);
            // Рамка кластера в эталоне НЕ залита, скруглена и обведена
            // толщиной 1; заголовок — кеглем 14 с базисом на 15 ниже
            // верха. Передаём это свойствами, чтобы не менять рамку
            // `mainframe` из sequence, которая рисуется тем же элементом.
            let mut properties = std::collections::HashMap::new();
            properties.insert("fill".to_string(), "none".to_string());
            properties.insert("rx".to_string(), SYSTEM_CORNER_RADIUS.to_string());
            properties.insert(
                "title-font-size".to_string(),
                SYSTEM_TITLE_FONT_SIZE.to_string(),
            );
            properties.insert("title-offset".to_string(), SYSTEM_TITLE_OFFSET.to_string());
            elements.push(LayoutElement {
                id: format!("system_{}", system_name.replace(' ', "_")),
                bounds: system_bounds,
                text: None,
                properties,
                element_type: ElementType::System { title: system_name },
            });
        }

        // Ось, на которой центрируются варианты использования.
        //
        // Эталон `UseCase: Простой`: центры ВСЕХ пяти эллипсов совпадают
        // (`cx = 247.626`) и равны центру рамки (`137.53 + 220.2/2`).
        // Эталон `usecase_basic`: оба эллипса и актёр стоят на одной оси
        // `82.13`. Прежний код выравнивал эллипсы ЛЕВЫМ краем, поэтому
        // центры расходились на половину разницы ширин (у нас 226.4…242.6
        // вместо 242.57) — «лесенка» вместо столбца.
        // Позиции вариантов: (индекс, центр по X, верх по Y).
        //
        // В LTR-режиме это ОДИН столбец на общей оси (эталон
        // `UseCase: Простой`: `cy = 62, 136, 208, 280, 356`), в режиме
        // сверху-вниз — ряды по слоям графа связей.
        let mut positions: Vec<(usize, f64, f64)> = Vec::with_capacity(all_usecases.len());
        if is_left_to_right {
            // Столбец начинается от верха рамки: эталон `UseCase: Простой`
            // ставит рамку на `y = 7`, а первый эллипс — на 42, то есть
            // `LTR_PACKAGE_TOP_INSET` ниже.
            let mut y = system_y
                + if has_package {
                    LTR_PACKAGE_TOP_INSET
                } else {
                    0.0
                };
            for i in 0..all_usecases.len() {
                positions.push((i, axis_x, y));
                y += usecase_height(i) + LTR_USECASE_SPACING;
            }
        } else {
            let mut y = first_row_top;
            for row in &rows {
                let mut x = axis_x - row_width(row) / 2.0;
                for &i in row {
                    let width = usecase_width(i);
                    positions.push((i, x + width / 2.0, y));
                    x += width + ROW_USECASE_SPACING;
                }
                y += row_height(row) + self.config.vertical_spacing;
            }
        }

        for (i, cx, y) in positions {
            let (name, alias) = all_usecases[i];
            let (elem, bounds) = self.create_usecase_element(name, cx, y);
            element_positions.insert(name.to_string(), bounds);
            if let Some(a) = alias {
                element_positions.insert(a.to_string(), bounds);
            }
            elements.push(elem);
        }

        // Размещаем актёров.
        //
        // В LTR-режиме актёры стоят СЛЕВА столбцом и центрируются по
        // СВЯЗАННЫМ вариантам по вертикали: эталон `UseCase: Простой` даёт
        // `Customer`, связанному с UC1/UC2/UC3 (cy 62, 136, 208), голову
        // на 106 — это среднее минус половина высоты актёра.
        //
        // В режиме сверху-вниз актёры идут РЯДОМ сверху и центрируются по
        // связанным вариантам ПО ГОРИЗОНТАЛИ: эталон `UC_noLTR` ставит
        // «Покупателя» на `cx = 323` при его вариантах с центрами 111, 323
        // и 526 (среднее 320), а «Менеджера» — на 845 при 736 и 954
        // (среднее 845).
        //
        // `actor_x` — ЦЕНТР блока актёра; сам блок шириной с подпись
        // (см. `create_actor_element`), поэтому подпись не выходит за
        // габарит и не обрезается холстом. Если два актёра оказываются на
        // одной высоте, второй сдвигается вправо на ширину блока.
        let mut occupied: Vec<(f64, f64)> = Vec::new(); // (y, x)
        for (actor_index, actor) in diagram.actors.iter().enumerate() {
            // Связи ссылаются на АЛИАС (`Customer`), а не на подпись
            // (`Покупатель`), поэтому сверяем оба.
            let alias = actor.alias.as_deref();
            let is_me = |name: &str| name == actor.name || Some(name) == alias;
            let mut sum_x = 0.0;
            let mut sum_y = 0.0;
            let mut count = 0.0;
            for rel in &diagram.relationships {
                let other = if is_me(&rel.from) {
                    Some(&rel.to)
                } else if is_me(&rel.to) {
                    Some(&rel.from)
                } else {
                    None
                };
                if let Some(name) = other {
                    if let Some(rect) = element_positions.get(name.as_str()) {
                        sum_x += rect.x + rect.width / 2.0;
                        sum_y += rect.y + rect.height / 2.0;
                        count += 1.0;
                    }
                }
            }

            let (y, default_x) = if is_left_to_right {
                (
                    if count > 0.0 {
                        (sum_y / count - self.config.actor_height / 2.0).max(self.config.margin)
                    } else {
                        system_y + actor_index as f64 * (self.config.actor_height + 10.0)
                    },
                    self.config.margin + actor_total_width / 2.0,
                )
            } else {
                (
                    self.config.margin,
                    if count > 0.0 {
                        sum_x / count
                    } else {
                        axis_x
                            + actor_index as f64
                                * (actor_total_width + self.config.horizontal_spacing)
                    },
                )
            };

            // Ищем свободную позицию по X среди актёров с такой же Y.
            let mut actor_x = default_x;
            let tolerance = self.config.actor_height / 2.0;
            while occupied.iter().any(|(oy, ox)| {
                (oy - y).abs() < tolerance && (ox - actor_x).abs() < actor_total_width
            }) {
                actor_x += actor_total_width + self.config.horizontal_spacing;
            }
            occupied.push((y, actor_x));

            let (elem, bounds) =
                self.create_actor_element(&actor.name, actor_x, y, actor_total_width);
            element_positions.insert(actor.name.clone(), bounds);
            if let Some(alias) = &actor.alias {
                element_positions.insert(alias.clone(), bounds);
            }
            elements.push(elem);
        }

        // Создаём связи
        for rel in &diagram.relationships {
            if let Some(edge) = self.create_relationship_element(rel, &element_positions) {
                elements.push(edge);
            }
        }

        // Вычисляем bounds
        let mut result = LayoutResult {
            elements,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        };
        result.calculate_bounds();

        result.bounds.width += self.config.margin;
        result.bounds.height += self.config.margin;

        result
    }

    /// Создаёт элемент актёра (stick figure).
    ///
    /// `x` — ЦЕНТР блока, а не его левый край: фигура актёра рисуется по
    /// центру габарита, а подпись — по центру под фигурой, как в эталоне
    /// (`usecase_basic`: подпись «Пользователь» центрирована на `82.13` —
    /// там же, где голова актёра).
    ///
    /// Ширина блока равна ширине подписи (`width`), а не 40: иначе
    /// подпись выходила за габарит элемента (у «Покупателя» это −10.48
    /// при холсте, начинающемся с 9), обрезалась левым краем картинки, а
    /// связи начинались в середине подписи и шли наискось через всю
    /// диаграмму.
    fn create_actor_element(
        &self,
        name: &str,
        x: f64,
        y: f64,
        width: f64,
    ) -> (LayoutElement, Rect) {
        let bounds = Rect::new(x - width / 2.0, y, width, self.config.actor_height);

        // Голова актёра в use case залита тем же `#F1F1F1`, что и
        // эллипсы (эталоны `usecase_basic`, `UseCase: Простой`); тема
        // даёт цвет узлов sequence — `#E2E2F0`.
        let mut properties = std::collections::HashMap::new();
        properties.insert("fill".to_string(), USECASE_FILL.to_string());

        (
            LayoutElement {
                id: format!("actor_{}", name.replace(' ', "_")),
                bounds,
                text: None,
                properties,
                element_type: ElementType::Actor {
                    label: name.to_string(),
                },
            },
            bounds,
        )
    }

    /// Натуральный размер эллипса use case (ширина, высота).
    ///
    /// Измерено по эталону PlantUML: «Оформить заказ» (15 символов) —
    /// 152.26 x 35.25, «Оплатить» (8 символов) — 106.92 x 29.05. Отсюда
    /// ширина ≈ 55.1 + 6.48 * n, высота ≈ 21.4 + 0.95 * n.
    fn usecase_natural_size(&self, name: &str) -> (f64, f64) {
        let chars = name.chars().count() as f64;
        (
            USE_CASE_BASE_WIDTH + USE_CASE_CHAR_WIDTH * chars,
            USE_CASE_BASE_HEIGHT + USE_CASE_CHAR_HEIGHT * chars,
        )
    }

    /// Создаёт элемент use case (эллипс).
    ///
    /// `x` — ЦЕНТР эллипса: в эталоне центры всех вариантов совпадают
    /// (`UseCase: Простой`: `cx = 247.626` у всех пяти).
    fn create_usecase_element(&self, name: &str, x: f64, y: f64) -> (LayoutElement, Rect) {
        // Размер эллипса PlantUML зависит от длины подписи. Измерено
        // по эталону (tests/golden/reference/usecase_basic.svg):
        //   «Оформить заказ» (15 символов) — 152.26 x 35.25
        //   «Оплатить»        (8 символов) — 106.92 x 29.05
        // Отсюда ширина ≈ 55.1 + 6.48 * n, высота ≈ 21.4 + 0.95 * n.
        // Раньше размер был фиксированным (160x40), из-за чего диаграмма
        // получалась шире эталона независимо от подписей.
        let chars = name.chars().count() as f64;
        let width = (USE_CASE_BASE_WIDTH + USE_CASE_CHAR_WIDTH * chars)
            .max(self.config.usecase_width.min(60.0));
        let height = (USE_CASE_BASE_HEIGHT + USE_CASE_CHAR_HEIGHT * chars)
            .max(self.config.usecase_height.min(24.0));
        let bounds = Rect::new(x - width / 2.0, y, width, height);

        // Заливка и толщина обводки эллипса — из эталона: `#F1F1F1` и
        // 0.5 (тема даёт `#E2E2F0` и 1).
        let mut properties = std::collections::HashMap::new();
        properties.insert("fill".to_string(), USECASE_FILL.to_string());
        properties.insert("stroke-width".to_string(), USECASE_STROKE_WIDTH.to_string());

        (
            LayoutElement {
                id: format!("usecase_{}", name.replace(' ', "_")),
                bounds,
                text: None,
                properties,
                element_type: ElementType::Ellipse {
                    label: Some(name.to_string()),
                },
            },
            bounds,
        )
    }

    /// Создаёт элемент связи
    fn create_relationship_element(
        &self,
        rel: &UseCaseRelationship,
        positions: &HashMap<String, Rect>,
    ) -> Option<LayoutElement> {
        let from_rect = positions.get(&rel.from)?;
        let to_rect = positions.get(&rel.to)?;

        let (start, end) = self.calculate_connection_points(from_rect, to_rect);

        let min_x = start.x.min(end.x);
        let min_y = start.y.min(end.y);
        let max_x = start.x.max(end.x);
        let max_y = start.y.max(end.y);

        let dashed = matches!(
            rel.relation_type,
            UseCaseRelationType::Include | UseCaseRelationType::Extend
        );

        Some(LayoutElement {
            id: format!(
                "rel_{}_{}",
                rel.from.replace(' ', "_"),
                rel.to.replace(' ', "_")
            ),
            bounds: Rect::new(
                min_x,
                min_y,
                (max_x - min_x).max(1.0),
                (max_y - min_y).max(1.0),
            ),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Edge {
                points: vec![start, end],
                label: rel.label.clone(),
                arrow_start: false,
                // В PlantUML --> всегда показывает стрелку
                // Стрелки нужны для всех типов кроме undirected (которые мы пока не поддерживаем)
                arrow_end: true,
                dashed,
                edge_type: match rel.relation_type {
                    UseCaseRelationType::Generalization => EdgeType::Inheritance,
                    UseCaseRelationType::Include => EdgeType::Dependency,
                    UseCaseRelationType::Extend => EdgeType::Dependency,
                    UseCaseRelationType::Association => EdgeType::Association,
                },
                from_cardinality: None,
                to_cardinality: None,
            },
        })
    }

    /// Вычисляет точки соединения для связи.
    ///
    /// Грань выбирается по взаимному расположению центров: связь идёт
    /// вдоль той оси, по которой фигуры разнесены сильнее. Прежнее
    /// правило «ширина меньше 50 — актёр, соединяем справа, иначе
    /// эллипс — соединяем слева» давало две ошибки:
    ///
    /// * блок актёра, расширенный до ширины подписи, перестал быть
    ///   «узким», и связь уходила из его ЛЕВОГО края;
    /// * связь актёр→вариант в режиме сверху-вниз, где фигуры стоят
    ///   одна над другой, шла по касательной слева: в эталоне
    ///   `usecase_basic` это вертикальная линия `x = 82.13`, а у нас
    ///   была диагональ из `x = 16` в `x = 16` через всю фигуру.
    fn calculate_connection_points(&self, from: &Rect, to: &Rect) -> (Point, Point) {
        let from_cx = from.x + from.width / 2.0;
        let from_cy = from.y + from.height / 2.0;
        let to_cx = to.x + to.width / 2.0;
        let to_cy = to.y + to.height / 2.0;

        let dx = to_cx - from_cx;
        let dy = to_cy - from_cy;

        if dx.abs() >= dy.abs() {
            // Фигуры разнесены по горизонтали: выходим правым/левым
            // краем на высоте центра.
            let start = Point::new(
                if dx >= 0.0 {
                    from.x + from.width
                } else {
                    from.x
                },
                from_cy,
            );
            let end = Point::new(if dx >= 0.0 { to.x } else { to.x + to.width }, to_cy);
            (start, end)
        } else {
            // Фигуры разнесены по вертикали: выходим низом/верхом на оси.
            let start = Point::new(
                from_cx,
                if dy >= 0.0 {
                    from.y + from.height
                } else {
                    from.y
                },
            );
            let end = Point::new(to_cx, if dy >= 0.0 { to.y } else { to.y + to.height });
            (start, end)
        }
    }
}

impl Default for UseCaseLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_ast::usecase::{UseCase, UseCaseActor};

    /// Актёры не должны накладываться друг на друга.
    ///
    /// Регрессия: раньше `actors_x` вычислялся один раз и использовался для
    /// всех актёров, поэтому несколько актёров на одной высоте рисовались в
    /// одной точке.
    #[test]
    fn test_actors_do_not_overlap() {
        let mut diagram = plantuml_ast::usecase::UseCaseDiagram::new();
        diagram.actors.push(UseCaseActor::new("A"));
        diagram.actors.push(UseCaseActor::new("B"));
        diagram.actors.push(UseCaseActor::new("C"));
        diagram.use_cases.push(UseCase::new("U"));

        let result = UseCaseLayoutEngine::new().layout(&diagram);

        let actors: Vec<&LayoutElement> = result
            .elements
            .iter()
            .filter(|e| e.id.starts_with("actor"))
            .collect();
        assert_eq!(actors.len(), 3, "не все актёры размещены");

        // Попарно проверяем, что прямоугольники не пересекаются
        for (i, a) in actors.iter().enumerate() {
            for b in actors.iter().skip(i + 1) {
                let overlap_x = a.bounds.x < b.bounds.x + b.bounds.width
                    && b.bounds.x < a.bounds.x + a.bounds.width;
                let overlap_y = a.bounds.y < b.bounds.y + b.bounds.height
                    && b.bounds.y < a.bounds.y + a.bounds.height;
                assert!(
                    !(overlap_x && overlap_y),
                    "актёры {} и {} перекрываются: {:?} и {:?}",
                    a.id,
                    b.id,
                    a.bounds,
                    b.bounds
                );
            }
        }
    }

    #[test]
    fn test_layout_simple() {
        let mut diagram = UseCaseDiagram::new();
        diagram.actors.push(UseCaseActor::new("User"));
        diagram.use_cases.push(UseCase::new("Login"));
        diagram
            .relationships
            .push(UseCaseRelationship::new("User", "Login"));

        let engine = UseCaseLayoutEngine::new();
        let result = engine.layout(&diagram);

        // Должны быть: actor + usecase + система + связь
        assert!(result.elements.len() >= 3);
    }

    #[test]
    fn test_layout_with_package() {
        use plantuml_ast::usecase::UseCasePackage;

        let mut diagram = UseCaseDiagram::new();
        diagram.actors.push(UseCaseActor::new("User"));

        let mut pkg = UseCasePackage::new("System");
        pkg.use_cases.push(UseCase::new("Login"));
        pkg.use_cases.push(UseCase::new("Logout"));
        diagram.packages.push(pkg);

        let engine = UseCaseLayoutEngine::new();
        let result = engine.layout(&diagram);

        // Должны быть: actor + system + 2 usecases
        assert!(result.elements.len() >= 4);
    }

    #[test]
    fn test_layout_include_extend() {
        let mut diagram = UseCaseDiagram::new();
        diagram.use_cases.push(UseCase::new("Login"));
        diagram.use_cases.push(UseCase::new("Authenticate"));
        diagram
            .relationships
            .push(UseCaseRelationship::include("Login", "Authenticate"));

        let engine = UseCaseLayoutEngine::new();
        let result = engine.layout(&diagram);

        assert!(result.elements.len() >= 3);
    }

    /// Варианты без связей между собой PlantUML кладёт РЯДОМ, а не столбцом.
    ///
    /// Регрессия: прежний код всегда ставил варианты в столбец, из-за чего
    /// пример с `rectangle` и двумя актёрами выходил 240x663 вместо
    /// эталонных 1063x207 — связи шли наискось через всю фигуру, а подпись
    /// второго актёра накладывалась на заголовок рамки.
    #[test]
    fn test_usecases_without_edges_go_in_a_row() {
        let mut diagram = UseCaseDiagram::new();
        diagram.actors.push(UseCaseActor::new("Покупатель"));
        diagram.use_cases.push(UseCase::new("Просмотр каталога"));
        diagram.use_cases.push(UseCase::new("Добавить в корзину"));
        diagram.use_cases.push(UseCase::new("Оформить заказ"));
        diagram
            .relationships
            .push(UseCaseRelationship::new("Покупатель", "Просмотр каталога"));

        let result = UseCaseLayoutEngine::new().layout(&diagram);

        let ellipses: Vec<Rect> = result
            .elements
            .iter()
            .filter(|e| matches!(e.element_type, ElementType::Ellipse { .. }))
            .map(|e| e.bounds)
            .collect();
        assert_eq!(ellipses.len(), 3, "не все варианты размещены");

        // Все три — на одной высоте и не перекрываются по X.
        let first_y = ellipses[0].y;
        for rect in &ellipses {
            assert!(
                (rect.y - first_y).abs() < 0.01,
                "варианты одного слоя должны стоять в ряд: {:?}",
                ellipses
            );
        }
        let mut sorted = ellipses.clone();
        sorted.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
        for pair in sorted.windows(2) {
            assert!(
                pair[0].x + pair[0].width <= pair[1].x + 0.01,
                "эллипсы ряда перекрываются: {:?} и {:?}",
                pair[0],
                pair[1]
            );
        }

        // Актёр стоит НАД рядом, а не сбоку: его низ выше верха эллипсов.
        let actor = result
            .elements
            .iter()
            .find(|e| matches!(e.element_type, ElementType::Actor { .. }))
            .expect("актёр размещён");
        assert!(
            actor.bounds.y + actor.bounds.height < first_y,
            "актёр должен стоять над вариантами: {:?} против y={first_y}",
            actor.bounds
        );
    }

    /// Цепочка связей между вариантами по-прежнему даёт СТОЛБЕЦ.
    ///
    /// Эталон `usecase_basic` (`Пользователь --> UC1`, `UC1 --> UC2`):
    /// эллипсы стоят друг под другом на общей оси.
    #[test]
    fn test_usecase_chain_stays_in_a_column() {
        let mut diagram = UseCaseDiagram::new();
        diagram.actors.push(UseCaseActor::new("Пользователь"));
        diagram.use_cases.push(UseCase::new("Оформить заказ"));
        diagram.use_cases.push(UseCase::new("Оплатить"));
        diagram
            .relationships
            .push(UseCaseRelationship::new("Пользователь", "Оформить заказ"));
        diagram
            .relationships
            .push(UseCaseRelationship::new("Оформить заказ", "Оплатить"));

        let result = UseCaseLayoutEngine::new().layout(&diagram);
        let ellipses: Vec<Rect> = result
            .elements
            .iter()
            .filter(|e| matches!(e.element_type, ElementType::Ellipse { .. }))
            .map(|e| e.bounds)
            .collect();
        assert_eq!(ellipses.len(), 2);

        let cx = |r: &Rect| r.x + r.width / 2.0;
        assert!(
            (cx(&ellipses[0]) - cx(&ellipses[1])).abs() < 0.01,
            "варианты цепочки должны стоять на одной оси: {:?}",
            ellipses
        );
        assert!(
            ellipses[0].y + ellipses[0].height < ellipses[1].y,
            "второй вариант цепочки должен быть НИЖЕ первого: {:?}",
            ellipses
        );
    }
}
