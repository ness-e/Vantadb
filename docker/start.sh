#!/bin/bash
set -e

# Autodetect RAM limit via cgroups
MEMORY_LIMIT_BYTES=0

if [ -f /sys/fs/cgroup/memory/memory.limit_in_bytes ]; then
    # cgroup v1
    MEMORY_LIMIT_BYTES=$(cat /sys/fs/cgroup/memory/memory.limit_in_bytes)
elif [ -f /sys/fs/cgroup/memory.max ]; then
    # cgroup v2
    CGROUP_MEM=$(cat /sys/fs/cgroup/memory.max)
    if [ "$CGROUP_MEM" != "max" ]; then
        MEMORY_LIMIT_BYTES=$CGROUP_MEM
    fi
fi

# 9223372036854771712 is effectively unbounded
if [ "$MEMORY_LIMIT_BYTES" -eq "9223372036854771712" ] || [ "$MEMORY_LIMIT_BYTES" -eq "0" ]; then
    echo "[NexusDB] Starting with unbounded memory limits."
else
    # Configurar variable de entorno para que StorageEngine / Governor pueda atraparla
    export NEXUSDB_MAX_MEMORY=$MEMORY_LIMIT_BYTES
    MEM_MB=$(($MEMORY_LIMIT_BYTES / 1024 / 1024))
    echo "[NexusDB] Docker limit detected: $MEM_MB MB. Survival Mode configured."
fi

# Iniciar servidor
exec nexusdb "$@"
