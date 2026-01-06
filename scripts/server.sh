#!/bin/bash
# ============================================================================
# server.sh — Запуск локального сервера для разработки playground
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

# Порт по умолчанию
PORT=${1:-8080}

# Директория для сервера — playground
PLAYGROUND_DIR="$PROJECT_ROOT/playground"

print_header "Локальный сервер Playground"

echo -e "Порт: ${GREEN}$PORT${NC}"
echo -e "Директория: ${GREEN}$PLAYGROUND_DIR${NC}"
echo ""

# Проверка наличия playground
if [ ! -d "$PLAYGROUND_DIR" ]; then
    print_error "Папка playground не найдена!"
    exit 1
fi

# Проверка наличия WASM файлов в playground/pkg
PLAYGROUND_PKG="$PLAYGROUND_DIR/pkg"
if [ ! -f "$PLAYGROUND_PKG/plantuml_wasm.js" ]; then
    print_warning "WASM модуль не найден в playground/pkg. Собираю..."
    ./scripts/wasm.sh
    print_success "WASM собран и скопирован"
fi

# Выбор сервера
if command -v python3 &> /dev/null; then
    SERVER_CMD="python3"
    SERVER_TYPE="Python 3"
elif command -v python &> /dev/null; then
    SERVER_CMD="python"
    SERVER_TYPE="Python"
elif command -v npx &> /dev/null; then
    SERVER_CMD="npx"
    SERVER_TYPE="npx serve"
elif command -v php &> /dev/null; then
    SERVER_CMD="php"
    SERVER_TYPE="PHP"
else
    print_error "Не найден подходящий сервер!"
    echo "Установите один из: python3, node (npx), php"
    exit 1
fi

echo -e "Используется: ${GREEN}$SERVER_TYPE${NC}"
echo ""
echo -e "${YELLOW}Нажмите Ctrl+C для остановки сервера${NC}"
echo ""
echo -e "Откройте в браузере: ${GREEN}http://localhost:$PORT${NC}"
echo ""

# Запуск сервера из папки playground
cd "$PLAYGROUND_DIR"

case $SERVER_TYPE in
    "Python 3")
        python3 -m http.server $PORT
        ;;
    "Python")
        python -m SimpleHTTPServer $PORT
        ;;
    "npx serve")
        npx serve -l $PORT
        ;;
    "PHP")
        php -S localhost:$PORT
        ;;
esac
