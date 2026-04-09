#!/bin/bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
LIBCLANG_ROOT="${ROOT_DIR}/../.local-apt/libclang18/extract/usr/lib/llvm-18/lib"
TARGET_DIR="${ROOT_DIR}/target/thumbv7em-none-eabihf/release"
DIST_DIR="${ROOT_DIR}/dist"

if [[ ! -e "${LIBCLANG_ROOT}/libclang.so.1" ]]; then
  if [[ -n "${CI:-}" ]]; then
    # In CI, use system libclang
    export LIBCLANG_PATH="/usr/lib/llvm-18/lib"
  else
    echo "libclang not found at ${LIBCLANG_ROOT}"
    echo "Run a single build first to populate the local libclang bundle."
    exit 1
  fi
else
  export LIBCLANG_PATH="${LIBCLANG_ROOT}"
fi

modules=(
  left_encoder
  left_trackball
  left_analog_stick
  right_encoder
  right_trackball
  right_analog_stick
)

mkdir -p "${DIST_DIR}"

# Build default first to generate keyboard.toml
echo "Building default firmware..."
MODULE="default" cargo build --release
install -m 755 "${TARGET_DIR}/rmk-mkb2" "${DIST_DIR}/default.elf"
install -m 644 "${ROOT_DIR}/keyboard.toml" "${DIST_DIR}/default.toml"

for module in "${modules[@]}"; do
  echo "Building ${module} firmware..."
  MODULE="${module}" cargo build --release
  install -m 755 "${TARGET_DIR}/rmk-mkb2" "${DIST_DIR}/${module}.elf"
  install -m 644 "${ROOT_DIR}/keyboard.toml" "${DIST_DIR}/${module}.toml"
done

echo "All firmware builds completed."
echo "Artifacts written to ${DIST_DIR}"
