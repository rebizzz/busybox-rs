#!/usr/bin/env bash
set -euo pipefail

CONFIG_FILE="${1:-.config}"

if [ ! -f "$CONFIG_FILE" ]; then
    echo "Config file $CONFIG_FILE not found" >&2
    exit 1
fi

FEATURES=()
while IFS= read -r line || [ -n "$line" ]; do
    line="$(echo "$line" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
    if [[ "$line" =~ ^CONFIG_([A-Za-z0-9_]+)=y$ ]]; then
        NAME="${BASH_REMATCH[1],,}"
        FEATURES+=("$NAME")
    fi
done < "$CONFIG_FILE"

FEATURE_LIST=$(IFS=,; echo "${FEATURES[*]}")
echo "$FEATURE_LIST"
