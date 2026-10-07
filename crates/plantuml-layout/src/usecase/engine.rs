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

const ACTOR_LABEL_GAP: f64 = 55.0;
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

        // Минимальная ширина для актёра с его label
        let actor_total_width = self.config.actor_width.max(max_actor_label_width);

        // Позиция актёров - центрируем по ширине их label
        let actors_x = self.config.margin + actor_total_width / 2.0;

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

        // Вычисляем размеры области use case
        let num_usecases = all_usecases.len().max(1);
        let usecase_row = |name: &str| self.usecase_natural_size(name).1;
        // Шаг берётся ТОТ ЖЕ, что и при расстановке ниже: в режиме
        // `left to right` он другой (34.6 вместо 60). Пока здесь стояла
        // общая константа, рамка системы выходила 507.8 при эталонных 386
        // — высота считалась по шагу, которым список уже не раскладывался.
        let usecase_step = if is_left_to_right {
            LTR_USECASE_SPACING
        } else {
            self.config.vertical_spacing
        };
        let inner_height: f64 = all_usecases
            .iter()
            .map(|(name, _)| usecase_row(name))
            .sum::<f64>()
            + (num_usecases.saturating_sub(1)) as f64 * usecase_step;

        // Рамка системы рисуется ТОЛЬКО если в диаграмме есть package.
        // В эталоне PlantUML при отсутствии package рамки нет вовсе
        // (usecase_basic: ни одного прямоугольника).
        let has_package = !diagram.packages.is_empty();

        let (system_width, system_height) = if has_package {
            // Ширина рамки системы.
            //
            // Измерено по эталону `UseCase: Простой`: рамка 220.2 при самом
            // широком эллипсе 188.2, то есть содержимое плюс 32. Прежняя
            // формула добавляла `package_padding * 2 + 40` = 90 и давала
            // 268.2. Ветка работает ТОЛЬКО при наличии package, поэтому
            // `usecase_basic` (без package) она не задевает.
            (
                max_usecase_width + USECASE_SYSTEM_PADDING,
                inner_height
                    + if is_left_to_right {
                        // Эталон: содержимое занимает 42..377, рамка 7..393,
                        // то есть 35 сверху и 16 снизу.
                        LTR_PACKAGE_TOP_INSET + LTR_PACKAGE_BOTTOM_INSET
                    } else {
                        self.config.package_header_height + self.config.package_padding * 2.0
                    },
            )
        } else {
            // Без package размеры области совпадают с содержимым
            (max_usecase_width, inner_height)
        };

        // PlantUML размещает актёров СВЕРХУ, а варианты использования —
        // вертикально ПОД ними (эталон usecase_basic: актёр на y=6..64,
        // первый use case на y=141.8, второй на y=237.06).
        // Раньше актёр стоял слева, а use case — справа, из-за чего
        // диаграмма была широкой и низкой (310x136 против 172x280).
        let layout_width = max_usecase_width.max(actor_total_width);
        let (system_x, system_y) = if is_left_to_right {
            // Актёры слева, система справа от них.
            (
                self.config.margin + actor_total_width + ACTOR_LABEL_GAP - 20.5,
                LTR_PACKAGE_TOP,
            )
        } else {
            (
                self.config.margin + (layout_width - system_width) / 2.0,
                self.config.margin + self.config.actor_height + ACTOR_LABEL_GAP,
            )
        };

        if has_package {
            let system_name = diagram.packages[0].name.clone();
            let system_bounds = Rect::new(system_x, system_y, system_width, system_height);
            elements.push(LayoutElement {
                id: format!("system_{}", system_name.replace(' ', "_")),
                bounds: system_bounds,
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::System { title: system_name },
            });
        }

        // Use case выравниваются по центру области
        let usecases_x = system_x + (system_width - max_usecase_width) / 2.0;
        // Отступ от верха рамки до первого use case.
        //
        // В LTR-режиме он МЕНЬШЕ: эталон `UseCase: Простой` даёт рамку на
        // y=7 и первый эллипс с верхом на 42, то есть 35. Общая формула
        // (заголовок 30 + padding 25) даёт 55.
        let usecases_start_y = system_y
            + if has_package {
                if is_left_to_right {
                    LTR_PACKAGE_TOP_INSET
                } else {
                    self.config.package_header_height + self.config.package_padding
                }
            } else {
                0.0
            };

        // Шаг между use case считается по их фактическим высотам, а не по
        // конфигу: высота эллипса зависит от длины подписи (см.
        // usecase_natural_size). Иначе длинные подписи наезжают друг на друга.
        let mut current_y = usecases_start_y;
        for (i, (name, alias)) in all_usecases.iter().enumerate() {
            let y = current_y;
            // Шаг в LTR-режиме СВОЙ. Общая константа 60 верна для
            // `usecase_basic` (там расхождение 1.0), но в LTR эталон даёт
            // шаг 72–76, что соответствует 34.6. Подстановка 34.6 в общую
            // константу ломала `usecase_basic` (1.0 -> 27.0).
            current_y += self.usecase_natural_size(name).1 + usecase_step;
            let _ = i;

            let (elem, bounds) = self.create_usecase_element(name, usecases_x, y);
            element_positions.insert(name.to_string(), bounds);
            if let Some(a) = alias {
                element_positions.insert(a.to_string(), bounds);
            }
            elements.push(elem);
        }

        // Размещаем актёров слева.
        //
        // Несколько актёров могут получить одинаковую Y (если связаны с
        // разными use case на одной высоте, либо оба не связаны ни с чем).
        // Раньше в этом случае они рисовались в одной точке и полностью
        // накладывались друг на друга. Теперь при совпадении Y актёр
        // сдвигается по X на ширину блока.
        let mut occupied: Vec<(f64, f64)> = Vec::new(); // (y, x)
        for (actor_index, actor) in diagram.actors.iter().enumerate() {
            // Вычисляем среднюю Y позицию use cases, с которыми связан актёр
            // Актёры ставятся СВЕРХУ, над всеми вариантами использования:
            // в эталоне usecase_basic актёр занимает y=6..64, а первый
            // эллипс начинается на y=141.8. Раньше актёр выравнивался по
            // средней Y связанных use case, из-за чего оказывался между
            // ними и диаграмма теряла вертикальный порядок.
            // В LTR актёр ЦЕНТРИРУЕТСЯ по связанным с ним use case.
            //
            // Измерено по эталону `UseCase: Простой`: `Customer` связан с
            // UC1/UC2/UC3, стоящими на cy 62, 136 и 208 (среднее 135.3), и
            // его голова оказывается на 106; `Manager` связан с UC4/UC5
            // (280 и 356, среднее 318) — голова на 288. Разница до среднего
            // в обоих случаях около 29, то есть актёр ставится ЦЕНТРОМ на
            // среднее своих use case.
            //
            // В режиме сверху-вниз актёры, наоборот, идут столбцом от верха:
            // это подтверждено эталоном `usecase_basic`, и там расхождение
            // всего 1.0 по высоте.
            let y = if is_left_to_right {
                let mut sum = 0.0;
                let mut count = 0.0;
                // Связи ссылаются на АЛИАС (`Customer`), а не на подпись
                // (`Покупатель`), поэтому сверяем оба.
                let alias = actor.alias.as_deref();
                let is_me = |name: &str| name == actor.name || Some(name) == alias;
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
                            sum += rect.y + rect.height / 2.0;
                            count += 1.0;
                        }
                    }
                }
                if count > 0.0 {
                    (sum / count - self.config.actor_height / 2.0).max(self.config.margin)
                } else {
                    system_y + actor_index as f64 * (self.config.actor_height + 10.0)
                }
            } else {
                self.config.margin + actor_index as f64 * (self.config.actor_height + 10.0)
            };

            // Ищем свободную позицию по X среди актёров с такой же Y
            let mut actor_x = if is_left_to_right {
                self.config.margin
            } else {
                actors_x
            };
            let tolerance = self.config.actor_height / 2.0;
            while occupied.iter().any(|(oy, ox)| {
                (oy - y).abs() < tolerance && (ox - actor_x).abs() < actor_total_width
            }) {
                actor_x += actor_total_width + self.config.horizontal_spacing;
            }
            occupied.push((y, actor_x));

            let (elem, bounds) = self.create_actor_element(&actor.name, actor_x, y);
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

    /// Создаёт элемент актёра (stick figure)
    fn create_actor_element(&self, name: &str, x: f64, y: f64) -> (LayoutElement, Rect) {
        let bounds = Rect::new(x, y, self.config.actor_width, self.config.actor_height);

        (
            LayoutElement {
                id: format!("actor_{}", name.replace(' ', "_")),
                bounds,
                text: None,
                properties: std::collections::HashMap::new(),
                element_type: ElementType::Actor {
                    label: name.to_string(),
                },
            },
            bounds,
        )
    }

    /// Создаёт элемент use case (эллипс)
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
        let bounds = Rect::new(x, y, width, height);

        (
            LayoutElement {
                id: format!("usecase_{}", name.replace(' ', "_")),
                bounds,
                text: None,
                properties: std::collections::HashMap::new(),
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

    /// Вычисляет точки соединения для связи
    fn calculate_connection_points(&self, from: &Rect, to: &Rect) -> (Point, Point) {
        let _from_center_x = from.x + from.width / 2.0;
        let from_center_y = from.y + from.height / 2.0;
        let _to_center_x = to.x + to.width / 2.0;
        let to_center_y = to.y + to.height / 2.0;

        // Для актёров (узкие) соединяем справа
        // Для эллипсов соединяем слева
        let start = if from.width < 50.0 {
            // Это актёр - соединяем справа
            Point::new(from.x + from.width, from_center_y)
        } else {
            // Это эллипс - соединяем слева
            Point::new(from.x, from_center_y)
        };

        let end = if to.width < 50.0 {
            // Это актёр - соединяем справа
            Point::new(to.x + to.width, to_center_y)
        } else {
            // Это эллипс - соединяем слева
            Point::new(to.x, to_center_y)
        };

        (start, end)
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
}
