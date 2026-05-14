# brush-palette-viewer

A real-time editor for palette-based 3D Gaussian Splatting, forked from [Brush](https://github.com/ArthurBrussee/brush). Edit palette colors and tone curves with the optimizer-driven solver running live in the loop.

**Tested on macOS Apple Silicon. Linux (x86-64) and Windows (x86-64) builds are supported but less tested.**

## Quickstart

### 1. Clone

```bash
git clone https://github.com/tedchao/brush-palette-viewer.git
cd brush-palette-viewer
```

### 2. Set up bundled Python

The viewer calls a Python optimizer (numpy + scipy) via PyO3. No conda or system Python required — run this once to download a self-contained Python 3.12 and install the dependencies:

```bash
./scripts/setup_python_standalone.sh
```

The paths are wired into `.cargo/config.toml`, so no environment variables need to be set or exported — `cargo build` picks them up automatically after this.

### 3. Build

```bash
cargo build --bin brush
```

First build takes a few minutes. Incremental rebuilds are 10-20 seconds.

### Distributing

**Via CI** — push a version tag and the GitHub Actions workflow builds and packages a release for all three platforms automatically:

```bash
git tag v0.3.1 && git push origin v0.3.1
```

**Locally** — build in release mode, then copy the standalone Python and scripts alongside the binary:

```bash
cargo build --release --bin brush

DEST=dist/brush-$(uname -s)-$(uname -m)
mkdir -p "$DEST"
cp target/release/brush "$DEST/"                     # brush.exe on Windows
cp -r vendor/python-standalone "$DEST/python-runtime"
cp -r crates/brush-palette/python "$DEST/python-scripts"
```

Either way the archive contains the binary alongside `python-runtime/` and `python-scripts/`. End-users need nothing installed.

### 4. Run

Each scene lives in `models/<name>/` as a `name.pply` + `name.gswp` pair. Pass the `.pply` path; the `.gswp` is auto-discovered alongside it.

```bash
./target/debug/brush --with-viewer models/statue/statue.pply
```

(after lauching, click the small gear icon on the top-right of the window and select "Recenter view" button in the very end of the pop-up window.)

## Controls

- **Left drag** orbit, **right drag** look around, **middle drag** pan, **scroll** zoom
- **WASD** / **QE** fly, **shift** for faster movement
- **C (shift+c)** toggle pixel-click constraint mode → click in the viewport to add an image-space constraint
- **F** fullscreen

## UI overview

- **Palette window** (top right): click any palette swatch to edit colors. Check "Edit tone curves" to overlay and edit per-palette tone curves. Edits trigger the constraint optimizer in real time.
- **Image-space constraints window**: appears once you place pixel constraints. Each row shows the original → target color; click the target to edit, click the original to jump back to the saved view, 💾 to export the swatch pair, ✕ to remove.
- **Top bar buttons**: 📷 save current view as PNG, ⚖ save K weight images (RGBA, one per palette index) to a folder, ⚙ open settings (FOV, splat scale, fly speed, grid, background color, auto-rotate turntable, recenter, reset layout).
- **Save palette** in the palette window: exports the palette as PNG (horizontal or vertical, color-only or with curves).

## Vanilla `.ply` files

The original Brush rendering path is untouched. Vanilla `.ply` files (without an adjacent `.gswp`) render through the upstream Brush shaders.

```bash
./target/debug/brush --with-viewer path/to/vanilla.ply
```

## License

Apache 2.0, following upstream Brush.