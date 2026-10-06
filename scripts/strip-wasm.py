#!/usr/bin/env python3
"""
strip-wasm.py — удаляет отладочную секцию `name` из WASM-модуля.

Зачем это нужно
===============

Собранный модуль plantuml-wasm содержит кастомную секцию `name` — имена
всех функций в отладочном виде. Для браузера она бесполезна: инструменты
отладки в стеке WASM всё равно не показывают привычные имена, а размер
файла растёт.

Измерено на сборке 4.77 МБ:

    code           4 034 234 байт  (80.6%)  — сам код, нужен
    data             590 271 байт  (11.8%)  — строки и ассеты, нужны
    custom:name      365 345 байт  ( 7.3%)  — имена функций, НЕ нужны

Удаление секции даёт 5 005 982 -> 4 640 633 байт, то есть минус 365 349
байт (7.3%) без единого изменения поведения.

Почему не `wasm-ld --strip-debug`
--------------------------------

Флаг `--strip-debug` у wasm-ld вырезает секции DWARF, но секцию `name`
оставляет: она хранит имена функций, а не отладочные записи о переменных.
Проверено: размер после флага не меняется ни на байт.

Почему не `wasm-opt --strip-debug`
----------------------------------

Было бы правильным решением, но требует бинарника binaryen. В CI и в
локальной сборке его может не быть, поэтому стриппер написан на Python:
он работает везде, где есть интерпретатор, и не тянет зависимостей.

Использование
=============

    python3 scripts/strip-wasm.py вход.wasm выход.wasm
    python3 scripts/strip-wasm.py --in-place модуль.wasm

Формат модуля не меняется: читаются все секции, секция `name`
пропускается, остальные переписываются без изменений.
"""

import argparse
import sys

# Идентификатор секции 0 означает «кастомная», её имя лежит в начале тела
# как LEB128-строка.
SECTION_CUSTOM = 0


def read_uleb(data: bytes, pos: int) -> tuple[int, int]:
    """Читает беззнаковое LEB128 число, возвращает (значение, новая позиция)."""
    result = 0
    shift = 0
    while True:
        byte = data[pos]
        pos += 1
        result |= (byte & 0x7F) << shift
        if not byte & 0x80:
            return result, pos
        shift += 7


def write_uleb(value: int) -> bytes:
    """Кодирует число в LEB128."""
    out = bytearray()
    while True:
        byte = value & 0x7F
        value >>= 7
        if value:
            out.append(byte | 0x80)
        else:
            out.append(byte)
            return bytes(out)


def strip_name_section(data: bytes) -> tuple[bytes, int]:
    """Возвращает модуль без секции `name` и число удалённых байт."""
    if data[:4] != b"\x00asm":
        raise ValueError("файл не является WASM-модулем")

    # Восемь байт заголовка (магия + версия) переносим как есть.
    out = bytearray(data[:8])
    pos = 8
    removed = 0

    while pos < len(data):
        section_start = pos
        section_id = data[pos]
        pos += 1
        size, pos = read_uleb(data, pos)
        body_start = pos
        pos += size

        if section_id == SECTION_CUSTOM:
            name_len, name_pos = read_uleb(data, body_start)
            name = data[name_pos : name_pos + name_len]
            if name == b"name":
                # Заголовок секции плюс тело, вместе с байтом типа.
                removed = (body_start + size) - section_start
                continue

        out += data[section_start:pos]

    return bytes(out), removed


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Удаляет отладочную секцию name из WASM-модуля"
    )
    parser.add_argument(
        "--in-place", action="store_true", help="перезаписать исходный файл"
    )
    parser.add_argument("input", help="исходный модуль")
    parser.add_argument("output", nargs="?", help="куда записать результат")
    args = parser.parse_args()

    if args.in_place and args.output:
        print("ошибка: нельзя одновременно указать выходной файл и --in-place", file=sys.stderr)
        return 1
    if not args.in_place and not args.output:
        print("ошибка: не указан выходной файл", file=sys.stderr)
        return 1

    source = args.input
    data = open(source, "rb").read()

    try:
        stripped, removed = strip_name_section(data)
    except ValueError as error:
        print(f"ошибка: {error}", file=sys.stderr)
        return 1

    target = source if args.in_place else args.output
    with open(target, "wb") as handle:
        handle.write(stripped)

    if removed:
        percent = removed / len(data) * 100
        print(
            f"секция name удалена: {len(data)} -> {len(stripped)} байт "
            f"(минус {removed}, {percent:.1f}%)"
        )
    else:
        print(f"секции name нет, модуль без изменений: {target}")

    return 0


if __name__ == "__main__":
    sys.exit(main())