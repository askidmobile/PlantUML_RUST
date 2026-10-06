//! SVG рендерер

use svg::node::element::{Definitions, Group, Marker, Path, Polygon, Rectangle, Text};
use svg::Document;

use crate::{
    ClassMember, ClassifierKind, EdgeType, ElementType, FragmentSection, LayoutElement,
    LayoutResult, MemberVisibility, Point, Rect, RenderOptions, Renderer, ZLayer,
};
use plantuml_themes::{Color, Theme};

/// Цвет рамки фрагмента (alt/opt/loop) в эталоне PlantUML.
const FRAGMENT_BORDER: &str = "#000";

/// Цвет заливки заголовка фрагмента в эталоне PlantUML.
const FRAGMENT_HEADER_FILL: &str = "#EEE";

/// Смещение кружка граничного элемента вправо от линии жизни.
const BOUNDARY_CIRCLE_SHIFT: f64 = 8.5;

/// Вершина «вороньей лапки» от конца линии (эталон `er_basic`: 129.24 → 139.24).
const CROW_FOOT_APEX: f64 = 10.0;

/// Длина лучей «лапки» (эталон: 139.24 → 147.24).
const CROW_FOOT_LENGTH: f64 = 18.0;

/// Разброс крайних лучей «лапки» (эталон: ±6).
const CROW_FOOT_SPREAD: f64 = 6.0;

/// Положение первой перекладины «один» от конца линии (эталон: 1).
const CROW_FOOT_BAR_FIRST: f64 = 1.0;

/// Шаг между перекладинами «один» (эталон: 94.79 и 91.79).
const CROW_FOOT_BAR_STEP: f64 = 3.0;

/// Половина длины перекладины «один» (эталон: 55.75…63.75).
const CROW_FOOT_BAR_HALF: f64 = 4.0;

/// Расстояние от конца линии до центра кружка «ноль».
const CROW_FOOT_CIRCLE: f64 = 4.0;

/// Радиус кружка «ноль» (эталон: 4).
const CROW_FOOT_CIRCLE_RADIUS: f64 = 4.0;

/// Является ли кардинальность записью ER-нотации.
///
/// Такие PlantUML рисует фигурами; числовые («1», «*») остаются текстом.
fn is_crow_foot(symbol: &str) -> bool {
    !symbol.is_empty() && symbol.chars().all(|c| matches!(c, '|' | 'o' | '{' | '}'))
}

/// Отступ типа фрагмента от левого края рамки.
///
/// Эталон `sequence_fragments`: «alt» на x = 31.95 при рамке от 16.955.
const FRAGMENT_TYPE_INSET: f64 = 15.0;

/// Базис типа фрагмента от верха рамки (эталон: 95.50 при 82.43).
const FRAGMENT_TYPE_BASELINE: f64 = 13.07;

/// Кегль типа фрагмента (эталон: `font-size="13"`).
const FRAGMENT_LABEL_FONT_SIZE: f64 = 13.0;

/// Кегль условия секции (эталон: `font-size="11"`).
const FRAGMENT_CONDITION_FONT_SIZE: f64 = 11.0;

/// Отступ условия от правого края рамки (эталон: 179.355 - 163.36).
const FRAGMENT_CONDITION_INSET: f64 = 16.0;

/// Базис условия от верха рамки (эталон: 94.64 при 82.43).
const FRAGMENT_CONDITION_BASELINE: f64 = 12.21;

/// Насколько разделитель секции выше её первого сообщения.
const FRAGMENT_SEPARATOR_OFFSET: f64 = 38.94;

/// Смещение подписи фигурного участника влево от его оси.
const PARTICIPANT_ICON_LABEL_SHIFT: f64 = 3.0;

/// Отступ подписи сообщения от левого конца стрелки.
const MESSAGE_LABEL_INSET: f64 = 7.0;

/// Длина наконечника стрелки: на столько подпись отодвигается от
/// левого конца, если наконечник стоит именно слева.
const MESSAGE_ARROW_HEAD: f64 = 10.0;

/// Насколько подпись сообщения поднята над линией.
const MESSAGE_LABEL_RISE: f64 = 5.0;

/// Цвет начального и конечного узлов UML (PlantUML пишет `#222`, а не
/// цвет границы темы: эталоны `activity_basic` и `state_simple` дают
/// `fill="#222" stroke="#222"`).
const UML_NODE_COLOR: &str = "#222";

/// Радиус внутреннего круга конечного узла UML относительно внешнего.
///
/// Эталон `activity_basic`: внешний круг `rx=11`, внутренний `rx=6`.
const UML_FINAL_INNER_RATIO: f64 = 6.0 / 11.0;

/// SVG рендерер
pub struct SvgRenderer {
    options: RenderOptions,
}

impl SvgRenderer {
    /// Создаёт новый рендерер
    pub fn new() -> Self {
        Self {
            options: RenderOptions::default(),
        }
    }

    /// Создаёт рендерер с опциями
    pub fn with_options(options: RenderOptions) -> Self {
        Self { options }
    }

    /// Рендерит в строку
    pub fn render_to_string(&self, layout: &LayoutResult, theme: &Theme) -> String {
        self.render(layout, theme)
    }

    /// Рисует растровый спрайт набором прямоугольников.
    ///
    /// PlantUML встраивает PNG в base64, но палитра фиксирована: 16
    /// оттенков серого с убывающей прозрачностью. Поэтому кодер PNG не
    /// нужен — каждая hex-цифра тела становится прямоугольником.
    /// Значения палитры сняты с эталонного PNG, который отдаёт сервер.
    fn render_sprite(
        &self,
        bounds: &Rect,
        rows: &[String],
        pixel_size: f64,
        group: Group,
    ) -> Group {
        self.render_sprite_data(bounds, rows, pixel_size, group)
    }

    /// Рисует векторный спрайт (`sprite имя <svg ...>...</svg>`).
    ///
    /// PlantUML поддерживает лишь небольшое подмножество SVG, поэтому
    /// тело переносится в вывод как есть, а система координат `viewBox`
    /// приводится к прямоугольнику элемента через `transform`:
    /// масштаб по обеим осям и сдвиг на минимум `viewBox`.
    ///
    /// Вложенные `transform` (например `translate(-19.992 -120.11)` в
    /// спрайтах Archimate) применяются средствами SVG, поэтому
    /// разбирать команды пути не требуется.
    fn render_vector_sprite(
        &self,
        bounds: &Rect,
        sprite: &plantuml_layout::SvgSprite,
        theme: &Theme,
        mut group: Group,
    ) -> Group {
        let (min_x, min_y, box_width, box_height) = view_box(&sprite.attrs);

        if box_width <= 0.0 || box_height <= 0.0 {
            return group;
        }

        // Масштаб: из системы координат `viewBox` в размеры прямоугольника.
        let scale_x = bounds.width / box_width;
        let scale_y = bounds.height / box_height;
        // Единый масштаб, чтобы не искажать пропорции; PlantUML вписывает
        // спрайт целиком.
        let scale = scale_x.min(scale_y);
        let offset_x = bounds.x - min_x * scale;
        let offset_y = bounds.y - min_y * scale;

        let body = svg::node::element::Group::new()
            .set(
                "transform",
                format!("translate({offset_x:.4} {offset_y:.4}) scale({scale:.6})"),
            )
            .set("fill", theme.text_color.to_css());

        // Тело переносится как «сырая» разметка: подмножество SVG мало,
        // а полноценный разбор путей не нужен.
        let raw = svg::node::Text::new(sprite.body.clone());
        group = group.add(body.add(raw));

        group
    }

    /// Рисует растр спрайта прямоугольниками палитры.
    fn render_sprite_data(
        &self,
        bounds: &Rect,
        rows: &[String],
        pixel_size: f64,
        mut group: Group,
    ) -> Group {
        // Пиксели одного цвета объединяются в ПРЯМОУГОЛЬНЫЕ БЛОКИ.
        //
        // Спрайт 64x63 содержит 4032 пикселя. Если рисовать каждый
        // отдельным `<rect>`, вывод раздувается до десятков мегабайт
        // (замерено 10.5 МБ на диаграмму). Сначала пробеги по строке
        // (10.5 МБ -> 1.3 МБ), затем слияние одинаковых пробегов по
        // вертикали — картинка при этом не меняется.
        let grid: Vec<Vec<char>> = rows.iter().map(|row| row.chars().collect()).collect();
        let height = grid.len();
        // Прогоны по каждой строке: (начало, длина, цифра).
        let runs: Vec<Vec<(usize, usize, char)>> = grid
            .iter()
            .map(|row| {
                let mut out = Vec::new();
                let mut col = 0;
                while col < row.len() {
                    let mut run = 1;
                    while col + run < row.len() && row[col + run] == row[col] {
                        run += 1;
                    }
                    out.push((col, run, row[col]));
                    col += run;
                }
                out
            })
            .collect();

        // Сливаем совпадающие прогоны соседних строк по вертикали.
        let mut used: Vec<Vec<bool>> = runs.iter().map(|r| vec![false; r.len()]).collect();
        for row_index in 0..height {
            for run_index in 0..runs[row_index].len() {
                if used[row_index][run_index] {
                    continue;
                }

                let (col, width, digit) = runs[row_index][run_index];
                let Some((color, alpha)) = sprite_color(digit) else {
                    used[row_index][run_index] = true;
                    continue;
                };
                used[row_index][run_index] = true;

                // Цифра `0` в палитре полностью прозрачна — не рисуем.
                if alpha == 0 {
                    continue;
                }

                // Считаем, сколько строк подряд совпадает этот прогон.
                let mut span = 1;
                'outer: while row_index + span < height {
                    for (next_index, (next_col, next_width, next_digit)) in
                        runs[row_index + span].iter().enumerate()
                    {
                        if used[row_index + span][next_index] {
                            continue;
                        }
                        if *next_col == col && *next_width == width && *next_digit == digit {
                            used[row_index + span][next_index] = true;
                            span += 1;
                            continue 'outer;
                        }
                    }
                    break;
                }

                let rect = Rectangle::new()
                    .set("x", bounds.x + col as f64 * pixel_size)
                    .set("y", bounds.y + row_index as f64 * pixel_size)
                    .set("width", width as f64 * pixel_size)
                    .set("height", span as f64 * pixel_size)
                    .set("fill", color)
                    .set("fill-opacity", fmt(alpha as f64 / 255.0));

                group = group.add(rect);
            }
        }

