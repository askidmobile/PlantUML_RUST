#!/bin/bash
# ============================================================================
# limit-memory.sh — жёсткий лимит памяти на команду
# ============================================================================
#
# Запускает команду и убивает её дерево процессов, если суммарный RSS
# превысит лимит. Нужно потому, что на macOS НЕ РАБОТАЮТ штатные механизмы:
#   ulimit -v / -d   — не поддерживаются ядром;
#   taskpolicy -m    — проверено, лимит игнорируется (процесс выделил
#                      3.9 ГБ при лимите 200 МиБ);
#   launchctl limit  — только системные значения, не на процесс.
#
# Использование:
#   ./scripts/limit-memory.sh 1024 cargo test --workspace
#   ./scripts/limit-memory.sh 2048 cargo build --release
#
# По умолчанию лимит 1 ГБ на дерево процессов.
# Код возврата 137 означает, что команда убита по превышению лимита.

LIMIT_MB="${1:-1024}"
shift

if [ $# -eq 0 ]; then
    echo "использование: $0 <лимит_МиБ> <команда...>" >&2
    exit 2
fi

"$@" &
PID=$!
LIMIT_KB=$((LIMIT_MB * 1024))
PEAK=0

while kill -0 "$PID" 2>/dev/null; do
    # Суммарный RSS процесса и ВСЕХ его потомков: cargo порождает
    # несколько rustc, и считать только родителя бесполезно.
    RSS=$(ps -Ao pid,ppid,rss | awk -v root="$PID" '
        {pid[NR]=$1; ppid[NR]=$2; rss[NR]=$3}
        END {
            inc[root]=1
            for (pass=0; pass<10; pass++)
                for (i in pid) if (inc[ppid[i]]) inc[pid[i]]=1
            for (i in pid) if (inc[pid[i]]) s+=rss[i]
            print s+0
        }')

    [ "$RSS" -gt "$PEAK" ] && PEAK=$RSS

    if [ "$RSS" -gt "$LIMIT_KB" ]; then
        echo "ПРЕВЫШЕН ЛИМИТ: $((RSS/1024)) МиБ > ${LIMIT_MB} МиБ — убиваю" >&2
        pkill -P "$PID" 2>/dev/null
        kill -9 "$PID" 2>/dev/null
        wait "$PID" 2>/dev/null
        echo "ПИК: $((PEAK/1024)) МиБ" >&2
        exit 137
    fi

    sleep 0.05
done

wait "$PID"
RC=$?
echo "ПИК: $((PEAK/1024)) МиБ (лимит ${LIMIT_MB} МиБ)" >&2
exit $RC
