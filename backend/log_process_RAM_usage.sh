#!/usr/bin/env bash

if [ -z "$1" ]; then
    echo "Usage: $0 [<binary-name> ...]"
    exit 1
fi

echo "RAM usage as on $(date)" >> memory_usage.out

for bin in "$@"
do
    process_id="$(pgrep "${bin}")"
    memory_usage_in_kb="$(pmap "${process_id}" | tail -n 1 | awk '{print $2}' | rev | cut -c2- | rev)"
    memory_usage_in_mb="$((memory_usage_in_kb / 1024))"
    echo "${bin} ${memory_usage_in_kb}kb ${memory_usage_in_mb}mb" >> memory_usage.out
done

echo "============================================================" >> memory_usage.out

cat memory_usage.out