        group
    }

    /// Ширина текста при заданном размере шрифта.
    ///
    /// Нужна для раскладки частей подписи вокруг вставки спрайта.
    fn measure_text(&self, text: &str, font_size: f64) -> f64 {
        plantuml_layout::text::TextMeasurer::default().width(text, font_size)
    }

    /// Поля страницы `(слева, сверху, справа, снизу)` для текущего типа.
    ///
    /// Значения измерены по эталонам PlantUML. Тип приходит в
    /// `RenderOptions::diagram_type` теми же именами, что PlantUML пишет
    /// в атрибут `data-diagram-type`.
    fn page_margins(&self) -> (f64, f64, f64, f64) {
        /// Поле по умолчанию: подходит class, state, sequence и другим,
        /// где контент начинается с отступа 7.
        const DEFAULT: (f64, f64, f64, f64) = (7.0, 7.0, 7.0, 7.0);
        /// GANTT: таблица идёт от самого края, справа остаётся место под
        /// подписи, снизу — небольшой отступ.
        // Нижний запас пересчитан после уточнения таблицы ширин: холст
        // выходил 131 вместо эталонных 130.
        const GANTT: (f64, f64, f64, f64) = (0.0, 0.0, 20.8, 1.59);

        /// WBS: движок сам добавляет `padding = 10`, поэтому рендереру
        /// нужно ещё 10, чтобы суммарное поле совпало с эталонным (20).
        const WBS: (f64, f64, f64, f64) = (10.0, 10.0, 10.0, 10.0);

        /// CLASS: поля асимметричны — слева и сверху 7, справа и снизу
        /// около 14.3 (измерено по class_inheritance, er_basic и
        /// object_basic: 14.28/14.41, 14.49/14.11, 13.90/13.41).
        const CLASS: (f64, f64, f64, f64) = (7.0, 7.0, 14.28, 14.41);

        /// JSON и YAML: таблица с отступом 10 со всех сторон и небольшим
        /// запасом справа и снизу (измерено 10/10/11.11/11.81 и
        /// 10/10/11.55/11.11).
        // Правый запас пересчитан после того, как колонка ключей стала
        // считаться по ЖИРНЫМ метрикам: ширина содержимого выросла на 6.36
        // и совпала с эталонной (131.45), а прежний запас давал холст
        // на 4 шире эталонного.
        const TABLE: (f64, f64, f64, f64) = (10.0, 10.0, 7.11, 11.81);

        /// MINDMAP: движок сам добавляет `padding = 10` со всех сторон,
        /// поэтому рендерер дополняет до эталонных 10/20/20.14/20.81.
        const MINDMAP: (f64, f64, f64, f64) = (0.0, 10.0, 10.14, 10.81);

        /// NWDIAG: слева 5 (контент начинается с padding движка),
        /// справа 6.27; сверху и снизу остаются поля по умолчанию.
        const NWDIAG: (f64, f64, f64, f64) = (0.0, 7.0, 6.27, 7.0);

        /// SEQUENCE: поля больше общих. Измерено по эталону
        /// `sequence_simple`: полоса участника стоит на y=10, а её низ
        /// на 206.125 при общей высоте 218, то есть поля 10 сверху и
        /// 11.875 снизу. Прежний профиль брал общие 7 — отсюда
        /// недобор высоты на 8 у всех sequence-диаграмм.
        /// ЛЕВОЕ и ВЕРХНЕЕ поля равны внутреннему отступу движка (10):
        /// только тогда `viewBox` начинается с нуля и рамка первого
        /// участника стоит на x=10, как в эталоне. Прежние 7 сдвигали
        /// содержимое на 3px влево — на картинке это заметно.
        const SEQUENCE: (f64, f64, f64, f64) = (10.0, 10.0, 10.879, 11.875);

        /// ACTIVITY: измерено по эталону `activity_basic`.
        ///
        /// Контент эталона занимает 179.91 x 287.91 (от 16,15 до
        /// 195.91,302.91), холст 215x322. Отсюда поля 16 и 19.09 по
        /// горизонтали, 15 и 19.09 по вертикали.
        ///
        /// ВАЖНО: предыдущий замер этих полей был НЕВЕРЕН — он не
        /// учитывал размеры эллипсов (у них нет width/height, только
        /// `cx/cy/rx/ry`), поэтому высота контента выходила заниженной
        /// на 21, а поля — завышенными.
        ///
        /// Левые и верхние поля задают СМЕЩЕНИЕ контента на странице
        /// (эталон начинает его в 16 и 15), правые и нижние добавлены к
        /// размеру: движок ACTIVITY уже прибавляет свои 16 к `bounds`,
        /// поэтому здесь нужен только остаток до 215x322.
        const ACTIVITY: (f64, f64, f64, f64) = (0.0, 0.0, 19.41, 18.14);

        // ВАЖНО: тип приходит из `data-diagram-type`, и он НЕ совпадает
        // с внутренним `DiagramType`:
        //   ER и OBJECT помечаются как CLASS (их считает движок классов);
        //   Deployment, Component и Usecase — все как DESCRIPTION,
        //   поэтому отдельного профиля на каждый из них быть не может.
        match self.options.diagram_type.as_deref() {
            Some("CLASS") => CLASS,
            Some("NWDIAG") => NWDIAG,
            Some("MINDMAP") => MINDMAP,
            Some("JSON") | Some("YAML") => TABLE,
            Some("GANTT") => GANTT,
            Some("WBS") => WBS,
            Some("SEQUENCE") => SEQUENCE,
            Some("ACTIVITY") => ACTIVITY,
            // STATE: измерено по эталону `state A` — холст 72x71 при
            // рамке 50x50 в точке (7, 7). Движок добавляет свои 7,
            // здесь остаётся довести до эталонных размеров.
            // Нижний запас пересчитан после уточнения таблицы ширин:
            // холст выходил 279 вместо эталонных 278.
            Some("STATE") => (0.0, 0.0, 8.0, 5.70),
            // SALT: измерено по эталону `salt_basic` — холст 113x71.
            // Начало координат остаётся нулевым: содержимое таблицы уже
            // начинается в (6, 7.84), как в эталоне. Правый запас
            // пересчитан после того, как движок стал считать ширину
            // контейнера по правому краю нарисованных элементов
            // (габарит вырос со 100.07 до 109.57).
            Some("SALT") => (0.0, 0.0, 3.43, 2.59),
            // TIMING: измерено по эталону `timing_basic` — холст 229x175.
            //
            // Снизу запас больше: подписи времени описываются
            // прямоугольником высотой в кегль НАД базисом, а у эталона
            // холст учитывает ещё и вынос строки под базисом (около 17).
            // Правый запас уточнён после пересчёта таблицы ширин
            // символов: холст стал на 1 уже эталонного.
            Some("TIMING") => (7.0, 7.0, 13.0, 29.0),
            _ => DEFAULT,
        }
    }

    /// Создаёт SVG документ
    /// PlantUML стиль: прозрачный/белый фон БЕЗ рамки вокруг диаграммы
    fn create_document(&self, layout: &LayoutResult, theme: &Theme) -> Document {
        let bounds = &layout.bounds;

        // Поля вокруг диаграммы зависят от типа: у PlantUML они разные.
        // Измерено по эталонам, снятым с сервера:
        //   class — слева 7, справа 14.28
        //   gantt — слева 0, справа 20.8, сверху 0, снизу 2.45
        // Раньше поле было фиксированным (7 со всех сторон), из-за чего
        // одни диаграммы выходили шире эталона, другие уже.
        let (left, top, right, bottom) = self.page_margins();
        let margin_h = left + right;
        let margin_v = top + bottom;

        let width = (bounds.width + margin_h) * self.options.scale;
        let height = (bounds.height + margin_v) * self.options.scale;

        // Набор атрибутов повторяет PlantUML: он важен для потребителей,
        // разбирающих вывод (contentStyleType), и для корректного
        // масштабирования (preserveAspectRatio, zoomAndPan).
        let doc = Document::new()
            .set("xmlns", "http://www.w3.org/2000/svg")
            .set("xmlns:xlink", "http://www.w3.org/1999/xlink")
            .set("version", "1.1")
            .set("width", format!("{:.0}px", width))
            .set("height", format!("{:.0}px", height))
            .set(
                "viewBox",
                (
                    bounds.x - left,
                    bounds.y - top,
                    bounds.width + margin_h,
                    bounds.height + margin_v,
                ),
            )
            .set("zoomAndPan", "magnify")
            .set("preserveAspectRatio", "none")
            .set("contentStyleType", "text/css")
            // PlantUML дублирует размеры ещё и в style, а также пишет
            // единицы измерения (`306px`) в width/height. Это важно для
            // потребителей, читающих размеры из style.
            .set(
                "style",
                format!("width:{width:.0}px;height:{height:.0}px;background:#FFFFFF;"),
            );

        // PlantUML помечает корневой `<svg>` типом диаграммы: потребители
        // вывода (редакторы, конвертеры) определяют по нему разметку.
        let mut doc = if let Some(kind) = &self.options.diagram_type {
            doc.set("data-diagram-type", kind.as_str())
        } else {
            doc
        };

        // PlantUML по умолчанию НЕ добавляет фон и рамку вокруг диаграммы
        // Фон добавляется только если явно указан через skinparam backgroundColor
        if let Some(bg) = &self.options.background_color {
            let bg_rect = Rectangle::new()
                .set("x", bounds.x - left)
                .set("y", bounds.y - top)
                .set("width", bounds.width + margin_h)
                .set("height", bounds.height + margin_v)
                .set("fill", bg.as_str())
                .set("stroke", "none");
            doc = doc.add(bg_rect);
        }

        // Определения (маркеры стрелок)
        let defs = self.create_definitions(theme);
        doc = doc.add(defs);

        doc
    }

    /// Создаёт определения (маркеры, градиенты)
    /// PlantUML стиль: разные стрелки для разных типов связей
    fn create_definitions(&self, theme: &Theme) -> Definitions {
        let arrow_color = theme.arrow_color.to_css();

        // Маркер стрелки в стиле PlantUML (ромб с вырезом) - для ассоциаций и сообщений
        let arrow_marker = Marker::new()
            .set("id", "arrow")
            .set("markerWidth", 10)
            .set("markerHeight", 8)
            .set("refX", 10)
            .set("refY", 4)
            .set("orient", "auto")
            .set("markerUnits", "userSpaceOnUse")
            .add(
                Path::new()
                    // PlantUML style: ромб с вырезом.
                    //
                    // Обводка не декоративна: PlantUML рисует наконечник
                    // полигоном со `stroke-width:1`, поэтому его видимый
                    // размер на пиксель больше самой геометрии. Без обводки
                    // наши стрелки выглядели тоньше эталонных.
                    .set("d", "M0,0 L10,4 L0,8 L4,4 Z")
                    .set("fill", arrow_color.as_str())
                    .set("stroke", arrow_color.as_str())
                    .set("stroke-width", 1)
                    .set("stroke-linejoin", "miter"),
            );

        // Открытый маркер стрелки (для async сообщений)
        let open_arrow_marker = Marker::new()
            .set("id", "arrow-open")
            .set("markerWidth", 10)
            .set("markerHeight", 8)
            .set("refX", 10)
            .set("refY", 4)
            .set("orient", "auto")
            .set("markerUnits", "userSpaceOnUse")
            .add(
                Path::new()
                    .set("d", "M0,0 L10,4 L0,8")
                    .set("fill", "none")
                    .set("stroke", arrow_color.as_str())
                    .set("stroke-width", 1),
            );

        // Маркер наследования (пустой треугольник) - для --|> и ..|>
        // PlantUML использует polygon fill="none" для inheritance
        let inheritance_marker = Marker::new()
            .set("id", "inheritance")
            .set("markerWidth", 20)
            .set("markerHeight", 20)
            .set("refX", 20)
            .set("refY", 10)
            .set("orient", "auto")
            .set("markerUnits", "userSpaceOnUse")
            .add(
                Path::new()
                    // Треугольник: верх, кончик, низ
                    .set("d", "M0,0 L20,10 L0,20 Z")
                    // PlantUML рисует полый треугольник с `fill="none"`,
                    // а не белой заливкой. Проверено по эталону
                    // class_inheritance: `<polygon ... fill="none">`.
                    // Разница видна, когда линия проходит через фигуру:
                    // белая заливка перекрывает её, `none` — нет.
                    .set("fill", "none")
                    .set("stroke", arrow_color.as_str())
                    .set("stroke-width", 1),
            );

        // Маркер композиции (закрашенный ромб) - для *--
        let composition_marker = Marker::new()
            .set("id", "composition")
            .set("markerWidth", 12)
            .set("markerHeight", 12)
            .set("refX", 0)
            .set("refY", 6)
            .set("orient", "auto")
            .set("markerUnits", "userSpaceOnUse")
            .add(
                Path::new()
                    // Ромб: лево, верх, право, низ
                    .set("d", "M0,6 L6,0 L12,6 L6,12 Z")
                    .set("fill", arrow_color.as_str()),
            );

        // Маркер агрегации (пустой ромб) - для o--
        let aggregation_marker = Marker::new()
            .set("id", "aggregation")
            .set("markerWidth", 12)
            .set("markerHeight", 12)
            .set("refX", 0)
            .set("refY", 6)
            .set("orient", "auto")
            .set("markerUnits", "userSpaceOnUse")
            .add(
                Path::new()
                    .set("d", "M0,6 L6,0 L12,6 L6,12 Z")
                    .set("fill", theme.background_color.to_css()) // белый внутри
                    .set("stroke", arrow_color.as_str())
                    .set("stroke-width", 1),
            );

        Definitions::new()
            .add(arrow_marker)
            .add(open_arrow_marker)
            .add(inheritance_marker)
            .add(composition_marker)
            .add(aggregation_marker)
    }

    /// Рендерит элемент
    fn render_element(&self, element: &LayoutElement, theme: &Theme) -> Group {
        self.render_element_with_id(element, theme, &element.id)
    }

    /// Рендерит элемент с заданным идентификатором.
    ///
    /// Отдельный метод нужен, чтобы `render` мог подставить уникальный id,
    /// не изменяя сам элемент layout.
    fn render_element_with_id(&self, element: &LayoutElement, theme: &Theme, id: &str) -> Group {
        let mut group = Group::new().set("id", id);

        // Цвета из свойств приводим к форме PlantUML ОДИН раз здесь:
        // сервер записывает их сокращённо (`#000000` → `#000`), а движки
        // раскладки задают полную запись. Нормализация в одной точке
        // избавляет от правок в каждом движке.
        let normalized;
        let element = if let Some(value) = normalize_element_colors(element) {
            normalized = value;
            &normalized
        } else {
            element
        };

        match &element.element_type {
            ElementType::Note { label, fold } => {
                group = self.render_note(
                    &element.bounds,
                    label,
                    *fold,
                    theme,
                    group,
                    &element.properties,
                );
            }
            ElementType::Rectangle {
                label,
                corner_radius,
            } => {
                group = self.render_rectangle(
                    &element.bounds,
                    label,
                    *corner_radius,
                    theme,
                    group,
                    &element.properties,
                );
            }
            ElementType::Sprite {
                rows,
                pixel_size,
                svg,
            } => {
                group = match svg {
                    Some(vector) => {
                        self.render_vector_sprite(&element.bounds, vector, theme, group)
                    }
                    None => self.render_sprite(&element.bounds, rows, *pixel_size, group),
                };
            }
            ElementType::Ellipse { label } => {
                group = self.render_ellipse(
                    &element.bounds,
                    label.as_deref(),
                    theme,
                    group,
                    &element.properties,
                );
            }
            ElementType::InitialState => {
                group = self.render_initial_state(&element.bounds, theme, group);
            }
            ElementType::FinalState => {
                group = self.render_final_state(&element.bounds, theme, group);
            }
            ElementType::State { name, description } => {
                group = self.render_uml_state(
                    &element.bounds,
                    name,
                    description.as_deref(),
                    theme,
                    group,
                );
            }
            ElementType::CompositeState {
                name,
                header_height,
            } => {
                group = self.render_composite_state(
                    &element.bounds,
                    name,
                    *header_height,
                    theme,
                    group,
                );
            }
            // Фигуры участников sequence. Нижние блоки (`footer_`) PlantUML
            // рисует зеркально: подпись сверху, фигура снизу.
            ElementType::Actor { label } => {
                let footer = id.starts_with("footer_");
                group = self.render_actor(&element.bounds, label, theme, group, footer);
            }
            ElementType::Database { label } => {
                let footer = id.starts_with("footer_");
                group = self.render_database(&element.bounds, label, theme, group, footer);
            }
            ElementType::Boundary { label } => {
                let footer = id.starts_with("footer_");
                group = self.render_boundary(&element.bounds, label, theme, group, footer);
            }
            ElementType::Control { label } => {
                let footer = id.starts_with("footer_");
                group = self.render_control(&element.bounds, label, theme, group, footer);
            }
            ElementType::Entity { label } => {
                let footer = id.starts_with("footer_");
                group = self.render_entity(&element.bounds, label, theme, group, footer);
            }
            ElementType::System { title } => {
                group = self.render_system(&element.bounds, title, theme, group);
            }
            ElementType::Polygon {
                points,
                label,
                font_size,
            } => {
                group = self.render_polygon(
                    &element.bounds,
                    points,
                    label.as_deref(),
                    *font_size,
                    theme,
                    group,
                    &element.properties,
                );
            }
            ElementType::Edge {
                points,
                label,
                arrow_start,
                arrow_end,
                dashed,
                edge_type,
                from_cardinality,
                to_cardinality,
            } => {
                let autonumber = element.properties.get("autonumber").map(|s| s.as_str());
                // Цвет и толщину линия может нести свойствами: PlantUML
                // рисует переходы состояний timing зелёным (#006400)
                // толщиной 2, деления оси — толщиной 2.
                let style = EdgeStyle {
                    color: element.properties.get("stroke").map(String::as_str),
                    width: element
                        .properties
                        .get("stroke-width")
                        .and_then(|value| value.parse().ok()),
                };
                group = self.render_edge(
                    points,
                    label.as_deref(),
                    autonumber,
                    *arrow_start,
                    *arrow_end,
                    *dashed,
                    *edge_type,
                    from_cardinality.as_deref(),
                    to_cardinality.as_deref(),
                    style,
                    theme,
                    group,
                );
            }
            ElementType::Text { text, font_size } => {
                let bold = element.properties.contains_key("font-weight");
                let fill = element
                    .properties
                    .get("text-fill")
                    .cloned()
                    .unwrap_or_else(|| theme.text_color.to_css());
                // Таблицы JSON и YAML кладут строку по своему базису,
                // а не по низу ячейки.
                let baseline = element
                    .properties
                    .get("baseline")
                    .and_then(|value| value.parse::<f64>().ok());
                group = self.render_text_weighted(
                    &element.bounds,
                    text,
                    *font_size,
                    TextStyle {
                        bold,
                        fill: &fill,
                        baseline,
                    },
                    theme,
                    group,
                );
            }
            ElementType::Group { label, children } => {
                let bold = element.properties.contains_key("font-weight");
                group = self.render_group(
                    &element.bounds,
                    label.as_deref(),
                    children,
                    bold,
                    theme,
                    group,
                );
            }
            ElementType::Fragment {
                fragment_type,
                sections,
            } => {
                let bold = element.properties.contains_key("font-weight");
                group = self.render_fragment(
                    &element.bounds,
                    fragment_type,
                    sections,
                    bold,
                    theme,
                    group,
                );
            }
            ElementType::Activation => {
                group = self.render_activation(&element.bounds, theme, group);
            }
            ElementType::RoundedRectangle => {
                // Рендерим как прямоугольник со скруглёнными углами.
                // Радиус берётся из темы (`skinparam roundcorner`), а не из
                // литерала: иначе настройка темы игнорировалась.
                let label = element.text.as_deref().unwrap_or("");
                group = self.render_rectangle(
                    &element.bounds,
                    label,
                    theme.corner_radius,
                    theme,
                    group,
                    &element.properties,
                );
            }
            ElementType::Path => {
                // Рендерим SVG path (для кривых Безье)
                if let Some(path_data) = element.properties.get("path") {
                    // Цвет, толщину и заливку путь может нести свойствами:
                    // так рисуется незамкнутый контур последнего состояния
                    // в timing (эталон `timing_basic`: `stroke:#006400;
                    // stroke-width:1.5` при заливке `#E2E2F0`).
                    let default_stroke = theme.node_border.to_css();
                    let path = svg::node::element::Path::new()
                        .set("d", path_data.as_str())
                        .set(
                            "fill",
                            element
                                .properties
                                .get("fill")
                                .map(String::as_str)
                                .unwrap_or("none"),
                        )
                        .set(
                            "stroke",
                            element
                                .properties
                                .get("stroke")
                                .map(String::as_str)
                                .unwrap_or(default_stroke.as_str()),
                        )
                        .set(
                            "stroke-width",
                            element
                                .properties
                                .get("stroke-width")
                                .map(String::as_str)
                                .unwrap_or("1"),
                        );
                    group = group.add(path);
                }
            }
            ElementType::ClassBox {
                classifier_type,
                name,
                stereotype,
                fields,
                methods,
            } => {
                let sprite_table = sprite_table(&element.properties);
                group = self.render_class_box(
                    &element.bounds,
                    *classifier_type,
                    name,
                    stereotype.as_deref(),
                    fields,
                    methods,
                    &sprite_table,
                    theme,
                    group,
                );
            }
            ElementType::ParticipantBox => {
                // Рендерим box для группировки участников
                let title = element.text.as_deref();
                let color = element.properties.get("color").map(|s| s.as_str());
                group = self.render_participant_box(&element.bounds, title, color, theme, group);
            }
        }

        group
    }

    /// Рисует многоугольник с необязательной подписью в центре.
    ///
    /// Так PlantUML изображает условие ветвления activity (шестиугольник)
    /// и точку слияния ветвей (ромб). Вершины приходят в локальных
    /// координатах `bounds`, поэтому движку раскладки достаточно задать
    /// прямоугольник фигуры.
    #[allow(clippy::too_many_arguments)]
    fn render_polygon(
        &self,
        bounds: &Rect,
        points: &[Point],
        label: Option<&str>,
        font_size: f64,
        theme: &Theme,
        mut group: Group,
        properties: &std::collections::HashMap<String, String>,
    ) -> Group {
        if points.len() < 3 {
            return group;
        }

        let fill = properties
            .get("fill")
            .cloned()
            .unwrap_or_else(|| theme.node_background.to_css());
        let stroke = properties
            .get("stroke")
            .cloned()
            .unwrap_or_else(|| theme.node_border.to_css());
        let stroke_width = properties
            .get("stroke-width")
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(theme.line_width * 0.5);

        let points_attr = points
            .iter()
            .map(|p| format!("{},{}", bounds.x + p.x, bounds.y + p.y))
            .collect::<Vec<_>>()
            .join(" ");

        group = group.add(
            Polygon::new()
                .set("points", points_attr)
                .set("fill", fill)
                .set("stroke", stroke)
                .set("stroke-width", stroke_width),
        );

        if let Some(text_content) = label.filter(|value| !value.is_empty()) {
            let label_color = properties
                .get("text-fill")
                .cloned()
                .unwrap_or_else(|| theme.text_color.to_css());

            group = group.add(
                Text::new(text_content)
                    .set("x", bounds.x + bounds.width / 2.0)
                    .set("y", bounds.y + bounds.height / 2.0)
                    .set("text-anchor", "middle")
                    .set("dominant-baseline", "middle")
                    .set("font-family", theme.font_family.as_str())
                    .set("font-size", font_size)
                    .set("fill", label_color),
            );
        }

        group
    }

    /// Рендерит прямоугольник
    fn render_rectangle(
        &self,
        bounds: &Rect,
        label: &str,
        corner_radius: f64,
        theme: &Theme,
        mut group: Group,
        properties: &std::collections::HashMap<String, String>,
    ) -> Group {
        // Получаем цвет заливки из properties или используем тему
        let default_fill = theme.node_background.to_css();
        let fill_color = properties
            .get("fill")
            .map(|s| s.as_str())
            .unwrap_or(default_fill.as_str());

        // Получаем прозрачность из properties
        let opacity = properties.get("opacity").map(|s| s.as_str());

        // Радиус скругления может быть задан явно в properties: так делает
        // таблица JSON/YAML (rx = 5). Раньше свойство не читалось, и таблица
        // рисовалась с радиусом темы.
        let corner_radius = properties
            .get("rx")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(corner_radius);

        // Толщина границы берётся из темы (`skinparam linetype`), значение
        // по умолчанию 0.5 совпадает с PlantUML для участников. Свойство
        // `stroke-width` перекрывает её: так рисуются кнопки salt —
        // в эталоне `salt_basic` у них обводка 2.5.
        let stroke_width = properties
            .get("stroke-width")
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(theme.line_width * 0.5);

        let mut rect = Rectangle::new()
            .set("x", bounds.x)
            .set("y", bounds.y)
            .set("width", bounds.width)
            .set("height", bounds.height)
            .set("rx", corner_radius)
            .set("ry", corner_radius)
            .set("fill", fill_color)
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", stroke_width);

        if let Some(op) = opacity {
            rect = rect.set("fill-opacity", op);
        }

        group = group.add(rect);

        // Тень: `skinparam shadowing true`. PlantUML рисует смещённый
        // прямоугольник под фигурой.
        if theme.shadow {
            group = group.add(
                Rectangle::new()
                    .set("x", bounds.x + SHADOW_OFFSET)
                    .set("y", bounds.y + SHADOW_OFFSET)
                    .set("width", bounds.width)
                    .set("height", bounds.height)
                    .set("rx", corner_radius)
                    .set("ry", corner_radius)
                    .set("fill", Color::new("#000000").to_css())
                    .set("fill-opacity", 0.2)
                    .set("stroke", "none"),
            );
        }

        // Пустая метка — это рамка без подписи: некоторые движки раскладки
        // помечают так фигуры, у которых текст рисуется отдельным
        // элементом (объекты, дорожки WBS). PlantUML в этом случае `<text>`
        // не пишет вовсе, а мы добавляли пустой текстовый узел.
        if label.is_empty() {
            return group;
        }

        // Текст по центру
        // Размер шрифта подписи.
        //
        // `ElementType::Rectangle` не несёт размер, поэтому используем
        // свойство `font-size`, если layout его проставил. Иначе берём
        // размер темы: так ведут себя все типы, кроме тех, у которых
        // подписи меньше (WBS и salt используют 12 при теме 14).
        let label_font_size = properties
            .get("font-size")
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(theme.font_size);
        // Цвет подписи тоже может быть переопределён: подписи timing
        // нарисованы цветом #333, тогда как тема даёт #000.
        let label_color = properties
            .get("text-fill")
            .cloned()
            .unwrap_or_else(|| theme.text_color.to_css());

        // Подпись может быть выровнена по ЛЕВОМУ краю с отступом.
        //
        // Так рисуются примечания: PlantUML ставит их текст с отступом
        // 6 от левого края рамки, а не по центру. Без этого длинный
        // текст примечания выходил за рамку.
        let left_aligned = properties
            .get("text-align")
            .map(|value| value == "left")
            .unwrap_or(false);
        let text_inset = properties
            .get("text-inset")
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(0.0);

        let (text_x, text_anchor) = if left_aligned {
            (bounds.x + text_inset, "start")
        } else {
            (bounds.x + bounds.width / 2.0, "middle")
        };

        let mut text = svg::node::element::Text::new(label)
            .set("x", text_x)
            .set("y", bounds.y + bounds.height / 2.0)
            .set("text-anchor", text_anchor)
            .set("dominant-baseline", "middle")
            .set("font-family", theme.font_family.as_str())
            .set("font-size", label_font_size)
            .set("fill", label_color.clone());

        // Рукописный стиль: `skinparam handwritten true`. PlantUML рисует
        // текст слегка наклонным.
        if theme.handwritten {
            text = text.set("font-style", "italic");
        }

        group.add(text)
    }

    /// Рисует примечание: прямоугольник с ЗАГНУТЫМ верхним правым углом.
    ///
    /// PlantUML рисует примечание полигоном со срезом угла и отдельным
    /// треугольником сгиба, а не простым прямоугольником. Измерено по
    /// эталону `sequence_notes`: правый край 204.987, срез начинается
    /// на 194.987 (размер среза 10).
    fn render_note(
        &self,
        bounds: &Rect,
        label: &str,
        fold: f64,
        theme: &Theme,
        mut group: Group,
        properties: &std::collections::HashMap<String, String>,
    ) -> Group {
        let fill = properties
            .get("fill")
            .cloned()
            .unwrap_or_else(|| theme.node_background.to_css());
        let stroke = properties
            .get("stroke")
            .cloned()
            .unwrap_or_else(|| theme.node_border.to_css());

        let x = bounds.x;
        let y = bounds.y;
        let right = bounds.x + bounds.width;
        let bottom = bounds.y + bounds.height;
        let fold_x = right - fold;
        let fold_y = y + fold;

        let body = format!(
            "M{x},{y} L{x},{bottom} L{right},{bottom} L{right},{fold_y} L{fold_x},{y} L{x},{y}"
        );
        let corner = format!("M{fold_x},{y} L{fold_x},{fold_y} L{right},{fold_y}");

        group = group.add(
            svg::node::element::Path::new()
                .set("d", body)
                .set("fill", fill)
                .set("stroke", stroke.clone())
                .set("stroke-width", theme.line_width * 0.5),
        );
        group = group.add(
            svg::node::element::Path::new()
                .set("d", corner)
                .set("fill", "none")
                .set("stroke", stroke)
                .set("stroke-width", theme.line_width * 0.5),
        );

        let label_font_size = properties
            .get("font-size")
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(theme.font_size);
        let label_color = properties
            .get("text-fill")
            .cloned()
            .unwrap_or_else(|| theme.text_color.to_css());
        let text_inset = properties
            .get("text-inset")
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(0.0);

        let text = svg::node::element::Text::new(label)
            .set("x", bounds.x + text_inset)
            .set("y", bounds.y + bounds.height / 2.0)
            .set("text-anchor", "start")
            .set("dominant-baseline", "middle")
            .set("font-family", theme.font_family.as_str())
            .set("font-size", label_font_size)
            .set("fill", label_color);

        group.add(text)
    }

    /// Рендерит эллипс
    fn render_ellipse(
        &self,
        bounds: &Rect,
        label: Option<&str>,
        theme: &Theme,
        mut group: Group,
        properties: &std::collections::HashMap<String, String>,
    ) -> Group {
        let cx = bounds.x + bounds.width / 2.0;
        let cy = bounds.y + bounds.height / 2.0;
        let rx = bounds.width / 2.0;
        let ry = bounds.height / 2.0;

        // Цвета могут быть переопределены: кружок обязательного атрибута
        // в ER-диаграмме — ЧЁРНЫЙ, тогда как тема даёт светлый фон.
        let fill = properties
            .get("fill")
            .cloned()
            .unwrap_or_else(|| theme.node_background.to_css());
        let stroke = properties
            .get("stroke")
            .cloned()
            .unwrap_or_else(|| theme.node_border.to_css());

        let ellipse = svg::node::element::Ellipse::new()
            .set("cx", cx)
            .set("cy", cy)
            .set("rx", rx)
            .set("ry", ry)
            .set("fill", fill)
            .set("stroke", stroke)
            .set("stroke-width", 1);

        group = group.add(ellipse);

        if let Some(label) = label {
            let text = svg::node::element::Text::new(label)
                .set("x", cx)
                .set("y", cy)
                .set("text-anchor", "middle")
                .set("dominant-baseline", "middle")
                .set("font-family", theme.font_family.as_str())
                .set("font-size", theme.font_size)
                .set("fill", theme.text_color.to_css());

            group = group.add(text);
        }

        group
    }

    /// Рендерит UML Initial State (чёрный заполненный круг)
    fn render_initial_state(&self, bounds: &Rect, _theme: &Theme, group: Group) -> Group {
        let cx = bounds.x + bounds.width / 2.0;
        let cy = bounds.y + bounds.height / 2.0;
        let r = bounds.width.min(bounds.height) / 2.0;

        // Заполненный чёрный круг (UML standard)
        let circle = svg::node::element::Ellipse::new()
            .set("cx", cx)
            .set("cy", cy)
            .set("rx", r)
            .set("ry", r)
            .set("fill", UML_NODE_COLOR)
            .set("stroke", UML_NODE_COLOR)
            .set("stroke-width", 1);

        group.add(circle)
    }

    /// Рендерит UML Final State (bullseye: внешний круг + внутренний заполненный круг)
    fn render_final_state(&self, bounds: &Rect, _theme: &Theme, mut group: Group) -> Group {
        let cx = bounds.x + bounds.width / 2.0;
        let cy = bounds.y + bounds.height / 2.0;
        let outer_r = bounds.width.min(bounds.height) / 2.0;
        // Внутренний круг — 6/11 внешнего (эталон `activity_basic`).
        let inner_r = outer_r * UML_FINAL_INNER_RATIO;

        // Внешний круг (пустой, с обводкой)
        let outer_circle = svg::node::element::Ellipse::new()
            .set("cx", cx)
            .set("cy", cy)
            .set("rx", outer_r)
            .set("ry", outer_r)
            .set("fill", "none")
            .set("stroke", UML_NODE_COLOR)
            .set("stroke-width", 1);

        group = group.add(outer_circle);

        // Внутренний круг (заполненный чёрный)
        let inner_circle = svg::node::element::Ellipse::new()
            .set("cx", cx)
            .set("cy", cy)
            .set("rx", inner_r)
            .set("ry", inner_r)
            .set("fill", UML_NODE_COLOR)
            .set("stroke", "none");

        group.add(inner_circle)
    }

    /// Рендерит UML State (скруглённый прямоугольник с разделителем и названием)
    fn render_uml_state(
        &self,
        bounds: &Rect,
        name: &str,
        description: Option<&str>,
        theme: &Theme,
        mut group: Group,
    ) -> Group {
        // Параметры по эталону PlantUML: rx/ry = 12.5, толщина границы 0.5,
        // разделитель на 26.3px от верха, имя без жирного начертания.
        // Раньше было rx=10, толщина 1, заголовок 25 и жирное имя.
        let corner_radius = 12.5;
        let header_height = STATE_HEADER_HEIGHT;

        // 1. Основной прямоугольник со скруглёнными углами.
        // Заливка в эталоне — #F1F1F1, а не цвет фона узлов темы.
        let rect = Rectangle::new()
            .set("x", bounds.x)
            .set("y", bounds.y)
            .set("width", bounds.width)
            .set("height", bounds.height)
            .set("rx", corner_radius)
            .set("ry", corner_radius)
            .set("fill", CLASS_BODY_FILL)
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 0.5);

        group = group.add(rect);

        // 2. Название состояния (центрировано сверху)
        let name_y = bounds.y + header_height / 2.0 + 5.0;
        let name_text = svg::node::element::Text::new(name)
            .set("x", bounds.x + bounds.width / 2.0)
            .set("y", name_y)
            .set("text-anchor", "middle")
            .set("dominant-baseline", "middle")
            .set("font-family", theme.font_family.as_str())
            .set("font-size", theme.font_size)
            .set("fill", theme.text_color.to_css());

        group = group.add(name_text);

        // 3. Горизонтальный разделитель (UML style)
        let separator_y = bounds.y + header_height;
        let separator = svg::node::element::Line::new()
            .set("x1", bounds.x)
            .set("y1", separator_y)
            .set("x2", bounds.x + bounds.width)
            .set("y2", separator_y)
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 0.5);

        group = group.add(separator);

        // 4. Описание (entry/exit/do actions) если есть
        if let Some(desc) = description {
            let desc_y = separator_y + 15.0;
            let desc_text = svg::node::element::Text::new(desc)
                .set("x", bounds.x + 5.0)
                .set("y", desc_y)
                .set("font-family", theme.font_family.as_str())
                .set("font-size", theme.font_size - 2.0)
                .set("fill", theme.text_color.to_css());

            group = group.add(desc_text);
        }

        group
    }

    /// Рендерит UML Composite State (контейнер с заголовком и вложенными состояниями)
    fn render_composite_state(
        &self,
        bounds: &Rect,
        name: &str,
        header_height: f64,
        theme: &Theme,
        mut group: Group,
    ) -> Group {
        let corner_radius = 10.0;

        // 1. Основной прямоугольник контейнера со скруглёнными углами
        let rect = Rectangle::new()
            .set("x", bounds.x)
            .set("y", bounds.y)
            .set("width", bounds.width)
            .set("height", bounds.height)
            .set("rx", corner_radius)
            .set("ry", corner_radius)
            .set("fill", theme.node_background.to_css())
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 1.5);

        group = group.add(rect);

        // 2. Заголовок состояния (название сверху, жирным, центрировано)
        let name_y = bounds.y + header_height / 2.0 + 2.0;
        let name_text = svg::node::element::Text::new(name)
            .set("x", bounds.x + bounds.width / 2.0)
            .set("y", name_y)
            .set("text-anchor", "middle")
            .set("dominant-baseline", "middle")
            .set("font-family", theme.font_family.as_str())
            .set("font-size", theme.font_size + 1.0)
            .set("font-weight", "bold")
            .set("fill", theme.text_color.to_css());

        group = group.add(name_text);

        // 3. Горизонтальный разделитель под заголовком
        let separator_y = bounds.y + header_height;
        let separator = svg::node::element::Line::new()
            .set("x1", bounds.x)
            .set("y1", separator_y)
            .set("x2", bounds.x + bounds.width)
            .set("y2", separator_y)
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 1);

        group = group.add(separator);

        group
    }

    /// Рендерит актёра (stick figure) для UseCase диаграмм
    /// Рисует фигуру участника sequence и его подпись.
    ///
    /// Геометрия снята с эталона PlantUML. Верхние блоки рисуют фигуру НАД
    /// подписью, нижние — зеркально, ПОД ней; подпись всегда прижата к
    /// линии жизни.
    fn render_participant_figure(
        &self,
        bounds: &Rect,
        label: &str,
        theme: &Theme,
        mut group: Group,
        footer: bool,
    ) -> Group {
        let cx = bounds.x + bounds.width / 2.0;
        let top = bounds.y;
        let bottom = bounds.y + bounds.height;

        // Подпись
        let label_y = if footer {
            top + PARTICIPANT_FOOTER_LABEL_BASELINE
        } else {
            bottom - PARTICIPANT_LABEL_BASELINE_GAP
        };
        // Подпись фигурных участников PlantUML смещает на 3 влево от оси:
        // эталон `sequence_participants` даёт центр «Database» на 553.089
        // при линии жизни 556.089 (у участника-рамки смещения нет).
        let text = svg::node::element::Text::new(label)
            .set("x", cx - PARTICIPANT_ICON_LABEL_SHIFT)
            .set("y", label_y)
            .set("text-anchor", "middle")
            .set("font-family", theme.font_family.as_str())
            .set("font-size", theme.font_size)
            .set("fill", theme.text_color.to_css());
        group = group.add(text);

        group
    }

    /// Стик-фигура актёра.
    fn render_actor(
        &self,
        bounds: &Rect,
        label: &str,
        theme: &Theme,
        group: Group,
        footer: bool,
    ) -> Group {
        let cx = bounds.x + bounds.width / 2.0;
        let stroke = theme.node_border.to_css();
        let fill = theme.node_background.to_css();

        // Все размеры сняты с эталона `sequence_participants`.
        let head_radius = ACTOR_HEAD_RADIUS;
        let (head_cy, neck_y, waist_y, arms_y, feet_y) = if footer {
            let base = bounds.y;
            (
                base + ACTOR_FOOTER_HEAD_CY,
                base + ACTOR_FOOTER_NECK,
                base + ACTOR_FOOTER_WAIST,
                base + ACTOR_FOOTER_ARMS,
                base + ACTOR_FOOTER_FEET,
            )
        } else {
            let base = bounds.y;
            (
                base + head_radius,
                base + ACTOR_HEAD_RADIUS * 2.0,
                base + ACTOR_WAIST_OFFSET,
                base + ACTOR_ARMS_OFFSET,
                base + ACTOR_FEET_OFFSET,
            )
        };

        let body = format!(
            "M{cx},{neck_y} L{cx},{waist_y} \
             M{arm_l},{arms_y} L{arm_r},{arms_y} \
             M{cx},{waist_y} L{leg_l},{feet_y} \
             M{cx},{waist_y} L{leg_r},{feet_y}",
            neck_y = fmt(neck_y),
            waist_y = fmt(waist_y),
            arms_y = fmt(arms_y),
            feet_y = fmt(feet_y),
            arm_l = fmt(cx - ACTOR_LIMB_SPREAD),
            arm_r = fmt(cx + ACTOR_LIMB_SPREAD),
            leg_l = fmt(cx - ACTOR_LIMB_SPREAD),
            leg_r = fmt(cx + ACTOR_LIMB_SPREAD),
        );

        let mut group = group;
        group = group.add(
            svg::node::element::Ellipse::new()
                .set("cx", cx)
                .set("cy", head_cy)
                .set("rx", head_radius)
                .set("ry", head_radius)
                .set("fill", fill)
                .set("stroke", stroke.clone())
                .set("stroke-width", 0.5),
        );
        group = group.add(
            svg::node::element::Path::new()
                .set("d", body)
                .set("fill", "none")
                .set("stroke", stroke)
                .set("stroke-width", 0.5),
        );

        self.render_participant_figure(bounds, label, theme, group, footer)
    }

    /// Цилиндр базы данных.
    fn render_database(
        &self,
        bounds: &Rect,
        label: &str,
        theme: &Theme,
        group: Group,
        footer: bool,
    ) -> Group {
        let cx = bounds.x + bounds.width / 2.0;
        let rx = DATABASE_RX;
        let ry = DATABASE_CAP_RY;
        let stroke = theme.node_border.to_css();
        let fill = theme.node_background.to_css();

        let (top_cap, bottom_cap) = if footer {
            // Зеркально: цилиндр висит под полосой подписи
            (
                bounds.y + bounds.height - DATABASE_FOOTER_TOP_CAP,
                bounds.y + bounds.height - DATABASE_FOOTER_BOTTOM_CAP,
            )
        } else {
            (bounds.y + ry, bounds.y + DATABASE_BODY_HEIGHT + ry)
        };

        // Боковины и нижняя крышка
        // Силуэт цилиндра.
        //
        // ВАЖНО: верхняя крышка входит в СИЛУЭТ выпуклостью ВВЕРХ, а не
        // рисуется отдельной дугой над плоским верхом. Эталон
        // `sequence_participants` даёт один замкнутый контур
        // `M538.089,34 C538.089,24 … 574.089,34 L574.089,60 C…538.089,60
        // L538.089,34`: прежний код начинал с прямой `L{left},{top_cap}`,
        // поэтому цилиндр выглядел скруглённым прямоугольником с плоским
        // верхом, а дуга висела над ним отдельной линией.
        let top_bulge = top_cap - ry;
        let body = format!(
            "M{left},{top_cap} C{left},{top_bulge} {cx},{top_bulge} {cx},{top_bulge} \
             C{cx},{top_bulge} {right},{top_bulge} {right},{top_cap} \
             L{right},{bottom_cap} \
             C{right},{bottom_bulge} {cx},{bottom_bulge} {cx},{bottom_bulge} \
             C{cx},{bottom_bulge} {left},{bottom_bulge} {left},{bottom_cap} Z",
            left = fmt(cx - rx),
            right = fmt(cx + rx),
            top_cap = fmt(top_cap),
            bottom_cap = fmt(bottom_cap),
            top_bulge = fmt(top_bulge),
            bottom_bulge = fmt(bottom_cap + ry),
        );

        let mut group = group;
        group = group.add(
            svg::node::element::Path::new()
                .set("d", body)
                .set("fill", fill)
                .set("stroke", stroke.clone())
                .set("stroke-width", 0.5),
        );
        // Нижняя дуга верхней крышки — «обод» цилиндра.
        //
        // Без неё верх читается как плоская крышка: эталон рисует вторую
        // половину эллипса отдельным путём `M538.089,34 C538.089,44 …
        // 574.089,34` без заливки.
        group = group.add(
            svg::node::element::Path::new()
                .set(
                    "d",
                    format!(
                        "M{left},{top_cap} C{left},{rim} {cx},{rim} {cx},{rim} \
                         C{cx},{rim} {right},{rim} {right},{top_cap}",
                        left = fmt(cx - rx),
                        right = fmt(cx + rx),
                        top_cap = fmt(top_cap),
                        rim = fmt(top_cap + ry),
                    ),
                )
                .set("fill", "none")
                .set("stroke", stroke)
                .set("stroke-width", 0.5),
        );

        self.render_participant_figure(bounds, label, theme, group, footer)
    }

    /// Граничный элемент: кружок со скобкой слева.
    fn render_boundary(
        &self,
        bounds: &Rect,
        label: &str,
        theme: &Theme,
        group: Group,
        footer: bool,
    ) -> Group {
        let cx = bounds.x + bounds.width / 2.0;
        let stroke = theme.node_border.to_css();
        let fill = theme.node_background.to_css();

        // Кружок прижат к линии жизни: сверху центр на 12 ниже верха,
        // снизу — на 12 выше низа.
        let circle_cy = if footer {
            bounds.y + bounds.height - PARTICIPANT_ICON_RADIUS
        } else {
            bounds.y + PARTICIPANT_ICON_RADIUS
        };
        let bracket_top = circle_cy - PARTICIPANT_ICON_RADIUS;
        let bracket_bottom = circle_cy + PARTICIPANT_ICON_RADIUS;
        // Кружок сдвинут ВПРАВО от линии жизни: эталон
        // `sequence_participants` даёт центр на 263.807 при линии 255.307,
        // то есть +8.5, а перекладина доходит до его левого края.
        let boundary_cx = cx + BOUNDARY_CIRCLE_SHIFT;

        let mut group = group;
        group = group.add(
            svg::node::element::Path::new()
                .set(
                    "d",
                    format!(
                        "M{bar},{top} L{bar},{bottom} M{bar},{mid} L{left_of_circle},{mid}",
                        bar = fmt(cx - BOUNDARY_BRACKET_OFFSET),
                        top = fmt(bracket_top),
                        bottom = fmt(bracket_bottom),
                        mid = fmt(circle_cy),
                        left_of_circle = fmt(boundary_cx - PARTICIPANT_ICON_RADIUS),
                    ),
                )
                .set("fill", "none")
                .set("stroke", stroke.clone())
                .set("stroke-width", 0.5),
        );
        group = group.add(
            svg::node::element::Ellipse::new()
                .set("cx", boundary_cx)
                .set("cy", circle_cy)
                .set("rx", PARTICIPANT_ICON_RADIUS)
                .set("ry", PARTICIPANT_ICON_RADIUS)
                .set("fill", fill)
                .set("stroke", stroke)
                .set("stroke-width", 0.5),
        );

        self.render_participant_figure(bounds, label, theme, group, footer)
    }

    /// Управляющий элемент: кружок со стрелкой сверху.
    fn render_control(
        &self,
        bounds: &Rect,
        label: &str,
        theme: &Theme,
        group: Group,
        footer: bool,
    ) -> Group {
        let cx = bounds.x + bounds.width / 2.0;
        let stroke = theme.node_border.to_css();
        let fill = theme.node_background.to_css();

        let circle_cy = if footer {
            bounds.y + bounds.height - PARTICIPANT_ICON_RADIUS
        } else {
            bounds.y + CONTROL_CIRCLE_OFFSET
        };
        // Стрелка прижата к внешнему краю фигуры
        let arrow_mid = if footer {
            bounds.y + bounds.height - CONTROL_FOOTER_ARROW_MID
        } else {
            bounds.y + CONTROL_ARROW_MID
        };

        let chevron = format!(
            "{x1},{y_mid} {x2},{y_top} {cx},{y_mid} {x2},{y_bot} {x1},{y_mid}",
            x1 = fmt(cx - CONTROL_ARROW_BACK),
            x2 = fmt(cx + CONTROL_ARROW_TIP),
            cx = fmt(cx),
            y_mid = fmt(arrow_mid),
            y_top = fmt(arrow_mid - CONTROL_ARROW_HALF),
            y_bot = fmt(arrow_mid + CONTROL_ARROW_HALF),
        );

        let mut group = group;
        group = group.add(
            svg::node::element::Ellipse::new()
                .set("cx", cx)
                .set("cy", circle_cy)
                .set("rx", PARTICIPANT_ICON_RADIUS)
                .set("ry", PARTICIPANT_ICON_RADIUS)
                .set("fill", fill)
                .set("stroke", stroke.clone())
                .set("stroke-width", 0.5),
        );
        group = group.add(
            svg::node::element::Polygon::new()
                .set("points", chevron)
                .set("fill", stroke.clone())
                .set("stroke", stroke)
                .set("stroke-width", 1.0),
        );

        self.render_participant_figure(bounds, label, theme, group, footer)
    }

    /// Сущность: кружок с подчёркиванием.
    fn render_entity(
        &self,
        bounds: &Rect,
        label: &str,
        theme: &Theme,
        group: Group,
        footer: bool,
    ) -> Group {
        let cx = bounds.x + bounds.width / 2.0;
        let stroke = theme.node_border.to_css();
        let fill = theme.node_background.to_css();

        let circle_cy = if footer {
            bounds.y + bounds.height - PARTICIPANT_ICON_RADIUS - ENTITY_FOOTER_GAP
        } else {
            bounds.y + PARTICIPANT_ICON_RADIUS
        };
        let underline_y = if footer {
            bounds.y + bounds.height
        } else {
            bounds.y + ENTITY_UNDERLINE_OFFSET
        };

        let mut group = group;
        group = group.add(
            svg::node::element::Ellipse::new()
                .set("cx", cx)
                .set("cy", circle_cy)
                .set("rx", PARTICIPANT_ICON_RADIUS)
                .set("ry", PARTICIPANT_ICON_RADIUS)
                .set("fill", fill)
                .set("stroke", stroke.clone())
                .set("stroke-width", 0.5),
        );
        group = group.add(
            svg::node::element::Line::new()
                .set("x1", cx - PARTICIPANT_ICON_RADIUS)
                .set("y1", underline_y)
                .set("x2", cx + PARTICIPANT_ICON_RADIUS)
                .set("y2", underline_y)
                .set("stroke", stroke)
                .set("stroke-width", 0.5),
        );

        self.render_participant_figure(bounds, label, theme, group, footer)
    }

    fn render_system(&self, bounds: &Rect, title: &str, theme: &Theme, mut group: Group) -> Group {
        let header_height = 25.0;

        // 1. Основной прямоугольник системы
        let rect = Rectangle::new()
            .set("x", bounds.x)
            .set("y", bounds.y)
            .set("width", bounds.width)
            .set("height", bounds.height)
            .set("fill", theme.node_background.to_css())
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 1);
        group = group.add(rect);

        // 2. Заголовок сверху по центру
        let title_text = svg::node::element::Text::new(title)
            .set("x", bounds.x + bounds.width / 2.0)
            .set("y", bounds.y + header_height / 2.0 + 5.0)
            .set("text-anchor", "middle")
            .set("font-family", theme.font_family.as_str())
            .set("font-size", theme.font_size + 1.0)
            .set("font-weight", "bold")
            .set("fill", theme.text_color.to_css());
        group = group.add(title_text);

        group
    }

    /// Рендерит линию/стрелку
    #[allow(clippy::too_many_arguments)]
    fn render_edge(
        &self,
        points: &[Point],
        label: Option<&str>,
        autonumber: Option<&str>,
        arrow_start: bool,
        arrow_end: bool,
        dashed: bool,
        edge_type: EdgeType,
        from_cardinality: Option<&str>,
        to_cardinality: Option<&str>,
        style: EdgeStyle<'_>,
        theme: &Theme,
        mut group: Group,
    ) -> Group {
        if points.len() < 2 {
            return group;
        }

        // Определяем, является ли это self-message (петля: 4 точки, первая и последняя
        // имеют одинаковый X но разный Y - прямоугольная петля вправо)
        let is_self_message = points.len() == 4
            && (points[0].x - points[3].x).abs() < 1.0
            && (points[0].y - points[3].y).abs() > 1.0;

        // Строим путь
        let d = if is_self_message {
            // PlantUML style self-message: прямые углы (3 линии)
            // points[0] = start (lifeline, top)
            // points[1] = right top
            // points[2] = right bottom
            // points[3] = end (lifeline, bottom)
            //
            // PlantUML SVG:
            // line 1: x1=28.8 → x2=70.8, y=67.4 (горизонтальная вправо)
            // line 2: x=70.8, y1=67.4 → y2=80.4 (вертикальная вниз)
            // line 3: x1=70.8 → x2=29.8, y=80.4 (горизонтальная влево)
            // polygon (стрелка): в конце line 3
            //
            // Путь: от lifeline вправо, вниз, обратно к lifeline
            format!(
                "M{},{} L{},{} L{},{} L{},{}",
                points[0].x,
                points[0].y, // начало (lifeline, верх)
                points[1].x,
                points[1].y, // вправо (верхний правый угол)
                points[2].x,
                points[2].y, // вниз (нижний правый угол)
                points[3].x,
                points[3].y, // влево к lifeline (конец)
            )
        } else {
            // Обычные линии
            let mut d = format!("M{},{}", points[0].x, points[0].y);
            for p in &points[1..] {
                d.push_str(&format!(" L{},{}", p.x, p.y));
            }
            d
        };

        // Цвет и толщина могут быть переопределены свойствами: PlantUML
        // рисует переходы состояний timing зелёным (#006400) толщиной 2,
        // деления оси — толщиной 2, а линии дорожек — цветом #333.
        // PlantUML использует stroke-width: 0.5 для lifelines, 1 для сообщений
        // Определяем по наличию стрелки - если есть стрелка, это сообщение
        // Толщину можно переопределить свойством: PlantUML рисует
        // переходы состояний timing и деления оси толщиной 2.
        let stroke_width = style
            .width
            .unwrap_or(if arrow_end || arrow_start { 1.0 } else { 0.5 });

        let mut path = Path::new()
            .set("d", d)
            .set("fill", "none")
            .set(
                "stroke",
                style
                    .color
                    .map(str::to_string)
                    .unwrap_or_else(|| theme.arrow_color.to_css()),
            )
            .set("stroke-width", stroke_width);

        // Пунктирная линия для lifelines и dashed arrows
        // PlantUML использует stroke-dasharray: 5,5 для lifelines, 2,2 для dashed сообщений
        if dashed {
            // Для lifelines (без стрелок) используем 5,5, для dashed сообщений - 2,2
            let dash_pattern = if arrow_end || arrow_start {
                "2,2"
            } else {
                "5,5"
            };
            path = path.set("stroke-dasharray", dash_pattern);
        }

        // Выбираем маркер на основе типа связи
        if arrow_end {
            let marker = match edge_type {
                EdgeType::Inheritance | EdgeType::Realization => "url(#inheritance)",
                EdgeType::Composition => "url(#arrow)", // composition marker на start
                EdgeType::Aggregation => "url(#arrow)", // aggregation marker на start
                EdgeType::Dependency => "url(#arrow-open)",
                EdgeType::Association => "url(#arrow)",
                EdgeType::Link => "", // без маркера
            };
            if !marker.is_empty() {
                path = path.set("marker-end", marker);
            }
        }
        if arrow_start {
            let marker = match edge_type {
                EdgeType::Composition => "url(#composition)",
                EdgeType::Aggregation => "url(#aggregation)",
                _ => "url(#arrow)",
            };
            path = path.set("marker-start", marker);
        }

        group = group.add(path);

        // Метка сообщения (в стиле PlantUML: текст рядом с линией)
        // По умолчанию в PlantUML: skinparam sequenceMessageAlign left
        // Если есть autonumber — рендерим его отдельно слева, текст справа от него
        if label.is_some() || autonumber.is_some() {
            // Определяем тип линии
            let dx = points[points.len() - 1].x - points[0].x;
            let dy = points[points.len() - 1].y - points[0].y;

            let is_vertical = points.len() == 2 && dy.abs() > dx.abs() * 3.0;
            let is_horizontal = points.len() == 2 && dx.abs() > dy.abs() * 3.0;
            let is_diagonal = points.len() == 2 && !is_vertical && !is_horizontal;

            // Позиция текста зависит от типа линии
            let (base_x, text_y, anchor) = if is_self_message {
                // PlantUML: для self-message текст НАД верхней линией петли
                let text_start = points[0].x + 5.0;
                let top_y = points[0].y - 5.0;
                (text_start, top_y, "start")
            } else if points.len() == 4 {
                // Ортогональный путь (4 точки): это обратный переход
                // Структура: start_h -> corner1 -> corner2 -> end_h
                // points[0]: начало горизонтального сегмента
                // points[1]: угол (конец первого горизонтального сегмента)
                // points[2]: угол (начало последнего горизонтального сегмента)
                // points[3]: конец горизонтального сегмента

                // Метка располагается на ПЕРВОМ горизонтальном сегменте (исходящем),
                // справа от точки выхода, на уровне Y этого сегмента
                let text_x = points[0].x + 5.0; // справа от точки выхода
                let text_y = points[0].y - 5.0; // чуть выше линии
                (text_x, text_y, "start")
            } else if is_diagonal {
                // Диагональная линия (state diagrams): текст РЯДОМ с линией
                // PlantUML style: текст размещается вдоль стрелки, с внешней стороны

                // Позиция вдоль линии (40% от начала для лучшего разделения расходящихся стрелок)
                let t = 0.40;
                let text_x = points[0].x + dx * t;
                let text_y = points[0].y + dy * t;

                // Смещаем текст в сторону от линии (перпендикулярно, на ВНЕШНЮЮ сторону)
                // Для расходящихся из одной точки стрелок:
                // - Линия влево-вниз — текст СЛЕВА от линии
                // - Линия вправо-вниз — текст СПРАВА от линии
                let offset = 8.0; // отступ от линии
                if dx > 0.0 {
                    // Линия идёт вправо-вниз — текст СПРАВА от линии (внешняя сторона)
                    (text_x + offset, text_y, "start")
                } else {
                    // Линия идёт влево-вниз — текст СЛЕВА от линии (внешняя сторона)
                    (text_x - offset, text_y, "end")
                }
            } else if is_vertical {
                // Вертикальная линия: метка СПРАВА, ПОСЕРЕДИНЕ по высоте
                let mid_y = (points[0].y + points[1].y) / 2.0;
                let text_x = points[0].x + 5.0;
                (text_x, mid_y, "start")
            } else if is_horizontal {
                // Горизонтальная стрелка (sequence diagrams).
                //
                // Отступ подписи от левого конца разный: у стрелки слева
                // направо 7, у обратной — 17, потому что там наконечник
                // занимает 10 и подпись начинается за ним. Эталон
                // `sequence_simple`: «Authentication Request» на 40.833
                // (33.833 + 7), «Authentication Response» на 50.833
                // (33.833 + 17).
                let is_left_to_right = dx > 0.0;
                let left_x = if is_left_to_right {
                    points[0].x + MESSAGE_LABEL_INSET
                } else {
                    points[1].x + MESSAGE_LABEL_INSET + MESSAGE_ARROW_HEAD
                };
                let top_y = points[0].y - MESSAGE_LABEL_RISE;
                (left_x, top_y, "start")
            } else {
                // Fallback: середина первого сегмента
                let mid_x = (points[0].x + points[1].x) / 2.0;
                let mid_y = (points[0].y + points[1].y) / 2.0;
                (mid_x, mid_y - 5.0, "middle")
            };

            // PlantUML не использует белый фон для текста — текст просто над стрелкой
            // PlantUML использует font-size 13 для сообщений
            let font_size = 13.0;

            // Вычисляем позицию для текста (учитывая autonumber)
            let text_x = if let Some(num_text) = autonumber {
                // Рендерим autonumber отдельно
                // Номер уже отформатирован (например "[01]" или "1." в зависимости от формата)

                // Autonumber слева
                let autonumber_element = svg::node::element::Text::new(num_text)
                    .set("x", base_x)
                    .set("y", text_y)
                    .set("text-anchor", anchor)
                    .set("dominant-baseline", "auto")
                    .set("font-family", theme.font_family.as_str())
                    .set("font-size", font_size)
                    .set("fill", theme.text_color.to_css());
                group = group.add(autonumber_element);

                // Вычисляем ширину autonumber для смещения текста
                // PlantUML: ~7px на символ + небольшой отступ 3px
                let num_width = num_text.len() as f64 * 7.0 + 3.0;
                base_x + num_width
            } else {
                base_x
            };

            // Рендерим текст сообщения (если есть)
            if let Some(label) = label {
                // Поддержка многострочного текста через \n
                group = self
                    .render_multiline_text(text_x, text_y, label, anchor, font_size, theme, group);
            }
        }

        // Рендерим кардинальности (для class diagrams)
        // PlantUML: кардинальности располагаются СЛЕВА от вертикальной линии,
        // близко к точкам соединения с классами
        if points.len() >= 2 {
            let font_size = 13.0; // PlantUML использует 13px
            let horizontal_offset = 10.0; // отступ слева от линии
            let vertical_offset = 12.0; // отступ от точки соединения вниз/вверх

            // Определяем, это вертикальная линия
            let is_vertical = (points[1].y - points[0].y).abs() > (points[1].x - points[0].x).abs();

            // Кардинальности.
            //
            // ER-нотацию (`||`, `}o`, `}|`, `|o`) PlantUML рисует ФИГУРАМИ
            // («вороньими лапками»), а числовые подписи классов («1», «*») —
            // текстом. См. `render_crow_foot` и `is_crow_foot`.
            let last_index = points.len() - 1;

            // Кардинальность у начальной точки (from)
            if let Some(card) = from_cardinality {
                if is_crow_foot(card) {
                    group = self.render_crow_foot(card, points[0], points[1], theme, group);
                } else {
                    let p = &points[0];
                    let (text_x, text_y) = if is_vertical {
                        // Вертикальная линия: текст СЛЕВА, чуть НИЖЕ точки соединения
                        (p.x - horizontal_offset, p.y + vertical_offset)
                    } else {
                        // Горизонтальная линия: текст сверху
                        (p.x + vertical_offset, p.y - horizontal_offset / 2.0)
                    };
                    let text_elem = svg::node::element::Text::new(card)
                        .set("x", text_x)
                        .set("y", text_y)
                        .set("text-anchor", "end") // выравнивание по правому краю (к линии)
                        .set("dominant-baseline", "middle")
                        .set("font-family", theme.font_family.as_str())
                        .set("font-size", font_size)
                        .set("fill", theme.text_color.to_css());
                    group = group.add(text_elem);
                }
            }

            // Кардинальность у конечной точки (to)
            if let Some(card) = to_cardinality {
                if is_crow_foot(card) {
                    group = self.render_crow_foot(
                        card,
                        points[last_index],
                        points[last_index - 1],
                        theme,
                        group,
                    );
                } else {
                    let p = &points[last_index];
                    let (text_x, text_y) = if is_vertical {
                        // Вертикальная линия: текст СЛЕВА, чуть ВЫШЕ точки соединения
                        (p.x - horizontal_offset, p.y - vertical_offset)
                    } else {
                        // Горизонтальная линия: текст сверху
                        (p.x - vertical_offset, p.y - horizontal_offset / 2.0)
                    };
                    let text_elem = svg::node::element::Text::new(card)
                        .set("x", text_x)
                        .set("y", text_y)
                        .set("text-anchor", "end") // выравнивание по правому краю (к линии)
                        .set("dominant-baseline", "middle")
                        .set("font-family", theme.font_family.as_str())
                        .set("font-size", font_size)
                        .set("fill", theme.text_color.to_css());
                    group = group.add(text_elem);
                }
            }
        }

        group
    }

    /// Рисует кардинальность ER фигурами («вороньими лапками»).
    ///
    /// PlantUML в ER-нотации НЕ пишет `||` и `}o` текстом, а рисует их
    /// линиями и кружком. Геометрия снята с эталона `er_basic` для связи
    /// `user ||--o{ order` (линия идёт сверху вниз, `tip` — конец у
    /// сущности, `inward` — соседняя точка линии):
    ///
    ///   * `||` — стойка длиной 8 и две перекладины на 1 и 4 от конца,
    ///     полушириной 4 (`line 55.75..63.75` на y=91.79 и 94.79);
    ///   * `}o` — кружок r=4 с центром на 4 от конца и «лапка»: вершина
    ///     на 10, три луча до 18 с разбросом ±6.
    fn render_crow_foot(
        &self,
        symbol: &str,
        tip: Point,
        inward: Point,
        theme: &Theme,
        mut group: Group,
    ) -> Group {
        let (dx, dy) = (tip.x - inward.x, tip.y - inward.y);
        let len = (dx * dx + dy * dy).sqrt();
        if len < f64::EPSILON {
            return group;
        }
        let (ux, uy) = (dx / len, dy / len);
        // Перпендикуляр к линии.
        let (nx, ny) = (-uy, ux);
        let at = |t: f64, s: f64| (tip.x + ux * t + nx * s, tip.y + uy * t + ny * s);
        let stroke = theme.node_border.to_css();
        let mut d = String::new();

        if symbol.contains('}') {
            let apex = at(CROW_FOOT_APEX, 0.0);
            for spread in [0.0, CROW_FOOT_SPREAD, -CROW_FOOT_SPREAD] {
                let end = at(CROW_FOOT_LENGTH, spread);
                d.push_str(&format!(
                    "M{},{} L{},{} ",
                    fmt(apex.0),
                    fmt(apex.1),
                    fmt(end.0),
                    fmt(end.1)
                ));
            }
        }

        // Перекладины «один»: по одной на каждый символ `|`.
        for index in 0..symbol.matches('|').count() {
            let t = CROW_FOOT_BAR_FIRST + CROW_FOOT_BAR_STEP * index as f64;
            let left = at(t, CROW_FOOT_BAR_HALF);
            let right = at(t, -CROW_FOOT_BAR_HALF);
            d.push_str(&format!(
                "M{},{} L{},{} ",
                fmt(left.0),
                fmt(left.1),
                fmt(right.0),
                fmt(right.1)
            ));
        }

        if !d.is_empty() {
            group = group.add(
                Path::new()
                    .set("d", d.trim_end().to_string())
                    .set("fill", "none")
                    .set("stroke", stroke.clone())
                    .set("stroke-width", 1),
            );
        }

        if symbol.contains('o') {
            let center = at(CROW_FOOT_CIRCLE, 0.0);
            group = group.add(
                svg::node::element::Ellipse::new()
                    .set("cx", fmt(center.0))
                    .set("cy", fmt(center.1))
                    .set("rx", CROW_FOOT_CIRCLE_RADIUS)
                    .set("ry", CROW_FOOT_CIRCLE_RADIUS)
                    .set("fill", "none")
                    .set("stroke", stroke)
                    .set("stroke-width", 1),
            );
        }

        group
    }

    /// Рендерит текст
    /// Рендерит текст с возможностью сделать его полужирным.
    ///
    /// PlantUML рисует метки состояний concise-дорожек timing полужирными
    /// (`font-weight="700"`), тогда как robust — обычными. Проверено
    /// прямым замером: одно слово даёт textLength 68.15 у robust и
    /// 75.088 у concise при одинаковом размере шрифта.
    fn render_text_weighted(
        &self,
        bounds: &Rect,
        text_content: &str,
        font_size: f64,
        style: TextStyle<'_>,
        theme: &Theme,
        group: Group,
    ) -> Group {
        // Базис можно задать свойством: таблицы JSON/YAML кладут строку
        // по `row + line_height - 5.302`, а не по низу прямоугольника.
        let baseline = style.baseline.unwrap_or(bounds.y + font_size);

        let mut text = svg::node::element::Text::new(text_content)
            .set("x", bounds.x)
            .set("y", baseline)
            .set("font-family", theme.font_family.as_str())
            .set("font-size", font_size)
            // Цвет можно переопределить свойством: PlantUML рисует
            // подписи timing цветом #333, тогда как остальные типы — #000.
            .set("fill", style.fill);

        if style.bold {
            text = text.set("font-weight", "700");
        }

        group.add(text)
    }

    /// Рендерит многострочный текст с поддержкой \n
    /// PlantUML использует <tspan> для каждой строки с dy для смещения
    #[allow(clippy::too_many_arguments)]
    fn render_multiline_text(
        &self,
        x: f64,
        y: f64,
        label: &str,
        anchor: &str,
        font_size: f64,
        theme: &Theme,
        mut group: Group,
    ) -> Group {
        // Конвертируем escape-последовательность \n в реальные переносы строк
        let processed_label = label.replace("\\n", "\n");
        let lines: Vec<&str> = processed_label.split('\n').collect();

        if lines.len() == 1 {
            // Одна строка — простой текст
            let text = svg::node::element::Text::new(label)
                .set("x", x)
                .set("y", y)
                .set("text-anchor", anchor)
                .set("dominant-baseline", "auto")
                .set("font-family", theme.font_family.as_str())
                .set("font-size", font_size)
                .set("fill", theme.text_color.to_css());
            group = group.add(text);
        } else {
            // Многострочный текст — используем <text> с <tspan> для каждой строки
            // PlantUML: последняя строка на y (ближе к стрелке), предыдущие строки ВВЕРХ
            // Так текст располагается над стрелкой, и не наезжает на неё
            let line_height = font_size + 2.0;

            // Начинаем с верхней строки (которая будет самой верхней визуально)
            let top_y = y - (lines.len() as f64 - 1.0) * line_height;

            let mut text_element = svg::node::element::Text::new("")
                .set("x", x)
                .set("text-anchor", anchor)
                .set("dominant-baseline", "auto")
                .set("font-family", theme.font_family.as_str())
                .set("font-size", font_size)
                .set("fill", theme.text_color.to_css());

            for (i, line) in lines.iter().enumerate() {
                let tspan = svg::node::element::TSpan::new(*line)
                    .set("x", x)
                    .set("y", top_y + (i as f64) * line_height);
                text_element = text_element.add(tspan);
            }

            group = group.add(text_element);
        }

        group
    }

    /// Рендерит группу (устаревший, для совместимости)
    /// Рисует контейнер (узел deployment, пакет component) объёмной рамкой.
    ///
    /// PlantUML рисует узлы deployment трёхмерными: основной прямоугольник
    /// плюс скошенный верхний правый угол со смещением 10px. Раньше здесь
    /// был плоский прямоугольник с полосой заголовка.
    fn render_group(
        &self,
        bounds: &Rect,
        label: Option<&str>,
        children: &[LayoutElement],
        bold: bool,
        theme: &Theme,
        mut group: Group,
    ) -> Group {
        let offset = NODE_3D_OFFSET;
        let stroke = theme.node_border.to_css();

        // Основной прямоугольник смещён вниз на величину скоса
        let left = bounds.x;
        let top = bounds.y + offset;
        let right = bounds.x + bounds.width - offset;
        let bottom = bounds.y + bounds.height;

        // Контур объёмной фигуры. Порядок точек снят с эталона
        // deployment_basic: (left,top) -> (left+offset,y) -> (right+offset,y)
        // -> (right+offset,bottom-offset) -> (right,bottom) -> (left,bottom).
        let polygon = format!(
            "{l},{t} {lo},{y} {ro},{y} {ro},{bo} {r},{b} {l},{b} {l},{t}",
            l = fmt(left),
            t = fmt(top),
            lo = fmt(left + offset),
            ro = fmt(right + offset),
            y = fmt(bounds.y),
            bo = fmt(bottom - offset),
            r = fmt(right),
            b = fmt(bottom),
        );

        group = group.add(
            svg::node::element::Polygon::new()
                .set("points", polygon)
                .set("fill", "none")
                .set("stroke", stroke.clone())
                .set("stroke-width", 1)
                .set("stroke-linejoin", "miter"),
        );

        // Внутренние рёбра объёма
        let edges = format!(
            "M{r},{t} L{ro},{y} M{l},{t} L{r},{t} M{r},{t} L{r},{b}",
            l = fmt(left),
            r = fmt(right),
            ro = fmt(right + offset),
            t = fmt(top),
            y = fmt(bounds.y),
            b = fmt(bottom),
        );
        group = group.add(
            svg::node::element::Path::new()
                .set("d", edges)
                .set("fill", "none")
                .set("stroke", stroke)
                .set("stroke-width", 1),
        );

        // Заголовок по центру основного прямоугольника
        if let Some(label) = label {
            let text = svg::node::element::Text::new(label)
                .set("x", (left + right) / 2.0)
                .set("y", top + NODE_TITLE_BASELINE)
                .set("text-anchor", "middle")
                .set("font-family", theme.font_family.as_str())
                .set("font-size", theme.font_size)
                .set("font-weight", if bold { "700" } else { "bold" })
                .set("fill", theme.text_color.to_css());

            group = group.add(text);
        }

        // Дочерние элементы
        for child in children {
            group = group.add(self.render_element(child, theme));
        }

        group
    }

    /// Рендерит Combined Fragment (alt, opt, loop, etc.) в стиле PlantUML
    fn render_fragment(
        &self,
        bounds: &Rect,
        fragment_type: &str,
        sections: &[FragmentSection],
        bold: bool,
        theme: &Theme,
        mut group: Group,
    ) -> Group {
        // 1. СПЛОШНАЯ рамка фрагмента (как в PlantUML)
        // PlantUML использует более толстую рамку для фрагментов (1.5px)
        // В эталоне PlantUML рамка фрагмента чёрная (#000), а не цвет
        // границ темы (#181818)
        let rect = Rectangle::new()
            .set("x", bounds.x)
            .set("y", bounds.y)
            .set("width", bounds.width)
            .set("height", bounds.height)
            .set("fill", "none")
            .set("stroke", FRAGMENT_BORDER)
            .set("stroke-width", 1.5);

        group = group.add(rect);

        // 2. Пятиугольный заголовок (pentagon) в левом верхнем углу.
        // Геометрия по эталону: высота 17.133, зазубрина 10.
        let label_text = fragment_type;
        let label_width = {
            // Ширина измеряется, а не оценивается по числу символов:
            // прежняя оценка `символы * font_size * 0.481` давала для
            // «alt» 26.9 при реальных 19.44. В эталоне заголовок занимает
            // 64.44px, то есть отступы по ~22px с каждой стороны.
            let measured = self.measure_text(label_text, theme.font_size);
            (measured + 45.0).max(40.0)
        };
        let label_height = 17.133;
        let notch_size = 10.0; // размер «зазубрины» пятиугольника

        // Пятиугольник: верхний левый угол рамки -> вправо -> вниз с зазубриной -> влево -> вверх
        let pentagon_path = format!(
            "M{},{} L{},{} L{},{} L{},{} L{},{} Z",
            bounds.x,
            bounds.y, // верхний левый
            bounds.x + label_width,
            bounds.y, // верхний правый
            bounds.x + label_width,
            bounds.y + label_height - notch_size, // правый до зазубрины
            bounds.x + label_width - notch_size,
            bounds.y + label_height, // зазубрина
            bounds.x,
            bounds.y + label_height, // нижний левый
        );

        // Заливка заголовка в эталоне — светло-серая #EEE
        let pentagon = Path::new()
            .set("d", pentagon_path)
            .set("fill", FRAGMENT_HEADER_FILL)
            .set("stroke", FRAGMENT_BORDER)
            .set("stroke-width", 1.5);

        group = group.add(pentagon);

        // Текст типа фрагмента ("alt", "opt", etc.)
        let type_text = svg::node::element::Text::new(label_text)
            .set("x", bounds.x + FRAGMENT_TYPE_INSET)
            .set("y", bounds.y + FRAGMENT_TYPE_BASELINE)
            .set("font-family", theme.font_family.as_str())
            .set("font-size", FRAGMENT_LABEL_FONT_SIZE)
            // PlantUML использует числовую запись `700`; визуально то же,
            // но для точного соответствия эталону приводим к ней.
            .set("font-weight", if bold { "700" } else { "bold" })
            .set("fill", theme.text_color.to_css());

        group = group.add(type_text);

        // 3. Условие первой секции справа от пятиугольника
        if let Some(first_section) = sections.first() {
            if let Some(condition) = &first_section.condition {
                // Условие секции тоже ПОЛУЖИРНОЕ: в эталоне
                // sequence_fragments элементы «[Успешно]» и «[Ошибка]»
                // имеют font-weight="700", как и тип «alt».
                // Условие прижато к ПРАВОМУ краю рамки с отступом 16,
                // кегль 11: эталон `sequence_fragments` даёт «[Успешно]» на
                // x = 96.40 при рамке 16.955..179.355 (96.40 + 66.96 + 16 =
                // 179.36), базис — на 12.21 ниже верха.
                let cond_value = format!("[{condition}]");
                let cond_width = plantuml_layout::text::TextMeasurer::default()
                    .width_bold(&cond_value, FRAGMENT_CONDITION_FONT_SIZE);
                let cond_text = svg::node::element::Text::new(cond_value)
                    .set(
                        "x",
                        bounds.x + bounds.width - FRAGMENT_CONDITION_INSET - cond_width,
                    )
                    .set("y", bounds.y + FRAGMENT_CONDITION_BASELINE)
                    .set("font-family", theme.font_family.as_str())
                    .set("font-size", FRAGMENT_CONDITION_FONT_SIZE)
                    .set("font-weight", if bold { "700" } else { "bold" })
                    .set("fill", theme.text_color.to_css());

                group = group.add(cond_text);
            }
        }

        // 4. Разделители между секциями (пунктирные линии с условиями else)
        for (i, section) in sections.iter().enumerate() {
            // Разделитель перед секцией (кроме первой)
            if i > 0 {
                // Линия разделителя находится между секциями
                // section.start_y — это Y позиция первого сообщения в секции
                // Нам нужно разместить линию ВЫШЕ этого сообщения с учётом:
                // 1. Места для текста условия [else] над линией (~18px)
                // 2. Отступа от линии до текста сообщения под ней (~10px)
                // Итого: линия на section.start_y - 28px
                // Эталон `sequence_fragments`: разделитель стоит на 38.94
                // выше первого сообщения секции (178.63 - 139.69), а не на
                // 28 — прежнее значение прижимало его к предыдущему
                // сообщению и растягивало блок «else».
                let separator_y = section.start_y - FRAGMENT_SEPARATOR_OFFSET;

                // Пунктирная линия
                let separator_line = Path::new()
                    .set(
                        "d",
                        format!(
                            "M{},{} L{},{}",
                            bounds.x,
                            separator_y,
                            bounds.x + bounds.width,
                            separator_y
                        ),
                    )
                    .set("fill", "none")
                    .set("stroke", theme.node_border.to_css())
                    .set("stroke-width", 1)
                    // В эталоне PlantUML разделитель else — пунктир 2,2,
                    // а не 5,3
                    .set("stroke-dasharray", "2,2");

                group = group.add(separator_line);

                // Текст условия else слева, ПОД линией.
                // В эталоне PlantUML линия на y=139.695, текст на y=151.906,
                // то есть на ~12px ниже. Раньше текст рисовался над линией.
                let else_label = if let Some(cond) = &section.condition {
                    format!("[{cond}]")
                } else {
                    "[else]".to_string()
                };

                let else_text = svg::node::element::Text::new(else_label)
                    .set("x", bounds.x + 5.0)
                    .set("y", separator_y + 12.0)
                    .set("font-family", theme.font_family.as_str())
                    // В эталоне размер 11 и жирный
                    .set("font-size", FRAGMENT_CONDITION_FONT_SIZE)
                    .set("font-weight", if bold { "700" } else { "bold" })
                    .set("fill", theme.text_color.to_css());

                group = group.add(else_text);
            }

            // Дочерние элементы секции
            for child in &section.children {
                group = group.add(self.render_element(child, theme));
            }
        }

        group
    }

    /// Рендерит Participant Box (фоновый прямоугольник с заголовком)
    fn render_participant_box(
        &self,
        bounds: &Rect,
        title: Option<&str>,
        color: Option<&str>,
        theme: &Theme,
        mut group: Group,
    ) -> Group {
        // Фоновый цвет (по умолчанию светло-серый)
        let fill_color = color.unwrap_or("#EEEEEE");

        // Основной прямоугольник
        let rect = Rectangle::new()
            .set("x", bounds.x)
            .set("y", bounds.y)
            .set("width", bounds.width)
            .set("height", bounds.height)
            .set("fill", fill_color)
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 1);
        group = group.add(rect);

        // Заголовок по центру сверху
        if let Some(title) = title {
            let title_y = bounds.y + 16.0;
            let title_text = svg::node::element::Text::new(title)
                .set("x", bounds.x + bounds.width / 2.0)
                .set("y", title_y)
                .set("text-anchor", "middle")
                .set("font-family", theme.font_family.as_str())
                .set("font-size", theme.font_size + 1.0)
                .set("font-weight", "bold")
                .set("fill", theme.text_color.to_css());
            group = group.add(title_text);
        }

        group
    }

    /// Рендерит Activation box (белый фон, чёрная рамка)
    fn render_activation(&self, bounds: &Rect, theme: &Theme, group: Group) -> Group {
        // Activation box: белый фон (как в PlantUML)
        let rect = Rectangle::new()
            .set("x", bounds.x)
            .set("y", bounds.y)
            .set("width", bounds.width)
            .set("height", bounds.height)
            .set("fill", theme.background_color.to_css()) // белый фон
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 1);

        group.add(rect)
    }

    /// Рендерит ClassBox (класс/интерфейс/enum) в стиле PlantUML
    #[allow(clippy::too_many_arguments)]
    fn render_class_box(
        &self,
        bounds: &Rect,
        classifier_type: ClassifierKind,
        name: &str,
        stereotype: Option<&str>,
        fields: &[ClassMember],
        methods: &[ClassMember],
        sprites: &std::collections::HashMap<String, SpriteData>,
        theme: &Theme,
        mut group: Group,
    ) -> Group {
        let padding = 5.0;
        let line_height = 16.0;
        let icon_size = 11.0; // радиус иконки класса

        // 1. Рамка класса
        // Иконка классификатора. Цвет зависит от типа, но заливается только
        // сам кружок: тело класса в эталоне PlantUML всегда #F1F1F1
        // (светло-серый), независимо от типа.
        let (icon_fill, icon_letter) = match classifier_type {
            ClassifierKind::Class => ("#ADD1B2", "C"),     // зелёный
            ClassifierKind::Interface => ("#B4A7E5", "I"), // фиолетовый
            ClassifierKind::AbstractClass => ("#A9DCDF", "A"), // голубой
            ClassifierKind::Enum => ("#EB937F", "E"),      // оранжевый
            ClassifierKind::Annotation => ("#FFDD8C", "@"), // жёлтый
            ClassifierKind::Entity => ("#CCCCCC", "E"),    // серый
        };

        // Тело класса: в эталоне PlantUML — #F1F1F1, а не цвет фона темы
        // (#E2E2F0) и не цвет иконки
        let rect = Rectangle::new()
            .set("x", bounds.x)
            .set("y", bounds.y)
            .set("width", bounds.width)
            .set("height", bounds.height)
            .set("rx", 2.5)
            .set("ry", 2.5)
            .set("fill", CLASS_BODY_FILL)
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 0.5);
        group = group.add(rect);

        let mut current_y = bounds.y + padding;

        // 2. Иконка классификатора (PlantUML style)
        let icon_x = bounds.x + padding + icon_size;
        let icon_y = current_y + icon_size;

        // Круг иконки
        let icon_circle = svg::node::element::Ellipse::new()
            .set("cx", icon_x)
            .set("cy", icon_y)
            .set("rx", icon_size)
            .set("ry", icon_size)
            .set("fill", icon_fill)
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 1);
        group = group.add(icon_circle);

        // Буква в иконке
        let icon_text = svg::node::element::Text::new(icon_letter)
            .set("x", icon_x)
            .set("y", icon_y + 4.0)
            .set("text-anchor", "middle")
            .set("font-family", theme.font_family.as_str())
            .set("font-size", 12)
            .set("font-weight", "bold")
            .set("fill", Color::new("#000000").to_css());
        group = group.add(icon_text);

        // 3. Стереотип (если есть)
        let name_x = icon_x + icon_size + 5.0;
        if let Some(stereo) = stereotype {
            let stereo_text = svg::node::element::Text::new(format!("«{}»", stereo))
                .set("x", name_x)
                .set("y", current_y + 10.0)
                .set("font-family", theme.font_family.as_str())
                .set("font-size", 10)
                .set("fill", theme.text_color.to_css());
            group = group.add(stereo_text);
            current_y += 12.0;
        }

        // 4. Название класса.
        // В эталоне PlantUML имя класса не жирное (font-size 14, обычное
        // начертание) — жирный был лишним.
        //
        // Вставка `<$имя>` заменяется спрайтом: PlantUML разбивает строку
        // на часть до вставки, растр и часть после. Растр рисуется
        // прямоугольниками палитры, как и объявленный отдельно.
        let baseline = current_y + line_height - 2.0;
        let mut cursor = name_x;

        for part in split_sprite_references(name, sprites) {
            match part {
                SpritePart::Text(text) => {
                    if !text.is_empty() {
                        let text_node = svg::node::element::Text::new(text.clone())
                            .set("x", cursor)
                            .set("y", baseline)
                            .set("font-family", theme.font_family.as_str())
                            .set("font-size", theme.font_size)
                            .set("fill", theme.text_color.to_css());
                        group = group.add(text_node);
                        cursor += self.measure_text(&text, theme.font_size);
                    }
                }
                SpritePart::Sprite(data, scale) => {
                    // Масштаб увеличивает и прямоугольник, и размер пикселя:
                    // иначе растр нарисуется в углу увеличенной области.
                    let pixel = SPRITE_INLINE_PIXEL * scale;
                    let size = data.width as f64 * pixel;
                    let height = data.height as f64 * pixel;
                    let top = baseline - height + 2.0;
                    let bounds = Rect::new(cursor, top, size, height);

                    group = match &data.svg {
                        Some(vector) => self.render_vector_sprite(&bounds, vector, theme, group),
                        None => self.render_sprite_data(&bounds, &data.rows, pixel, group),
                    };

                    cursor += size + SPRITE_INLINE_GAP;
                }
            }
        }
        current_y += line_height + padding;

        // 5. Разделитель после имени
        let separator1 = svg::node::element::Line::new()
            .set("x1", bounds.x + 1.0)
            .set("y1", current_y)
            .set("x2", bounds.x + bounds.width - 1.0)
            .set("y2", current_y)
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 0.5);
        group = group.add(separator1);
        current_y += padding;

        // 6. Поля
        for field in fields {
            group = self.render_class_member(
                bounds.x + padding,
                current_y,
                bounds.width - padding * 2.0,
                field,
                theme,
                group,
            );
            current_y += line_height;
        }

        // 7. Разделитель между полями и методами
        let separator2 = svg::node::element::Line::new()
            .set("x1", bounds.x + 1.0)
            .set("y1", current_y)
            .set("x2", bounds.x + bounds.width - 1.0)
            .set("y2", current_y)
            .set("stroke", theme.node_border.to_css())
            .set("stroke-width", 0.5);
        group = group.add(separator2);
        current_y += padding;

        // 8. Методы
        for method in methods {
            group = self.render_class_member(
                bounds.x + padding,
                current_y,
                bounds.width - padding * 2.0,
                method,
                theme,
                group,
            );
            current_y += line_height;
        }

        group
    }

    /// Рендерит член класса (поле или метод) с иконкой видимости
    fn render_class_member(
        &self,
        x: f64,
        y: f64,
        _width: f64,
        member: &ClassMember,
        theme: &Theme,
        mut group: Group,
    ) -> Group {
        let icon_radius = 3.0;
        let icon_x = x + icon_radius;
        let icon_y = y + 8.0;

        // Иконка видимости (цветной кружок)
        let (fill_color, stroke_color) = match member.visibility {
            MemberVisibility::Public => ("#84BE84", "#038048"), // зелёный
            MemberVisibility::Private => ("#C82829", "#C80000"), // красный
            MemberVisibility::Protected => ("#FFCC00", "#B38600"), // жёлтый
            MemberVisibility::Package => ("#66CCFF", "#0099CC"), // голубой
        };

        let icon = svg::node::element::Ellipse::new()
            .set("cx", icon_x)
            .set("cy", icon_y)
            .set("rx", icon_radius)
            .set("ry", icon_radius)
            .set("fill", fill_color)
            .set("stroke", stroke_color)
            .set("stroke-width", 1);
        group = group.add(icon);

        // Текст члена
        let text_x = icon_x + icon_radius + 5.0;
        let mut text = svg::node::element::Text::new(&member.text)
            .set("x", text_x)
            .set("y", y + 12.0)
            .set("font-family", theme.font_family.as_str())
            .set("font-size", theme.font_size)
            .set("fill", theme.text_color.to_css());

        // Статический - подчёркивание
        if member.is_static {
            text = text.set("text-decoration", "underline");
        }

        // Абстрактный - курсив
        if member.is_abstract {
            text = text.set("font-style", "italic");
        }

        group.add(text)
    }
}

