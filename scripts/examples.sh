#!/bin/bash
# ============================================================================
# examples.sh — Запуск примеров диаграмм
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

print_step() {
    echo -e "${CYAN}→ $1${NC}"
}

# Переход в корень проекта
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

print_header "Запуск примеров plantuml-rs"

# Список доступных примеров
EXAMPLES=(
    "sequence_demo"
    "class_demo"
    "activity_demo"
    "state_demo"
    "component_demo"
    "deployment_demo"
    "usecase_demo"
    "object_demo"
    "timing_demo"
    "gantt_demo"
    "mindmap_demo"
    "wbs_demo"
    "json_demo"
    "yaml_demo"
    "network_demo"
    "salt_demo"
    "er_demo"
)

# Если передан конкретный пример — запустить его
if [ -n "$1" ]; then
    EXAMPLE=$1
    
    # Проверка существования примера
    if [[ " ${EXAMPLES[*]} " =~ " ${EXAMPLE} " ]] || [[ "$EXAMPLE" == *_demo ]]; then
        print_step "Запуск примера: $EXAMPLE"
        cargo run -p plantuml-core --example "$EXAMPLE"
        print_success "Пример $EXAMPLE выполнен"
        exit 0
    else
        echo -e "${RED}Пример '$EXAMPLE' не найден${NC}"
        echo ""
        echo "Доступные примеры:"
        for ex in "${EXAMPLES[@]}"; do
            echo "  - $ex"
        done
        exit 1
    fi
fi

# Интерактивный выбор примера
echo "Доступные примеры:"
echo ""

PS3=$'\n'"Выберите пример (1-${#EXAMPLES[@]}, q для выхода): "

select EXAMPLE in "${EXAMPLES[@]}" "Все примеры" "Выход"; do
    case $EXAMPLE in
        "Выход")
            echo "Выход"
            exit 0
            ;;
        "Все примеры")
            print_header "Запуск всех примеров"
            for ex in "${EXAMPLES[@]}"; do
                print_step "Запуск $ex..."
                cargo run -p plantuml-core --example "$ex" || true
                print_success "$ex выполнен"
                echo ""
            done
            print_success "Все примеры выполнены"
            exit 0
            ;;
        *)
            if [ -n "$EXAMPLE" ]; then
                print_step "Запуск примера: $EXAMPLE"
                cargo run -p plantuml-core --example "$EXAMPLE"
                print_success "Пример $EXAMPLE выполнен"
                echo ""
                echo "Сгенерированные файлы находятся в текущей директории"
            else
                echo "Неверный выбор. Попробуйте снова."
            fi
            ;;
    esac
done
