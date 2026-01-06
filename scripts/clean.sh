#!/bin/bash
# ============================================================================
# clean.sh — Очистка временных файлов и артефактов сборки
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

print_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

# Переход в корень проекта
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

print_header "Очистка проекта plantuml-rs"

# Режим очистки
MODE=${1:-"normal"}

case $MODE in
    "all"|"--all"|"-a")
        echo -e "Режим: ${RED}Полная очистка${NC}"
        CLEAN_ALL=true
        ;;
    "normal"|"")
        echo -e "Режим: ${GREEN}Обычная очистка${NC}"
        CLEAN_ALL=false
        ;;
    *)
        echo "Использование: $0 [normal|all]"
        echo ""
        echo "  normal (по умолчанию) — очистка target/ и временных файлов"
        echo "  all                   — полная очистка включая pkg/, Cargo.lock"
        exit 1
        ;;
esac

echo ""

# Функция для безопасного удаления
safe_remove() {
    local path="$1"
    local description="$2"
    
    if [ -e "$path" ]; then
        rm -rf "$path"
        print_success "Удалено: $description ($path)"
    else
        print_info "Не найдено: $description"
    fi
}

# Подсчёт размера до очистки
if [ -d "target" ]; then
    SIZE_BEFORE=$(du -sh target 2>/dev/null | cut -f1)
    echo -e "Размер target/ до очистки: ${YELLOW}$SIZE_BEFORE${NC}"
    echo ""
fi

# Шаг 1: Очистка cargo
print_header "Шаг 1: Очистка cargo"
cargo clean
print_success "Cargo clean выполнен"

# Шаг 2: Удаление временных файлов
print_header "Шаг 2: Удаление временных файлов"

# Backup файлы
find . -name "*.bak" -type f -delete 2>/dev/null && print_success "Удалены *.bak файлы" || true
find . -name "*~" -type f -delete 2>/dev/null && print_success "Удалены *~ файлы" || true
find . -name "*.swp" -type f -delete 2>/dev/null && print_success "Удалены *.swp файлы" || true
find . -name ".DS_Store" -type f -delete 2>/dev/null && print_success "Удалены .DS_Store файлы" || true

# Snapshot новые файлы (insta)
find . -name "*.snap.new" -type f -delete 2>/dev/null && print_success "Удалены *.snap.new файлы" || true

# Логи
find . -name "*.log" -type f -not -path "./target/*" -delete 2>/dev/null && print_success "Удалены *.log файлы" || true

# Шаг 3: Удаление сгенерированных SVG/PNG
print_header "Шаг 3: Сгенерированные файлы"

# SVG файлы в корне (результаты демо)
for f in *.svg; do
    if [ -f "$f" ] && [ "$f" != "*.svg" ]; then
        rm -f "$f"
        print_success "Удалён $f"
    fi
done

# PNG файлы в корне
for f in *.png; do
    if [ -f "$f" ] && [ "$f" != "*.png" ]; then
        rm -f "$f"
        print_success "Удалён $f"
    fi
done

# Шаг 4: Полная очистка (если запрошена)
if [ "$CLEAN_ALL" = true ]; then
    print_header "Шаг 4: Полная очистка"
    
    safe_remove "pkg" "WASM пакет"
    safe_remove "Cargo.lock" "Cargo.lock"
    safe_remove ".cargo" ".cargo кэш"
    
    # node_modules если есть
    safe_remove "node_modules" "node_modules"
    
    # Документация
    safe_remove "target/doc" "Сгенерированная документация"
fi

# Итоги
print_header "Очистка завершена"

echo ""
echo -e "${GREEN}Проект очищен!${NC}"
echo ""

if [ "$CLEAN_ALL" = true ]; then
    echo "Для полной пересборки выполните:"
    echo -e "  ${YELLOW}cargo build --workspace${NC}"
    echo -e "  ${YELLOW}./scripts/wasm.sh${NC}"
else
    echo "Для пересборки выполните:"
    echo -e "  ${YELLOW}cargo build --workspace${NC}"
fi
