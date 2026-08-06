#!/usr/bin/env bash
# Automated CLI / packaging smoke checks for desktop integration.
# GUI items (Ctrl+Alt+T, Nautilus menu, Wayland/X11) are listed as a manual checklist.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${EL_TERMINAL_BIN:-$ROOT/target/release/el-terminal}"
TMP="$(mktemp -d /tmp/el-terminal-desktop-XXXXXX)"
trap 'rm -rf "$TMP"' EXIT

pass=0
fail=0

ok() { echo "OK  $*"; pass=$((pass + 1)); }
bad() { echo "FAIL $*"; fail=$((fail + 1)); }

if [[ ! -x "$BIN" ]]; then
  echo "Building release binary…"
  (cd "$ROOT" && cargo build --release)
fi

HELP="$("$BIN" --help-all 2>&1 || true)"
for opt in --working-directory --dir --new-window --command -e; do
  if grep -q -- "$opt" <<<"$HELP"; then
    ok "help lists $opt"
  else
    bad "help missing $opt"
  fi
done

# Path fixtures
mkdir -p "$TMP/normal/nested/deep"
mkdir -p "$TMP/with spaces/inner"
mkdir -p "$TMP/ünïcode-πάθος"
mkdir -p "$TMP/--dash-name"
ln -s "$TMP/normal/nested" "$TMP/symlink-to-nested"

run_cwd_case() {
  local label=$1
  local dir=$2
  local flag=$3
  local marker="$TMP/marker-${label//[^a-zA-Z0-9]/_}"
  rm -f "$marker"
  # Command writes pwd then exits; window should close.
  if ! "$BIN" "$flag=$dir" --command "pwd > '$marker'"; then
    bad "$label: process exited non-zero"
    return
  fi
  if [[ ! -f "$marker" ]]; then
    bad "$label: no marker written"
    return
  fi
  local got
  got="$(<"$marker")"
  local want
  want="$(readlink -f "$dir" 2>/dev/null || realpath "$dir")"
  if [[ "$got" == "$want" ]]; then
    ok "$label → $got"
  else
    bad "$label: expected '$want', got '$got'"
  fi
}

run_cwd_case "working-directory nested" "$TMP/normal/nested/deep" --working-directory
run_cwd_case "dir spaces" "$TMP/with spaces/inner" --dir
run_cwd_case "dir unicode" "$TMP/ünïcode-πάθος" --dir
run_cwd_case "dir symlink" "$TMP/symlink-to-nested" --working-directory
# Dash-leading path via equals form
run_cwd_case "dir dash-name" "$TMP/--dash-name" --working-directory

# Desktop entry sanity
DESKTOP="$ROOT/packaging/el-terminal.desktop"
for key in "Type=Application" "Name=El-Terminal" "Exec=el-terminal" "Terminal=false" \
  "Categories=System;TerminalEmulator;" "X-TerminalArgDir=--working-directory" \
  "X-TerminalArgExec=-e" "TryExec=el-terminal"; do
  if grep -qxF "$key" "$DESKTOP"; then
    ok "desktop: $key"
  else
    bad "desktop missing: $key"
  fi
done

# Nautilus extension present and uses argv launch (no shell=True / os.system)
EXT="$ROOT/packaging/nautilus/el_terminal_nautilus.py"
if [[ -f "$EXT" ]]; then
  ok "nautilus extension file exists"
else
  bad "nautilus extension missing"
fi
if grep -qF 'Gio.Subprocess.new' "$EXT" && grep -qF -- '--working-directory' "$EXT"; then
  ok "nautilus launches via Gio.Subprocess + --working-directory"
else
  bad "nautilus extension does not use safe argv launch"
fi
if grep -Eq 'os\.system|shell=True|bash -c' "$EXT"; then
  bad "nautilus extension appears to use a shell"
else
  ok "nautilus extension avoids shell invocation"
fi

echo
echo "Results: $pass passed, $fail failed"
echo
cat <<'EOF'
Manual checklist (not automated):
  1. Open el-terminal normally
  2. Set as default: sudo update-alternatives --config x-terminal-emulator
  3. Press Ctrl+Alt+T
  4. Open Nautilus; install el-terminal-nautilus; nautilus -q
  5. Right-click folder / background → Open in El-Terminal
  6. Verify cwd matches the folder (spaces, Unicode, nested, symlink)
  7. Multiple Nautilus windows and multiple terminal windows
  8. Wayland session; X11 if available
  9. Ask AI with share context off/on — indicator appears only when sharing
EOF

exit "$fail"
