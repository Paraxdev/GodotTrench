# Development

Contributing with an AI agent? [AGENTS.md](https://github.com/Paraxdev/GodotTrench/blob/main/AGENTS.md) lists the rules
and the checks a pull request has to pass.

## Building

The Godot addon in `godot/addons/func_godot` is a git submodule, so clone with
`git clone --recursive https://github.com/Paraxdev/GodotTrench.git`, or run `git submodule update --init` in an
existing clone. The editor builds without it, but the tests and the Godot project need it.

```sh
cargo run -p gt_editor --release            # start the editor
cargo run -p gt_editor --release -- map.gtm # open a map
```

You need a GPU with Vulkan, DX12 or Metal, and on Linux an X11 or Wayland session. Rust 1.95 is the minimum, since
egui 0.36 needs it. [rust-toolchain.toml](https://github.com/Paraxdev/GodotTrench/blob/main/rust-toolchain.toml) pins
1.98.1, the version CI uses, so rustup installs it on the first build.

## Checks

Run `./tools/fmt --install` once per clone. It enables a pre-commit hook that formats staged Rust.

```sh
./tools/fmt --check
python tools/gen_licenses.py --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p gt_editor --test e2e -- --ignored --test-threads=1
godot --headless --path godot --import                            # twice on a fresh checkout
godot --headless --path godot --script res://tests/run_tests.gd
godot --headless --path godot --script res://tests/build_showcase.gd
```

CI runs all of these. The e2e tests run on Linux with software Vulkan and drive the editor over MCP, and
`example_mcp_scripts_replay` replays every script in `examples/mcp`. To test the live link without a window, see
[Live link protocol](godot/live-link.md#testing-without-a-window).

To time map builds, run `godot --headless --path godot --script res://tests/bench_build.gd -- runs=5 threaded=1`.
Do not time them with `build_showcase.gd`, it overwrites the committed showcase scenes.

## Map files in git

`.gtm` maps are binary, see [the .gtm map format](format/README.md). For readable diffs, tell git once per clone to print
them as JSON with the editor, which has to be on your `PATH`:

```sh
git config diff.gtm.textconv "godottrench --dump"
```

`godottrench --to-json map.gtm map.json` writes the JSON to a file, and `godottrench --to-gtm map.json map.gtm` turns
it back into a binary map with the same content.

## Generated content

| Command | Regenerates |
| --- | --- |
| `cargo run -p gt_formats --example demo_maps` | The test maps in `godot/tests/maps` and the small demo maps in `godot/demo/maps` |
| `python tools/fetch_demo_textures.py` | The demo's CC0 photo textures, see [texture size](godot/materials.md#texture-size) |
| `python tools/fetch_demo_props.py` | The CC0 Poly Haven props in `godot/models/polyhaven`, with the alpha maps of glass and flames restored. `--fix-alpha` only restores them |
| `cargo run -p gt_samples --bin showcase --release` | The demo Blockbench models |
| `cargo run -p gt_editor --example export_addon_fgd` | The addon's FGD files from the built in entity definitions, written into the addon submodule |
| `python tools/gen_licenses.py` | The Rust crate table in [LICENSES.md](https://github.com/Paraxdev/GodotTrench/blob/main/LICENSES.md) |

## The Godot addon

The addon has its own repository, [godottrench_func](https://github.com/Paraxdev/godottrench_func), checked out at
`godot/addons/func_godot`. Its tests stay here in `godot/tests`, since they need the demo project. To change the addon:

1. Commit the change inside `godot/addons/func_godot` and push it to godottrench_func.
2. Stage the new submodule commit here with `git add godot/addons/func_godot` and commit it with the GodotTrench
   changes that need it.

Push the addon first. A GodotTrench commit that points at an addon commit GitHub does not have fails to check out in
CI.

## Documentation

The docs are a HonKit book in `docs/`. `docs/SUMMARY.md` is the table of contents, a page not listed there is not
built. `docs/package.json` pins HonKit and the
[darkening theme](https://github.com/Stuyk/honkit-plugin-theme-darkening), and `docs/styles/website.css` adjusts it.

```sh
cd docs
npm ci                        # once
npx honkit serve              # live preview on http://localhost:4000
npx honkit build . _site      # the build CI runs, lands in docs/_site
```

`.github/workflows/pages.yml` builds the book for pull requests that touch `docs/`, and publishes it to GitHub Pages
when they reach `main`.

## CI and releases

CI skips pushes that only change docs or Markdown. Every green push to `main` replaces the
[rolling beta](https://github.com/Paraxdev/GodotTrench/releases/tag/beta) with editor builds for Linux, Windows and macOS, the addon zip and the demo project zip.

To release, bump `version` in `Cargo.toml`, add `.github/release-notes/vX.Y.Z.md`, then push a `vX.Y.Z` tag. CI runs
the full suite on the tag and publishes the release with the same downloads.

With a `BUTLER_API_KEY` repository secret, both also go to [itch.io](https://paraxdev.itch.io/godottrench): releases on
the channels `linux`, `windows`, `mac`, `addon` and `demo-project`, the rolling beta on the same names with `-beta`.
