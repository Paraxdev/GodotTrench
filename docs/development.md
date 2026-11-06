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

## Translations

The editor UI comes in English, German, French, Spanish, Portuguese, Russian, Simplified Chinese and Japanese, see
[Language](editor/interface.md#language). English is the source, and the other seven live in
[crates/td_editor/locales](https://github.com/Paraxdev/TrenchDot/tree/main/crates/td_editor/locales), one JSON file
each, embedded when the editor is built. The README there has the format, the plural forms and a glossary.

In code, the English text is the key. Wrap a literal in `en!("...")` where it is stored and in `tr(...)`,
`trf!("... {n} ...", n = count)` or `tr_n!(count, "{n} brush", "{n} brushes")` where it is drawn, and add the same key
to every file. A text with no entry shows in English. Code under `crates/td_editor/src/mcp`, log lines and map content
never go through them, MCP replies and errors stay English. A `Msg` keeps the English text beside the translation, so
the status bar can show one while MCP reads the other.

```sh
TD_L10N_TODO=/tmp/l10n cargo test -p td_editor --lib i18n
```

That run fails while a file lacks a key the code uses or still holds one it no longer does. With `TD_L10N_TODO` set to
an existing folder it also writes one file per language with the missing keys. Another test scans the source for UI text
that skips the lookup, so a raw literal handed to a label fails it. The translations are written by AI and unchecked, a
fix from a native speaker touches only that language's file. The editor started with `--default-prefs`, as the e2e tests
do, is always in English.

The manual has translations too, see [Translating the manual](#translating-the-manual). Its pages name menu items and
buttons the way the editor shows them, so after you change a value in a catalogue run
`python3 tools/translations.py check`. It lists the pages that still use the old wording.

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
zensical serve                         # live preview of the English site on http://localhost:8000/TrenchDot/
python3 tools/translations.py check    # the translated pages against their English source
python3 tools/build_site.py            # the build CI runs, English at the root of _site and a folder per language
python3 tools/test_docs.py             # tests of the three tools and of the built site
python3 tools/check_site.py            # the link check CI runs on _site
```

`build_site.py` runs `zensical build --clean --strict` once for English and once for every language, so it fails on a
link to a page or heading that doesn't exist in any of them. A mistyped `SUMMARY.md` entry, a missing image or a page
left out of the navigation get through it, `tools/check_site.py` catches those in every language. To build only some
languages, name them: `python3 tools/build_site.py de ja`.

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

`.github/workflows/pages.yml` builds the site for pull requests that touch `docs/`, `zensical.toml`, the docs tools or
the editor catalogues, and publishes it to GitHub Pages once they reach `main`.

### Translating the manual

The site is also built in German, French, Spanish, Portuguese, Russian, Simplified Chinese and Japanese, the languages
of the editor. English is the source, and the other languages are named like the editor's codes: `de`, `fr`, `es`,
`pt`, `ru`, `zh-Hans` and `ja`. The site of a language lives at `/TrenchDot/<code>/`, and the editor opens the manual
there when it runs in that language. The translations are written by AI and no native speaker has checked them, so each
translated site says that in a banner a reader can close. A fix from someone fluent is welcome.

**Where things live.** A translated page sits in `docs/translations/<code>/` under the name of the English one,
`docs/translations/de/editor/brushes.md` for `docs/editor/brushes.md`. Next to the pages, `site.toml` holds the texts
that are not pages: the theme language, the site description, the names of the color scheme button and the MCP switch,
the banner, the note for untranslated pages, the titles of the alerts and a `[nav]` table. The `[nav]` table maps each
English title in `SUMMARY.md` to its translation, a title it lacks shows in English. The list of languages is
`[[project.extra.alternate]]` in `zensical.toml`.

**The source line.** A translated page starts with front matter that names the English page it follows:

```yaml
---
source: 0123456789abcdef0123456789abcdef01234567
---
```

`source` is what `git hash-object docs/<page>` prints for the English page. While it matches, the translation is
current and the build uses it. Once the English page changes it no longer matches, the translation is outdated, and the
build shows the English page with a short note after its title instead. So editing an English page never breaks CI and
needs no translation edit, the page only turns English on the other sites until someone updates it. A missing page falls
back the same way, so every page exists in every language and the language button always lands on the same page.

**Commands.** All of them run from the repository root.

| Command | What it does |
| --- | --- |
| `python3 tools/translations.py status` | Lists outdated, missing and orphaned pages and nav titles without an entry. It never fails. Add `--lang de` for one language and `--diff` to see what changed in English since each page was translated. |
| `python3 tools/translations.py prep --lang de editor/brushes.md` | Prints what a translator needs for one page: the front matter, the heading ids, the labels with their translation, the glossary words and the English text. |
| `python3 tools/translations.py check [--lang de] [page ...]` | Fails on a current translation that breaks a rule below. Pages you name are checked whatever their state, so you can check one before it is current. |
| `python3 tools/translations.py stamp --lang de editor/brushes.md` | Sets `source` to the English page as it is now and touches nothing else. |

**Translating or updating a page.** Four steps:

1. Run `prep` for the page. For an outdated page it also shows the English diff since the old `source`.
2. Write the page in `docs/translations/<code>/`, with the front matter `prep` printed first. Keep the English
    structure line for line: same headings, lists, images, links, alerts and `td-mcp` blocks. Translate the text, the
    alt text of images and the link text, nothing else, in the voice of the language, see below.
3. Run `check` for the page and fix what it reports until it is clean.
4. Run `python3 tools/build_site.py`, `python3 tools/check_site.py` and `python3 tools/translations.py check`, then
    commit the pages of one language together.

Use `stamp` only when the English change needs nothing in the translation, like a typo, a changed link target or a
rewording that says the same. It does not look at the page, so read the English diff first. A change in what the page
says needs the translation edited, and the new `source` comes with it.

**Voice.** Every language keeps the casual voice of [getting-started.md](getting-started.md), in the form software docs
use there:

| Language | Address |
| --- | --- |
| de | du, lower case |
| fr | vous |
| es | tú, in neutral Spanish, ustedes for the plural as the catalogue README says |
| pt | você, Brazilian Portuguese |
| ru | вы, lower case |
| zh-Hans | 你 |
| ja | です/ます form, no pronoun |

**What `check` enforces.** The rules hold for every current translation, and `check` runs before the build in CI.

* Heading ids stay the English ones. Every heading, the title too, carries the id of its English heading,
    `## Größe ändern { #reshape }`, in the same order and with the same repeat suffixes (`{ #inputs_1 }`). Zensical
    drops letters outside ASCII when it makes an id, and the editor's F1 links, the links between pages, the glossary
    and the language button all rely on the English ones.
* Code stays English and byte for byte. Fenced blocks are copied with their comments, and every inline code span of the
    English page appears in the translation. Entity classnames, keys, Inputs and Outputs, MCP tool names, JSON, file
    names, key names (`Ctrl`, `F1`) and product names (TrenchDot, Godot, MCP, Live Link) are not translated either.
* Links and images keep their targets, only the link text and the alt text change. The screenshots are the English
    ones, a translated page points at the same files, so the deployed site holds one copy of each.
* Labels are the editor's. A bold span, or an italic one holding ` > `, whose parts are all keys of the catalogue
    `crates/td_editor/locales/<code>.json` has to appear in the translated page as the catalogue writes it, the parts
    joined with ` > `. Write the label as the editor shows it, uninflected, and build the sentence around it. A label
    that is not in the catalogue, like a Godot or addon button, stays in English.
* Alerts, `td-mcp` blocks and their `## MCP` heading stay where they are. Write the alert marker alone on its line,
    `> [!NOTE]`, the build puts the translated title after it. GDScript blocks keep their `gdscript` tag.
* No em dash, en dash, spaced hyphen or emoji, the rules of the English pages hold in every language.
* In Chinese and Japanese each paragraph, list item and table cell is one source line. A line break between two CJK
    characters renders as a space in the browser, so `check` flags it. Full width punctuation is fine, but the
    doubled dash characters U+2014 and U+2015 count as dashes and are not allowed.

**The glossary.** `GLOSSARY.md` is translated like any page, and each term keeps its English id,
`## Aktivator { #activator }`. A language that inflects a term lists the other forms in `data-forms`,
`{ #baked-lighting data-forms="gebackenen Beleuchtung|gebackener Beleuchtung" }`, so the hover card finds them too.
A term that is a UI name uses the catalogue word, and `check` compares them. A translated page uses a term in the
glossary's own form where it first appears, that is what links it to the card. When you add a term to the English
glossary, add it to the translated ones if you can, until then the glossary of each language is English.

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