impl Default for SvgRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for SvgRenderer {
    type Output = String;

    fn render(&self, layout: &LayoutResult, theme: &Theme) -> String {
        let doc = self.create_document(layout, theme);

        // PlantUML оборачивает всё содержимое в один `<g>` с общим шрифтом
        // и режимом подгонки текста. Группа нужна потребителям вывода:
        // по ней наследуется font-family, а lengthAdjust="spacing" задаёт
        // способ подгонки по textLength.
        let mut group = Group::new()
            .set("font-family", theme.font_family.as_str())
            .set("lengthAdjust", "spacing");

        // Сортируем элементы по z-layer для правильного порядка рендеринга
        // Элементы с меньшим z-layer рендерятся первыми (внизу)
        let mut sorted_elements: Vec<_> = layout.elements.iter().collect();
        sorted_elements.sort_by_key(|e| ZLayer::from_element(e));

        // Уникализируем идентификаторы.
        //
        // Layout-движки формируют id из статических частей без счётчика
        // (`msg_Alice_Bob`, `edge_User_Order`, `trans_A_B`), поэтому два
        // сообщения между одной парой участников дают одинаковые id.
        // В SVG атрибут id обязан быть уникальным: дубликаты — невалидный
        // документ, ломающий селекторы, якоря и `<use>`.
        let mut seen: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();

        for element in sorted_elements {
            let unique_id = {
                let base = element.id.as_str();
                let counter = seen.entry(base).or_insert(0);
                *counter += 1;
                if *counter == 1 {
                    base.to_string()
                } else {
                    format!("{base}_{counter}")
                }
            };
            let rendered = self.render_element_with_id(element, theme, &unique_id);
            group = group.add(rendered);
        }

        let doc = doc.add(group);

        // Собственного XML-заголовка PlantUML не пишет ни в одном из
        // эталонов: документ начинается сразу с `<svg>`. Опция оставлена
        // для потребителей, которым заголовок нужен.
        let svg = if self.options.xml_header {
            format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", doc)
        } else {
            doc.to_string()
        };

        // PlantUML проставляет textLength каждому текстовому узлу — это
        // ширина строки в метриках шрифта. Атрибут добавляется одной
        // постобработкой, а не в каждом из методов отрисовки текста:
        // так его невозможно забыть при добавлении нового метода.
        annotate_text_length(&svg)
    }
}

