#!/bin/bash
# ============================================================================
# test.sh — Запуск тестов проекта plantuml-rs
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

print_header "Тестирование plantuml-rs"

# Режим тестирования
MODE=${1:-"all"}

case $MODE in
    "all")
        echo -e "Режим: ${GREEN}Все тесты${NC}"
        ;;
    "unit")
        echo -e "Режим: ${YELLOW}Unit тесты${NC}"
        ;;
    "integration")
        echo -e "Режим: ${YELLOW}Интеграционные тесты${NC}"
        ;;
    "quick")
        echo -e "Режим: ${CYAN}Быстрые тесты (без snapshot)${NC}"
        ;;
    *)
        # Если передано имя crate или теста — запустить конкретный
        if [[ "$MODE" == plantuml-* ]]; then
            echo -e "Тестируется crate: ${YELLOW}$MODE${NC}"
            cargo test -p "$MODE" "${@:2}"
            exit $?
        else
            echo "Использование: $0 [all|unit|integration|quick|<crate-name>|<test-name>]"
            echo ""
            echo "  all          — все тесты (по умолчанию)"
            echo "  unit         — только unit тесты"
            echo "  integration  — только интеграционные тесты"
            echo "  quick        — быстрые тесты без snapshot"
            echo "  <crate>      — тесты конкретного crate (например: plantuml-parser)"
            echo ""
            echo "Примеры:"
            echo "  $0                              # все тесты"
            echo "  $0 plantuml-parser              # тесты парсера"
            echo "  $0 plantuml-core sequence       # тесты sequence в plantuml-core"
            exit 1
        fi
        ;;
esac

echo ""

# Начало отсчёта времени
START_TIME=$(date +%s)

# Запуск тестов в зависимости от режима
case $MODE in
    "all")
        print_header "Unit тесты"
        print_step "Запуск unit тестов..."
        cargo test --workspace --lib
        print_success "Unit тесты пройдены"
        
        print_header "Интеграционные тесты"
        print_step "Запуск интеграционных тестов..."
        cargo test --workspace --test '*'
        print_success "Интеграционные тесты пройдены"
        
        print_header "Doc тесты"
        print_step "Запуск doc тестов..."
        cargo test --workspace --doc
        print_success "Doc тесты пройдены"
        ;;
        
    "unit")
        print_header "Unit тесты"
        print_step "Запуск unit тестов..."
        cargo test --workspace --lib
        print_success "Unit тесты пройдены"
        ;;
        
    "integration")
        print_header "Интеграционные тесты"
        print_step "Запуск интеграционных тестов..."
        cargo test --workspace --test '*'
        print_success "Интеграционные тесты пройдены"
        ;;
        
    "quick")
        print_header "Быстрые тесты"
        print_step "Запуск тестов без snapshot..."
        cargo test --workspace --lib -- --skip snapshot
        print_success "Быстрые тесты пройдены"
        ;;
esac

# Итоги
print_header "Тестирование завершено"

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo ""
echo -e "Время тестирования: ${GREEN}${DURATION}s${NC}"
echo ""

# Дополнительные команды
echo "Дополнительные опции:"
echo -e "  Тесты с выводом:     ${YELLOW}cargo test --workspace -- --nocapture${NC}"
echo -e "  Конкретный тест:     ${YELLOW}cargo test -p plantuml-parser test_name${NC}"
echo -e "  Обновить snapshots:  ${YELLOW}cargo insta review${NC}"
echo ""

print_success "Все тесты пройдены успешно!"
