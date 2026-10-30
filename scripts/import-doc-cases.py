#!/usr/bin/env python3
"""Импорт примеров из справочников в корпус сверки tests/golden/corpus/ref_*.

Берёт блоки ```plantuml с @start из docs/PLANTUML_LANGUAGE_REFERENCE.md и
docs/SYNTAX.md. Блок с несколькими схемами делится на отдельные кейсы.
Запускать при изменении справочников; эталоны затем снимает
tests/golden/fetch_references.py --corpus.
"""

import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[1]
OUT = ROOT / "tests" / "golden" / "corpus"
DOCS = {"lang": "docs/PLANTUML_LANGUAGE_REFERENCE.md", "syn": "docs/SYNTAX.md"}


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    for prefix, doc in DOCS.items():
        text = (ROOT / doc).read_text(encoding="utf-8")
        index = 0
        for block in re.findall(r"```plantuml\n(.*?)```", text, re.S):
            for diagram in re.findall(r"@start\w*.*?@end\w*", block, re.S):
                index += 1
                (OUT / f"ref_{prefix}_{index:03}.puml").write_text(diagram + "\n", encoding="utf-8")
        print(f"{doc}: {index}")


if __name__ == "__main__":
    main()
