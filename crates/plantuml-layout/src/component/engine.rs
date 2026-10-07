//! Component Diagram Layout Engine
//!
//! Алгоритм layout для диаграмм компонентов.

use std::collections::HashMap;

use plantuml_ast::component::{Component, ComponentDiagram, ComponentType, Connection};
use plantuml_model::{Point, Rect};

use super::config::ComponentLayoutConfig;

/// Базовая ширина компонента, измеренная по эталону PlantUML.
/// Место под заголовок контейнера при переходе между пакетами.
///
/// Измерено на сервере: `package` -> `package` даёт шаг 122 при 106
/// внутри пакета.
/// Базовый шаг между уровнями для компонентов ВНУТРИ пакетов.
///
/// Измерено на сервере: шаг 106 = 46 (высота блока) + 60. У компонентов
/// верхнего уровня шаг другой — 123 = 46 + 77 (эталон `component_basic`).
const COMPONENT_IN_PACKAGE_SPACING: f64 = 60.0;

/// Верхний отступ над первым уровнем, если он внутри пакета.
///
/// Измерено по эталону `Component: Простой`: подпись `Frontend` на y=21,
/// первый блок на 41 при `margin` 7.
const CONTAINER_TOP_EXTRA: f64 = 34.0;

const CONTAINER_HEADER_EXTRA: f64 = 16.0;

/// То же при переходе из `package` в `database`.
///
/// Измерено на сервере: `package` -> `database` даёт шаг 137.
const CONTAINER_DATABASE_EXTRA: f64 = 31.0;

const COMPONENT_BASE_WIDTH: f64 = 4.6;
/// Прибавка к ширине компонента на каждый символ подписи.
const COMPONENT_CHAR_WIDTH: f64 = 11.43;
/// Высота компонента по эталону.
const COMPONENT_HEIGHT: f64 = 46.297;
/// Насколько контейнер шире своего заголовка (измерено по эталону).
const PACKAGE_TITLE_EXTRA: f64 = 76.0;

/// Добавка к ширине артефакта (измерено по эталону deployment_basic).
const ARTIFACT_TEXT_PADDING: f64 = 30.0;

/// Высота артефакта (измерено по эталону).
const ARTIFACT_HEIGHT: f64 = 39.297;
use crate::{EdgeType, ElementType, LayoutElement, LayoutResult};

/// Layout engine для component diagrams
/// Добавка к ширине для плоских компонентов (эталон 13.81 при поле
/// рендерера 7).
const COMPONENT_RIGHT_EXTRA: f64 = 6.81;

/// Дополнительное правое поле для диаграмм с объёмными узлами.
///
/// Эталон: правый край узла на 25 от края холста при поле рендерера 7,
/// то есть 18 сверх общего margin.
const NODE_RIGHT_EXTRA: f64 = 18.0;

pub struct ComponentLayoutEngine {
    config: ComponentLayoutConfig,
}

impl ComponentLayoutEngine {
    /// Создаёт новый engine с конфигурацией по умолчанию
    pub fn new() -> Self {
        Self {
            config: ComponentLayoutConfig::default(),
        }
    }

    /// Создаёт engine с заданной конфигурацией
    pub fn with_config(config: ComponentLayoutConfig) -> Self {
        Self { config }
    }