/// Высота заголовка состояния: в эталоне разделитель на 113.297 при
/// верхней границе 87, то есть 26.297.
const STATE_HEADER_HEIGHT: f64 = 26.297;

/// Размер пикселя спрайта, вставленного внутрь подписи.
const SPRITE_INLINE_PIXEL: f64 = 1.0;

/// Отступ после спрайта внутри подписи.
const SPRITE_INLINE_GAP: f64 = 2.0;

/// Разбирает `viewBox` в `(minX, minY, ширина, высота)`.
///
/// Возвращает нули, если атрибут отсутствует или задан неполно.
fn view_box(attrs: &str) -> (f64, f64, f64, f64) {
    let Some(start) = attrs.find("viewBox=\"") else {
        return (0.0, 0.0, 0.0, 0.0);
    };
    let rest = &attrs[start + 9..];
    let Some(end) = rest.find('"') else {
        return (0.0, 0.0, 0.0, 0.0);
    };

    let parts: Vec<f64> = rest[..end]
        .split([' ', ','])
        .filter_map(|value| value.trim().parse().ok())
        .collect();

    if parts.len() == 4 {
        (parts[0], parts[1], parts[2], parts[3])
    } else {
        (0.0, 0.0, 0.0, 0.0)
    }
}

/// Приводит цвета в свойствах элемента к форме PlantUML.
///
/// Возвращает `None`, если менять нечего — тогда вызывающий использует
/// исходный элемент и лишней копии не создаётся.
fn normalize_element_colors(element: &LayoutElement) -> Option<LayoutElement> {
    const COLOR_KEYS: [&str; 3] = ["fill", "stroke", "text-fill"];

    let mut changed = false;
    let mut properties = element.properties.clone();

    for key in COLOR_KEYS {
        if let Some(value) = properties.get(key) {
            let shortened = Color::new(value.as_str()).to_css();
            if &shortened != value {
                properties.insert(key.to_string(), shortened);
                changed = true;
            }
        }
    }

    changed.then(|| LayoutElement {
        id: element.id.clone(),
        bounds: element.bounds,
        element_type: element.element_type.clone(),
        text: element.text.clone(),
        properties,
    })
}

