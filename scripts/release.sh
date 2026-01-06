#!/bin/bash
# ============================================================================
# release.sh — Создание релиза проекта plantuml-rs
# ============================================================================

set -e

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${BLUE}============================================${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}============================================${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Переход в корень проекта
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

print_header "Создание релиза plantuml-rs"

# Проверка версии
if [ -z "$1" ]; then
    echo "Использование: $0 <версия>"
    echo "Пример: $0 0.3.0"
    exit 1
fi

VERSION=$1
echo -e "Версия релиза: ${GREEN}$VERSION${NC}"
echo ""

# Проверка чистого git состояния
if [ -n "$(git status --porcelain)" ]; then
    print_error "В репозитории есть незакоммиченные изменения!"
    echo "Пожалуйста, закоммитьте или отмените изменения перед релизом."
    exit 1
fi

print_success "Git репозиторий чистый"

# Шаг 1: Запуск тестов
print_header "Шаг 1: Запуск тестов"
cargo test --workspace
print_success "Все тесты прошли"

# Шаг 2: Проверка clippy
print_header "Шаг 2: Проверка clippy"
cargo clippy --workspace -- -D warnings
print_success "Clippy проверка пройдена"

# Шаг 3: Проверка форматирования
print_header "Шаг 3: Проверка форматирования"
cargo fmt --all -- --check
print_success "Форматирование корректное"

# Шаг 4: Сборка WASM
print_header "Шаг 4: Сборка WASM"
./scripts/wasm.sh
print_success "WASM собран"

# Шаг 5: Обновление версий в Cargo.toml
print_header "Шаг 5: Обновление версий"

# Список crates для обновления
CRATES=(
    "crates/plantuml-ast/Cargo.toml"
    "crates/plantuml-core/Cargo.toml"
    "crates/plantuml-layout/Cargo.toml"
    "crates/plantuml-model/Cargo.toml"
    "crates/plantuml-parser/Cargo.toml"
    "crates/plantuml-preprocessor/Cargo.toml"
    "crates/plantuml-renderer/Cargo.toml"
    "crates/plantuml-stdlib/Cargo.toml"
    "crates/plantuml-themes/Cargo.toml"
    "crates/plantuml-wasm/Cargo.toml"
)

for crate in "${CRATES[@]}"; do
    if [ -f "$crate" ]; then
        sed -i.bak "s/^version = \"[0-9]*\.[0-9]*\.[0-9]*\"/version = \"$VERSION\"/" "$crate"
        rm -f "$crate.bak"
        print_success "Обновлён $crate"
    fi
done

# Шаг 6: Создание git тега
print_header "Шаг 6: Создание git коммита и тега"

git add -A
git commit -m "chore: релиз версии v$VERSION"
git tag -a "v$VERSION" -m "Версия $VERSION"

print_success "Создан тег v$VERSION"

# Шаг 7: Инструкции для публикации
print_header "Релиз готов!"

echo ""
echo -e "Следующие шаги:"
echo -e "  1. Проверьте изменения: ${YELLOW}git log --oneline -5${NC}"
echo -e "  2. Запушьте в репозиторий: ${YELLOW}git push origin main --tags${NC}"
echo -e "  3. Опубликуйте на crates.io:"
echo ""
echo -e "     ${YELLOW}cargo publish -p plantuml-model${NC}"
echo -e "     ${YELLOW}cargo publish -p plantuml-ast${NC}"
echo -e "     ${YELLOW}cargo publish -p plantuml-parser${NC}"
echo -e "     ${YELLOW}cargo publish -p plantuml-preprocessor${NC}"
echo -e "     ${YELLOW}cargo publish -p plantuml-themes${NC}"
echo -e "     ${YELLOW}cargo publish -p plantuml-stdlib${NC}"
echo -e "     ${YELLOW}cargo publish -p plantuml-layout${NC}"
echo -e "     ${YELLOW}cargo publish -p plantuml-renderer${NC}"
echo -e "     ${YELLOW}cargo publish -p plantuml-core${NC}"
echo ""

print_success "Релиз v$VERSION создан успешно!"
