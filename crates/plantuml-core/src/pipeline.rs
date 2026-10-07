//! Pipeline рендеринга диаграмм

use std::path::Path;

use crate::{Error, RenderOptions, Result};
use plantuml_ast::diagram::DiagramType;
use plantuml_ast::Diagram;
use plantuml_layout::{
    ActivityLayoutEngine, ClassLayoutEngine, ComponentLayoutEngine, ErLayoutEngine,
    GanttLayoutEngine, JsonLayoutEngine, LayoutConfig, LayoutResult, MindMapLayoutEngine,
    NetworkLayoutEngine, ObjectLayoutEngine, SaltLayoutEngine, SequenceLayoutEngine,
    StateLayoutEngine, TimingLayoutEngine, UseCaseLayoutEngine, WbsLayoutEngine, YamlLayoutEngine,
};
use plantuml_preprocessor::{FsFileResolver, Preprocessor};
use plantuml_renderer::{Renderer, SvgRenderer};
use plantuml_themes::Theme;

/// Выполняет полный pipeline рендеринга
pub fn render_pipeline(source: &str, options: &RenderOptions) -> Result<String> {
    // Проверка на пустой исходник
    let source = source.trim();
    if source.is_empty() {
        return Err(Error::EmptySource);
    }

    // 1. Препроцессинг. Возвращает и текст, и тему, разобранную из
    //    `!theme` и `skinparam` — раньше она терялась в препроцессоре.
    let (processed, theme) = preprocess(source)?;

    // 2. Парсинг
    let diagram = parse(&processed)?;

    // 3. Layout
    let layout = layout(&diagram, options)?;

    // 4. Рендеринг с темой из исходника
    let svg = render_svg(&layout, options, &theme, svg_diagram_type(&diagram))?;

    Ok(svg)
}

/// Выполняет полный pipeline с поддержкой !include
pub fn render_pipeline_with_includes(
    source: &str,
    base_path: &Path,
    options: &RenderOptions,
) -> Result<String> {
    // Проверка на пустой исходник
    let source = source.trim();
    if source.is_empty() {
        return Err(Error::EmptySource);
    }

    // 1. Препроцессинг с поддержкой файлов
    let (processed, theme) = preprocess_with_includes(source, base_path)?;

    // 2. Парсинг
    let diagram = parse(&processed)?;

    // 3. Layout
    let layout = layout(&diagram, options)?;

    // 4. Рендеринг с темой из исходника
    let svg = render_svg(&layout, options, &theme, svg_diagram_type(&diagram))?;

    Ok(svg)
}

/// Имя типа диаграммы для атрибута `data-diagram-type`.
///
/// Имена не совпадают с нашими один в один: PlantUML помечает component,
/// deployment и usecase как `DESCRIPTION`, а ER и object — как `CLASS`.
fn svg_diagram_type(diagram: &Diagram) -> &'static str {
    match diagram.diagram_type() {
        DiagramType::Sequence => "SEQUENCE",
        DiagramType::Class | DiagramType::Er | DiagramType::Object => "CLASS",
        DiagramType::Activity => "ACTIVITY",
        DiagramType::State => "STATE",
        DiagramType::Timing => "TIMING",
        DiagramType::Component | DiagramType::Deployment | DiagramType::UseCase => "DESCRIPTION",
        DiagramType::Gantt => "GANTT",
        DiagramType::MindMap => "MINDMAP",
        DiagramType::Wbs => "WBS",
        DiagramType::Json => "JSON",
        DiagramType::Yaml => "YAML",
        DiagramType::Network => "NWDIAG",
        DiagramType::Salt => "SALT",
        DiagramType::Archimate => "ARCHIMATE",
    }
}

/// Этап препроцессинга: возвращает обработанный текст и тему из исходника.
fn preprocess(source: &str) -> Result<(String, Theme)> {
    let preprocessor = Preprocessor::new();
    preprocessor
        .process_with_theme(source)
        .map_err(|e: plantuml_preprocessor::PreprocessError| Error::Preprocess(e.to_string()))
}