/// Оформление линии: цвет и толщина.
///
/// Оба поля необязательны: если не заданы, берутся значения по умолчанию
/// (цвет темы и толщина 0.5/1 в зависимости от наличия наконечника).
#[derive(Default)]
struct EdgeStyle<'a> {
    color: Option<&'a str>,
    width: Option<f64>,
}

/// Оформление текста: жирность и цвет.
///
/// Сгруппировано, чтобы не плодить параметры: `render_text_weighted`
/// иначе превышает порог clippy по числу аргументов.
struct TextStyle<'a> {
    bold: bool,
    fill: &'a str,
    /// Явный базис строки; `None` — считать от `bounds`.
    baseline: Option<f64>,
}

/// Данные спрайта, разобранные из свойства элемента.
#[derive(Debug, Clone)]
struct SpriteData {
    width: usize,
    height: usize,
    rows: Vec<String>,
    /// Векторное тело: заполнено для `sprite имя <svg ...>...</svg>`.
    svg: Option<plantuml_layout::SvgSprite>,
}

/// Часть подписи: обычный текст либо вставка спрайта.
enum SpritePart {
    Text(String),
    /// Вставка спрайта и её масштаб (по умолчанию 1.0).
    Sprite(SpriteData, f64),
}

/// Разбирает масштаб из модификаторов вставки спрайта.
///
/// PlantUML поддерживает три равнозначные формы (проверено на сервере —
/// все дают одинаковый размер):
///
/// ```text
/// <$имя*3>              — звёздочка
/// <$имя{scale=3}>       — в фигурных скобках
/// <$имя,scale=3,...>    — в списке опций через запятую
/// ```
fn sprite_scale(modifiers: &str) -> f64 {
    // `*N`
    if let Some(rest) = modifiers.strip_prefix('*') {
        let digits: String = rest
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        if let Ok(value) = digits.parse::<f64>() {
            return value;
        }
    }

    // `scale=N` — в любой из форм с разделителями.
    if let Some(index) = modifiers.find("scale=") {
        let rest = &modifiers[index + 6..];
        let digits: String = rest
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        if let Ok(value) = digits.parse::<f64>() {
            return value;
        }
    }

    1.0
}