    /// Выполняет layout диаграммы
    pub fn layout(&self, diagram: &ComponentDiagram) -> LayoutResult {
        let mut elements = Vec::new();
        let mut component_positions: HashMap<String, Rect> = HashMap::new();

        // Сначала располагаем компоненты в grid-layout
        let components: Vec<&Component> = diagram.components.iter().collect();
        // PlantUML размещает компоненты вертикально (в эталоне три
        // компонента стоят на x=7 друг под другом). Раньше использовалась
        // сетка по sqrt(n), дававшая 2-3 колонки и широкую низкую диаграмму.
        let num_cols = 1;

        for (i, comp) in components.iter().enumerate() {
            let row = i / num_cols;
            let col = i % num_cols;

            let x = self.config.margin
                + col as f64 * (self.config.component_width + self.config.horizontal_spacing);
            let y = self.config.margin
                + row as f64 * (self.config.component_height + self.config.vertical_spacing);

            let (elem, bounds) = self.create_component_element(comp, x, y);

            // Сохраняем позицию по имени и алиасу
            component_positions.insert(comp.name.clone(), bounds);
            if let Some(alias) = &comp.alias {
                component_positions.insert(alias.clone(), bounds);
            }

            elements.push(elem);
        }

        // Располагаем пакеты
        // Узлы начинаются сразу под компонентами.
        //
        // Раньше здесь безусловно резервировалась одна строка
        // (`components.len() / num_cols + 1`), даже когда компонентов нет
        // вовсе. Из-за этого диаграмма из одного узла сдвигалась вниз на
        // 123px, а холст начинался с y=123.3 вместо 7.
        let component_rows = components.len().div_ceil(num_cols.max(1));
        let mut package_y = self.config.margin
            + component_rows as f64 * (self.config.component_height + self.config.vertical_spacing);

        // Узлы ставятся с отступом node_margin, а не общим margin.
        let node_x = if diagram.packages.is_empty() {
            self.config.margin
        } else {
            self.config.node_margin
        };

        // ПЕРВЫЙ ПРОХОД: раскладываем узлы от общего начала.
        //
        // Раньше все узлы ставились по одному `node_x`, то есть
        // выравнивались РАМКИ. Эталон выравнивает ВЛОЖЕННЫЕ фигуры:
        // измерено на сервере по четырём вариантам диаграммы — центр
        // вложенной фигуры первого узла совпадает с центром второго
        // (97/97, 82/82, 214/214). Именно поэтому ребро в эталоне
        // вертикально: `M97,85.64 C97,112.56 97,156.7 97,185.1`.
        // ГЛОБАЛЬНАЯ РАСКЛАДКА ПО УРОВНЯМ СВЯЗЕЙ — но ТОЛЬКО когда пакеты
        // не являются узлами (`node`).
        //
        // Эталон `Component: Простой` выстраивает компоненты из РАЗНЫХ
        // пакетов в общую цепочку по связям: React App (y=41), Redux Store
        // (147), API Gateway (269), затем двое рядом — Auth Service (375,
        // x=22) и User Service (375, x=187), потом Users DB (511). Уровни
        // по longest-path совпадают с этими y.
        //
        // Но у `deployment_basic` пакеты — это `node`, и там раскладка
        // ПОПАКЕТНАЯ: попытка применить общий граф сломала его с 295 до 404
        // по ширине, хотя он был точен (0.0 x 2.0).
        let uses_nodes = diagram
            .packages
            .iter()
            .any(|pkg| pkg.package_type == plantuml_ast::component::PackageType::Node);

        let mut global_positions: HashMap<String, (f64, f64)> = HashMap::new();
        if !uses_nodes {
            let mut all_components: Vec<(
                &Component,
                Option<&plantuml_ast::component::ComponentPackage>,
            )> = Vec::new();
            for comp in &diagram.components {
                all_components.push((comp, None));
            }
            Self::collect_all_components(&diagram.packages, &mut all_components);

            let levels = Self::assign_component_levels(&all_components, &diagram.connections);
            let level_count = levels.values().copied().max().unwrap_or(0) + 1;
            let mut by_level: Vec<Vec<usize>> = vec![Vec::new(); level_count];
            for (i, (comp, _)) in all_components.iter().enumerate() {
                by_level[levels.get(&comp.name).copied().unwrap_or(0)].push(i);
            }

            // УРОВНИ ЦЕНТРИРУЮТСЯ по общей оси, а не прижимаются влево.
            //
            // Измерено по эталону `Component: Простой`: уровни 0, 1, 2 и 3
            // имеют центры 169, 169, 169 и 168.5 — то есть выровнены по
            // одной вертикали. Прежде каждый уровень начинался от
            // `margin`, из-за чего диаграмма выходила на 57 шире эталонной.
            let row_widths: Vec<f64> = by_level
                .iter()
                .map(|row| {
                    let mut probe: Vec<LayoutElement> = Vec::new();
                    let mut w = 0.0_f64;
                    for &i in row {
                        let (comp, _) = all_components[i];
                        let (_, b) = self.create_component_element(comp, 0.0, 0.0);
                        w += b.width + self.config.horizontal_spacing;
                        probe.clear();
                    }
                    (w - self.config.horizontal_spacing).max(0.0)
                })
                .collect();
            let widest_row = row_widths.iter().cloned().fold(0.0_f64, f64::max);

            // Верхний отступ: над первым уровнем тоже стоит ЗАГОЛОВОК
            // пакета. Эталон `Component: Простой`: подпись `Frontend` на
            // y=21, первый блок на 41 при margin 7 — то есть 34.
            let mut cursor_y = self.config.margin
                + if by_level
                    .first()
                    .and_then(|r| r.first())
                    .and_then(|&i| all_components[i].1)
                    .is_some()
                {
                    CONTAINER_TOP_EXTRA
                } else {
                    0.0
                };
            for (row_index, row) in by_level.iter().enumerate() {
                let mut cursor_x = self.config.margin + (widest_row - row_widths[row_index]) / 2.0;
                let mut row_height = self.component_natural_height();
                for &i in row {
                    let (comp, _) = all_components[i];
                    let (_, bounds) = self.create_component_element(comp, cursor_x, cursor_y);
                    global_positions.insert(comp.name.clone(), (cursor_x, cursor_y));
                    if let Some(alias) = &comp.alias {
                        global_positions.insert(alias.clone(), (cursor_x, cursor_y));
                    }
                    cursor_x += bounds.width + self.config.horizontal_spacing;
                    row_height = row_height.max(bounds.height);
                }
                // СМЕНА КОНТЕЙНЕРА ДОБАВЛЯЕТ МЕСТО ПОД ЕГО ЗАГОЛОВОК.
                //
                // Измерено на сервере:
                //   внутри пакета                       -> шаг 106
                //   package -> package                  -> шаг 122 (+16)
                //   package -> database                 -> шаг 137 (+31)
                // (база 106 = 46 высота блока + 60 отступ). Проверено на
                // playground `Backend` -> `PostgreSQL`, где шаг 136 —
                // совпадает с предсказанными 137 в пределах округления.
                // БАЗОВЫЙ ШАГ РАЗНЫЙ: у компонентов ВЕРХНЕГО уровня 77
                // (эталон `component_basic`: блоки на y=7 и y=130, шаг 123
                // = 46 + 77), у компонентов ВНУТРИ ПАКЕТОВ 60 (эталон
                // `Component: Простой`: шаг 106 = 46 + 60).
                let in_package = row.first().and_then(|&i| all_components[i].1).is_some();
                let base_spacing = if in_package {
                    COMPONENT_IN_PACKAGE_SPACING
                } else {
                    self.config.vertical_spacing
                };
                cursor_y += row_height + base_spacing;
                if let Some(next) = by_level.get(row_index + 1) {
                    // Важна СМЕНА контейнера, а не смена его типа:
                    // `Frontend` -> `Backend` — разные пакеты ОДНОГО типа,
                    // и добавка там тоже есть (замер: шаг 122).
                    let container_of =
                        |row: &Vec<usize>| row.first().and_then(|&i| all_components[i].1);
                    if let (Some(cur), Some(nxt)) = (container_of(row), container_of(next)) {
                        if cur.name != nxt.name {
                            cursor_y += if matches!(
                                nxt.package_type,
                                plantuml_ast::component::PackageType::Package
                            ) {
                                CONTAINER_HEADER_EXTRA
                            } else {
                                CONTAINER_DATABASE_EXTRA
                            };
                        }
                    }
                }
            }
        }

        let mut laid: Vec<(Vec<LayoutElement>, Rect, HashMap<String, Rect>)> = Vec::new();
        for pkg in &diagram.packages {
            let laid_pkg = self.layout_package(pkg, node_x, package_y, &global_positions);
            package_y = laid_pkg.1.y + laid_pkg.1.height + self.config.package_vertical_spacing;
            laid.push(laid_pkg);
        }

        // ВТОРОЙ ПРОХОД: сдвигаем узлы так, чтобы связанные фигуры встали
        // на одну вертикаль. Ограничение — не левее поля.
        let shifts = self.align_package_children(&laid, &diagram.connections, node_x);

        for (index, (pkg_elements, _bounds, inner_positions)) in laid.into_iter().enumerate() {
            let shift = shifts.get(index).copied().unwrap_or(0.0);

            for mut elem in pkg_elements {
                elem.bounds.x += shift;
                elements.push(elem);
            }

            // Добавляем позиции вложенных компонентов
            for (name, rect) in inner_positions {
                component_positions.insert(
                    name,
                    Rect::new(rect.x + shift, rect.y, rect.width, rect.height),
                );
            }
        }

        // Создаём связи
        for conn in &diagram.connections {
            if let Some(edge) = self.create_connection_element(conn, &component_positions) {
                elements.push(edge);
            }
        }

        // Вычисляем bounds
        let mut result = LayoutResult {
            elements,
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        };
        result.calculate_bounds();

        // Отступы.
        //
        // У объёмных узлов deployment поля БОЛЬШЕ, чем у плоских
        // компонентов. Измерено по эталонам, снятым с сервера:
        //   один узел:         слева 16, справа 25
        //   плоский компонент: слева  7, справа 13.8
        // Узлы размещаются с отступом `node_margin`, а начало координат
        // остаётся на `margin`, поэтому левое поле складывается из обоих.
        let has_nodes = !diagram.packages.is_empty();
        if has_nodes {
            result.bounds.x = self.config.margin;
            let content_right = result.bounds.x + result.bounds.width;
            result.bounds.width = (content_right - result.bounds.x) + NODE_RIGHT_EXTRA;
        } else {
            // Плоские компоненты: добавки к габаритам измерены отдельно по
            // осям. Эталон component_basic имеет поля слева 7, справа 13.81,
            // сверху 7, снизу 29.42; поле рендерера даёт по 7, остальное
            // добавляет движок.
            result.bounds.width += COMPONENT_RIGHT_EXTRA;
        }
        // Нижний отступ РАЗНЫЙ для двух случаев.
        //
        // У компонентов ВЕРХНЕГО уровня (эталон `component_basic`, без
        // пакетов) верны два отступа: 323 при содержимом 7..309.
        // У компонентов В ПАКЕТАХ (эталон `Component: Простой`) нижний
        // отступ вдвое меньше: содержимое кончается на 559, холст 598 при
        // верхнем крае 41, то есть снизу 41, а не 48.
        //
        // Прежняя константа `margin * 2` верна только для первого случая;
        // проверка показала, что её замена на `margin` ломает
        // `component_basic` (2.0 -> 9.0) и `deployment_basic` (2.0 -> 5.0).
        // Уточнение: у `deployment_basic` пакеты — это `node`, и там
        // удвоенный отступ ВЕРЕН (замена ломала его: 2.0 -> 5.0). Малый
        // отступ нужен только для обычных `package`/`database`.
        let packages_are_nodes = diagram
            .packages
            .iter()
            .any(|pkg| pkg.package_type == plantuml_ast::component::PackageType::Node);
        let bottom_margin = if diagram.packages.is_empty() || packages_are_nodes {
            self.config.margin * 2.0
        } else {
            self.config.margin
        };
        result.bounds.height += bottom_margin;

        result
    }

