#!/bin/bash
# ============================================================================
# wasm.sh — Сборка WASM модуля для plantuml-rs
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

print_header "Сборка WASM модуля"

# Проверка наличия wasm-pack
if ! command -v wasm-pack &> /dev/null; then
    print_warning "wasm-pack не установлен. Устанавливаю..."
    cargo install wasm-pack
    print_success "wasm-pack установлен"
fi

# Проверка наличия target wasm32
if ! rustup target list --installed | grep -q "wasm32-unknown-unknown"; then
    print_warning "Target wasm32-unknown-unknown не установлен. Устанавливаю..."
    rustup target add wasm32-unknown-unknown
    print_success "Target wasm32-unknown-unknown установлен"
fi

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
    *)
        echo "Использование: $0 [release|debug]"
        exit 1
        ;;
esac

# Шаг 1: Базовая сборка cargo для WASM
print_header "Шаг 1: Cargo сборка для wasm32"
cargo build --target wasm32-unknown-unknown -p plantuml-wasm $BUILD_FLAG
print_success "Cargo сборка завершена"

# Шаг 2: Сборка через wasm-pack (если нужен npm пакет)
print_header "Шаг 2: wasm-pack сборка"

OUTPUT_DIR="$PROJECT_ROOT/pkg"

if [ "$MODE" = "release" ]; then
    wasm-pack build crates/plantuml-wasm --target web --out-dir "$OUTPUT_DIR" --release
else
    wasm-pack build crates/plantuml-wasm --target web --out-dir "$OUTPUT_DIR" --dev
fi

print_success "wasm-pack сборка завершена"

# Шаг 3: Информация о результатах
print_header "Результаты сборки"

echo ""
echo "Файлы сборки:"
ls -lh "$OUTPUT_DIR"/*.wasm 2>/dev/null || echo "  (wasm файлы не найдены)"
ls -lh "$OUTPUT_DIR"/*.js 2>/dev/null || echo "  (js файлы не найдены)"
echo ""

if [ -f "$OUTPUT_DIR/plantuml_wasm_bg.wasm" ]; then
    SIZE=$(ls -lh "$OUTPUT_DIR/plantuml_wasm_bg.wasm" | awk '{print $5}')
    echo -e "Размер WASM: ${GREEN}$SIZE${NC}"
fi

# Шаг 4: Копируем в playground/pkg
print_header "Шаг 4: Обновление playground"

PLAYGROUND_PKG="$PROJECT_ROOT/playground/pkg"
if [ -d "$PLAYGROUND_PKG" ]; then
    cp -f "$OUTPUT_DIR/plantuml_wasm.js" "$PLAYGROUND_PKG/"
    cp -f "$OUTPUT_DIR/plantuml_wasm_bg.wasm" "$PLAYGROUND_PKG/"
    cp -f "$OUTPUT_DIR/plantuml_wasm.d.ts" "$PLAYGROUND_PKG/" 2>/dev/null || true
    cp -f "$OUTPUT_DIR/plantuml_wasm_bg.wasm.d.ts" "$PLAYGROUND_PKG/" 2>/dev/null || true
    print_success "Файлы скопированы в playground/pkg"
else
    print_warning "Папка playground/pkg не найдена"
fi

echo ""
echo -e "Выходная директория: ${YELLOW}$OUTPUT_DIR${NC}"
echo ""
echo "Использование в браузере:"
echo ""
echo -e "  ${YELLOW}import init, { render } from './pkg/plantuml_wasm.js';${NC}"
echo -e "  ${YELLOW}await init();${NC}"
echo -e "  ${YELLOW}const svg = render('@startuml\\nAlice -> Bob\\n@enduml');${NC}"
echo ""

print_success "WASM сборка завершена успешно!"
