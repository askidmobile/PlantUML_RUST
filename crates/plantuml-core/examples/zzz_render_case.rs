//! Временный прогон одного golden-кейса в SVG (для сверки с эталоном).
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let case = &args[1];
    let src = fs::read_to_string(format!("tests/golden/cases/{case}.puml")).unwrap();
    let out = plantuml_core::render(&src, &plantuml_core::RenderOptions::default()).unwrap();
    fs::write(format!("/tmp/vis/{case}.our.svg"), &out).unwrap();
    println!("{case}: {} bytes", out.len());
}