    /// Создаёт элемент компонента
    /// Создаёт элемент компонента и возвращает его **фактические** границы.
    ///
    /// Раньше возвращался прямоугольник фиксированного размера
    /// (`component_width × component_height`, 140×60) независимо от того,
    /// что реально нарисовано. Для `Interface` (эллипс 20×20), `Actor`
    /// (0.6 ширины) и `Cloud` (1.2 ширины) это неверно: связи привязывались
    /// к несуществующему блоку и визуально «висели» мимо фигуры.
    fn create_component_element(&self, comp: &Component, x: f64, y: f64) -> (LayoutElement, Rect) {
        let elem = match comp.component_type {
            ComponentType::Database => self.create_database_element(&comp.name, x, y),
            ComponentType::Cloud => self.create_cloud_element(&comp.name, x, y),
            ComponentType::Interface => self.create_interface_element(&comp.name, x, y),
            ComponentType::Queue => self.create_queue_element(&comp.name, x, y),
            ComponentType::Node => self.create_node_element(&comp.name, x, y),
            ComponentType::Folder => self.create_folder_element(&comp.name, x, y),
            ComponentType::Actor => self.create_actor_element(&comp.name, x, y),
            // Артефакт имеет свои пропорции: измерено по эталону
            // deployment_basic — «app.jar» при тексте 49.027 занимает
            // 79.027x39.297, то есть текст плюс 30.
            ComponentType::Artifact => self.create_artifact_element(&comp.name, x, y),
            _ => self.create_standard_component_element(comp, x, y),
        };

        // Границы берём у фактически созданного элемента, а не назначаем
        // по конфигу: иначе связи не попадают в фигуру.
        let bounds = elem.bounds;

        (elem, bounds)
    }

