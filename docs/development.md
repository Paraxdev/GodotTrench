# Development

Contributing with an AI agent? [AGENTS.md](https://github.com/Paraxdev/GodotTrench/blob/main/AGENTS.md) lists the rules
and the checks a pull request has to pass.

## Building

```sh
cargo run -p gt_editor --release            # start the editor
cargo run -p gt_editor --release -- map.gtm # open a map
```

| Requirement | Notes |
| --- | --- |
| GPU | Vulkan, DX12 or Metal. On Linux the window needs X11 or Wayland |
| Rust | 1.95 or newer, the minimum egui 0.36 supports. Cargo stops an older compiler with a clear error |

[rust-toolchain.toml](https://github.com/Paraxdev/GodotTrench/blob/main/rust-toolchain.toml) pins 1.98.1, the version CI
uses, so rustup installs the right toolchain on the first build. Clippy warns when code uses an API newer than 1.95.

## Formatting

Run `tools/fmt --install` once per clone. It enables a pre-commit hook that formats staged Rust.

`tools/fmt` runs rustfmt with
[rustfmt.toml](https://github.com/Paraxdev/GodotTrench/blob/main/rustfmt.toml), then
[tools/space_blocks.py](https://github.com/Paraxdev/GodotTrench/blob/main/tools/space_blocks.py), which adds one blank
line after a multi-line block. It only ever adds blank lines, never touches tokens. `tools/fmt --check` verifies both,
which is what CI runs.

## Tests

```sh
cargo test --workspace                                            # unit tests and headless UI tests
cargo test -p gt_editor --test e2e -- --ignored --test-threads=1  # end to end, drives the editor over MCP

cargo run -p gt_formats --example demo_maps                       # regenerate the Godot test maps
godot --headless --path godot --import                            # run twice on a fresh checkout
godot --headless --path godot --script res://tests/run_tests.gd   # addon tests
```

The addon tests cover the `.gtm` build, I/O, gameplay entities, scatter, blend, C#, live sessions and a showcase
playthrough. The `example_mcp_scripts_replay` end to end test replays every script in `examples/mcp`.

To test the live link without a window, see [Live link protocol](godot/live-link.md#testing-without-a-window).

## Map files in git

`.gtm` maps are compressed binary files, see [the .gtm map format](format-gtm.md), and `.gitattributes` marks them as
binary so git never changes their line endings. To see readable diffs of maps, tell git once per clone to print them as
JSON with the editor:

```sh
git config diff.gtm.textconv "godottrench --dump"
```

`godottrench` has to be on your `PATH`, otherwise give the full path to the binary. The same flag prints any map to the
terminal. `godottrench --to-json map.gtm map.json` writes the readable JSON to a file, which the editor opens and
can save again, and `godottrench --to-gtm map.json map.gtm` turns it back into a binary map with exactly the same
content. Older JSON maps load as they are and are saved in the binary format from then on.

## Generated content

| Command | Regenerates |
| --- | --- |
| `python tools/fetch_demo_textures.py` | The demo's CC0 photo textures and their materials, see [texture size](godot/materials.md#texture-size) |
| `python tools/fetch_demo_props.py` | The CC0 Poly Haven props in `godot/models/polyhaven`, with their alpha maps fixed |
| `cargo run -p gt_samples --bin showcase --release` | The demo Blockbench models |
| `cargo run -p gt_editor --example export_addon_fgd` | The addon's FGD resources from the built in entity definitions |
| `python tools/gen_licenses.py` | The Rust crate table in [LICENSES.md](https://github.com/Paraxdev/GodotTrench/blob/main/LICENSES.md) |

Poly Haven ships glass and flame colors as JPGs, which lose the alpha map. `fetch_demo_props.py` merges it back into a
PNG (`--fix-alpha` does only that) and turns `KHR_materials_transmission`, which Godot ignores, into `BLEND`.

## Documentation

These docs live in `docs/` as a GitBook style book, built with [HonKit](https://github.com/honkit/honkit) and published
to GitHub Pages by `.github/workflows/pages.yml` on every push to `main` that touches them. Pull requests build the book
too, so a broken page fails before it merges.

`docs/SUMMARY.md` is the table of contents. A page not listed there is not built. To preview:

```sh
npx honkit@6.2.2 serve docs          # live preview on http://localhost:4000
npx honkit@6.2.2 build docs _site    # the same build CI runs
```

`.gitbook.yaml` points GitBook's Git Sync at the same folder, so the repository can also back a gitbook.com space.

## CI and releases

GitHub Actions runs fmt, clippy and the Rust tests on Linux and Windows, the end to end tests on Linux with software
Vulkan, and the Godot addon tests.

Every green push to `main` replaces the
[rolling alpha](https://github.com/Paraxdev/GodotTrench/releases/tag/alpha) with editor builds for Linux, Windows and
macOS, the addon zip and the demo project. It also uploads to [itch.io](https://paraxdev.itch.io/godottrench) with
[butler](https://itch.io/docs/butler/):

| Channel | Upload |
| --- | --- |
| `linux`, `windows`, `mac` | The editor builds |
| `addon` | The addon zip |
| `demo-project` | The demo project zip |

Add a repository secret named `BUTLER_API_KEY` to enable the upload, the step skips itself without it.
