fn main() {
    let src = std::fs::read_to_string("tests/golden/cases/component_basic.puml").unwrap();
    if let Ok(svg) = plantuml_core::render(&src, &Default::default()) {
        std::fs::write("/tmp/component_basic.our.svg", svg).unwrap();
    }
}
