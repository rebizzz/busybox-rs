#!/usr/bin/env bash
set -euo pipefail

TARGET="${1:-target/release/busybox}"

if [ ! -f "$TARGET" ]; then
    echo "Binary $TARGET does not exist. Run 'make' or 'cargo build --release' first." >&2
    exit 1
fi

SIZE=$(stat -c %s "$TARGET")
echo "=== BusyBox Binary Size Report ==="
echo "Path: $TARGET"
echo "Bytes: $SIZE"
echo "Readable: $(ls -lh "$TARGET" | awk '{print $5}')"

if command -v size >/dev/null 2>&1; then
    echo "--- Section Sizes ---"
    size "$TARGET"
fi

if command -v readelf >/dev/null 2>&1; then
    echo "--- Key ELF Sections ---"
    readelf -S "$TARGET" | grep -E "\.text|\.rodata|\.eh_frame|\.data" || true
fi
