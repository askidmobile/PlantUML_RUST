#!/bin/bash
# ============================================================================
# docs.sh — Генерация и просмотр документации
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

# Переход в корень проекта
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

print_header "Генерация документации"

# Режим
MODE=${1:-"build"}

case $MODE in
    "build")
        echo -e "Режим: ${GREEN}Сборка документации${NC}"
        echo ""
        cargo doc --workspace --no-deps
        print_success "Документация сгенерирована"
        echo ""
        echo -e "Путь: ${YELLOW}target/doc/plantuml_core/index.html${NC}"
        ;;
        
    "open")
        echo -e "Режим: ${GREEN}Сборка и открытие${NC}"
        echo ""
        cargo doc --workspace --no-deps --open
        print_success "Документация открыта в браузере"
        ;;
        
    "watch")
        echo -e "Режим: ${YELLOW}Автообновление${NC}"
        echo ""
        
        if ! command -v cargo-watch &> /dev/null; then
            echo "Установка cargo-watch..."
            cargo install cargo-watch
        fi
        
        echo "Документация будет автоматически пересобираться при изменениях"
        echo -e "${YELLOW}Нажмите Ctrl+C для остановки${NC}"
        echo ""
        
        cargo watch -s "cargo doc --workspace --no-deps"
        ;;
        
    *)
        echo "Использование: $0 [build|open|watch]"
        echo ""
        echo "  build (по умолчанию) — сгенерировать документацию"
        echo "  open                 — сгенерировать и открыть в браузере"
        echo "  watch                — автообновление при изменениях"
        exit 1
        ;;
esac
