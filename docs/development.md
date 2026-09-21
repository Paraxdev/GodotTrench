# Development

## Building

```sh
cargo run -p gt_editor --release            # start the editor
cargo run -p gt_editor --release -- map.gtm # open a map
```

Requires a GPU with Vulkan, DX12 or Metal support. On Linux the window needs X11 or Wayland.

## Formatting

All Rust is formatted with the settings in [rustfmt.toml](../rustfmt.toml), so hand written and AI written commits share one
style. Run `tools/fmt --install` once per clone to enable the pre-commit hook in [.githooks](../.githooks); it formats staged
Rust before each commit. `tools/fmt` formats the workspace on demand and `tools/fmt --check` verifies it, which is what CI runs.

## Tests

```sh
cargo test --workspace                                            # unit tests + headless UI tests (egui_kittest)
cargo test -p gt_editor --test e2e -- --ignored --test-threads=1  # end to end: launches the editor, drives it over MCP

cargo run -p gt_formats --example demo_maps                       # regenerate the Godot test maps
godot --headless --path godot --import                            # run twice on a fresh checkout
godot --headless --path godot --script res://tests/run_tests.gd   # addon tests: .gtm build, I/O, gameplay entities, scatter, blend, C#, live sessions, a showcase playthrough
godot --headless --path godot --script res://tests/bench_build.gd -- runs=5 threaded=1  # time each build step of the showcase maps
```

To try the live link end to end without a window, open a scene with a `FuncGodotMap` in a headless Godot editor
(`godot --headless --editor --path godot res://path/scene.tscn`), start the editor with `--mcp-http --project godot`, and
check what Godot built with the `inspect` event described in [Working with Godot](godot.md).

The `example_mcp_scripts_replay` end to end test replays every script in `examples/mcp` and checks the result.

## Generated content

* `cargo run -p gt_samples --bin showcase --release` regenerates the showcase textures and the demo Blockbench models.
* `cargo run -p gt_editor --example export_addon_fgd` regenerates the addon's FGD resources from the built in entity definitions.
* `python tools/gen_licenses.py` regenerates the Rust crate table in [LICENSES.md](../LICENSES.md).

## CI and releases

GitHub Actions runs fmt, clippy and the Rust tests on Linux and Windows, the end to end tests on Linux with software Vulkan,
and the Godot addon tests. Every green push to `main` replaces the
[rolling alpha](https://github.com/Paraxdev/GodotTrench/releases/tag/alpha) with editor builds for Linux, Windows and macOS,
the addon zip and the demo project.

The same push also uploads the three editor builds to [itch.io](https://paraxdev.itch.io/godottrench) with
[butler](https://itch.io/docs/butler/) on the `linux`, `windows` and `mac` channels. Add a repository secret named
`BUTLER_API_KEY` (a butler API key from itch) to enable it; the step skips itself while the secret is absent.