    /// Создаёт стандартный компонент
    fn create_standard_component_element(&self, comp: &Component, x: f64, y: f64) -> LayoutElement {
        let name = &comp.name;
        let mut properties = std::collections::HashMap::new();

        // Цвет элемента, если он задан (например, слоем Archimate).
        // Раньше поле `color` не читалось вовсе, поэтому archimate-элементы
        // всех слоёв выглядели одинаково белыми.
        if let Some(color) = &comp.color {
            properties.insert("fill".to_string(), color.to_css());
        }

        // Размер компонента зависит от длины подписи: измерено по эталону
        // («Веб-интерфейс», 13 символов — 153.189x46.297; «Сервис API», 10 —
        // 118.9x46.297). Отсюда ширина ≈ 4.6 + 11.43 * n, высота 46.297.
        // Раньше размер был фиксированным (40x40), из-за чего диаграмма
        // расходилась с эталоном.
        let chars = name.chars().count() as f64;
        let width =
            (COMPONENT_BASE_WIDTH + COMPONENT_CHAR_WIDTH * chars).max(self.config.component_width);
        let height = self.config.component_height.max(COMPONENT_HEIGHT);

        LayoutElement {
            id: format!("component_{}", name.replace(' ', "_")),
            bounds: Rect::new(x, y, width, height),
            text: None,
            properties,
            element_type: ElementType::Rectangle {
                label: name.to_string(),
                corner_radius: self.config.corner_radius,
            },
        }
    }

    /// Высота компонента (по эталону).
    fn component_natural_height(&self) -> f64 {
        self.config.component_height.max(COMPONENT_HEIGHT)
    }

