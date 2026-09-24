#!/usr/bin/env bash
# End-to-end check of the bin/ toolbox against real mise and real GitHub releases.
# Installs land in a throwaway MISE_DATA_DIR, so the machine's own mise is untouched.
#   bash plugins/ora/tests/toolbox.sh
set -uo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

mise_bin=$(command -v mise) || {
  echo "mise is required to run this test" >&2
  exit 1
}
mkdir -p "$work/mise-only" "$work/fake"
ln -s "$mise_bin" "$work/mise-only/mise"
export MISE_DATA_DIR=$work/mise/data MISE_CACHE_DIR=$work/mise/cache \
  MISE_CONFIG_DIR=$work/mise/config MISE_STATE_DIR=$work/mise/state
# uv's own Python downloads too, or a newer Python already on this machine hides what a stock
# macOS (python3 3.9) gets.
export UV_PYTHON_INSTALL_DIR=$work/uv/python UV_CACHE_DIR=$work/uv/cache
timeout_bin=$(command -v timeout || command -v gtimeout) || {
  echo "timeout (coreutils) is required to run this test" >&2
  exit 1
}
# sd is not on a stock macOS or Ubuntu image, so it exercises the install path.
base=/usr/bin:/bin

failures=0
check() {
  local label=$1 expected=$2 actual=$3
  if [[ $actual == *"$expected"* ]]; then
    echo "ok   $label"
  else
    echo "FAIL $label: expected '$expected', got '$actual'"
    failures=$((failures + 1))
  fi
}

printf '#!/bin/sh\necho system-sd "$@"\n' >"$work/fake/sd"
chmod +x "$work/fake/sd"
out=$(PATH="$root/bin:$work/fake:$base" sd --x 2>&1)
check "a copy already on PATH wins" "system-sd --x" "$out"

out=$(PATH="$root/bin:$base" sd --version 2>&1)
check "exit 127 without mise" "mise is not on PATH" "$out"

out=$(PATH="$root/bin:$work/mise-only:$base" sd --version 2>&1)
check "first use installs through mise" "sd 1." "$out"

start=$(date +%s)
out=$(PATH="$root/bin:$work/mise-only:$base" sd --version 2>&1)
check "second use is served from the cache" "sd 1." "$out"
elapsed=$(($(date +%s) - start))
check "cached call under 2 s" "fast" "$([[ $elapsed -lt 2 ]] && echo fast || echo "${elapsed}s")"

# Linux ships an unrelated /usr/bin/sg, so the aliases are ones no base image carries.
out=$(PATH="$root/bin:$work/mise-only:$base" difft --version 2>&1)
check "aliased name (difft -> difftastic) installs and runs" "Difftastic" "$out"

out=$(PATH="$root/bin:$work/mise-only:$base" ugrep --version 2>&1)
check "a conda-backed tool (ugrep) installs and runs" "ugrep 7." "$out"

# npm-backed, so the install needs node from the caller's PATH.
node_dir=$(dirname "$(command -v node)")
out=$(PATH="$root/bin:$work/mise-only:$node_dir:$base" tinyfish --version 2>&1)
check "an npm-backed tool (tinyfish) installs and runs" "." "$out"

printf '#!/bin/sh\necho "ctags: illegal option -- -" >&2\nexit 1\n' >"$work/fake/ctags"
chmod +x "$work/fake/ctags"
out=$(PATH="$root/bin:$work/fake:$work/mise-only:$base" uctags --version 2>&1)
check "a BSD ctags on PATH is passed over for Universal Ctags" "Universal Ctags" "$out"

out=$(PATH="$root/bin:$work/mise-only:$base" lightpanda version 2>&1)
check "a pinned tool (lightpanda) installs that version" "0.4.1" "$out"

python3 - "$work/hello.pdf" <<'PDF'
import sys
objs = [b"<< /Type /Catalog /Pages 2 0 R >>", b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 100] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>",
        None, b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>"]
stream = b"BT /F1 18 Tf 20 40 Td (hello from ora) Tj ET"
objs[3] = b"<< /Length %d >>\nstream\n" % len(stream) + stream + b"\nendstream"
out, offsets = b"%PDF-1.4\n", []
for i, body in enumerate(objs, 1):
    offsets.append(len(out)); out += b"%d 0 obj\n" % i + body + b"\nendobj\n"
xref = len(out)
out += b"xref\n0 %d\n0000000000 65535 f \n" % (len(objs) + 1) + b"".join(b"%010d 00000 n \n" % o for o in offsets)
out += b"trailer\n<< /Size %d /Root 1 0 R >>\nstartxref\n%d\n%%%%EOF\n" % (len(objs) + 1, xref)
open(sys.argv[1], "wb").write(out)
PDF
out=$(PATH="$root/bin:$work/mise-only:$base" markitdown "$work/hello.pdf" 2>&1)
check "markitdown installs with its PDF extra" "hello from ora" "$out"

cp -R "$root/bin" "$root/libexec" "$work/" 2>/dev/null
mkdir -p "$work/second" && mv "$work/bin" "$work/libexec" "$work/second/"
out=$(PATH="$work/second/bin:$root/bin:$work/mise-only:$base" "$timeout_bin" 60 difft --version 2>&1)
check "two ora installs on PATH do not loop" "Difftastic" "$out"

# A mise or asdf shim with no version set falls through to the next copy on PATH: this link.
mkdir -p "$work/other-shims"
printf '#!/bin/sh\nexec "%s/bin/sd" "$@"\n' "$root" >"$work/other-shims/sd"
chmod +x "$work/other-shims/sd"
out=$(PATH="$work/other-shims:$root/bin:$work/mise-only:$base" "$timeout_bin" 60 sd --version 2>&1)
check "another manager's shim that falls back to this link does not loop" "sd " "$out"

out=$(ORA_TOOLBOX_NAME=sd ORA_TOOLBOX_TRIED=":mise exec" PATH="$root/bin:$work/mise-only:$base" "$root/bin/sd" 2>&1)
check "re-entry after mise exec stops instead of looping" "provides no 'sd' executable" "$out"

echo
if ((failures)); then
  echo "$failures failed"
  exit 1
fi
echo "all passed"
