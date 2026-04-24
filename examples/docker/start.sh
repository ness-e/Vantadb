#!/bin/bash
set -e

# VantaDB Intelligent Entrypoint
# Detects Docker CGroup Memory Limits and injects them to HardwareScout

MEMORY_LIMIT=""

if [ -f "/sys/fs/cgroup/memory.max" ]; then
    # Cgroups v2
    CGROUP_MEM=$(cat /sys/fs/cgroup/memory.max)
    if [ "$CGROUP_MEM" != "max" ]; then
        MEMORY_LIMIT=$CGROUP_MEM
    fi
elif [ -f "/sys/fs/cgroup/memory/memory.limit_in_bytes" ]; then
    # Cgroups v1
    CGROUP_MEM=$(cat /sys/fs/cgroup/memory/memory.limit_in_bytes)
    # 9223372036854771712 indicates no limit
    if [ "$CGROUP_MEM" != "9223372036854771712" ] && [ -n "$CGROUP_MEM" ]; then
        MEMORY_LIMIT=$CGROUP_MEM
    fi
fi

if [ -n "$MEMORY_LIMIT" ]; then
    # Subtract 10% for OS / buffer safety margin
    # Using awk for large number arithmetic natively
    SAFE_LIMIT=$(awk -v mem="$MEMORY_LIMIT" 'BEGIN { printf "%.0f", mem * 0.9 }')
    export VANTADB_MEMORY_LIMIT=$SAFE_LIMIT
    echo "🛡️  [DOCKER] Memory Limit detected: $MEMORY_LIMIT bytes. Setting Safe Cap: $SAFE_LIMIT bytes."
else
    echo "🛡️  [DOCKER] No Memory Limit detected. HardwareScout will use Host RAM."
fi

exec "/usr/local/bin/vanta-server" "$@"