/// Этап препроцессинга с поддержкой !include
/// Этап препроцессинга с `!include`: возвращает текст и тему из исходника.
fn preprocess_with_includes(source: &str, base_path: &Path) -> Result<(String, Theme)> {
    let resolver = FsFileResolver::new(base_path);
    let preprocessor = Preprocessor::with_resolver(resolver);
    preprocessor
        .process_with_theme(source)
        .map_err(|e: plantuml_preprocessor::PreprocessError| Error::Preprocess(e.to_string()))
}

/// Этап парсинга
fn parse(source: &str) -> Result<Diagram> {
    plantuml_parser::parse(source)
        .map_err(|e: plantuml_parser::ParseError| Error::Parse(e.to_string()))
}

/// Этап layout
fn layout(diagram: &Diagram, _options: &RenderOptions) -> Result<LayoutResult> {
    let _config = LayoutConfig::default();

    // Выбираем layout engine в зависимости от типа диаграммы
    match diagram {
        Diagram::Sequence(seq) => {
            // Используем SequenceLayoutEngine для sequence diagrams
            let engine = SequenceLayoutEngine::new();
            Ok(engine.layout(seq))
        }
        Diagram::Class(class) => {
            // Используем ClassLayoutEngine для class diagrams (Sugiyama algorithm)
            let engine = ClassLayoutEngine::new();
            Ok(engine.layout_diagram(class))
        }
        Diagram::Activity(act) => {
            // Используем ActivityLayoutEngine для activity diagrams
            let engine = ActivityLayoutEngine::new();
            Ok(engine.layout(act))
        }
        Diagram::State(state) => {
            // Используем StateLayoutEngine для state diagrams
            let engine = StateLayoutEngine::new();
            Ok(engine.layout(state))
        }
        Diagram::Component(comp) => {
            // Используем ComponentLayoutEngine для component diagrams
            let engine = ComponentLayoutEngine::new();
            Ok(engine.layout(comp))
        }
        Diagram::UseCase(uc) => {
            // Используем UseCaseLayoutEngine для use case diagrams
            let engine = UseCaseLayoutEngine::new();
            Ok(engine.layout(uc))
        }
        Diagram::Deployment(dep) => {
            // Deployment использует ComponentLayoutEngine (та же структура)
            let engine = ComponentLayoutEngine::new();
            Ok(engine.layout(dep))
        }
        Diagram::Object(obj) => {
            // Используем ObjectLayoutEngine для object diagrams
            let engine = ObjectLayoutEngine::new();
            Ok(engine.layout(obj))
        }
        Diagram::Timing(timing) => {
            // Используем TimingLayoutEngine для timing diagrams
            let engine = TimingLayoutEngine::new();
            Ok(engine.layout(timing))
        }
        Diagram::Gantt(gantt) => {
            // Используем GanttLayoutEngine для gantt diagrams
            let engine = GanttLayoutEngine::new();
            Ok(engine.layout(gantt))
        }
        Diagram::MindMap(mindmap) => {
            // Используем MindMapLayoutEngine для mindmap diagrams
            let engine = MindMapLayoutEngine::new();
            Ok(engine.layout(mindmap))
        }
        Diagram::Wbs(wbs) => {
            // Используем WbsLayoutEngine для wbs diagrams
            let engine = WbsLayoutEngine::new();
            Ok(engine.layout(wbs))
        }
        Diagram::Json(json) => {
            // Используем JsonLayoutEngine для json diagrams
            use plantuml_layout::traits::LayoutEngine as _;
            let engine = JsonLayoutEngine::new();
            Ok(engine.layout(json, &_config))
        }
        Diagram::Yaml(yaml) => {
            // Используем YamlLayoutEngine для yaml diagrams
            use plantuml_layout::traits::LayoutEngine as _;
            let engine = YamlLayoutEngine::new();
            Ok(engine.layout(yaml, &_config))
        }
        Diagram::Er(er) => {
            // Используем ErLayoutEngine для ER diagrams
            use plantuml_layout::traits::LayoutEngine as _;
            let engine = ErLayoutEngine::new();
            Ok(engine.layout(er, &_config))
        }
        Diagram::Network(net) => {
            // Используем NetworkLayoutEngine для network diagrams
            use plantuml_layout::traits::LayoutEngine as _;
            let engine = NetworkLayoutEngine::new();
            Ok(engine.layout(net, &_config))
        }
        Diagram::Salt(salt) => {
            // Используем SaltLayoutEngine для salt diagrams
            use plantuml_layout::traits::LayoutEngine as _;
            let engine = SaltLayoutEngine::new();
            Ok(engine.layout(salt, &_config))
        }
        Diagram::Archimate(arch) => {
            // Archimate использует ComponentLayoutEngine
            let engine = ComponentLayoutEngine::new();
            Ok(engine.layout(arch))
        }
    }
}

