#!/usr/bin/env python3
"""
measure-activity.py — надёжный замер раскладки activity против PlantUML.

Зачем этот скрипт
=================

Позиция ветви условия задаётся шириной её содержимого, и правило выведено
замерами на сервере:

    отступ = (ширина_ветви_then + ширина_ветви_else) / 4 + 10

Формула совпала с эталоном до сотых на независимых наборах (ширины
88.32/85.37 -> 53.42; 137.00/203.36 -> 95.09).

Но чтобы её further уточнять, нужен замер, который не врёт. Первая версия
обхода брала блоки по порядку следования `y`, и при нескольких действиях
в ветви второй блок принадлежал НЕ ТОЙ ветви. Из-за этого появился ложный
вывод «каждый дополнительный элемент сдвигает отступ примерно на 10».
На самом деле число элементов не влияет ни на что.

Этот скрипт сопоставляет блок с подписью по близости горизонтальных
центров, поэтому разные количества блоков в ветвях измеряются верно.

Использование
=============

    python3 scripts/measure-activity.py

Печатает таблицу: число действий в ветвях, ширины, измеренный отступ,
предсказание формулы и расхождение. Расхождение, отличное от нуля,
означает, что правило неполно — это повод для нового замера, а не для
правки движка наугад.
"""

from __future__ import annotations

import re
import sys

sys.path.insert(0, "tests/golden")
import fetch_references as fr  # noqa: E402

# Подписи служебных элементов: условия, метки ветвей и петли — они не
# являются блоками и в расчёте ширины не участвуют.
SERVICE_LABELS = {"да", "нет", "у?"}


def measure(source: str) -> dict[str, tuple[float, float, float]]:
    """Возвращает подпись -> (x, ширина, центр) для блоков диаграммы."""
    svg = fr.fetch(source)

    items = [
        (float(m.group(2)), float(m.group(1)), float(m.group(3)), m.group(4))
        for m in re.finditer(
            r'<text[^>]*x="([\d.]+)"[^>]*y="([\d.]+)"[^>]*textLength="([\d.]+)"[^>]*>([^<]*)</text>',
            svg,
        )
    ]
    raw = [
        (float(m.group(2)), float(m.group(1)), float(m.group(3)), float(m.group(4)))
        for m in re.finditer(
            r'<rect[^>]*x="([\d.]+)"[^>]*y="([\d.]+)"[^>]*width="([\d.]+)"[^>]*height="([\d.]+)',
            svg,
        )
    ]
    # Настоящий блок действия шире 40 и выше 15.
    boxes = [b for b in raw if b[2] > 40 and b[3] > 15]

    found: dict[str, tuple[float, float, float]] = {}
    for (text_y, text_x, text_w, label) in items:
        text_center = text_x + text_w / 2
        best: tuple[float, float, float, float] | None = None
        for (box_y, box_x, box_w, box_h) in boxes:
            if not (box_y <= text_y <= box_y + box_h):
                continue
            distance = abs((box_x + box_w / 2) - text_center)
            if best is None or distance < best[0]:
                best = (distance, box_x, box_w, box_x + box_w / 2)
        # 60 — допуск: подпись может выходить за рамку на ширину полей.
        if best and best[0] < 60:
            found.setdefault(label, (best[1], best[2], best[3]))
    return found


TEMPLATE = (
    "@startuml\nstart\n:Верх;\nif (у?) then (да)\n{then_body}\nelse\n{else_body}\nendif\nstop\n@enduml"
)

LOOP_NARROW = "  while (д?) is (да)\n    :Узкое тело;\n  endwhile (нет)"
LOOP_WIDE = "  while (д?) is (да)\n    :Очень длинное тело цикла здесь шире;\n  endwhile (нет)"

CASES = [
    ("одно действие", "  :Короткое;", "  :Ветка else;", ["Короткое"], ["Ветка else"]),
    ("два действия", "  :Короткое;\n  :Второе;", "  :Ветка else;", ["Короткое", "Второе"], ["Ветка else"]),
    (
        "три действия",
        "  :Короткое;\n  :Второе;\n  :Третье;",
        "  :Ветка else;",
        ["Короткое", "Второе", "Третье"],
        ["Ветка else"],
    ),
    (
        "широкая ветвь else",
        "  :Короткое;",
        "  :Очень длинная подпись в ветке else;\n  :Второе;",
        ["Короткое"],
        ["Очень длинная подпись в ветке else", "Второе"],
    ),
    ("цикл с узким телом", f"  :Короткое;\n{LOOP_NARROW}", "  :Ветка else;", ["Короткое", "Узкое тело"], ["Ветка else"]),
    ("цикл с широким телом", f"  :Короткое;\n{LOOP_WIDE}", "  :Ветка else;", ["Короткое", "Очень длинное тело цикла здесь шире"], ["Ветка else"]),
    (
        "два цикла с широким телом",
        f"  :Короткое;\n{LOOP_WIDE}\n{LOOP_WIDE}",
        "  :Ветка else;",
        ["Короткое", "Очень длинное тело цикла здесь шире"],
        ["Ветка else"],
    ),
]


def main() -> int:
    print(f"{'случай':<28} {'w_then':>8} {'w_else':>8} {'измерен':>9} {'формула':>9} {'разница':>9}")
    print("-" * 76)

    failures = 0
    for name, then_body, else_body, then_labels, else_labels in CASES:
        source = TEMPLATE.format(then_body=then_body, else_body=else_body)
        try:
            blocks = measure(source)
        except Exception as error:  # noqa: BLE001 — сервер может ответить 509
            print(f"{name:<28} ошибка запроса: {error}")
            failures += 1
            continue

        top = blocks.get("Верх")
        else_blocks = [blocks[label] for label in else_labels if label in blocks]
        then_blocks = [blocks[label] for label in then_labels if label in blocks]
        if top is None or not else_blocks or not then_blocks:
            print(f"{name:<28} не все блоки найдены: {sorted(blocks)}")
            failures += 1
            continue

        width_then = max(block[1] for block in then_blocks)
        width_else = max(block[1] for block in else_blocks)
        measured = top[2] - then_blocks[0][2]
        predicted = (width_then + width_else) / 4 + 10
        delta = measured - predicted

        print(
            f"{name:<28} {width_then:>8.2f} {width_else:>8.2f} "
            f"{measured:>9.2f} {predicted:>9.2f} {delta:>+9.2f}"
        )
        if abs(delta) > 0.5:
            failures += 1

    print()
    if failures:
        print(
            "Расхождение больше 0.5 px означает, что правило неполно.\n"
            "Это повод поставить новый случай в tests/golden/cases и\n"
            "зафиксировать разрыв в baseline, а не править движок наугад."
        )
    else:
        print("Формула подтверждена на всех случаях.")
    return 0


if __name__ == "__main__":
    sys.exit(main())