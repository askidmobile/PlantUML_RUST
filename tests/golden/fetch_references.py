#!/usr/bin/env python3
"""Генерация эталонных SVG с официального сервера PlantUML.

Использование:
    python3 tests/golden/fetch_references.py [--force]

Для каждого .puml из tests/golden/cases/ запрашивает SVG у plantuml.com
и сохраняет в tests/golden/reference/<имя>.svg вместе с метаданными
(версия PlantUML и хеш исходника), чтобы расхождение версий было видно.

Сервер отдаёт 403 без браузерного User-Agent.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import re
import sys
import urllib.error
import time
import urllib.request
import zlib

ROOT = pathlib.Path(__file__).resolve().parents[2]
CASES_DIR = ROOT / "tests" / "golden" / "cases"
REF_DIR = ROOT / "tests" / "golden" / "reference"
META_FILE = REF_DIR / "metadata.json"

SERVER = "https://www.plantuml.com/plantuml/svg/"

# Сколько раз перезапрашивать эталон при вырожденном ответе сервера.
FETCH_ATTEMPTS = 3
USER_AGENT = (
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 "
    "(KHTML, like Gecko) Chrome/120.0 Safari/537.36"
)


def _encode6bit(b: int) -> str:
    """Символ алфавита PlantUML для 6-битного значения.

    ВНИМАНИЕ: это НЕ стандартный base64. PlantUML использует собственный
    алфавит 0-9 A-Z a-z - _ , поэтому обычный base64.b64encode даёт
    неверный URL (сервер отвечает страницей «bad URL»).
    """
    if b < 10:
        return chr(48 + b)  # 0-9
    b -= 10
    if b < 26:
        return chr(65 + b)  # A-Z
    b -= 26
    if b < 26:
        return chr(97 + b)  # a-z
    b -= 26
    if b == 0:
        return "-"
    if b == 1:
        return "_"
    return "?"


def _append3bytes(b1: int, b2: int, b3: int) -> str:
    c1 = b1 >> 2
    c2 = ((b1 & 0x3) << 4) | (b2 >> 4)
    c3 = ((b2 & 0xF) << 2) | (b3 >> 6)
    c4 = b3 & 0x3F
    return (
        _encode6bit(c1 & 0x3F)
        + _encode6bit(c2 & 0x3F)
        + _encode6bit(c3 & 0x3F)
        + _encode6bit(c4 & 0x3F)
    )


def encode(source: str) -> str:
    """Кодирует исходник PlantUML в формат URL сервера (raw deflate + алфавит PlantUML)."""
    data = zlib.compress(source.encode("utf-8"))[2:-4]  # raw deflate
    chunks = []
    for i in range(0, len(data), 3):
        chunk = data[i : i + 3]
        b1 = chunk[0]
        b2 = chunk[1] if len(chunk) > 1 else 0
        b3 = chunk[2] if len(chunk) > 2 else 0
        chunks.append(_append3bytes(b1, b2, b3))
    return "".join(chunks)


def fetch(source: str) -> str:
    """Запрашивает SVG у сервера PlantUML.

    Проверяет, что вернулся именно SVG: при ошибке кодирования сервер
    отдаёт SVG со страницей «The plugin you are using seems to generated
    a bad URL», и такой файл нельзя сохранять как эталон.
    """
    url = SERVER + encode(source)
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(req, timeout=30) as resp:
        svg = resp.read().decode("utf-8")

    if "seems to generated a bad URL" in svg or "does not look like DEFLATE" in svg:
        raise ValueError(
            "сервер PlantUML отверг URL (ошибка кодирования), а не отрендерил диаграмму"
        )
    if "<svg" not in svg:
        raise ValueError("ответ сервера не содержит SVG")
    if is_degenerate(svg):
        raise DegenerateResponse(
            "сервер вернул вырожденный SVG: у всех подписей textLength=\"0\""
        )
    return svg


class DegenerateResponse(ValueError):
    """Сервер отдал SVG, в котором не измерена ни одна подпись.

    Такое случается: PlantUML иногда возвращает ответ, где у ВСЕХ элементов
    `<text>` стоит `textLength="0"`, а габариты занижены. Пример —
    `network_nwdiag`: вырожденный ответ 117x129 против правильного 273x140.

    Такой файл нельзя сохранять как эталон: расхождения по нему
    недостоверны. Раньше проверки не было, и четыре эталона из двадцати
    оказались вырожденными, потому что скрипт по умолчанию пропускает уже
    существующие файлы и не перезапрашивал их.
    """


def is_degenerate(svg: str) -> bool:
    """Все подписи без длины — признак вырожденного ответа."""
    texts = re.findall(r"<text\b", svg)
    if not texts:
        return False
    lengths = re.findall(r'textLength="([\d.]+)"', svg)
    if not lengths:
        return False
    return all(float(value) == 0.0 for value in lengths)


def plantuml_version(svg: str) -> str | None:
    """Извлекает версию PlantUML из ответа сервера.

    Сервер вставляет инструкцию обработки вида `<?plantuml 1.2026.9beta4?>`.
    Версия важна: эталон, снятый другой версией PlantUML, может отличаться
    геометрией, и это нужно видеть при разборе расхождений.
    """
    match = re.search(r"<\?plantuml\s+([^?\s]+)\s*\?>", svg)
    return match.group(1) if match else None


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--force",
        action="store_true",
        help="перезаписать существующие эталоны",
    )
    args = parser.parse_args()

    if not CASES_DIR.is_dir():
        print(f"нет директории с кейсами: {CASES_DIR}", file=sys.stderr)
        return 1

    REF_DIR.mkdir(parents=True, exist_ok=True)
    meta: dict[str, dict[str, str]] = {}
    if META_FILE.is_file() and not args.force:
        meta = json.loads(META_FILE.read_text(encoding="utf-8"))

    cases = sorted(CASES_DIR.glob("*.puml"))
    if not cases:
        print(f"нет .puml в {CASES_DIR}", file=sys.stderr)
        return 1

    failed = 0
    for case in cases:
        out = REF_DIR / f"{case.stem}.svg"
        if out.is_file() and not args.force:
            print(f"  пропуск (есть): {out.name}")
            continue

        source = case.read_text(encoding="utf-8")
        svg = None
        for attempt in range(1, FETCH_ATTEMPTS + 1):
            try:
                svg = fetch(source)
                break
            except DegenerateResponse as exc:
                # Вырожденный ответ обычно разовый — пробуем ещё раз.
                print(
                    f"  попытка {attempt}/{FETCH_ATTEMPTS} {case.name}: {exc}",
                    file=sys.stderr,
                )
                time.sleep(1.5)
            except (urllib.error.URLError, TimeoutError) as exc:
                print(f"  ОШИБКА {case.name}: {exc}", file=sys.stderr)
                break

        if svg is None:
            print(f"  ПРОПУЩЕНО {case.name}: эталон не получен", file=sys.stderr)
            failed += 1
            continue

        out.write_text(svg, encoding="utf-8")
        meta[case.stem] = {
            "source_sha256": hashlib.sha256(source.encode("utf-8")).hexdigest(),
            "svg_sha256": hashlib.sha256(svg.encode("utf-8")).hexdigest(),
            "plantuml_version": plantuml_version(svg) or "не указана",
            "svg_bytes": str(len(svg)),
        }
        print(f"  сохранено: {out.name} ({len(svg)} байт)")

    META_FILE.write_text(
        json.dumps(meta, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(f"метаданные: {META_FILE.relative_to(ROOT)}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
