#!/bin/bash
# ============================================================================
#                         plantuml-rs — Главное меню
# ============================================================================
#
# Интерактивный скрипт для управления проектом plantuml-rs
# Использование: ./run.sh [команда] [аргументы]
#
# ============================================================================

set -e

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Переход в директорию проекта
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# ============================================================================
# Функции вывода
# ============================================================================

print_logo() {
    echo -e "${BLUE}"
    echo "  ╔═══════════════════════════════════════════════════════════╗"
    echo "  ║                                                           ║"
    echo "  ║     ██████╗ ██╗      █████╗ ███╗   ██╗████████╗██╗   ██╗  ║"
    echo "  ║     ██╔══██╗██║     ██╔══██╗████╗  ██║╚══██╔══╝██║   ██║  ║"
    echo "  ║     ██████╔╝██║     ███████║██╔██╗ ██║   ██║   ██║   ██║  ║"
    echo "  ║     ██╔═══╝ ██║     ██╔══██║██║╚██╗██║   ██║   ██║   ██║  ║"
    echo "  ║     ██║     ███████╗██║  ██║██║ ╚████║   ██║   ╚██████╔╝  ║"
    echo "  ║     ╚═╝     ╚══════╝╚═╝  ╚═╝╚═╝  ╚═══╝   ╚═╝    ╚═════╝   ║"
    echo "  ║                       M L - R S                           ║"
    echo "  ║                                                           ║"
    echo "  ║          Pure Rust PlantUML Renderer                      ║"
    echo "  ╚═══════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

print_menu() {
    echo -e "${BOLD}Выберите действие:${NC}"
    echo ""
    echo -e "  ${GREEN}Сборка и разработка${NC}"
    echo -e "    ${CYAN}1)${NC} build      — Полная сборка проекта (release)"
    echo -e "    ${CYAN}2)${NC} build-dev  — Отладочная сборка (debug)"
    echo -e "    ${CYAN}3)${NC} check      — Проверка кода (clippy + fmt)"
    echo -e "    ${CYAN}4)${NC} clean      — Очистка временных файлов"
    echo -e "    ${CYAN}5)${NC} clean-all  — Полная очистка (включая WASM, Cargo.lock)"
    echo ""
    echo -e "  ${GREEN}Тестирование${NC}"
    echo -e "    ${CYAN}6)${NC} test       — Запуск всех тестов"
    echo -e "    ${CYAN}7)${NC} test-unit  — Только unit тесты"
    echo -e "    ${CYAN}8)${NC} test-quick — Быстрые тесты"
    echo ""
    echo -e "  ${GREEN}WASM${NC}"
    echo -e "    ${CYAN}9)${NC} wasm       — Сборка WASM модуля"
    echo -e "   ${CYAN}10)${NC} server     — Запуск локального сервера"
    echo ""
    echo -e "  ${GREEN}Документация и примеры${NC}"
    echo -e "   ${CYAN}11)${NC} docs       — Генерация документации"
    echo -e "   ${CYAN}12)${NC} docs-open  — Открыть документацию в браузере"
    echo -e "   ${CYAN}13)${NC} examples   — Запуск примеров диаграмм"
    echo ""
    echo -e "  ${GREEN}Релиз${NC}"
    echo -e "   ${CYAN}14)${NC} release    — Создание нового релиза"
    echo ""
    echo -e "  ${MAGENTA}0) exit${NC}      — Выход"
    echo ""
}

print_help() {
    echo -e "${BOLD}plantuml-rs — Управление проектом${NC}"
    echo ""
    echo "Использование: ./run.sh [команда] [аргументы]"
    echo ""
    echo "Команды:"
    echo ""
    echo -e "  ${GREEN}Сборка${NC}"
    echo "    build [release|debug]   Сборка проекта"
    echo "    check                   Проверка кода (clippy + fmt)"
    echo "    clean [all]             Очистка временных файлов"
    echo ""
    echo -e "  ${GREEN}Тестирование${NC}"
    echo "    test [all|unit|quick]   Запуск тестов"
    echo "    test <crate-name>       Тесты конкретного crate"
    echo ""
    echo -e "  ${GREEN}WASM${NC}"
    echo "    wasm [release|debug]    Сборка WASM модуля"
    echo "    server [port]           Локальный сервер (по умолчанию порт 8080)"
    echo ""
    echo -e "  ${GREEN}Документация${NC}"
    echo "    docs [build|open|watch] Генерация документации"
    echo "    examples [name]         Запуск примеров"
    echo ""
    echo -e "  ${GREEN}Релиз${NC}"
    echo "    release <version>       Создание релиза (например: 0.3.0)"
    echo ""
    echo -e "  ${GREEN}Прочее${NC}"
    echo "    help                    Показать эту справку"
    echo "    menu                    Интерактивное меню"
    echo ""
    echo "Примеры:"
    echo "  ./run.sh build            # Сборка release"
    echo "  ./run.sh test unit        # Unit тесты"
    echo "  ./run.sh wasm             # Сборка WASM"
    echo "  ./run.sh server 3000      # Сервер на порту 3000"
    echo "  ./run.sh examples sequence_demo"
    echo ""
}

# ============================================================================
# Команды
# ============================================================================

cmd_build() {
    local mode=${1:-"release"}
    ./scripts/build.sh "$mode"
}

cmd_check() {
    ./scripts/build.sh check
}

cmd_clean() {
    local mode=${1:-"normal"}
    ./scripts/clean.sh "$mode"
}

cmd_test() {
    ./scripts/test.sh "$@"
}

cmd_wasm() {
    local mode=${1:-"release"}
    ./scripts/wasm.sh "$mode"
}

cmd_server() {
    local port=${1:-8080}
    ./scripts/server.sh "$port"
}

cmd_docs() {
    local mode=${1:-"build"}
    ./scripts/docs.sh "$mode"
}

cmd_examples() {
    ./scripts/examples.sh "$@"
}

cmd_release() {
    if [ -z "$1" ]; then
        echo -e "${RED}Ошибка: укажите версию релиза${NC}"
        echo "Использование: ./run.sh release <версия>"
        echo "Пример: ./run.sh release 0.3.0"
        exit 1
    fi
    ./scripts/release.sh "$1"
}

# ============================================================================
# Интерактивное меню
# ============================================================================

interactive_menu() {
    while true; do
        clear
        print_logo
        print_menu
        
        read -p "Введите номер действия: " choice
        echo ""
        
        case $choice in
            1)  cmd_build release ;;
            2)  cmd_build debug ;;
            3)  cmd_check ;;
            4)  cmd_clean ;;
            5)  cmd_clean all ;;
            6)  cmd_test all ;;
            7)  cmd_test unit ;;
            8)  cmd_test quick ;;
            9)  cmd_wasm ;;
            10) cmd_server ;;
            11) cmd_docs build ;;
            12) cmd_docs open ;;
            13) cmd_examples ;;
            14)
                read -p "Введите версию релиза (например 0.3.0): " version
                cmd_release "$version"
                ;;
            0|q|exit|quit)
                echo -e "${GREEN}До свидания!${NC}"
                exit 0
                ;;
            *)
                echo -e "${RED}Неверный выбор. Попробуйте снова.${NC}"
                ;;
        esac
        
        echo ""
        read -p "Нажмите Enter для продолжения..."
    done
}

