# Reflection Puzzler

A small puzzle game built with Bevy where you rotate mirrors to guide a light beam from the source to the target.

## Features

- Multiple levels loaded from JSON assets
- Click-to-rotate mirrors
- Fixed and rotatable mirror pieces
- Native build and wasm target

## Project Layout

- `src/` - gameplay code (grid, beam, input, UI, level loading)
- `assets/levels.json` - level manifest loaded at runtime
- `index.html` - Trunk entry for web build

## Run in Browser (WASM)

WASM build:

```bash
cargo build-wasm
```

Serve with Trunk:

```bash
trunk serve
```

## Controls

- Left click on a rotatable mirror: rotate mirror
- Menu button: return to level select
- Reset button: restore mirrors to original orientation
- Back to Menu (win overlay): return to level select
