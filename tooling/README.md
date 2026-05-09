# tooling/

Blender source files for voxel block models. Not consumed at runtime.

## Files

- `guided_block.blend` — reference cube with face/orientation guides; used to validate orientation tables (`src/core/math/orient_table.rs`).
- `corner_inner_wedge.blend` — inner-corner wedge geometry reference for slope/wedge block variants.

## Workflow

1. Edit in Blender.
2. Export to `assets/debug/` (or appropriate `assets/` subdir) as `.glb` for runtime use.
3. Commit both the `.blend` source and the exported asset.

`.blend1` autosave files are gitignored.

## Notes

- Binary `.blend` bloats history. If edits become frequent, migrate to Git LFS.
- Keep these in sync with the orientation/winding conventions documented in `CLAUDE.md` (Orientation system).
