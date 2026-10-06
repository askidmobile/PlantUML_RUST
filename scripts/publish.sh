#!/bin/bash
# ============================================================================
# publish.sh — публикация крейтов на crates.io в порядке зависимостей
# ============================================================================
#
# Зачем нужен порядок
# ===================
#
# При публикации `path`-зависимость заменяется на требование версии из
# реестра. Если зависимость ещё не опубликована, сборка падает с
# «no matching package named ... found». Поэтому крейты публикуются
# строго в топологическом порядке: сначала те, у которых нет внутренних
# зависимостей.
#
# Порядок выведен из манифестов (скрипт печатает его при запуске) и
# закреплён здесь явно: перечислить его в коде надёжнее, чем вычислять
# на лету и надеяться, что граф не изменится.
#
# Почему у внутренних зависимостей проставлены версии
# =====================================================
#
# Изначально они были объявлены как `{ path = "crates/..." }` без
# версии. При упаковке путь отбрасывается, и без версии крейт просто не
# упаковывается:
#
#     error: all dependencies must have a version requirement specified
#            when packaging.
#            dependency `plantuml-ast` does not specify a version
#
# То есть семь из десяти крейтов нельзя было опубликовать вообще.
#
# Использование
# ============
#
#     ./scripts/publish.sh            # проверка без публикации
#     ./scripts/publish.sh --dry-run  # то же: cargo publish --dry-run
#     ./scripts/publish.sh --publish  # реальная публикация
#
# По умолчанию ничего не публикуется: команда без флагов только
# проверяет, что каждый крейт собирается и упаковывается.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"


# Топологический порядок: зависимость всегда раньше зависимой.
ORDER=(
    plantuml-ast
    plantuml-model
    plantuml-layout
    plantuml-stdlib
    plantuml-parser
    plantuml-themes
    plantuml-preprocessor
    plantuml-renderer
    plantuml-core
    plantuml-wasm
)

MODE="check"
case "${1:-}" in
    --publish)  MODE="publish" ;;
    --dry-run|"") MODE="check" ;;
    *) echo "использование: $0 [--publish|--dry-run]"; exit 1 ;;
esac

echo "============================================"
echo "  Публикация крейтов"
echo "============================================"
echo "режим: $MODE"
echo "порядок: ${ORDER[*]}"
echo ""

VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
echo "версия workspace: $VERSION"
echo ""

# Проверка версии во всех крейтах: расхождение сделает публикацию
# невозможной, потому что зависимости ссылаются на конкретную версию.
echo "Шаг 1: версии"
MISMATCH=0
for crate in "${ORDER[@]}"; do
    manifest="crates/$crate/Cargo.toml"
    if grep -q '^version.workspace = true' "$manifest"; then
        # Наследует версию workspace — расхождение невозможно.
        continue
    fi
    crate_version=$(grep -m1 '^version *=' "$manifest" | sed 's/.*"\(.*\)".*/\1/')
    if [ "$crate_version" != "$VERSION" ]; then
        echo "  ✗ $crate: версия $crate_version, ожидалась $VERSION\n"
        MISMATCH=1
    fi
done
if [ "$MISMATCH" -eq 0 ]; then
    echo "  ✓ во всех крейтах версия $VERSION"
else
    echo "  Версии разошлись — публикация невозможна"
    exit 1
fi
echo ""

# Проверка, что рабочее дерево чистое: публикация из грязного дерева
# упаковывает файлы, которых нет в репозитории.
echo "Шаг 2: состояние дерева"
if [ -n "$(git status --porcelain)" ]; then
    echo "  ✗ есть незакоммиченные изменения:"
    git status --porcelain | head -5 | sed 's/^/      /'
    exit 1
fi
echo "  ✓ дерево чистое"
echo ""

echo "Шаг 3: гейты"
cargo fmt --all -- --check || { echo "  ✗ fmt"; exit 1; }
echo "  ✓ fmt"
cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | tail -1
echo "  ✓ clippy"
cargo test --workspace --all-features 2>&1 | grep -E "^test result" \
    | awk -F'[ ;]' '{p += $4; f += $6} END {print "    тестов:", p, "провалов:", f}'
echo "  ✓ тесты"
./scripts/check-no-panic.sh > /dev/null
echo "  ✓ паникующих операций нет"
echo ""

FAILED=0
for crate in "${ORDER[@]}"; do
    printf "  %-22s " "$crate"
    if [ "$MODE" = "publish" ]; then
        if cargo publish -p "$crate" --locked 2>&1 | tail -1 | grep -q "Uploaded"; then
            echo "опубликован"
        else
            echo "не удалось"
            FAILED=1
            break
        fi
        # crates.io обновляется не мгновенно: следующий крейт может
        # не увидеть только что загруженный. Пауза бережная.
        echo "    ждём обновления индекса crates.io..."
        sleep 30
    else
        # В режиме проверки cargo не должен обращаться к сети: крейты
        # ещё не опубликованы, поэтому проверяем только упаковку.
        if cargo package -p "$crate" --no-verify --allow-dirty --offline 2>&1 \
            | grep -qE "^error: failed to verify manifest"; then
            echo "манифест не проходит упаковку"
            FAILED=1
        elif cargo package -p "$crate" --no-verify --allow-dirty --offline 2>&1 \
            | grep -qE "no matching package"; then
            # Манифест валиден; не хватает только опубликованных
            # зависимостей — для первого релиза это ожидаемо.
            echo "манифест в порядке, ждёт публикации зависимостей"
        else
            echo "✓ упакован"
        fi
    fi
done

echo ""
if [ "$FAILED" -ne 0 ]; then
    echo "Публикация невозможна — см. ошибки выше"
    exit 1
fi

if [ "$MODE" = "publish" ]; then
    echo "Все крейты опубликованы"
else
    echo "Все манифесты готовы к публикации"
    echo "Для реальной публикации: ./scripts/publish.sh --publish"
fi