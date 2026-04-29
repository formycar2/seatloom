#!/usr/bin/env bash
# verify-rust-foundation.sh
# Bounded Rust foundation checks for SeatLoom.
# Runs the same sequence used by CI.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

echo "=== SeatLoom Rust Foundation Verification ==="
echo "repo root: $REPO_ROOT"
echo ""

# Source cargo env if available (handles seats where cargo is not in default PATH)
if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck source=/dev/null
  . "$HOME/.cargo/env"
fi

# Resolve cargo from active toolchain
CARGO="$(command -v cargo 2>/dev/null || true)"
if [ -z "$CARGO" ] || [ ! -x "$CARGO" ]; then
  echo "ERROR: cargo not found. Install Rust via https://rustup.rs" >&2
  exit 1
fi

echo "cargo:  $($CARGO --version)"
echo "rustc:  $(rustc --version 2>/dev/null || echo 'not found')"
echo "clippy: $(cargo clippy --version 2>/dev/null || echo 'not found')"
echo "rustfmt: $(rustfmt --version 2>/dev/null || echo 'not found')"
echo ""

echo "--- Step 1: cargo check (workspace) ---"
$CARGO check
echo ""

echo "--- Step 2: cargo test -p seatloom-core ---"
$CARGO test -p seatloom-core
echo ""

echo "--- Step 3: cargo fmt --all --check ---"
$CARGO fmt --all --check
echo ""

echo "--- Step 4: cargo clippy -p seatloom-core --all-targets -- -D warnings ---"
$CARGO clippy -p seatloom-core --all-targets -- -D warnings
echo ""

echo "=== All checks passed ==="