/// Этап SVG рендеринга
/// Этап SVG-рендеринга.
///
/// `source_theme` — тема, разобранная из самого исходника (`!theme`,
/// `skinparam`). Она имеет приоритет над темой из `RenderOptions`: настройки
/// внутри диаграммы в PlantUML переопределяют внешние.
fn render_svg(
    layout: &LayoutResult,
    options: &RenderOptions,
    source_theme: &Theme,
    diagram_type: &str,
) -> Result<String> {
    // Фон холста: явная опция важнее темы, но и тема из исходника
    // (`skinparam backgroundColor`, `!theme`) должна доходить до
    // рендерера. Раньше сюда попадала только опция, поэтому `skinparam
    // backgroundColor` на вывод не влиял — тест ловил это лишь случайно:
    // варианты различались блоком `<defs>` с маркерами.
    let background_color = options.background_color.clone().or_else(|| {
        let from_source = source_theme.background_color.to_css();
        if from_source == Theme::default().background_color.to_css() {
            None
        } else {
            Some(from_source)
        }
    });

    let render_options = plantuml_renderer::RenderOptions {
        xml_header: options.xml_header,
        scale: options.scale,
        background_color,
        diagram_type: Some(diagram_type.to_string()),
    };

    // Тема из исходника накладывается поверх темы из опций.
    //
    // Сравниваем с темой по умолчанию целиком, а не по нескольким полям:
    // иначе изменение, не попавшее в список проверяемых (например
    // `FontColor`, влияющий на `text_color`), отбрасывалось, и `skinparam`
    // не действовал.
    let mut theme = options.theme.clone();
    let default_theme = Theme::default();
    let source_is_default = source_theme.name == default_theme.name
        && source_theme.background_color.to_css() == default_theme.background_color.to_css()
        && source_theme.node_background.to_css() == default_theme.node_background.to_css()
        && source_theme.node_border.to_css() == default_theme.node_border.to_css()
        && source_theme.text_color.to_css() == default_theme.text_color.to_css()
        && source_theme.arrow_color.to_css() == default_theme.arrow_color.to_css()
        && source_theme.font_family == default_theme.font_family
        && (source_theme.font_size - default_theme.font_size).abs() < f64::EPSILON
        && (source_theme.line_width - default_theme.line_width).abs() < f64::EPSILON
        && (source_theme.corner_radius - default_theme.corner_radius).abs() < f64::EPSILON
        && source_theme.shadow == default_theme.shadow
        && source_theme.handwritten == default_theme.handwritten;
    if !source_is_default {
        theme = source_theme.clone();
    }

    let renderer = SvgRenderer::with_options(render_options);

    Ok(renderer.render(layout, &theme))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `skinparam` и `!theme` обязаны влиять на вывод.
    ///
    /// Регрессия: препроцессор возвращал только `String`, поэтому тема,
    /// разобранная из `skinparam` и `!theme`, терялась — все варианты давали
    /// байт-идентичный результат с базовым.
    #[test]
    fn test_skinparam_affects_output() {
        let base =
            render_pipeline("@startuml\nA -> B\n@enduml", &RenderOptions::default()).unwrap();

        let cases = [
            (
                "monochrome",
                "@startuml\nskinparam monochrome true\nA -> B\n@enduml",
            ),
            (
                "backgroundColor",
                "@startuml\nskinparam backgroundColor #FF0000\nA -> B\n@enduml",
            ),
            (
                "defaultFontName",
                "@startuml\nskinparam defaultFontName ComicSansMS\nA -> B\n@enduml",
            ),
            (
                "FontColor в блоке",
                "@startuml\nskinparam rectangle {\n FontColor #FF0000\n}\nA -> B\n@enduml",
            ),
            (
                "BorderColor в блоке",
                "@startuml\nskinparam rectangle {\n BorderColor #00FF00\n}\nA -> B\n@enduml",
            ),
            ("!theme dark", "@startuml\n!theme dark\nA -> B\n@enduml"),
        ];

        for (name, source) in cases {
            let svg = render_pipeline(source, &RenderOptions::default())
                .unwrap_or_else(|e| panic!("{name}: ошибка рендера: {e}"));
            assert_ne!(
                svg, base,
                "{name}: вывод совпал с базовым — настройка не применилась"
            );
        }
    }

    /// `monochrome true` делает фон и границы чёрно-белыми.
    #[test]
    fn test_monochrome_applies_colors() {
        let svg = render_pipeline(
            "@startuml\nskinparam monochrome true\nA -> B\n@enduml",
            &RenderOptions::default(),
        )
        .unwrap();

        // PlantUML записывает цвета в сокращённой форме: `#FFFFFF` → `#FFF`,
        // `#000000` → `#000`. Проверено на сервере: заданный в полной
        // записи цвет возвращается сокращённым.
        assert!(svg.contains("#FFF"), "нет белого фона при monochrome");
        assert!(svg.contains("#000"), "нет чёрных границ при monochrome");
    }

    /// Тема из исходника переопределяет тему из опций.
    #[test]
    fn test_source_theme_overrides_options() {
        let options = RenderOptions::new().with_theme(Theme::cerulean());
        let svg = render_pipeline(
            "@startuml\nskinparam monochrome true\nA -> B\n@enduml",
            &options,
        )
        .unwrap();

        // monochrome в исходнике должен победить тему cerulean из опций
        assert!(
            svg.contains("#FFFFFF"),
            "тема из исходника не переопределила тему из опций"
        );
    }

    #[test]
    fn test_pipeline_basic() {
        let source = "@startuml\nAlice -> Bob\n@enduml";
        let result = render_pipeline(source, &RenderOptions::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_pipeline_empty_source() {
        let result = render_pipeline("", &RenderOptions::default());
        assert!(matches!(result, Err(Error::EmptySource)));
    }

    #[test]
    fn test_pipeline_whitespace_only() {
        let result = render_pipeline("   \n  \t  ", &RenderOptions::default());
        assert!(matches!(result, Err(Error::EmptySource)));
    }

    #[test]
    fn test_pipeline_box_sequence() {
        let source = r#"@startuml
box "Фронтенд" #LightBlue
    participant "React App" as React
    participant "Redux Store" as Redux
end box

box "Бэкенд" #LightGreen
    participant "API Gateway" as API
    participant "Auth Service" as Auth
    participant "User Service" as User
end box

React -> Redux: dispatch(login)
Redux -> API: POST /auth/login
API -> Auth: validateCredentials
Auth -> User: getUserById
User --> Auth: user data
Auth --> API: JWT token
API --> Redux: { token, user }
Redux --> React: state updated
@enduml"#;
        let result = render_pipeline(source, &RenderOptions::default());
        assert!(result.is_ok(), "Pipeline error: {:?}", result.err());
    }
}
