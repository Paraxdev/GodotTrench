# Development

This page is for you if you want to build the editor from source, change the addon or write docs.

## Building

Clone with submodules, the Godot addon in `godot/addons/func_godot` is one:

```sh
git clone --recursive https://github.com/Paraxdev/TrenchDot.git
```

Already cloned without it? Run `git submodule update --init`. The editor builds without the addon, but the tests and
the Godot project need it.

```sh
cargo run -p td_editor --release            # start the editor
cargo run -p td_editor --release -- map.tdm # open a map
```

You need a GPU with Vulkan, DX12 or Metal, and on Linux an X11 or Wayland session.

Rust 1.95 is the minimum, egui 0.36 needs it.
[rust-toolchain.toml](https://github.com/Paraxdev/TrenchDot/blob/main/rust-toolchain.toml) pins 1.98.1, the version
CI uses, so rustup installs that on your first build.

## Checks

Run `./tools/fmt --install` once per clone. It enables a pre-commit hook that formats staged Rust.

```sh
./tools/fmt --check
python tools/gen_licenses.py --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p td_editor --test e2e -- --ignored --test-threads=1
godot --headless --path godot --import                            # twice on a fresh checkout
godot --headless --path godot --script res://tests/run_tests.gd
godot --headless --path godot --script res://tests/build_showcase.gd
```

CI runs all of these, so run them before you push.

The e2e tests launch the real editor and drive it with simulated input. That needs a GPU and a desktop session, so
`cargo test` skips them unless you pass `--ignored`. CI runs them on Linux with software Vulkan instead. To test the
live link without a window, see [Live link protocol](godot/live-link.md#testing-without-a-window).

> **Note:** `godot --headless --script` renders nothing (screenshots and rendered checks come out black) and doesn't
> run project autoloads, only the script itself. Write tests against data and nodes, not pixels or autoload state.

To time map builds, run `godot --headless --path godot --script res://tests/bench_build.gd -- runs=5 threaded=1`.
Don't use `build_showcase.gd` for that, it overwrites the committed showcase scenes.

## Map files in git

`.tdm` maps are binary, see [the .tdm map format](format/README.md). The repository's `.gitattributes` already marks
them. For readable diffs, all you need is the `git config` line from
[Reviewing changes](editor/reviewing.md#maps-in-git), once per clone.

## Generated content

Some files are made by a script, not by hand. When you change what a script reads, run it again.

| Command | Regenerates |
| --- | --- |
| `cargo run -p td_formats --example demo_maps` | The test maps in `godot/tests/maps` and the small demo maps in `godot/demo/maps` |
| `python tools/fetch_demo_textures.py` | The demo's CC0 photo textures, see [texture size](godot/materials.md#texture-size) |
| `python tools/fetch_demo_props.py` | The CC0 Poly Haven props in `godot/models/polyhaven`, with the alpha maps of glass and flames restored. `--fix-alpha` only restores them |
| `cargo run -p td_samples --bin showcase --release` | The demo Blockbench models |
| `cargo run -p td_editor --example export_addon_fgd` | The addon's core FGD files and the Gameplay entities pack template from `builtin_entities.json` and `gameplay_entities.json`, written into the addon submodule, then the demo project's installed copy of the pack |
| `python tools/gen_licenses.py` | The Rust crate table in [LICENSES.md](https://github.com/Paraxdev/TrenchDot/blob/main/LICENSES.md) |

## The Godot addon

The addon has its own repository, [trenchdot_func](https://github.com/Paraxdev/trenchdot_func), checked out at
`godot/addons/func_godot`. Its tests live here in `godot/tests`, since they need the demo project. To change the addon:

1. Commit the change inside `godot/addons/func_godot` and push it to trenchdot_func.
2. Stage the new submodule commit here with `git add godot/addons/func_godot` and commit it with the TrenchDot
   changes that need it.

Push the addon first. A TrenchDot commit pointing at an addon commit GitHub doesn't have fails to check out in CI.

`plugin.cfg`'s `version` matches [Cargo.toml](https://github.com/Paraxdev/TrenchDot/blob/main/Cargo.toml)'s
workspace version. Bump both together, the editor warns when a project's addon doesn't match it.

## Documentation

The docs are a HonKit book in `docs/`. `docs/SUMMARY.md` is the table of contents, a page that isn't listed there
doesn't get built. `docs/package.json` pins HonKit and the
[darkening theme](https://github.com/Stuyk/honkit-plugin-theme-darkening), and `docs/styles/website.css` tweaks it.

```sh
cd docs
npm ci                        # once
npx honkit serve              # live preview on http://localhost:4000
npx honkit build . _site      # the build CI runs, lands in docs/_site
```

Pictures for a page go in `docs/assets/<page>/`.

If a newcomer wouldn't know a term, give it an entry in `GLOSSARY.md`. Every page then shows the definition when you
hover the term.

`.github/workflows/pages.yml` builds the book for pull requests that touch `docs/`, and publishes it to GitHub Pages
once they reach `main`.

## CI

CI skips pushes to `main` that only change docs or Markdown. Every other green push to `main` replaces the
[rolling beta](https://github.com/Paraxdev/TrenchDot/releases/tag/beta) with fresh editor, addon, demo and content
downloads.

The editor downloads the addon and content from the GitHub releases only. itch.io is a storefront that gets the same
files on its own channels, `addon`, `nature-pack`, `demo-content` and so on. The rolling beta is the only build, so it
fills those plain channels.

| Zip in the release | Installed by |
| --- | --- |
| `func_godot-trenchdot-addon.zip` | *Godot > Install or Update Addon…* |
| `trenchdot-nature-pack.zip` | *Godot > Add Content to Project…* |
| `trenchdot-demo-content.zip` | *Godot > Add Content to Project…* |

The folders each zip holds are listed in
[content.rs](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_editor/src/content.rs) and
[addon_install.rs](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_editor/src/addon_install.rs), and tests
check the zip commands in `ci.yml` against them.

The editor finds the zips through the GitHub API. CI builds it with `TRENCHDOT_CHANNEL=beta`, so it looks in the
rolling beta first. A local build tries a release named after its own version first, then falls back to the beta. To
try a zip before it's published, point the editor at another server with the `TRENCHDOT_RELEASES_API` environment
variable.

{% mcp %}

## MCP

Contributing with an AI agent? [AGENTS.md](https://github.com/Paraxdev/TrenchDot/blob/main/AGENTS.md) lists the rules
and the checks a pull request has to pass.

The e2e tests drive the editor over the [MCP server](mcp.md), and `example_mcp_scripts_replay` replays every script in
`examples/mcp`.

### MCP in the docs

The **MCP** switch in the header, off by default, hides everything about MCP so the pages stay readable for people who
never touch it. On a page, put all MCP text in one `## MCP` section at the very end, wrapped in
{% raw %}`{% mcp %}` and `{% endmcp %}`{% endraw %}. Pages only about MCP belong in the **MCP** part of `SUMMARY.md`,
which the switch hides too.

{% endmcp %}
