# vibevoga

Voxel engine in Rust on Bevy. Crate name is `unvoga` because the original
author called it that. The repo is `vibevoga` because the fork started as
a joke after a "no LLM was involved" post on r/rust — turns out the
codebase had bugs, was stuck on Bevy 0.13, and could use the help.

Now on Bevy 0.16. Renders. Generates a flat world. Flickers a bit when
you click. Working on it.

## Run

```bash
cargo run --bin sandbox
cargo run --bin citygen   # emit a demo public-data-style city tile as JSON
cargo run --bin city_viewer
```

Run from the repo root (Bevy resolves assets relative to cwd).

Controls:

- WASD + mouse — move/look (cursor confined to window)
- Left click — place selected block
- Right click — break
- Number keys — switch block type
- Esc — save world and exit

World saves to `ignore/worldgen/`. To start fresh:

```bash
rm -rf ignore/worldgen
```

## Build

```bash
cargo build --bin sandbox
cargo test --lib
cargo bench --bench chunk_occlusion   # voxel meshing perf
```

Toolchain is pinned to Rust 1.94.0 via `rust-toolchain.toml` (edition 2024).

For release, remove the `dynamic_linking` feature in `Cargo.toml`:

```toml
bevy = { version = "0.16", features = ["dynamic_linking"] }  # drop the feature
```

## What's in here

- `src/core/voxel/` — block/state/region/world/meshing/occlusion/lighting
- `src/core/city/` — geospatial projection, city tile data, demo city generation
- `src/core/math/` — orientation, rotation, flip, bit primitives
- `src/bin/sandbox/` — interactive test app
- `src/bin/visualizer/` — face/orientation debug viewer
- `assets/shaders/voxel/voxel.wgsl` — the chunk shader
- `benches/` — criterion suites driving the perf work
- `docs/lookup_table_audit.md` — orientation lookup table audit coverage
- `tooling/` — Blender source files for block models (see `tooling/README.md`)

## Notable bugs the fork fixed

- `bit::move_bits` was reading from already-mutated state during
  permutation. Snapshot the source first.
- `Occluder::occluded_by` Rect-vs-S{8,4,2} arms read out of bounds on
  the destination grid because they iterated the un-downsampled rect.
- `OcclusionRect::rotate` computed `ymax` from x coordinates.
- `ObjectPool::pop()` shrank the pool but left index slots pointing
  past the new end. Subsequent `remove()` indexed past the end and
  panicked. Now idempotent.
- `world.rs::hide_face` had an inverted dirty mark — neighbor sections
  weren't getting re-meshed on edits.
- Six `static mut` UB sites. Refactored to `OnceLock<Mutex<…>>`.

## Perf wins from the bench suite

The default-orientation case dominates chunk meshing. Bitwise fast
paths in `Occluder::occluded_by` for that case:

```
chunk_occlusion (16³ block scan, 24K face-pair queries)
all_detailed_16x16:  10.1 ms  ->  191 µs   (~53× faster)
mixed:                620 µs  ->  300 µs   (~2× faster)
```

See `src/core/voxel/occluder.rs` for the fast-path arms and the
matching equivalence tests.

## Status

Not playable. Engine launches, terrain renders, blocks place/remove
without flicker (fixed: populate mesh on spawn + share material across
chunks). Chunk-pool churn under high render distance is still fragile.
Patches welcome.

## License

See `LICENSE`.
