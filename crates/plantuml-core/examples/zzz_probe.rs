fn main() {
    let src = std::fs::read_to_string("tests/golden/cases/activity_branch.puml").unwrap();
    let svg = plantuml_core::render(&src, &Default::default()).unwrap();
    std::fs::write("/tmp/ab.our.svg", &svg).unwrap();
    println!("наш SVG {} байт", svg.len());
}
