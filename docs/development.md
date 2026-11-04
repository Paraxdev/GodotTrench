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

> [!NOTE]
> `godot --headless --script` renders nothing (screenshots and rendered checks come out black) and doesn't
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

The docs are the Markdown pages in `docs/`, and [Zensical](https://zensical.org) turns them into the site.
`zensical.toml` in the repository root sets it up, and `docs/SUMMARY.md` is the table of contents. A page missing there
still gets built, it just has no place in the navigation. `docs/requirements.txt` pins Zensical,
`docs/stylesheets/td.css` holds the colors, `docs/javascripts/td.js` the MCP switch and the glossary cards, and
`docs/overrides` replaces a few theme templates.

Run these from the repository root:

```sh
python3 -m venv .venv                  # once
. .venv/bin/activate                   # .venv\Scripts\activate on Windows
pip install -r docs/requirements.txt   # once, and again when the pin changes
zensical serve                         # live preview on http://localhost:8000/TrenchDot/
zensical build --clean --strict        # the build CI runs, lands in _site
python3 tools/check_site.py            # the link check CI runs on _site
```

`--strict` fails the build on a link to a page or heading that doesn't exist. A mistyped `SUMMARY.md` entry, a missing
image or a page left out of the navigation get through it, `tools/check_site.py` catches those.

A heading's id is its text in lower case, spaces become hyphens and other punctuation is dropped, so `## path_corner`
is `#path_corner`. The editor's help links in
[help.rs](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_editor/src/help.rs) point at these ids, and a test
checks them, so rename a heading only together with its links.

Callouts are GitHub style alerts: `> [!NOTE]`, `> [!TIP]` or `> [!WARNING]` on a line of their own, with the text on
the quoted lines below. That way they render on GitHub too. Everything that belongs to a numbered step, a second
paragraph, an image or a code block, is indented by 4 spaces, or it falls out of the list.

Pictures for a page go in `docs/assets/<page>/`.

If a newcomer wouldn't know a term, give it an entry in `GLOSSARY.md`: a `## term` heading with a one paragraph
definition. Its first use on every page then links to it and shows the definition when you hover it. Code, links and
headings are left alone.

`.github/workflows/pages.yml` builds the site for pull requests that touch `docs/` or `zensical.toml`, and publishes it
to GitHub Pages once they reach `main`.

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

CI also stamps each beta, in `TRENCHDOT_BUILD` for the editor and in the zipped addon's `plugin.cfg`, as
`0.1.0-beta.202611041532.d194b57`: the release number, the UTC commit time and the commit. That is how the editor
tells an addon from an older beta and offers the update. A local build keeps the plain version.

<div class="td-mcp" markdown>

## MCP

Contributing with an AI agent? [AGENTS.md](https://github.com/Paraxdev/TrenchDot/blob/main/AGENTS.md) lists the rules
and the checks a pull request has to pass.

The e2e tests drive the editor over the [MCP server](mcp.md), and `example_mcp_scripts_replay` replays every script in
`examples/mcp`.

To drive an editor of your own from the shell:

```sh
trenchdot --mcp-http=7850 --default-prefs &
TRENCHDOT_MCP_URL=http://127.0.0.1:7850/mcp python3 tools/mcp_client.py call get_state
```

Set `TRENCHDOT_MCP_TOKEN` to the same value for both to try it with a [token](mcp.md#the-http-token).

### MCP in the docs

The **MCP** switch in the header, off by default, hides everything about MCP so the pages stay readable for people who
never touch it. On a page, put all MCP text in one `## MCP` section at the very end, inside a `td-mcp` block:

```html
<div class="td-mcp" markdown>

## MCP

What only MCP users need.

</div>
```

Keep the blank lines around the content, GitHub only reads the Markdown inside with them. Pages only about MCP belong
in the **MCP** part of `SUMMARY.md`, which the switch hides too.

</div>
