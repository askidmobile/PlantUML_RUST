use plantuml_core::{render_with_includes, RenderOptions};
use std::path::Path;
fn main() {
    println!("=== 1. Циклический !include ===");
    let r = std::panic::catch_unwind(|| {
        render_with_includes(
            "@startuml\n!include \"b.puml\"\nA -> B\n@enduml",
            Path::new("/tmp/puml_cycle"),
            &RenderOptions::default(),
        )
    });
    match r {
        Ok(Ok(s)) => println!("OK: {} байт", s.len()),
        Ok(Err(e)) => println!("ОШИБКА (правильно): {}", e),
        Err(_) => println!("PANIC — баг не исправлен"),
    }

    println!("\n=== 2. stdlib-включения ===");
    for (n, s) in [
        (
            "C4/C4_Context",
            "@startuml\n!include <C4/C4_Context>\nA -> B\n@enduml",
        ),
        (
            "office/Users/user",
            "@startuml\n!include <office/Users/user>\nA -> B\n@enduml",
        ),
        (
            "несуществующий",
            "@startuml\n!include <nope/nope>\nA -> B\n@enduml",
        ),
    ] {
        match render_with_includes(s, Path::new("/tmp/puml_cycle"), &RenderOptions::default()) {
            Ok(v) => println!("  OK   | {:<20} | {} байт", n, v.len()),
            Err(e) => println!("  FAIL | {:<20} | {}", n, e),
        }
    }
}
