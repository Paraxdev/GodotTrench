# Development

This page is for people who build the editor from source, change the addon or write the docs.

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

CI runs all of these. The e2e tests launch the real
editor and drive it with simulated input. That needs a GPU and a desktop session, so `cargo test` skips them unless
you pass `--ignored`. CI runs them on Linux with software Vulkan instead of a GPU. To test the live link without a
window, see [Live link protocol](godot/live-link.md#testing-without-a-window).

> **Note:** `godot --headless --script` renders nothing (screenshots and rendered checks come out black) and does not
> run project autoloads, only the script itself. Write tests against data and nodes, not pixels or autoload state.

To time map builds, run `godot --headless --path godot --script res://tests/bench_build.gd -- runs=5 threaded=1`.
Do not time them with `build_showcase.gd`, it overwrites the committed showcase scenes.

## Map files in git

`.gtm` maps are binary, see [the .gtm map format](format/README.md). The repository's `.gitattributes` already marks
them, so for readable diffs run the `git config` line from [Reviewing changes](editor/reviewing.md#maps-in-git) once
per clone.

## Generated content

Some files in the repository are produced by a script rather than edited by hand. Run the matching command again after
changing what it reads.

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

`plugin.cfg`'s `version` matches [Cargo.toml](https://github.com/Paraxdev/GodotTrench/blob/main/Cargo.toml)'s
workspace version. Bump both together, the editor warns when a project's addon does not match it.

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

A term a newcomer would not know gets an entry in `GLOSSARY.md`. Every page then shows its definition when you hover
the term.

`.github/workflows/pages.yml` builds the book for pull requests that touch `docs/`, and publishes it to GitHub Pages
when they reach `main`.

## CI

CI skips pushes that only change docs or Markdown. Every green push to `main` replaces the
[rolling beta](https://github.com/Paraxdev/GodotTrench/releases/tag/beta) with fresh editor, addon and demo downloads.

The releases also carry `godottrench-nature-pack.zip` and `godottrench-demo-content.zip`, which *Godot > Add Content to
Project…* downloads into a project. The folders they hold are listed in
[content.rs](https://github.com/Paraxdev/GodotTrench/blob/main/crates/gt_editor/src/content.rs), and a test checks the
zip commands in `ci.yml` against that list. The editor looks the zips up through the GitHub API, and the
`GODOTTRENCH_RELEASES_API` environment variable points it at another server, for trying a zip before it is published.

{% mcp %}

## MCP

Contributing with an AI agent? [AGENTS.md](https://github.com/Paraxdev/GodotTrench/blob/main/AGENTS.md) lists the rules
and the checks a pull request has to pass.

The e2e tests drive the editor over the [MCP server](mcp.md), and `example_mcp_scripts_replay` replays every script in
`examples/mcp`.

### MCP in the docs

The **MCP** switch in the header, off by default, hides everything about MCP so the pages stay readable for people who
never touch it. On a page, put all MCP text in one `## MCP` section at the very end, wrapped in
{% raw %}`{% mcp %}` and `{% endmcp %}`{% endraw %}. Pages only about MCP belong in the **MCP** part of `SUMMARY.md`,
which the switch hides too.

{% endmcp %}
