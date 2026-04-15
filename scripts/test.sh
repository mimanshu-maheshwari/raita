#!/usr/bin/env bash

set -uo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MAELSTROM_BIN="${MAELSTROM_BIN:-"$ROOT_DIR/res/maelstrom/maelstrom"}"
TARGET_DIR="${TARGET_DIR:-target-codex}"
PROFILE="${PROFILE:-debug}"

add_to_path_if_exists() {
  local candidate="$1"
  if [ -d "$candidate" ]; then
    case ":$PATH:" in
      *":$candidate:"*) ;;
      *) PATH="$PATH:$candidate" ;;
    esac
  fi
}

setup_windows_path_helpers() {
  add_to_path_if_exists "/c/Users/miman/.cargo/bin"
  add_to_path_if_exists "/c/Program Files/Common Files/Oracle/Java/javapath"
}

require_command() {
  local command_name="$1"
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "Missing required command: $command_name" >&2
    exit 1
  fi
}

setup_windows_path_helpers
require_command cargo
require_command java

build() {
  cargo build --bins --target-dir "$TARGET_DIR"
}

run_workload() {
  local name="$1"
  shift

  echo "==> Running ${name}"
  if "$@"; then
    echo "==> ${name} completed successfully"
    return 0
  fi

  echo "==> ${name} exited with a non-zero status; continuing so saved Maelstrom artifacts can still be inspected" >&2
  return 0
}

bin_path() {
  local name="$1"
  printf '%s/%s/%s/%s' "$ROOT_DIR" "$TARGET_DIR" "$PROFILE" "$name"
}

run_echo() {
  "$MAELSTROM_BIN" test \
    -w echo \
    --bin "$(bin_path echo)" \
    --node-count 1 \
    --time-limit 10
}

run_unique_ids() {
  "$MAELSTROM_BIN" test \
    -w unique-ids \
    --bin "$(bin_path unique_id)" \
    --node-count 3 \
    --time-limit 30 \
    --rate 1000 \
    --availability total \
    --nemesis partition
}

run_broadcast_basic() {
  "$MAELSTROM_BIN" test \
    -w broadcast \
    --bin "$(bin_path broadcast)" \
    --node-count 3 \
    --time-limit 20 \
    --rate 10
}

run_broadcast_strict() {
  "$MAELSTROM_BIN" test \
    -w broadcast \
    --bin "$(bin_path broadcast)" \
    --node-count 5 \
    --time-limit 20 \
    --rate 100 \
    --topology tree
}

run_g_counter() {
  "$MAELSTROM_BIN" test \
    -w g-counter \
    --bin "$(bin_path g_counter)" \
    --node-count 3 \
    --rate 100 \
    --time-limit 20 \
    --nemesis partition
}

main() {
  local target="${1:-all}"
  build

  case "$target" in
    echo)
      run_workload "echo" run_echo
      ;;
    unique-id|unique-ids)
      run_workload "unique-ids" run_unique_ids
      ;;
    broadcast-basic)
      run_workload "broadcast-basic" run_broadcast_basic
      ;;
    broadcast|broadcast-strict)
      run_workload "broadcast" run_broadcast_strict
      ;;
    g-counter)
      run_workload "g-counter" run_g_counter
      ;;
    all)
      run_workload "echo" run_echo
      run_workload "unique-ids" run_unique_ids
      run_workload "broadcast-basic" run_broadcast_basic
      run_workload "broadcast" run_broadcast_strict
      run_workload "g-counter" run_g_counter
      ;;
    *)
      echo "Usage: $0 [echo|unique-ids|broadcast-basic|broadcast|g-counter|all]" >&2
      exit 1
      ;;
  esac
}

main "$@"