    /// Создаёт артефакт.
    ///
    /// Измерено по эталону deployment_basic: «app.jar» при тексте 49.027
    /// занимает 79.027x39.297, то есть ширина — текст плюс 30, высота 39.297.
    fn create_artifact_element(&self, name: &str, x: f64, y: f64) -> LayoutElement {
        let width = self.config.text.width(name, self.config.font_size) + ARTIFACT_TEXT_PADDING;

        // Заливка и толщина рамки — как в эталоне `deployment_basic`:
        // `fill="#F1F1F1"`, `stroke-width:0.5`.
        LayoutElement {
            id: format!("artifact_{}", name.replace(' ', "_")),
            bounds: Rect::new(x, y, width, ARTIFACT_HEIGHT),
            text: None,
            properties: [
                ("fill".to_string(), "#F1F1F1".to_string()),
                ("stroke".to_string(), "#181818".to_string()),
                ("stroke-width".to_string(), "0.5".to_string()),
            ]
            .into_iter()
            .collect(),
            element_type: ElementType::Rectangle {
                label: name.to_string(),
                corner_radius: 2.5,
            },
        }
    }

    /// Иконка документа в правом верхнем углу артефакта.
    ///
    /// Геометрия снята с эталона `deployment_basic`: рамка артефакта
    /// 79.027x39.297 в точке (57.49, 46), иконка 12x14 в точке
    /// (119.517, 51) и загнутый уголок 6x6.
    fn create_artifact_icon(&self, bounds: Rect) -> Vec<LayoutElement> {
        /// Отступ иконки от правого края рамки.
        const INSET_RIGHT: f64 = 17.0;
        /// Отступ иконки от верхнего края рамки.
        const INSET_TOP: f64 = 5.0;
        /// Ширина и высота иконки.
        const WIDTH: f64 = 12.0;
        const HEIGHT: f64 = 14.0;
        /// Сторона загнутого уголка.
        const FOLD: f64 = 6.0;

        let x = bounds.x + bounds.width - INSET_RIGHT;
        let y = bounds.y + INSET_TOP;

        let outline = LayoutElement {
            id: "artifact_icon".to_string(),
            bounds: Rect::new(x, y, WIDTH, HEIGHT),
            text: None,
            properties: [
                ("fill".to_string(), "#F1F1F1".to_string()),
                ("stroke".to_string(), "#181818".to_string()),
                ("stroke-width".to_string(), "0.5".to_string()),
            ]
            .into_iter()
            .collect(),
            // Вершины — в ЛОКАЛЬНЫХ координатах `bounds`: рендерер
            // прибавляет к ним начало рамки.
            element_type: ElementType::Polygon {
                points: vec![
                    Point::new(0.0, 0.0),
                    Point::new(0.0, HEIGHT),
                    Point::new(WIDTH, HEIGHT),
                    Point::new(WIDTH, FOLD),
                    Point::new(WIDTH - FOLD, 0.0),
                ],
                label: None,
                font_size: 0.0,
            },
        };

        // Загнутый уголок: вертикаль и горизонталь от точки сгиба.
        // Двумя отрезками, потому что `ElementType::Path` — единичный
        // вариант без геометрии, а ломаная здесь из двух звеньев.
        let mut fold = Vec::new();
        for (index, (from, to)) in [
            (
                Point::new(x + WIDTH - FOLD, y),
                Point::new(x + WIDTH - FOLD, y + FOLD),
            ),
            (
                Point::new(x + WIDTH - FOLD, y + FOLD),
                Point::new(x + WIDTH, y + FOLD),
            ),
        ]
        .into_iter()
        .enumerate()
        {
            fold.push(LayoutElement {
                id: format!("artifact_icon_fold_{index}"),
                bounds: Rect::new(from.x, from.y, (to.x - from.x).abs(), (to.y - from.y).abs()),
                text: None,
                properties: [
                    ("fill".to_string(), "none".to_string()),
                    ("stroke".to_string(), "#181818".to_string()),
                    ("stroke-width".to_string(), "0.5".to_string()),
                ]
                .into_iter()
                .collect(),
                element_type: ElementType::Edge {
                    points: vec![from, to],
                    label: None,
                    arrow_start: false,
                    arrow_end: false,
                    dashed: false,
                    edge_type: EdgeType::Link,
                    from_cardinality: None,
                    to_cardinality: None,
                },
            });
        }

        let mut result = vec![outline];
        result.extend(fold);
        result
    }

    /// Создаёт элемент базы данных (цилиндр)
    fn create_database_element(&self, name: &str, x: f64, y: f64) -> LayoutElement {
        let chars = name.chars().count() as f64;
        let width =
            (COMPONENT_BASE_WIDTH + COMPONENT_CHAR_WIDTH * chars).max(self.config.component_width);

        LayoutElement {
            id: format!("database_{}", name.replace(' ', "_")),
            bounds: Rect::new(x, y, width, COMPONENT_HEIGHT),
            text: None,
            properties: std::collections::HashMap::new(),
            // База данных рисуется цилиндром, а не эмодзи: символ 🛢
            // зависит от наличия шрифта в системе и визуально не совпадает
            // с оригиналом PlantUML
            element_type: ElementType::Database {
                label: name.to_string(),
            },
        }
    }

