#!/usr/bin/env bash
# Downloads a python-build-standalone distribution, installs numpy + scipy
# into it, and prints the env-var exports needed to build brush with bundled Python.
#
# Run once per host platform before `cargo build --release`:
#
#   ./scripts/setup_python_standalone.sh
#   # then copy-paste the "export …" lines it prints, or source it:
#   eval "$(./scripts/setup_python_standalone.sh)"
#   cargo build --release -p brush-app
#
# The resulting standalone Python is placed at vendor/python-standalone/.
# Bundle it with your release artifact as python-runtime/ next to the binary.

set -euo pipefail

PYTHON_VERSION="3.12.9"
PBS_DATE="20250317"          # python-build-standalone release tag
OUTDIR="vendor/python-standalone"

# ── Detect platform ────────────────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS-$ARCH" in
    Darwin-arm64)  TRIPLE="aarch64-apple-darwin" ;;
    Darwin-x86_64) TRIPLE="x86_64-apple-darwin" ;;
    Linux-x86_64)  TRIPLE="x86_64-unknown-linux-gnu" ;;
    Linux-aarch64) TRIPLE="aarch64-unknown-linux-gnu" ;;
    *)
        echo "Unsupported platform: $OS-$ARCH" >&2
        echo "See https://github.com/indygreg/python-build-standalone/releases for available triples." >&2
        exit 1
        ;;
esac

ARCHIVE="cpython-${PYTHON_VERSION}+${PBS_DATE}-${TRIPLE}-install_only.tar.gz"
URL="https://github.com/indygreg/python-build-standalone/releases/download/${PBS_DATE}/${ARCHIVE}"

# ── Download ───────────────────────────────────────────────────────────────
mkdir -p "$OUTDIR"
STAMP="$OUTDIR/.setup_done_${PBS_DATE}_${PYTHON_VERSION}"

if [[ -f "$STAMP" ]]; then
    echo "✓ Standalone Python already set up at $OUTDIR" >&2
else
    echo "Downloading $ARCHIVE …" >&2
    TMP="$(mktemp -d)"
    trap 'rm -rf "$TMP"' EXIT

    curl -fL --progress-bar "$URL" -o "$TMP/$ARCHIVE"

    echo "Extracting …" >&2
    # The archive extracts to a 'python/' subdirectory.
    tar -xzf "$TMP/$ARCHIVE" -C "$TMP"
    # Move contents into our vendor dir (overwrite if re-running).
    rm -rf "$OUTDIR"
    mv "$TMP/python" "$OUTDIR"

    touch "$STAMP"
    echo "✓ Extracted to $OUTDIR" >&2
fi

# ── Install Python packages ────────────────────────────────────────────────
PYTHON_BIN="$OUTDIR/bin/python3"

# On macOS the standalone may ship as python3.12 without a python3 symlink.
if [[ ! -f "$PYTHON_BIN" ]]; then
    PYTHON_BIN="$(ls "$OUTDIR/bin/python3."* 2>/dev/null | head -1)"
fi
if [[ -z "$PYTHON_BIN" || ! -f "$PYTHON_BIN" ]]; then
    echo "Could not find a python3 binary in $OUTDIR/bin" >&2
    exit 1
fi

echo "Installing numpy and scipy …" >&2
"$PYTHON_BIN" -m pip install --quiet --upgrade pip
"$PYTHON_BIN" -m pip install --quiet numpy scipy

echo "✓ numpy and scipy installed" >&2

# ── Fix libpython install name on macOS ────────────────────────────────────
# python-build-standalone dylibs have their install name set to the build
# prefix (/install/lib/…), which doesn't exist on the user's machine.
# Change it to @rpath so the linker's rpath entries resolve it at runtime.
if [[ "$OS" == "Darwin" ]]; then
    DYLIB="$(ls "$OUTDIR"/lib/libpython3.*.dylib 2>/dev/null | head -1)"
    if [[ -n "$DYLIB" ]]; then
        DYLIB_NAME="$(basename "$DYLIB")"
        echo "Fixing dylib install name: $DYLIB_NAME …" >&2
        install_name_tool -id "@rpath/$DYLIB_NAME" "$DYLIB"
        # Re-sign with an ad-hoc signature; macOS refuses to load a modified
        # dylib whose original signature no longer matches.
        codesign --sign - --force "$DYLIB"
        echo "✓ install name fixed and dylib re-signed" >&2
    fi
fi

# ── Print exports ──────────────────────────────────────────────────────────
ABS_OUTDIR="$(cd "$OUTDIR" && pwd)"

cat <<EOF
export PYO3_PYTHON="${ABS_OUTDIR}/bin/python3"
export BRUSH_PYTHON_HOME="${ABS_OUTDIR}"
EOF
