#!/usr/bin/env python3
"""
extract-text-corpus.py — корпус «текст → ширина» из эталонов PlantUML.

Зачем
=====

Точность измерения текста задаёт ВСЁ остальное: ширины коробок, расстояния
между участниками, ширину рамок фрагментов. Ошибка в 2% на подписи
превращается в десятки пикселей на большой диаграмме.

PlantUML пишет на каждом `<text>` атрибут `textLength` — свою измеренную
ширину строки. Это готовый эталон: берём подпись, кегль, начертание и
ширину, и получаем материал для сверки с нашей таблицей глифов.

Результат — TSV со столбцами:

    подпись <TAB> кегль <TAB> полужирный(0/1) <TAB> ширина

Его читает `crates/plantuml-core/examples/text_calibration.rs`, который
считает наши ширины и печатает расхождения по кеглям и начертаниям.

Использование
=============

    python3 scripts/extract-text-corpus.py                 # в /tmp/textcorpus.tsv
    python3 scripts/extract-text-corpus.py corpus.tsv      # в свой файл

Повторы одной и той же пары схлопываются по медиане: PlantUML может
округлить ширину по-разному в зависимости от соседних элементов.
"""

from __future__ import annotations

import glob
import os
import re
import statistics
import sys

DEFAULT_OUTPUT = "/tmp/textcorpus.tsv"
DEFAULT_REFERENCES = "tests/golden/reference/*.svg"

TEXT_RE = re.compile(r"<text([^>]*)>([^<]*)</text>")


def collect(references_glob: str) -> dict[tuple[str, float, bool], list[float]]:
    """Собирает ширины по (подпись, кегль, полужирный)."""
    collected: dict[tuple[str, float, bool], list[float]] = {}

    for path in sorted(glob.glob(references_glob)):
        with open(path, encoding="utf-8") as handle:
            svg = handle.read()

        for match in TEXT_RE.finditer(svg):
            attrs, label = match.group(1), match.group(2)
            if not label.strip():
                continue

            length = re.search(r'(?<![\w-])textLength="([\d.]+)"', attrs)
            size = re.search(r'(?<![\w-])font-size="([\d.]+)"', attrs)
            if not (length and size):
                continue

            bold = 'font-weight="700"' in attrs or 'font-weight="bold"' in attrs
            key = (label, float(size.group(1)), bold)
            collected.setdefault(key, []).append(float(length.group(1)))

    return collected


def main() -> int:
    output = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_OUTPUT
    collected = collect(DEFAULT_REFERENCES)

    if not collected:
        print("Не найдено ни одной пары «текст — textLength».", file=sys.stderr)
        print("Сначала скачайте эталоны: tests/golden/fetch_references.py", file=sys.stderr)
        return 1

    rows = sorted(
        ((label, size, bold, statistics.median(widths))
         for (label, size, bold), widths in collected.items()),
        key=lambda row: -row[3],
    )

    with open(output, "w", encoding="utf-8") as handle:
        for label, size, bold, width in rows:
            handle.write(f"{label}\t{size}\t{1 if bold else 0}\t{width:.4f}\n")

    by_kind: dict[tuple[float, bool], int] = {}
    for _, size, bold, _ in rows:
        by_kind[(size, bold)] = by_kind.get((size, bold), 0) + 1

    print(f"Записано {len(rows)} пар в {output}")
    print(f"Эталонов просмотрено: {len(glob.glob(DEFAULT_REFERENCES))}")
    print("По кеглю и начертанию:")
    for (size, bold), count in sorted(by_kind.items()):
        kind = "полужирный" if bold else "обычный"
        print(f"  кегль {size:>4} {kind:<11} {count:>3}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
