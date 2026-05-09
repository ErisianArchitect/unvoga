# Lookup Table Audit

This note documents the audit coverage for the voxel orientation lookup tables.
The tables are intentionally retained because the orientation system is on a hot
path and the precomputed maps avoid recalculating face coordinate transforms at
runtime.

## Scope

The audit covers the orientation and face-coordinate tables used by:

- `Rotation::reface`, `Rotation::source_face`, `Rotation::rotate`
- `Rotation::from_up_and_forward`
- `Orientation::reface`, `Orientation::source_face`, `Orientation::transform`
- `Orientation::reorient`, `Orientation::deorient`, `Orientation::invert`
- `Orientation::map_face_coord`
- `Orientation::source_face_coord`
- `MAP_COORD_TABLE`
- `SOURCE_FACE_COORD_TABLE`

The active audit tests live in `src/core/math/orientation.rs`.

## What Is Verified

The test suite now checks every valid rotation and orientation:

- 24 rotations: 6 possible up directions times 4 angles
- 8 flips: X/Y/Z bit combinations
- 192 full orientations
- 6 cube faces per orientation

The rotation tests verify that:

- all 24 rotations produce unique face mappings
- left/right, up/down, and forward/backward are inverse pairs
- rotating a unit face normal lands on the same face reported by `reface`
- `source_face` and `reface` are exact inverses
- `from_up_and_forward` resolves to the same rotation found by exhaustive search

The orientation tests verify that:

- orientation pack/unpack round trips for every orientation
- transforming a unit face normal agrees with `reface`
- `source_face` and `reface` are exact inverses after rotation and flip
- `invert`, `reorient`, and `deorient` round trip correctly
- every pairwise orientation composition round trips

The face-coordinate table tests verify that:

- `MAP_COORD_TABLE` matches an algorithmic reference implementation
- `SOURCE_FACE_COORD_TABLE` matches an algorithmic reference implementation
- source and target face coordinate mappings are mutual inverses
- representative centered UV coordinates round trip across every orientation and face

## Generator Tests

The old table-generation tests are still present, but marked `#[ignore]` so
normal test runs verify the current tables without writing generated files under
`ignore/`.

Run ignored generator tests explicitly only when regenerating table data:

```bash
cargo test orientation --lib -- --ignored
```

## How To Run The Audit

Focused orientation audit:

```bash
cargo test orientation --lib
```

Full library regression pass:

```bash
cargo test --lib
```

## Current Result

The current lookup tables pass the focused orientation audit and the full library
test suite.

No lookup table inaccuracies were found by the audit.

## Caveat

This audit verifies internal mathematical consistency and checks the table data
against the algorithmic face-coordinate reference in the codebase. It does not
claim that the chosen default face-up/face-right conventions are the only valid
conventions, only that the implemented conventions are complete, invertible, and
accurately represented by the lookup tables.
