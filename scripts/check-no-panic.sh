#!/bin/bash
# ============================================================================
# check-no-panic.sh — в библиотечном коде не должно быть паникующих операций
# ============================================================================
#
# Зачем это нужно
# ---------------
#
# В release профиль собирается с `panic = "abort"` (Cargo.toml, раздел
# [profile.release]). Это значит:
#
#   * паника в коде НЕотличима от аварийной остановки процесса;
#   * под wasm32 паника превращается в ловушку модуля, и весь снимок
#     приложения в браузере падает вместе с ней;
#   * `catch_unwind` не сработает — раскрутки нет.
#
# Для библиотеки, которую встраивают в чужие приложения, это особенно
# неприятно: одна ошибка разбора диаграммы роняет всё приложение целиком.
#
# Что проверяем
# -------------
#
# Ни в одном файле `crates/*/src/**` — до секции `#[cfg(test)]`, то есть
# вне тестов — не должно встречаться:
#
#   unwrap()   — съём значения без проверки
#   expect(    — то же самое с сообщением
#   panic!     — явная паника
#   todo!      — заглушка «не дописано», в релизе это ошибка
#   unimplemented! — то же
#
# Файлы `benches/`, `examples/` и `tests/` НЕ проверяются: там паника
# допустима и даже полезна для диагностики.
#
# Историческая справка
# --------------------
#
# До этой проверки в парсере было одиннадцать таких мест:
#
#   salt.rs, mindmap.rs, wbs.rs     stack.pop().unwrap()
#   class.rs                        package_stack.last_mut().unwrap()
#   json.rs, yaml.rs                pair.into_inner().next().unwrap()
#
# Все они были защищены инвариантами: съём со стека — проверкой `len() > 1`
# рядом, а правила грамматики pest гарантируют непустое тело `json_value`,
# `yaml_value`, `yaml_scalar` и `yaml_inline_value`, так что пустой пары там
# быть не может. То есть достижимой паники не было: 20 000 мутационных
# прогонов это подтвердили.
#
# Правка — страховка от будущих правок грамматики или раскладки, после
# которых инвариант перестанет держаться, а паника станет реальной.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

echo "============================================"
echo "  Проверка паникующих операций в библиотеке"
echo "============================================"

python3 - "$@" <<'PYEOF'
import pathlib
import re
import sys

# Операции, которые в библиотечном коде недопустимы.
PATTERNS = {
    "unwrap()": re.compile(r"\.unwrap\(\)"),
    "expect(": re.compile(r"\.expect\("),
    "panic!": re.compile(r"\bpanic!"),
    "todo!": re.compile(r"\btodo!"),
    "unimplemented!": re.compile(r"\bunimplemented!"),
    "unreachable!": re.compile(r"\bunreachable!"),
}

offenders = []

for path in sorted(pathlib.Path("crates").glob("*/src/**/*.rs")):
    text = path.read_text(encoding="utf-8")
    lines = text.split("\n")

    # Код после `#[cfg(test)]` — тестовый, панить в нём можно.
    cut = len(lines)
    for index, line in enumerate(lines):
        if "#[cfg(test)]" in line:
            cut = index
            break

    in_doc_comment = False
    for number, line in enumerate(lines[:cut], 1):
        stripped = line.strip()

        # Многострочные комментарии-примеры `///` и `//!` пропускаем:
        # там `.unwrap()` входит в состав примера кода.
        if stripped.startswith("//"):
            continue

        for name, pattern in PATTERNS.items():
            if pattern.search(line):
                offenders.append((str(path), number, name, stripped[:90]))

if not offenders:
    print("  паникующих операций в библиотечном коде нет")
    sys.exit(0)

print(f"  НАЙДЕНО {len(offenders)} мест с паникующими операциями:\n")
for path, number, name, code in offenders:
    print(f"    {path}:{number}  [{name}]")
    print(f"      {code}\n")

print("  Что делать:")
print("   * заменить unwrap() на let ... else или ok_or_else с ParseError;")
print("   * заменить expect() на вариант с сообщением об ошибке;")
print("   * todo!/unimplemented! в релизе считать дефектом.")
print()
sys.exit(1)
PYEOF

echo "Проверка пройдена."