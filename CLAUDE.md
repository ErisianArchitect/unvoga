# unvoga / vibevoga

Voxel game engine in Rust on Bevy. Crate name `unvoga`; repo `vibevoga`.

## Commands

```bash
cargo check --all-targets       # Lib + 4 bins + tests + benches
cargo run --bin sandbox         # Main interactive bin (run from repo root)
cargo run --bin visualizer      # Orientation/face debug viewer
cargo run --bin game            # Game shell
cargo test --lib                # Unit tests
cargo bench                     # Criterion (invert_bit)
```

Run binaries from repo root — assets path is `./assets/...` (Bevy looks
relative to cwd, not exe).

## Release builds

Remove `dynamic_linking` feature in `Cargo.toml` before `cargo build --release`:
```toml
bevy = { version = "0.16", features = ["dynamic_linking"] }  # remove for release
```

## Versions (pinned)

- Bevy 0.16, bevy_egui 0.34
- hashbrown with `serde` feature
- Rust edition 2021

## Architecture

- `src/core/` — engine internals (math, voxel, collections, util, io)
- `src/core/voxel/` — block/state/region/world/meshing/lighting
- `src/game/` — game-side glue (cameras, voxel_world, settings)
- `src/bin/{sandbox,game,visualizer,quick}/` — entry points
- `src/main.rs` — default bin (also named `unvoga`)
- `assets/` — shaders (wgsl), textures, debug models

## Block registry (gotcha)

Globals in `src/core/voxel/blocks.rs`. Migrated from `static mut` to
`OnceLock<Mutex<Registry>>` + `Box::leak` for `&'static dyn Block` refs.
**Block trait requires `Send + Sync`.** Adding fields with `Rc`/`RefCell`
to a Block impl will fail to compile.

## VoxelMaterial (Bevy 0.16)

Storage fields are `Handle<ShaderStorageBuffer>`, not `Vec<f32>` (Bevy 0.16
removed inline auto-upload). `VoxelMaterial::new(handle, &mut Assets<ShaderStorageBuffer>)`
populates buffers. Use `storage_buffers.get_mut(&handle).set_data(vec)` to
update.

## Required components (Bevy 0.16)

No `MaterialMeshBundle`/`Camera3dBundle`/`SpriteBundle`/`TransformBundle`.
Spawn:
```rust
commands.spawn((
    Mesh3d(handle), MeshMaterial3d(mat), Transform::from_xyz(...),
));
commands.spawn((
    Camera3d::default(),
    Projection::from(PerspectiveProjection { ... }),
    Transform::...,
));
```

## Known issues

- 2 pre-existing test failures: `sandbox::write_read_test` and
  `core::voxel::region::regionfile::tests::write_read_test` —
  Tag round-trip via RegionFile drops Flip arrays / corrupts U8 buffers.
  Real bug, not env-dependent.
- ~30 panics on hot data paths in `world.rs`/`blockdata.rs`/`regionfile.rs`
  — internal-invariant style, not converted to Result.
- `Result` ambiguity warnings: crate has `core::error::Result` alias
  re-exported via `prelude::*` colliding with `std::result::Result`.
  Use fully-qualified or rename if you touch these.

## Testing notes

`ignore/` directory used as scratch for test region files. Tests assume cwd = repo root.
