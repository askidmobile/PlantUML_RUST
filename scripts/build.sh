#!/bin/bash
# ============================================================================
# build.sh — Полная сборка проекта plantuml-rs
# ============================================================================

set -e

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
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

print_step() {
    echo -e "${CYAN}→ $1${NC}"
}

# Переход в корень проекта
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

# Начало отсчёта времени
START_TIME=$(date +%s)

print_header "Полная сборка plantuml-rs"

# Режим сборки
MODE=${1:-"release"}

case $MODE in
    "release")
        echo -e "Режим: ${GREEN}release${NC}"
        BUILD_FLAG="--release"
        ;;
    "debug")
        echo -e "Режим: ${YELLOW}debug${NC}"
        BUILD_FLAG=""
        ;;
    "check")
        echo -e "Режим: ${CYAN}check (только проверка)${NC}"
        CHECK_ONLY=true
        ;;
    *)
        echo "Использование: $0 [release|debug|check]"
        echo ""
        echo "  release (по умолчанию) — оптимизированная сборка"
        echo "  debug                  — отладочная сборка"
        echo "  check                  — только проверка без сборки"
        exit 1
        ;;
esac

echo ""

# Шаг 1: Проверка форматирования
print_header "Шаг 1: Проверка форматирования"
if cargo fmt --all -- --check; then
    print_success "Форматирование корректное"
else
    print_warning "Обнаружены проблемы с форматированием"
    echo -e "Выполните: ${YELLOW}cargo fmt --all${NC}"
    exit 1
fi

# Шаг 2: Проверка clippy
print_header "Шаг 2: Проверка clippy"
print_step "Запуск clippy..."
if cargo clippy --workspace -- -D warnings; then
    print_success "Clippy проверка пройдена"
else
    print_error "Clippy нашёл ошибки"
    exit 1
fi

# Если режим check — завершаем здесь
if [ "$CHECK_ONLY" = true ]; then
    print_header "Проверка завершена"
    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))
    echo -e "Время: ${GREEN}${DURATION}s${NC}"
    print_success "Все проверки пройдены!"
    exit 0
fi

# Шаг 3: Сборка workspace
print_header "Шаг 3: Сборка workspace"
print_step "Компиляция..."
cargo build --workspace $BUILD_FLAG
print_success "Workspace собран"

# Шаг 4: Сборка WASM
print_header "Шаг 4: Сборка WASM"
print_step "Компиляция для wasm32..."
cargo build --target wasm32-unknown-unknown -p plantuml-wasm $BUILD_FLAG
print_success "WASM собран"

# Шаг 5: Генерация документации
print_header "Шаг 5: Генерация документации"
print_step "Сборка документации..."
cargo doc --workspace --no-deps $BUILD_FLAG
print_success "Документация сгенерирована"

# Итоги
print_header "Сборка завершена"

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo ""
echo -e "Время сборки: ${GREEN}${DURATION}s${NC}"
echo ""

# Информация о артефактах
if [ "$MODE" = "release" ]; then
    echo "Артефакты:"
    echo -e "  Библиотека: ${YELLOW}target/release/libplantuml_*.rlib${NC}"
    echo -e "  WASM:       ${YELLOW}target/wasm32-unknown-unknown/release/plantuml_wasm.wasm${NC}"
    echo -e "  Документация: ${YELLOW}target/doc/plantuml_core/index.html${NC}"
else
    echo "Артефакты:"
    echo -e "  Библиотека: ${YELLOW}target/debug/libplantuml_*.rlib${NC}"
    echo -e "  WASM:       ${YELLOW}target/wasm32-unknown-unknown/debug/plantuml_wasm.wasm${NC}"
    echo -e "  Документация: ${YELLOW}target/doc/plantuml_core/index.html${NC}"
fi

echo ""
echo "Следующие шаги:"
echo -e "  Запустить тесты:  ${YELLOW}./scripts/test.sh${NC}"
echo -e "  Собрать WASM пакет: ${YELLOW}./scripts/wasm.sh${NC}"
echo -e "  Открыть документацию: ${YELLOW}open target/doc/plantuml_core/index.html${NC}"
echo ""

print_success "Сборка успешно завершена!"