    /// Создаёт элемент облака
    fn create_cloud_element(&self, name: &str, x: f64, y: f64) -> LayoutElement {
        LayoutElement {
            id: format!("cloud_{}", name.replace(' ', "_")),
            bounds: Rect::new(
                x,
                y,
                self.config.component_width * 1.2,
                self.config.component_height,
            ),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Rectangle {
                label: name.to_string(),
                corner_radius: self.config.component_height / 2.0,
            },
        }
    }

    /// Создаёт элемент интерфейса (кружок)
    fn create_interface_element(&self, name: &str, x: f64, y: f64) -> LayoutElement {
        let r = self.config.interface_radius;
        LayoutElement {
            id: format!("interface_{}", name.replace(' ', "_")),
            bounds: Rect::new(x, y, r * 2.0, r * 2.0),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Ellipse {
                label: Some(name.to_string()),
            },
        }
    }

    /// Создаёт элемент очереди
    fn create_queue_element(&self, name: &str, x: f64, y: f64) -> LayoutElement {
        LayoutElement {
            id: format!("queue_{}", name.replace(' ', "_")),
            bounds: Rect::new(
                x,
                y,
                self.config.component_width,
                self.config.component_height,
            ),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Rectangle {
                label: name.to_string(),
                corner_radius: self.config.component_height / 4.0,
            },
        }
    }

    /// Создаёт элемент node
    fn create_node_element(&self, name: &str, x: f64, y: f64) -> LayoutElement {
        LayoutElement {
            id: format!("node_{}", name.replace(' ', "_")),
            bounds: Rect::new(
                x,
                y,
                self.config.component_width,
                self.config.component_height,
            ),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Rectangle {
                label: name.to_string(),
                corner_radius: 0.0, // Node — с углами
            },
        }
    }

    /// Создаёт элемент folder
    fn create_folder_element(&self, name: &str, x: f64, y: f64) -> LayoutElement {
        LayoutElement {
            id: format!("folder_{}", name.replace(' ', "_")),
            bounds: Rect::new(
                x,
                y,
                self.config.component_width,
                self.config.component_height,
            ),
            text: None,
            properties: std::collections::HashMap::new(),
            element_type: ElementType::Rectangle {
                label: name.to_string(),
                corner_radius: self.config.corner_radius,
            },
        }
    }

    /// Создаёт элемент actor
    fn create_actor_element(&self, name: &str, x: f64, y: f64) -> LayoutElement {
        LayoutElement {
            id: format!("actor_{}", name.replace(' ', "_")),
            bounds: Rect::new(
                x,
                y,
                self.config.component_width * 0.6,
                self.config.component_height,
            ),
            text: None,
            properties: std::collections::HashMap::new(),
            // Актёр — стик-фигура, а не эмодзи
            element_type: ElementType::Actor {
                label: name.to_string(),
            },
        }
    }

