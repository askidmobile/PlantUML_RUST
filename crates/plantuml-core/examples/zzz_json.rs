fn main() {
    let src = std::fs::read_to_string("/tmp/pg/cases/JSON_Диаграмма.puml").unwrap();
    let svg = plantuml_core::render(&src, &Default::default()).unwrap();
    std::fs::write("/tmp/pg/our_json.svg", &svg).unwrap();
    println!(
        "{}",
        svg.split("viewBox=\"")
            .nth(1)
            .and_then(|x| x.split('"').next())
            .unwrap_or("?")
    );
}