/// Разбирает свойство `sprites` элемента в таблицу.
///
/// Формат: `имя|ШxВ|строка,строка;имя2|...` — та же раскладка, что
/// заполняет layout. Свойство отсутствует, если в подписях нет вставок.
fn sprite_table(
    properties: &std::collections::HashMap<String, String>,
) -> std::collections::HashMap<String, SpriteData> {
    let mut table = parse_sprite_property(properties.get("sprites"));

    // Векторные спрайты лежат отдельными свойствами `sprite-svg-<имя>`:
    // тело SVG содержит разделители `|`, `;` и `,`, поэтому упаковать
    // их в общую строку нельзя.
    for (key, value) in properties {
        let Some(name) = key.strip_prefix("sprite-svg-") else {
            continue;
        };

        let Some((attrs, body)) = value.split_once('\n') else {
            continue;
        };

        let (width, height) = vector_extent(attrs);
        table.insert(
            name.to_string(),
            SpriteData {
                width,
                height,
                rows: Vec::new(),
                svg: Some(plantuml_layout::SvgSprite {
                    attrs: attrs.to_string(),
                    body: body.to_string(),
                }),
            },
        );
    }

    table
}

/// Размер векторного спрайта по его атрибутам (`viewBox`, затем `width`).
fn vector_extent(attrs: &str) -> (usize, usize) {
    let (_, _, width, height) = view_box(attrs);
    if width > 0.0 && height > 0.0 {
        return (width.round() as usize, height.round() as usize);
    }
    (0, 0)
}

