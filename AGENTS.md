# Working on GodotTrench with an AI agent

GodotTrench accepts AI assisted contributions. This file is what an agent (and the person running it) has to follow.
Whoever opens the pull request vouches for every line in it, see the disclosure in the [README](README.md).

## Layout

* `crates/gt_core` math primitives, `gt_geom` brushes, CSG, displacements and terrain geometry, `gt_doc` the map
  document and entities, `gt_formats` `.gtm` and other file formats plus the Godot API data, `gt_render` the wgpu
  renderer, `gt_editor` the egui app and its MCP server, `gt_samples` the demo Blockbench models.
* `godot/addons/func_godot` is our FuncGodot fork that builds `.gtm` maps in Godot, see its `FORK.md`.
  `godot/tests` holds the addon tests, `godot/demo` the demo project.
* `examples/mcp` holds the showcase maps as MCP tool call scripts.
* `wiki/` is the static documentation site, `docs/` the Markdown docs.

## Before you open a pull request

Everything CI runs has to pass locally. The commands are in [docs/development.md](docs/development.md):

```sh
./tools/fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p gt_editor --test e2e -- --ignored --test-threads=1
godot --headless --path godot --script res://tests/run_tests.gd
```

CI uses the Rust version in `RUST_TOOLCHAIN` in [ci.yml](.github/workflows/ci.yml). Clippy on a newer toolchain
catches things an older one misses, so match it when you can.

Add or extend tests for what you change. UI changes are checked by running the editor with `--mcp-http=PORT
--default-prefs` and driving it with `tools/mcp_client.py`, or by an e2e test in `crates/gt_editor/tests/e2e.rs`.

Run `./tools/fmt --install` once per clone. It enables a pre-commit hook that formats staged Rust with rustfmt and
`tools/space_blocks.py`. Do not hand format around it.

## Writing style

This applies to code comments, docs, the wiki, UI text, commit messages and pull request descriptions.

* No em dashes, en dashes or a spaced hyphen as punctuation. Join clauses with a comma or start a new sentence.
  Hyphens inside words, kebab-case names and CLI flags are fine.
* No emoji.
* Write plain explanatory prose. Avoid bullet dumps, marketing tone and stacked jargon.

## Code

* Read the surrounding code first and match its naming, idiom and comment density.
* Keep comments minimal. Only explain what a reader could not work out from the code, like a quirk, a workaround or a
  non-obvious constraint. Never narrate what the next line does.
* The accent colors in `crates/gt_editor/src/theme.rs` stay colorblind safe. The `cvd_pairs_stay_distinguishable`
  test enforces it, so adjust the color rather than the threshold.
* Map units are 32 per meter.

## Content

* Showcase and demo maps are built by driving the editor through its MCP tools, never by Rust code that writes a
  map. Put the calls in a script in `examples/mcp`. If a map needs something the tools cannot do, add or extend a
  tool. The `example_mcp_scripts_replay` e2e test replays every script.
* Assets like textures and models may be generated or fetched by code, see the scripts in `tools/`. Only ship
  assets whose license allows it (CC0 preferred) and list them in [LICENSES.md](LICENSES.md).
* After changing `builtin_entities.json` or `code_refs.rs`, regenerate the addon FGDs with
  `cargo run -p gt_editor --example export_addon_fgd`. The `addon_fgd_sync` test fails otherwise.

## Wiki

* Every element on a page must be one the wiki editor itself can create. If a page needs a new kind of element,
  add it to the editor first, then use it. See [wiki/README.md](wiki/README.md) for what the editor writes.
* Text renders with `breaks: true`, so a single newline in page JSON is a hard line break.
* Verify facts against the code before writing them. Explain why and when, not only what.

## Godot gotchas

* On a fresh checkout, or after adding a GDScript `class_name`, run `godot --headless --path godot --import` twice
  before the tests. One pass only registers classes and textures then fail to load.
* Put a timeout on headless Godot runs, a stuck test can hang for good.
* Check the exit code, not only the "0 failures" line. A crash on shutdown still prints a clean summary.
* Do not keep Resources in a GDScript `static var`. They get freed after the servers shut down and Godot crashes on exit.
* Headless Godot drops MultiMesh buffers when packing scenes. New MultiMesh output needs the same metadata copy that
  `GodotTrenchScatter` restores in `_ready`.

## Commits and pull requests

* Short plain subject in the imperative, the way a person would write it, for example
  "Add map overlays for Godot-side content". Add a few sentences of body only when the why is not obvious.
* Group related work into one commit instead of a chain of follow-up fixes.
* Describe the change, not the conversation that led to it. No "as requested", no prompt text, no AI attribution
  trailers.