# ============================================================================
# Точка входа
# ============================================================================

# Проверка наличия скриптов
if [ ! -d "$SCRIPT_DIR/scripts" ]; then
    echo -e "${RED}Ошибка: папка scripts не найдена${NC}"
    exit 1
fi

# Сделать все скрипты исполняемыми
chmod +x "$SCRIPT_DIR/scripts/"*.sh 2>/dev/null || true

# Обработка аргументов командной строки
if [ $# -eq 0 ]; then
    # Без аргументов — интерактивное меню
    interactive_menu
else
    # С аргументами — выполнить команду напрямую
    case $1 in
        build)      shift; cmd_build "$@" ;;
        check)      cmd_check ;;
        clean)      shift; cmd_clean "$@" ;;
        test)       shift; cmd_test "$@" ;;
        wasm)       shift; cmd_wasm "$@" ;;
        server)     shift; cmd_server "$@" ;;
        docs)       shift; cmd_docs "$@" ;;
        examples)   shift; cmd_examples "$@" ;;
        release)    shift; cmd_release "$@" ;;
        help|-h|--help)
            print_help
            ;;
        menu)
            interactive_menu
            ;;
        *)
            echo -e "${RED}Неизвестная команда: $1${NC}"
            echo ""
            print_help
            exit 1
            ;;
    esac
fi