/// Разбирает свойство `sprites` элемента в таблицу.
fn parse_sprite_property(value: Option<&String>) -> std::collections::HashMap<String, SpriteData> {
    let mut table = std::collections::HashMap::new();
    let Some(raw) = value else {
        return table;
    };

    for entry in raw.split(';') {
        let mut fields = entry.split('|');
        let (Some(name), Some(size), Some(rows)) = (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };

        let mut dims = size.split('x');
        let width = dims.next().and_then(|v| v.parse().ok()).unwrap_or(0);
        let height = dims.next().and_then(|v| v.parse().ok()).unwrap_or(0);

        table.insert(
            name.to_string(),
            SpriteData {
                width,
                height,
                rows: rows.split(',').map(str::to_string).collect(),
                svg: None,
            },
        );
    }

    table
}

/// Разбивает подпись на текст и вставки `<$имя>`.
fn split_sprite_references(
    text: &str,
    sprites: &std::collections::HashMap<String, SpriteData>,
) -> Vec<SpritePart> {
    let mut parts = Vec::new();
    let mut rest = text;

    while let Some(start) = rest.find("<$") {
        let after = &rest[start + 2..];
        let Some(end) = after.find('>') else {
            break;
        };

        // Тело вставки: имя плюс необязательные модификаторы
        // (`*N`, `{scale=N}`, `,scale=N`). Имя заканчивается на первом
        // символе-разделителе.
        let body = &after[..end];
        let name_end = body.find(['*', '{', ',']).unwrap_or(body.len());
        let name = &body[..name_end];
        let modifiers = &body[name_end..];

        let Some(data) = sprites.get(name) else {
            // Неизвестный спрайт: оставляем вставку как обычный текст.
            parts.push(SpritePart::Text(rest[..start + 2 + end + 1].to_string()));
            rest = &after[end + 1..];
            continue;
        };

        if start > 0 {
            parts.push(SpritePart::Text(rest[..start].to_string()));
        }
        parts.push(SpritePart::Sprite(data.clone(), sprite_scale(modifiers)));
        rest = &after[end + 1..];
    }

    if !rest.is_empty() {
        parts.push(SpritePart::Text(rest.to_string()));
    }

    parts
}