    /// Располагает пакет и возвращает элементы, bounds и позиции вложенных компонентов
    /// Сдвиги узлов, выравнивающие связанные вложенные фигуры по вертикали.
    ///
    /// Первый узел остаётся на месте, остальные сдвигаются так, чтобы
    /// центр фигуры-источника совпал с центром фигуры-приёмника. Если
    /// после этого узел вылезает левее поля, весь набор сдвигается вправо.
    fn align_package_children(
        &self,
        laid: &[(Vec<LayoutElement>, Rect, HashMap<String, Rect>)],
        connections: &[plantuml_ast::component::Connection],
        node_x: f64,
    ) -> Vec<f64> {
        let mut shifts = vec![0.0_f64; laid.len()];
        let mut known = vec![false; laid.len()];
        if let Some(first) = known.first_mut() {
            *first = true;
        }

        // Какому узлу принадлежит вложенная фигура.
        let mut owner: HashMap<&str, usize> = HashMap::new();
        for (index, (_, _, inner)) in laid.iter().enumerate() {
            for name in inner.keys() {
                owner.insert(name.as_str(), index);
            }
        }

        let centre = |index: usize, name: &str| -> Option<f64> {
            laid[index]
                .2
                .get(name)
                .map(|rect| rect.x + rect.width / 2.0)
        };

        // Проходов столько же, сколько узлов: сдвиг распространяется по цепочке.
        for _ in 0..laid.len() {
            for conn in connections {
                let (Some(&source), Some(&target)) =
                    (owner.get(conn.from.as_str()), owner.get(conn.to.as_str()))
                else {
                    continue;
                };
                if source == target {
                    continue;
                }

                match (known[source], known[target]) {
                    (true, false) => {
                        if let (Some(from), Some(to)) =
                            (centre(source, &conn.from), centre(target, &conn.to))
                        {
                            shifts[target] = from + shifts[source] - to;
                            known[target] = true;
                        }
                    }
                    (false, true) => {
                        if let (Some(from), Some(to)) =
                            (centre(source, &conn.from), centre(target, &conn.to))
                        {
                            shifts[source] = to + shifts[target] - from;
                            known[source] = true;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Ни один узел не должен уехать левее поля.
        let leftmost = laid
            .iter()
            .enumerate()
            .map(|(index, (_, bounds, _))| bounds.x + shifts[index])
            .fold(f64::INFINITY, f64::min);
        if leftmost < node_x && leftmost.is_finite() {
            let correction = node_x - leftmost;
            for shift in &mut shifts {
                *shift += correction;
            }
        }

        shifts
    }

    /// Собирает компоненты ВСЕХ пакетов (рекурсивно) в один список.
    fn collect_all_components<'a>(
        packages: &'a [plantuml_ast::component::ComponentPackage],
        out: &mut Vec<(
            &'a Component,
            Option<&'a plantuml_ast::component::ComponentPackage>,
        )>,
    ) {
        for pkg in packages {
            for comp in &pkg.components {
                out.push((comp, Some(pkg)));
            }
            Self::collect_all_components(&pkg.packages, out);
        }
    }

    /// Присваивает уровням компоненты по длине longest-path.
    fn assign_component_levels(
        all: &[(
            &Component,
            Option<&plantuml_ast::component::ComponentPackage>,
        )],
        connections: &[plantuml_ast::component::Connection],
    ) -> HashMap<String, usize> {
        let mut level: HashMap<String, usize> = HashMap::new();
        for (comp, _) in all {
            level.entry(comp.name.clone()).or_insert(0);
        }
        for _ in 0..=all.len() {
            let mut changed = false;
            for conn in connections {
                if let (Some(from), Some(to)) =
                    (level.get(&conn.from).copied(), level.get(&conn.to).copied())
                {
                    if to < from + 1 {
                        level.insert(conn.to.clone(), from + 1);
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        level
    }

    fn layout_package(
        &self,
        pkg: &plantuml_ast::component::ComponentPackage,
        x: f64,
        y: f64,
        global: &HashMap<String, (f64, f64)>,
    ) -> (Vec<LayoutElement>, Rect, HashMap<String, Rect>) {
        let mut elements = Vec::new();
        let mut positions = HashMap::new();

        // Компоненты внутри контейнера располагаются вертикально: в эталоне
        // `node` содержит один элемент под заголовком. Раньше использовалась
        // сетка по sqrt(n), а размеры брались из конфига (40x40) вместо
        // контентных, из-за чего контейнер расходился по габаритам.
        let mut max_row = 0;
        let mut inner_width: f64 = 0.0;
        let mut inner_height: f64 = 0.0;

        for (i, comp) in pkg.components.iter().enumerate() {
            max_row = i;

            let (comp_x, comp_y) = match global.get(&comp.name) {
                Some(&(gx, gy)) => (gx, gy),
                None => (
                    x + self.config.package_padding,
                    y + self.config.package_header_height
                        + self.config.package_padding
                        + i as f64
                            * (self.component_natural_height() + self.config.vertical_spacing),
                ),
            };
            let _ = i;

            let (elem, bounds) = self.create_component_element(comp, comp_x, comp_y);
            inner_width = inner_width.max(bounds.width);
            inner_height = comp_y - y + bounds.height
                - self.config.package_header_height
                - self.config.package_padding;
            positions.insert(comp.name.clone(), bounds);
            if let Some(alias) = &comp.alias {
                positions.insert(alias.clone(), bounds);
            }
            elements.push(elem);

            // У артефакта в правом верхнем углу — иконка документа.
            // Измерено по эталону `deployment_basic`: рамка 57.49..136.52,
            // иконка 119.517..131.517 по x и 51..65 по y, то есть 12x14
            // на 17 от правого края и 5 от верхнего.
            if comp.component_type == ComponentType::Artifact {
                elements.extend(self.create_artifact_icon(bounds));
            }
        }

        // Вычисляем размер пакета по фактическому содержимому
        let _ = max_row;

        // Ширина контейнера: PlantUML растягивает его под заголовок.
        // Измерено по эталону deployment_basic: контейнер «Сервер приложений»
        // имеет ширину 241 при заголовке 165.136, «Сервер БД» — 162 при
        // 86.181, то есть ширина = заголовок + 76. Берётся максимум из
        // содержимого и заголовка.
        // Заголовок узла рисуется ЖИРНЫМ, а он шире обычного примерно
        // на 8.3%. Без этого узел выходил уже эталонного.
        let title_width = self
            .config
            .text
            .width_bold(&pkg.name, self.config.font_size);
        let pkg_width = (inner_width + self.config.package_padding * 2.0)
            .max(title_width + PACKAGE_TITLE_EXTRA);
        let pkg_height =
            inner_height + self.config.package_header_height + self.config.package_padding * 2.0;

        let pkg_bounds = Rect::new(x, y, pkg_width.max(150.0), pkg_height.max(100.0));

        // Создаём элемент пакета (group)
        let pkg_elem = LayoutElement {
            id: format!("package_{}", pkg.name.replace(' ', "_")),
            bounds: pkg_bounds,
            text: None,
            // Заголовок узла PlantUML рисует ПОЛУЖИРНЫМ: в эталоне
            // deployment_basic обе подписи («Сервер приложений» и
            // «Сервер БД») имеют font-weight="700".
            properties: [("font-weight".to_string(), "700".to_string())]
                .into_iter()
                .collect(),
            element_type: ElementType::Group {
                label: Some(pkg.name.clone()),
                children: Vec::new(),
            },
        };

        // Вставляем пакет первым (под компонентами)
        elements.insert(0, pkg_elem);

        (elements, pkg_bounds, positions)
    }

    /// Создаёт элемент связи
    fn create_connection_element(
        &self,
        conn: &Connection,
        positions: &HashMap<String, Rect>,
    ) -> Option<LayoutElement> {
        let from_rect = positions.get(&conn.from)?;
        let to_rect = positions.get(&conn.to)?;

        let (start, end) = self.calculate_connection_points(from_rect, to_rect);

        let min_x = start.x.min(end.x);
        let min_y = start.y.min(end.y);
        let max_x = start.x.max(end.x);
        let max_y = start.y.max(end.y);

        Some(LayoutElement {
            id: format!(
                "conn_{}_{}",
                conn.from.replace(' ', "_"),
                conn.to.replace(' ', "_")
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
                label: conn.label.clone(),
                arrow_start: false,
                arrow_end: true,
                dashed: conn.dashed,
                edge_type: EdgeType::Association,
                from_cardinality: None,
                to_cardinality: None,
            },
        })
    }

    /// Вычисляет точки соединения для связи
    fn calculate_connection_points(&self, from: &Rect, to: &Rect) -> (Point, Point) {
        let from_center_x = from.x + from.width / 2.0;
        let from_center_y = from.y + from.height / 2.0;
        let to_center_x = to.x + to.width / 2.0;
        let to_center_y = to.y + to.height / 2.0;

        let dx = to_center_x - from_center_x;
        let dy = to_center_y - from_center_y;

        let start;
        let end;

        if dy.abs() > dx.abs() {
            if dy > 0.0 {
                start = Point::new(from_center_x, from.y + from.height);
                end = Point::new(to_center_x, to.y);
            } else {
                start = Point::new(from_center_x, from.y);
                end = Point::new(to_center_x, to.y + to.height);
            }
        } else {
            if dx > 0.0 {
                start = Point::new(from.x + from.width, from_center_y);
                end = Point::new(to.x, to_center_y);
            } else {
                start = Point::new(from.x, from_center_y);
                end = Point::new(to.x + to.width, to_center_y);
            }
        }

        (start, end)
    }
}

impl Default for ComponentLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_simple_components() {
        let mut diagram = ComponentDiagram::new();
        diagram.components.push(Component::new("API"));
        diagram.components.push(Component::database("MySQL"));
        diagram.connections.push(Connection::new("API", "MySQL"));

        let engine = ComponentLayoutEngine::new();
        let result = engine.layout(&diagram);

        // Должны быть: 2 компонента + 1 связь
        assert!(result.elements.len() >= 3);
    }

    #[test]
    fn test_layout_with_package() {
        use plantuml_ast::component::ComponentPackage;

        let mut diagram = ComponentDiagram::new();
        let mut pkg = ComponentPackage::new("Backend");
        pkg.components.push(Component::new("API"));
        pkg.components.push(Component::new("Worker"));
        diagram.packages.push(pkg);

        let engine = ComponentLayoutEngine::new();
        let result = engine.layout(&diagram);

        // Должен быть пакет + 2 компонента
        assert!(result.elements.len() >= 3);
    }

    #[test]
    fn test_layout_various_types() {
        let mut diagram = ComponentDiagram::new();
        diagram.components.push(Component::new("App"));
        diagram.components.push(Component::database("PostgreSQL"));
        diagram.components.push(Component::cloud("AWS"));
        diagram.components.push(Component::node("Server"));

        let engine = ComponentLayoutEngine::new();
        let result = engine.layout(&diagram);

        assert_eq!(result.elements.len(), 4);
    }
}