/// Цвет и прозрачность пикселя спрайта по его шестнадцатеричной цифре.
///
/// Палитра PlantUML — серый градиент от `#F1F1F1` до `#121212`. Значения
/// сняты с эталонного PNG, который сервер встраивает в вывод: тип цвета 6
/// (RGBA), поэтому у первых индексов прозрачность меньше 255.
fn sprite_color(digit: char) -> Option<(&'static str, u8)> {
    const PALETTE: [(&str, u8); 16] = [
        ("#F1F1F1", 0),
        ("#E2E2E2", 60),
        ("#D3D3D3", 123),
        ("#C5C5C5", 186),
        ("#B6B6B6", 238),
        ("#A7A7A7", 255),
        ("#989898", 255),
        ("#898989", 255),
        ("#7A7A7A", 255),
        ("#6B6B6B", 255),
        ("#5D5D5D", 255),
        ("#4E4E4E", 255),
        ("#3F3F3F", 255),
        ("#303030", 255),
        ("#212121", 255),
        ("#121212", 255),
    ];

    let index = digit.to_digit(16)? as usize;
    PALETTE.get(index).copied()
}

/// Величина скоса объёмной рамки узла deployment.
const NODE_3D_OFFSET: f64 = 10.0;

/// Отступ базовой линии заголовка узла от верха основного прямоугольника.
const NODE_TITLE_BASELINE: f64 = 16.0;

/// Смещение тени от фигуры (`skinparam shadowing true`).
/// Форматирует координату для атрибута SVG.
///
/// Убирает хвостовые нули: PlantUML пишет `26.5`, а не `26.500000`.
fn fmt(value: f64) -> String {
    let text = format!("{value:.3}");
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() || trimmed == "-" {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

// === Геометрия фигур участников sequence ===
//
// Все числа сняты с эталона `sequence_participants` (PlantUML 1.2026.9beta4):
// верхняя полоса подписи 55..85.297, нижняя 249.961..280.258.

/// Радиус головы стик-фигуры.
const ACTOR_HEAD_RADIUS: f64 = 8.0;

/// Полуразнос рук и ног стик-фигуры.
const ACTOR_LIMB_SPREAD: f64 = 13.0;

/// Смещения стик-фигуры от верха элемента (верхний блок).
const ACTOR_WAIST_OFFSET: f64 = 43.0;
const ACTOR_ARMS_OFFSET: f64 = 24.0;
const ACTOR_FEET_OFFSET: f64 = 58.0;

/// Смещения стик-фигуры от верха элемента (нижний блок, зеркально).
const ACTOR_FOOTER_HEAD_CY: f64 = 24.797;
const ACTOR_FOOTER_NECK: f64 = 32.797;
const ACTOR_FOOTER_WAIST: f64 = 59.797;
const ACTOR_FOOTER_ARMS: f64 = 40.797;
const ACTOR_FOOTER_FEET: f64 = 74.797;

/// Полуширина цилиндра базы данных.
const DATABASE_RX: f64 = 18.0;

/// Полувысота крышки цилиндра.
const DATABASE_CAP_RY: f64 = 10.0;

/// Высота тела цилиндра между крышками.
const DATABASE_BODY_HEIGHT: f64 = 26.0;

/// Смещения крышек цилиндра в нижнем блоке.
const DATABASE_FOOTER_TOP_CAP: f64 = 36.0;
const DATABASE_FOOTER_BOTTOM_CAP: f64 = 10.0;

/// Радиус кружка у boundary/control/entity.
const PARTICIPANT_ICON_RADIUS: f64 = 12.0;

/// Отступ подписи от низа верхней полосы.
const PARTICIPANT_LABEL_BASELINE_GAP: f64 = 2.302;

/// Отступ подписи от верха нижней полосы.
const PARTICIPANT_FOOTER_LABEL_BASELINE: f64 = 12.995;

/// Вынос скобки boundary влево от центра.
///
/// Эталон `sequence_participants`: вертикаль скобки на 234.807 при линии
/// жизни 255.307, то есть на 20.5 левее. Прежние 29 ставили её на 226.3,
/// а перекладина всё равно доходила до кружка — скобка выглядела длиннее
/// эталонной.
const BOUNDARY_BRACKET_OFFSET: f64 = 20.5;

/// Смещение центра кружка control от верха элемента.
const CONTROL_CIRCLE_OFFSET: f64 = 17.0;

/// Смещение стрелки control от верха элемента.
const CONTROL_ARROW_MID: f64 = 5.0;

/// Смещение стрелки control в нижнем блоке от низа элемента.
const CONTROL_FOOTER_ARROW_MID: f64 = 24.0;

/// Размеры стрелки control.
const CONTROL_ARROW_BACK: f64 = 4.0;
const CONTROL_ARROW_TIP: f64 = 2.0;
const CONTROL_ARROW_HALF: f64 = 5.0;

/// Смещение подчёркивания entity от верха элемента.
const ENTITY_UNDERLINE_OFFSET: f64 = 26.0;

/// Зазор между кружком entity и подчёркиванием в нижнем блоке.
const ENTITY_FOOTER_GAP: f64 = 2.0;

/// Смещение тени от фигуры (`skinparam shadowing true`).
const SHADOW_OFFSET: f64 = 3.0;

/// Цвет тела класса в эталоне PlantUML.
const CLASS_BODY_FILL: &str = "#F1F1F1";

/// Округляет до трёх знаков: PlantUML пишет `33.667`, а не `33.6670001`.
fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

/// Извлекает значение `font-size` из строки тега.
fn extract_font_size(tag: &str) -> Option<f64> {
    let marker = "font-size=\"";
    let start = tag.find(marker)? + marker.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    rest[..end].parse().ok()
}

/// Добавляет атрибут `textLength` каждому текстовому узлу, где его нет.
///
/// PlantUML проставляет его всегда: это ширина строки в метриках шрифта.
/// Разбор разметки простым поиском достаточен, потому что рендерер сам
/// формирует теги `<text>` и не вставляет экранированные `>` в атрибуты.
fn annotate_text_length(svg: &str) -> String {
    let mut out = String::with_capacity(svg.len() + svg.len() / 20);
    let mut rest = svg;

    while let Some(open) = rest.find("<text") {
        out.push_str(&rest[..open]);
        let after = &rest[open..];

        let Some(tag_end) = after.find('>') else {
            out.push_str(after);
            return out;
        };
        let tag = &after[..tag_end];

        // Уже есть — оставляем как есть
        if tag.contains("textLength") {
            out.push_str(&after[..=tag_end]);
            rest = &after[tag_end + 1..];
            continue;
        }

        let content_start = tag_end + 1;
        let content = &after[content_start..];
        let Some(close) = content.find("</text>") else {
            out.push_str(after);
            return out;
        };

        let label = content[..close].trim();
        let font_size = extract_font_size(tag).unwrap_or(13.0);
        // Полужирный текст шире обычного: PlantUML считает его ширину по
        // своей таблице, и в эталонах жирные подписи длиннее на 8–14 %.
        // Без этой поправки `textLength` жирной подписи совпадал с обычной
        // и браузер сжимал её.
        let measurer = plantuml_layout::text::TextMeasurer::default();
        let measure = |line: &str| {
            if tag.contains("font-weight") {
                measurer.width_bold(line, font_size)
            } else {
                measurer.width(line, font_size)
            }
        };
        // Многострочный текст: берём самую длинную строку
        let measured = label.lines().map(measure).fold(0.0_f64, f64::max);

        out.push_str(tag);
        out.push_str(&format!(" textLength=\"{}\"", round3(measured)));
        out.push('>');
        out.push_str(&content[..close]);
        out.push_str("</text>");

        rest = &content[close + "</text>".len()..];
    }

    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use plantuml_layout::ElementType;

    #[test]
    fn test_render_empty() {
        let renderer = SvgRenderer::new();
        let layout = LayoutResult::empty();
        let theme = Theme::default();

        let svg = renderer.render(&layout, &theme);
        assert!(svg.contains("<?xml"));
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn test_render_rectangle() {
        let renderer = SvgRenderer::new();
        let layout = LayoutResult {
            elements: vec![LayoutElement::new(
                "test",
                Rect::new(10.0, 10.0, 100.0, 50.0),
                ElementType::Rectangle {
                    label: "Hello".to_string(),
                    corner_radius: 5.0,
                },
            )],
            bounds: Rect::new(0.0, 0.0, 120.0, 70.0),
        };
        let theme = Theme::default();

        let svg = renderer.render(&layout, &theme);
        assert!(svg.contains("<rect"));
        assert!(svg.contains("Hello"));
    }
}
