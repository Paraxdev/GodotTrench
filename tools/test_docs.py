#!/usr/bin/env python3
"""Tests of the docs tools, `python3 tools/test_docs.py`. The tests that look at a built site are skipped until
`python3 tools/build_site.py` has produced _site."""
import contextlib
import copy
import html as htmllib
import importlib.util
import io
import json
import pathlib
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import unittest
from unittest import mock

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

import build_site as bs  # noqa: E402
import check_site  # noqa: E402
import translations as tr  # noqa: E402

CONFIG = bs.load_config()
ZH = {"theme_language": "zh"}
DE = {
    "theme_language": "de",
    "description": "Ein Level-Editor mit Brushes",
    "light_toggle": "Zum dunklen Modus",
    "dark_toggle": "Zum hellen Modus",
    "mcp_switch": "MCP zeigen",
    "notice": "Von einer KI übersetzt",
}


def write(path, text):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(text.encode("utf-8"))
    return path


def stamped(page_text, text="Übersetzt\n"):
    return f"---\nsource: {bs.blob_id(page_text.encode())}\n---\n{text}"


class BlobIdTest(unittest.TestCase):
    def test_known_ids(self):
        self.assertEqual(bs.blob_id(b"hello\n"), "ce013625030ba8dba906f756967f9e9ca394464a")
        self.assertEqual(bs.blob_id(b""), "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391")

    def test_crlf_gives_the_id_of_the_lf_text(self):
        self.assertEqual(bs.blob_id(b"hello\r\n"), "ce013625030ba8dba906f756967f9e9ca394464a")
        self.assertEqual(bs.blob_id(b"a\r\nb\r\n"), bs.blob_id(b"a\nb\n"))
        self.assertNotEqual(bs.blob_id(b"a\r\nb\r\n"), bs.blob_id(b"a\nb"))

    def test_matches_what_git_stores(self):
        page = "docs/GLOSSARY.md"
        git = subprocess.run(["git", "rev-parse", f"HEAD:{page}"], cwd=bs.ROOT, capture_output=True, text=True)
        diff = subprocess.run(["git", "diff", "--quiet", "HEAD", "--", page], cwd=bs.ROOT)
        if git.returncode or diff.returncode:
            self.skipTest("no committed, unchanged page to compare with")
        self.assertEqual(bs.blob_id((bs.ROOT / page).read_bytes()), git.stdout.strip())


class FrontMatterTest(unittest.TestCase):
    def test_block_and_body(self):
        meta, body = bs.split_front_matter("---\nsource: abc\nother:  x: y \n---\n# T\n\ntext\n")
        self.assertEqual(meta, {"source": "abc", "other": "x: y"})
        self.assertEqual(body, "# T\n\ntext\n")

    def test_no_block_keeps_the_text(self):
        for text in ("# T\n", "", "text\n---\nsource: a\n---\n"):
            self.assertEqual(bs.split_front_matter(text), ({}, text))

    def test_block_never_closed_is_no_front_matter(self):
        text = "---\nsource: abc\n# T\n"
        self.assertEqual(bs.split_front_matter(text), ({}, text))

    def test_only_the_first_closing_line_ends_the_block(self):
        meta, body = bs.split_front_matter("---\nsource: a\n---\n---\nsource: b\n---\n")
        self.assertEqual((meta, body), ({"source": "a"}, "---\nsource: b\n---\n"))


class PageStateTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.docs = pathlib.Path(self.tmp.name) / "docs"
        self.addCleanup(self.tmp.cleanup)
        self.pages = {"a.md": "# A\n", "b.md": "# B\n", "c.md": "# C\n", "d.md": "# D\n", "e.md": "# E\n",
                      "sub/f.md": "# F\n", "SUMMARY.md": "# Summary\n\n* [A](a.md)\n"}
        for page, text in self.pages.items():
            write(self.docs / page, text)
        de = self.docs / "translations" / "de"
        write(de / "a.md", stamped(self.pages["a.md"]))
        write(de / "b.md", stamped("# B old\n"))
        write(de / "c.md", "# C ohne Kopf\n")
        write(de / "d.md", "---\nsource: " + bs.blob_id(b"# D\n") + "\n# D\n")
        write(de / "sub/f.md", stamped(self.pages["sub/f.md"]))
        write(de / "orphan.md", stamped("# Gone\n"))

    def test_states_of_german(self):
        states = {page: bs.page_state(self.docs, "de", page) for page in (*self.pages, "orphan.md")}
        self.assertEqual(states, {"a.md": "current", "b.md": "outdated", "c.md": "outdated", "d.md": "outdated",
                                  "e.md": "missing", "sub/f.md": "current", "SUMMARY.md": "missing",
                                  "orphan.md": "orphan"})

    def test_other_language_has_its_own_states(self):
        self.assertEqual({bs.page_state(self.docs, "fr", p) for p in self.pages}, {"missing"})
        self.assertEqual(bs.page_state(self.docs, "fr", "nothing.md"), "missing")

    def test_a_change_of_the_english_page_makes_it_outdated(self):
        write(self.docs / "a.md", "# A\n\nmore\n")
        self.assertEqual(bs.page_state(self.docs, "de", "a.md"), "outdated")

    def test_english_page_list_skips_summary_and_translations(self):
        self.assertEqual(bs.english_pages(self.docs), ["a.md", "b.md", "c.md", "d.md", "e.md", "sub/f.md"])

    def test_render_pages_builds_only_english_pages(self):
        strings = dict(bs.DEFAULTS, tip="Tipp")
        pages, translated = bs.render_pages(self.docs, "de", strings)
        self.assertEqual(sorted(pages), ["SUMMARY.md", "a.md", "b.md", "c.md", "d.md", "e.md", "sub/f.md"])
        self.assertEqual(translated, 2)
        self.assertEqual(pages["a.md"], "Übersetzt\n")
        self.assertEqual(pages["sub/f.md"], "Übersetzt\n")
        for page in ("b.md", "c.md", "d.md", "e.md"):
            title = self.pages[page].split("\n")[0]
            self.assertEqual(pages[page], f"{title}\n\n> [!NOTE] Note\n> {bs.DEFAULTS['fallback']}\n")

    def test_render_pages_of_a_second_language_keeps_nothing_of_the_first(self):
        write(self.docs / "translations" / "fr" / "e.md", stamped("# E\n", "Traduit\n"))
        bs.render_pages(self.docs, "de", bs.DEFAULTS)
        pages, translated = bs.render_pages(self.docs, "fr", bs.DEFAULTS)
        self.assertEqual(translated, 1)
        self.assertEqual(pages["e.md"], "Traduit\n")
        self.assertTrue(pages["a.md"].startswith("# A\n\n> [!NOTE]"))

    def test_render_pages_localises_alerts_of_current_pages_only(self):
        write(self.docs / "translations" / "de" / "a.md", stamped("# A\n", "# A\n\n> [!WARNING]\n> Achtung\n"))
        write(self.docs / "b.md", "# B\n\n> [!NOTE]\n> english\n")
        titles = dict(bs.DEFAULTS, warning="Warnung", note="Hinweis")
        pages, _ = bs.render_pages(self.docs, "de", titles)
        self.assertEqual(pages["a.md"], "# A\n\n> [!WARNING] Warnung\n> Achtung\n")
        self.assertIn("> [!NOTE]\n> english", pages["b.md"])

    def test_summary_is_translated_with_the_nav_table(self):
        pages, _ = bs.render_pages(self.docs, "de", dict(bs.DEFAULTS, nav={"A": "Ä"}))
        self.assertEqual(pages["SUMMARY.md"], "# Summary\n\n* [Ä](a.md)\n")


class NoteTest(unittest.TestCase):
    def test_note_after_the_h1(self):
        text = "# Title { #t }\n\nFirst.\n\n## Next\n"
        out = bs.with_note(text, "Hinweis", "Nur Englisch.")
        self.assertEqual(out, "# Title { #t }\n\n> [!NOTE] Hinweis\n> Nur Englisch.\n\nFirst.\n\n## Next\n")
        self.assertTrue(out.endswith("\nFirst.\n\n## Next\n"))

    def test_note_at_the_top_without_h1(self):
        text = "Intro.\n\n## Two\n"
        out = bs.with_note(text, "Hinweis", "x")
        self.assertEqual(out, "> [!NOTE] Hinweis\n> x\n\nIntro.\n\n## Two\n")
        self.assertTrue(out.endswith(text))

    def test_h1_in_a_fence_and_h2_do_not_count(self):
        text = "```sh\n# comment\n```\n\n## Two\n\n# Late\n"
        out = bs.with_note(text, "N", "x")
        self.assertTrue(out.startswith("```sh\n# comment\n```\n\n## Two\n\n# Late\n\n> [!NOTE] N\n> x\n"))

    def test_text_right_after_the_h1_gets_a_blank_line(self):
        self.assertEqual(bs.with_note("# T\ntext\n", "N", "x"), "# T\n\n> [!NOTE] N\n> x\n\ntext\n")

    def test_h1_only_page_without_final_newline(self):
        self.assertEqual(bs.with_note("# T", "N", "x"), "# T\n\n> [!NOTE] N\n> x")

    def test_empty_page(self):
        self.assertEqual(bs.with_note("", "N", "x"), "> [!NOTE] N\n> x\n\n")


class AlertTest(unittest.TestCase):
    TITLES = {"NOTE": "Hinweis", "TIP": "Tipp", "WARNING": "Warnung"}

    def test_three_kinds_outside_code(self):
        text = "> [!NOTE]\n> a\n\n> [!TIP]\n> b\n\n> [!WARNING]\n> c\n"
        self.assertEqual(bs.localise_alerts(text, self.TITLES),
                         "> [!NOTE] Hinweis\n> a\n\n> [!TIP] Tipp\n> b\n\n> [!WARNING] Warnung\n> c\n")

    def test_fenced_markers_stay(self):
        text = ("```md\n> [!NOTE]\n```\n\n    ```md\n    > [!TIP]\n    ```\n\n~~~\n> [!WARNING]\n~~~\n\n"
                "````\n```\n> [!NOTE]\n```\n````\n\n> [!NOTE]\n")
        self.assertEqual(bs.localise_alerts(text, self.TITLES), text[:-len("> [!NOTE]\n")] + "> [!NOTE] Hinweis\n")

    def test_alert_in_a_list_item_and_after_a_fence_closes(self):
        text = "1. Step\n\n    > [!TIP]\n    > Inside.\n\n    ```sh\n    x\n    ```\n\n    > [!NOTE]\n    > After.\n"
        out = bs.localise_alerts(text, self.TITLES)
        self.assertIn("    > [!TIP] Tipp\n", out)
        self.assertIn("    > [!NOTE] Hinweis\n", out)

    def test_existing_title_and_other_kinds_stay(self):
        text = "> [!NOTE] Mine\n> a\n\n> [!IMPORTANT]\n> b\n\n> not [!NOTE]\n\n[!NOTE]\n"
        self.assertEqual(bs.localise_alerts(text, self.TITLES), text)

    def test_trailing_space_and_nested_quote(self):
        self.assertEqual(bs.localise_alerts("> [!TIP]  \n>> [!NOTE]\n", self.TITLES),
                         "> [!TIP] Tipp\n>> [!NOTE] Hinweis\n")

    def test_inline_backtick_line_is_not_a_fence(self):
        text = "```inline``` and more\n> [!NOTE]\n"
        self.assertEqual(bs.localise_alerts(text, self.TITLES), "```inline``` and more\n> [!NOTE] Hinweis\n")


class NavTest(unittest.TestCase):
    SUMMARY = """# Summary

* [Introduction](README.md)
* Editor
    * [Brushes and CSG](editor/brushes.md)
    * [Brushes](editor/brushes2.md)
        * [Brushes and CSG](editor/deep.md)
* [The Godot addon](https://github.com/Paraxdev/trenchdot_func#readme)
* [Unmapped](x.md)
* Reference [odd]
"""

    def test_whole_text(self):
        nav = {"Introduction": "Einführung", "Editor": "Editor (de)", "Brushes and CSG": "Brushes und CSG",
               "The Godot addon": "Das Godot-Addon", "Summary": "Not a nav line", "Reference": "no"}
        self.assertEqual(bs.translate_nav(self.SUMMARY, nav), """# Summary

* [Einführung](README.md)
* Editor (de)
    * [Brushes und CSG](editor/brushes.md)
    * [Brushes](editor/brushes2.md)
        * [Brushes und CSG](editor/deep.md)
* [Das Godot-Addon](https://github.com/Paraxdev/trenchdot_func#readme)
* [Unmapped](x.md)
* Reference [odd]
""")

    def test_empty_table_changes_nothing(self):
        self.assertEqual(bs.translate_nav(self.SUMMARY, {}), self.SUMMARY)

    def test_titles_in_order_with_repeats(self):
        self.assertEqual(bs.nav_titles(self.SUMMARY), [
            "Introduction", "Editor", "Brushes and CSG", "Brushes", "Brushes and CSG", "The Godot addon", "Unmapped",
            "Reference [odd]"])


class FenceSpanTest(unittest.TestCase):
    def test_spans_of_adjacent_indented_and_quoted_fences(self):
        lines = ["a", "```x", "1", "```", "```y", "2", "```", "  ~~~~", "  3", "  ~~~", "  ~~~~", "> ```", "> 4", "> ```"]
        self.assertEqual(bs.fence_spans(lines), [(1, 3), (4, 6), (7, 10), (11, 13)])

    def test_unclosed_fence_runs_to_the_end_and_no_fence_gives_nothing(self):
        self.assertEqual(bs.fence_spans(["a", "```", "b", "c"]), [(1, 3)])
        self.assertEqual(bs.fence_spans(["a", "`x` and ```y```"]), [])
        self.assertEqual(bs.fence_spans([]), [])


class TomlTest(unittest.TestCase):
    def test_real_config_round_trips(self):
        self.assertEqual(tomllib.loads(bs.toml_dump(CONFIG)), CONFIG)

    def test_awkward_values_round_trip(self):
        config = {"project": {"a": "quote \" back \\ tab \t line\nnext", "b": "日本語 Русский", "c": True, "d": False,
                              "e": 3, "f": 2.5, "g": [], "h": [1, [2, "x"], {"k": "v"}], "i": {},
                              "j": {"with space": {"x.y": 1, "z-z": {"deep": [{"p": 1}, {"p": 2}]}}}},
                  "other": {"k": "v"}}
        self.assertEqual(tomllib.loads(bs.toml_dump(config)), config)

    def test_unknown_type_is_refused(self):
        with self.assertRaises(TypeError):
            bs.toml_dump({"t": {"k": None}})


class LanguageConfigTest(unittest.TestCase):
    def test_zh_hans_changes(self):
        original = copy.deepcopy(CONFIG)
        out = bs.language_config(CONFIG, "zh-Hans", ZH)
        self.assertEqual(CONFIG, original)
        project, base = out["project"], CONFIG["project"]
        self.assertEqual(project["site_url"], "https://paraxdev.github.io/TrenchDot/zh-Hans/")
        self.assertEqual((project["docs_dir"], project["site_dir"]), ("docs", "site"))
        self.assertNotIn("edit_uri", project)
        self.assertEqual(project["theme"]["language"], "zh")
        self.assertEqual(project["theme"]["custom_dir"], "docs/overrides")
        self.assertNotIn("content.action.edit", project["theme"]["features"])
        self.assertEqual(project["theme"]["features"],
                         [f for f in base["theme"]["features"] if f != "content.action.edit"])
        self.assertIn("announce.dismiss", project["theme"]["features"])
        self.assertEqual(project["extra"]["alternate"], base["extra"]["alternate"])
        self.assertEqual(project["extra"]["root_url"], "https://paraxdev.github.io/TrenchDot/")
        # Everything else is as in the English config.
        expected = copy.deepcopy(base)
        for key in ("site_url", "docs_dir", "site_dir", "edit_uri"):
            expected.pop(key, None)
            project.pop(key, None)
        for table, keys in ((expected["theme"], ("custom_dir", "language", "features")),
                            (project["theme"], ("custom_dir", "language", "features"))):
            for key in keys:
                table.pop(key)
        project["extra"].pop("root_url")
        self.assertEqual(project["extra"].pop("notice"), bs.DEFAULTS["notice"])
        self.assertEqual(project, expected)

    def test_strings_reach_the_config(self):
        out = bs.language_config(CONFIG, "de", DE)["project"]
        expected = copy.deepcopy(CONFIG["project"])
        expected["site_url"] = "https://paraxdev.github.io/TrenchDot/de/"
        expected["docs_dir"], expected["site_dir"] = "docs", "site"
        del expected["edit_uri"]
        expected["site_description"] = "Ein Level-Editor mit Brushes"
        expected["extra"].update(root_url="https://paraxdev.github.io/TrenchDot/", mcp_switch="MCP zeigen",
                                 notice="Von einer KI übersetzt")
        theme = expected["theme"]
        theme["language"] = "de"
        theme["features"].remove("content.action.edit")
        palette = {p["scheme"]: p for p in theme["palette"]}
        palette["default"]["toggle"]["name"] = "Zum dunklen Modus"
        palette["slate"]["toggle"]["name"] = "Zum hellen Modus"
        self.assertEqual(out, expected)
        self.assertEqual([p["scheme"] for p in out["theme"]["palette"]], ["default", "slate"])

    def test_missing_strings_keep_the_english_ones(self):
        out = bs.language_config(CONFIG, "de", {"theme_language": "de", "description": "Nur das"})["project"]
        base = CONFIG["project"]
        self.assertEqual(out["site_description"], "Nur das")
        self.assertEqual([p["toggle"]["name"] for p in out["theme"]["palette"]],
                         [p["toggle"]["name"] for p in base["theme"]["palette"]])
        self.assertEqual(out["extra"]["mcp_switch"], base["extra"]["mcp_switch"])
        self.assertEqual(out["extra"]["notice"], bs.DEFAULTS["notice"])

    def test_english_config_has_the_tooltip_and_no_notice(self):
        self.assertTrue(CONFIG["project"]["extra"]["mcp_switch"].startswith("Show the parts"))
        self.assertNotIn("notice", CONFIG["project"]["extra"])

    def test_default_theme_language_is_the_code(self):
        self.assertEqual(bs.language_config(CONFIG, "de", {})["project"]["theme"]["language"], "de")

    def test_a_site_url_without_slash_still_gets_the_folder(self):
        config = copy.deepcopy(CONFIG)
        config["project"]["site_url"] = "https://example.org/docs"
        self.assertEqual(bs.language_config(config, "ja", {})["project"]["site_url"], "https://example.org/docs/ja/")

    def test_languages_in_the_real_config(self):
        self.assertEqual(bs.languages(CONFIG), [
            ("de", "Deutsch", "de"), ("fr", "Français", "fr"), ("es", "Español", "es"), ("pt", "Português", "pt-BR"),
            ("ru", "Русский", "ru"), ("zh-Hans", "简体中文", "zh-Hans"), ("ja", "日本語", "ja")])

    def test_every_language_has_site_strings(self):
        for code, _, _ in bs.languages(CONFIG):
            strings = bs.load_strings(bs.ROOT / "docs", code)
            self.assertTrue(strings["theme_language"], code)
            for key in ("description", "light_toggle", "dark_toggle", "mcp_switch", "notice", "fallback", "note", "tip",
                        "warning"):
                self.assertIn(key, tomllib.loads((bs.ROOT / "docs" / "translations" / code / "site.toml")
                                                 .read_text(encoding="utf-8")), f"{code} {key}")

    def test_site_strings_are_translated_and_follow_the_style(self):
        english = {**bs.DEFAULTS, "description": CONFIG["project"]["site_description"],
                   "mcp_switch": CONFIG["project"]["extra"]["mcp_switch"],
                   "light_toggle": CONFIG["project"]["theme"]["palette"][0]["toggle"]["name"],
                   "dark_toggle": CONFIG["project"]["theme"]["palette"][1]["toggle"]["name"]}
        titles = set()
        for code, _, _ in bs.languages(CONFIG):
            strings = bs.load_strings(bs.ROOT / "docs", code)
            for key, value in english.items():
                self.assertNotEqual(strings[key], value, f"{code} {key} is still English")
            for key, value in strings.items():
                if isinstance(value, str):
                    self.assertNotRegex(value, r"[\u2013\u2014\u2015]| - ", f"{code} {key}")
            self.assertEqual(len({strings["note"], strings["tip"], strings["warning"]}), 3, code)
            titles.add(strings["warning"])
        self.assertGreater(len(titles), 4)

    def test_strings_fall_back_to_the_defaults(self):
        with tempfile.TemporaryDirectory() as tmp:
            docs = pathlib.Path(tmp)
            write(docs / "translations" / "de" / "site.toml", 'theme_language = "de"\nnote = "Hinweis"\n')
            strings = bs.load_strings(docs, "de")
            self.assertEqual((strings["note"], strings["tip"], strings["theme_language"]), ("Hinweis", "Tip", "de"))
            self.assertEqual(bs.load_strings(docs, "fr"), bs.DEFAULTS)


class ImageTest(unittest.TestCase):
    FOLDERS = ["assets/brushes", "screenshots"]

    def test_image_folders(self):
        with tempfile.TemporaryDirectory() as tmp:
            docs = pathlib.Path(tmp)
            for path in ("assets/logo.png", "assets/brushes/a.png", "assets/brushes/deep/b.jpg",
                         "assets/other/c.GIF", "screenshots/s.webp", "assets/notes/readme.txt",
                         "javascripts/x.png", "overrides/o.png", "translations/de/assets/i.png",
                         "top.png", "sub/page/d.svg"):
                write(docs / path, "x")
            (docs / "assets" / "empty").mkdir()
            self.assertEqual(bs.image_folders(docs), ["assets/brushes", "assets/other", "screenshots", "sub/page"])

    def test_real_docs_folders(self):
        folders = bs.image_folders(bs.ROOT / "docs")
        self.assertIn("screenshots", folders)
        self.assertIn("assets/brushes", folders)
        self.assertNotIn("assets", folders)
        self.assertNotIn("assets/images", folders)

    def test_point_up(self):
        up = lambda url, page_dir=("editor",), base="/TrenchDot/de/": bs.point_up(url, page_dir, self.FOLDERS, base)
        self.assertEqual(up("../assets/brushes/a b.png"), "../../assets/brushes/a b.png")
        self.assertEqual(up("../assets/brushes/a%20b.png?x=1#f"), "../../assets/brushes/a%20b.png?x=1#f")
        self.assertEqual(up("assets/brushes/a.png", ()), "../assets/brushes/a.png")
        self.assertEqual(up("./screenshots/s.gif", ()), ".././screenshots/s.gif")
        self.assertEqual(up("../../screenshots/s.gif", ("gameplay", "entities")), "../../../screenshots/s.gif")
        self.assertEqual(up("../x/../assets/brushes/a.png"), "../../x/../assets/brushes/a.png")
        self.assertEqual(up("../assets/logo.png"), "../assets/logo.png")
        self.assertEqual(up("../assets/brushes2/a.png"), "../assets/brushes2/a.png")
        self.assertEqual(up("../assets/brushes"), "../assets/brushes")
        self.assertEqual(up("assets/brushes/a.png"), "assets/brushes/a.png")
        self.assertEqual(up("../../../assets/brushes/a.png"), "../../../assets/brushes/a.png")
        self.assertEqual(up("../assets/stylesheets/x.css"), "../assets/stylesheets/x.css")
        for url in ("https://example.org/assets/brushes/a.png", "//cdn.example.org/screenshots/s.gif",
                    "mailto:a@b.c", "#assets/brushes/a.png", "", "data:image/png;base64,AAAA", "index.html"):
            self.assertEqual(up(url), url)

    def test_point_up_absolute_urls(self):
        up = lambda url: bs.point_up(url, ("editor",), self.FOLDERS, "/TrenchDot/de/")
        self.assertEqual(up("/TrenchDot/de/assets/brushes/a.png"), "/TrenchDot/assets/brushes/a.png")
        self.assertEqual(up("/TrenchDot/de/screenshots/s.gif#x"), "/TrenchDot/screenshots/s.gif#x")
        self.assertEqual(up("/TrenchDot/de/assets/logo.png"), "/TrenchDot/de/assets/logo.png")
        self.assertEqual(up("/TrenchDot/assets/brushes/a.png"), "/TrenchDot/assets/brushes/a.png")
        self.assertEqual(up("/other/de/assets/brushes/a.png"), "/other/de/assets/brushes/a.png")

    def test_rewrite_images(self):
        with tempfile.TemporaryDirectory() as tmp:
            site = pathlib.Path(tmp)
            for path in ("assets/brushes/a.png", "assets/brushes/deep/b.png", "screenshots/s.gif", "assets/logo.png",
                         "assets/stylesheets/m.css"):
                write(site / path, "x")
            page = write(site / "editor" / "x.html", "\n".join([
                '<link rel="icon" href="../assets/brushes/a.png">',
                '<img alt="a > b" src="../assets/brushes/a.png" />',
                "<IMG SRC='../assets/brushes/deep/b.png'>",
                '<img src="../assets/logo.png">',
                '<picture><source srcset="../assets/brushes/a.png 1x, ../assets/brushes/b.png 2x, ../assets/logo.png 3x">'
                "</picture>",
                '<video poster="../screenshots/s.gif"></video>',
                '<a href="../screenshots/s.gif">gif</a> <a href="x.html">x</a> <a href="#top">top</a>',
                '<pre><code>&lt;img src="../assets/brushes/a.png"&gt;</code></pre>',
                '<p>&lt;a href="../screenshots/s.gif"&gt;</p>',
                '<span>&lt;</span><span class="nt">img</span> src="../assets/brushes/a.png"',
                "",
            ]))
            top = write(site / "index.html", '<img src="screenshots/s.gif"><a href="assets/brushes/a.png">')
            changed = bs.rewrite_images(site, self.FOLDERS, "/TrenchDot/de/")
            self.assertEqual(changed, 7)
            self.assertEqual(page.read_text(encoding="utf-8"), "\n".join([
                '<link rel="icon" href="../assets/brushes/a.png">',
                '<img alt="a > b" src="../../assets/brushes/a.png" />',
                "<IMG SRC='../../assets/brushes/deep/b.png'>",
                '<img src="../assets/logo.png">',
                '<picture><source srcset="../../assets/brushes/a.png 1x, ../../assets/brushes/b.png 2x, '
                '../assets/logo.png 3x"></picture>',
                '<video poster="../../screenshots/s.gif"></video>',
                '<a href="../../screenshots/s.gif">gif</a> <a href="x.html">x</a> <a href="#top">top</a>',
                '<pre><code>&lt;img src="../assets/brushes/a.png"&gt;</code></pre>',
                '<p>&lt;a href="../screenshots/s.gif"&gt;</p>',
                '<span>&lt;</span><span class="nt">img</span> src="../assets/brushes/a.png"',
                "",
            ]))
            self.assertEqual(top.read_text(encoding="utf-8"),
                             '<img src="../screenshots/s.gif"><a href="../assets/brushes/a.png">')
            self.assertEqual(sorted(p.relative_to(site).as_posix() for p in site.rglob("*") if p.is_file()),
                             ["assets/logo.png", "assets/stylesheets/m.css", "editor/x.html", "index.html"])

    def test_untouched_page_is_not_rewritten(self):
        with tempfile.TemporaryDirectory() as tmp:
            site = pathlib.Path(tmp)
            page = write(site / "a.html", '<a href="b.html">b</a>\r\n')
            self.assertEqual(bs.rewrite_images(site, self.FOLDERS, "/TrenchDot/de/"), 0)
            self.assertEqual(page.read_bytes(), b'<a href="b.html">b</a>\r\n')


def nav_page(nav=(), body=""):
    links = "".join(f'<a class="md-nav__link" href="{href}">x</a>' for href in nav)
    return f"<html><body><nav>{links}</nav>{body}</body></html>"


def clean_site(site, languages=("de",)):
    for root in ("", *languages):
        prefix = f"/TrenchDot/{root + '/' if root else ''}"
        alternates = "".join(f'<a href="/TrenchDot/{r + "/" if r else ""}a.html" hreflang="{r}">l</a>'
                             for r in ("", *languages))
        write(site / root / "index.html", nav_page(["index.html", "a.html"], f'<h1 id="top">T</h1>'))
        write(site / root / "a.html", nav_page(["index.html", "a.html"],
                                               f'<h2 id="sec">S</h2><a href="#sec">s</a><a href="index.html">i</a>{alternates}'))
        write(site / root / "404.html", nav_page(body=f'<a href="{prefix}">home</a>'))
    write(site / "assets" / "brushes" / "a.png", "x")


class CheckSiteTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.site = pathlib.Path(self.tmp.name)
        self.addCleanup(self.tmp.cleanup)
        clean_site(self.site)

    def check(self, roots=("", "de")):
        return check_site.check(self.site, "/TrenchDot/", list(roots))

    def test_clean_tree(self):
        self.assertEqual(self.check(), [])

    def test_page_missing_from_the_german_nav(self):
        write(self.site / "de" / "index.html", nav_page(["index.html"]))
        self.assertEqual(self.check(), ["de/a.html is not in the navigation, add its page to docs/SUMMARY.md"])

    def test_english_page_only_in_the_german_nav(self):
        write(self.site / "extra.html", nav_page(["index.html", "a.html"]))
        write(self.site / "de" / "index.html", nav_page(["index.html", "a.html", "../extra.html"]))
        self.assertEqual(self.check(), ["extra.html is not in the navigation, add its page to docs/SUMMARY.md"])

    def test_german_page_outside_the_english_nav_is_not_counted_there(self):
        write(self.site / "de" / "b.html", nav_page())
        write(self.site / "de" / "index.html", nav_page(["index.html", "a.html", "b.html"]))
        self.assertEqual(self.check(), [])

    def test_language_link_to_a_missing_page(self):
        write(self.site / "a.html", nav_page(["index.html", "a.html"], '<a href="/TrenchDot/de/gone.html">de</a>'
                                                                       '<a href="/TrenchDot/de/gone.html">de</a>'))
        write(self.site / "index.html", nav_page(["index.html", "a.html"], '<a href="/TrenchDot/de/gone.html">de</a>'))
        self.assertEqual(self.check(), ["de/gone.html does not exist, linked from a.html and 1 other page"])

    def test_language_link_to_a_missing_heading(self):
        write(self.site / "de" / "a.html", nav_page(["index.html", "a.html"], '<a href="/TrenchDot/a.html#nope">en</a>'))
        self.assertEqual(self.check(), ["a.html#nope does not exist, linked from de/a.html"])

    def test_absolute_link_outside_the_base(self):
        write(self.site / "de" / "a.html", nav_page(["index.html", "a.html"], '<a href="/elsewhere/a.html">x</a>'))
        problems = self.check()
        self.assertEqual(len(problems), 1)
        self.assertTrue(problems[0].startswith("/elsewhere/a.html is outside /TrenchDot/, linked from de/a.html"))

    def test_missing_image_in_a_language(self):
        write(self.site / "de" / "a.html", nav_page(["index.html", "a.html"], '<img src="../assets/brushes/gone.png">'))
        self.assertEqual(self.check(), ["assets/brushes/gone.png does not exist, linked from de/a.html"])

    def test_missing_language_root_names_the_build_script(self):
        problems = self.check(("", "de", "ja"))
        self.assertEqual(problems, ["ja/index.html does not exist, build the languages with tools/build_site.py"])

    def test_pages_of_a_language_without_root_are_not_taken_for_english(self):
        write(self.site / "ja" / "a.html", nav_page())
        self.assertEqual(self.check(), ["ja/a.html is not in the navigation, add its page to docs/SUMMARY.md"])

    def test_404_is_not_in_the_nav_of_any_root(self):
        self.assertTrue((self.site / "de" / "404.html").is_file())
        self.assertEqual(self.check(), [])

    def test_rewritten_images_of_a_translated_page_resolve_to_the_root_copy(self):
        write(self.site / "de" / "editor" / "p.html", nav_page(["../index.html", "../a.html", "p.html"],
                                                               '<img src="../assets/brushes/a.png">'))
        write(self.site / "de" / "index.html", nav_page(["index.html", "a.html", "editor/p.html"]))
        write(self.site / "de" / "assets" / "brushes" / "a.png", "x")
        bs.rewrite_images(self.site / "de", ["assets/brushes"], "/TrenchDot/de/")
        self.assertIn('src="../../assets/brushes/a.png"', (self.site / "de" / "editor" / "p.html").read_text())
        self.assertEqual(self.check(), [])
        (self.site / "assets" / "brushes" / "a.png").unlink()
        self.assertEqual(self.check(), ["assets/brushes/a.png does not exist, linked from de/editor/p.html"])


class CompressedSizeTest(unittest.TestCase):
    def test_counts_what_a_tar_gz_would_hold(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder = pathlib.Path(tmp) / "site"
            write(folder / "a.html", "<p>" + "text " * 5000 + "</p>")
            write(folder / "sub" / "b.txt", "other")
            archive = io.BytesIO()
            with tarfile.open(fileobj=archive, mode="w:gz") as tar:
                tar.add(folder, arcname="site")
            self.assertAlmostEqual(bs.compressed_size(folder), len(archive.getvalue()), delta=16)
            self.assertLess(bs.compressed_size(folder), 5000)


@unittest.skipUnless((bs.ROOT / "_site" / "de" / "index.html").is_file(), "run tools/build_site.py first")
class BuiltSiteTest(unittest.TestCase):
    SITE = bs.ROOT / "_site"

    def test_every_language_is_built_and_passes_the_check(self):
        roots = [""] + [code for code, _, _ in bs.languages(CONFIG)]
        for root in roots[1:]:
            self.assertTrue((self.SITE / root / "editor" / "brushes.html").is_file(), root)
        self.assertEqual(check_site.check(self.SITE, bs.base_path(CONFIG), roots), [])

    def test_language_sites_keep_no_image_folders(self):
        folders = bs.image_folders(bs.ROOT / "docs")
        for code, _, _ in bs.languages(CONFIG):
            for folder in folders:
                self.assertFalse((self.SITE / code / folder).exists(), f"{code}/{folder}")
            self.assertTrue((self.SITE / code / "assets" / "logo.png").is_file(), code)
        for folder in folders:
            self.assertTrue((self.SITE / folder).is_dir(), folder)

    def test_translated_pages_point_at_the_root_images(self):
        html = (self.SITE / "de" / "editor" / "brushes.html").read_text(encoding="utf-8")
        self.assertIn('src="../../assets/brushes/draw-box-3d.png"', html)
        english = (self.SITE / "editor" / "brushes.html").read_text(encoding="utf-8")
        self.assertIn('src="../assets/brushes/draw-box-3d.png"', english)
        index = (self.SITE / "ja" / "index.html").read_text(encoding="utf-8")
        self.assertIn('src="../screenshots/editor.png"', index)

    def test_german_page_has_the_banner_and_the_german_strings(self):
        strings = bs.load_strings(bs.ROOT / "docs", "de")
        html = htmllib.unescape((self.SITE / "de" / "editor" / "brushes.html").read_text(encoding="utf-8"))
        self.assertIn('class="md-banner"', html)
        self.assertIn(strings["notice"], html)
        self.assertIn(f'<meta name="td-mcp-switch" content="{strings["mcp_switch"]}">', html)
        self.assertIn(f'<meta name="description" content="{strings["description"]}">', html)
        self.assertIn(f'title="{strings["light_toggle"]}"', html)
        self.assertIn(f'title="{strings["dark_toggle"]}"', html)
        self.assertIn(f'<p class="admonition-title">{strings["note"]}</p>', html)

    def test_english_page_has_no_banner_and_the_english_tooltip(self):
        html = (self.SITE / "editor" / "brushes.html").read_text(encoding="utf-8")
        self.assertNotIn("md-banner", html)
        self.assertIn(f'<meta name="td-mcp-switch" content="{CONFIG["project"]["extra"]["mcp_switch"]}">', html)

    def test_every_language_page_has_its_own_notice(self):
        for code, _, _ in bs.languages(CONFIG):
            strings = bs.load_strings(bs.ROOT / "docs", code)
            html = htmllib.unescape((self.SITE / code / "index.html").read_text(encoding="utf-8"))
            self.assertEqual(html.count('class="md-banner"'), 1, code)
            self.assertIn(strings["notice"], html, code)

    def test_heading_ids_stay_english(self):
        for code, _, _ in bs.languages(CONFIG):
            self.assertIn('id="reshape"', (self.SITE / code / "editor" / "brushes.html").read_text(encoding="utf-8"),
                          code)

    def test_language_button_keeps_the_page(self):
        html = (self.SITE / "ru" / "editor" / "brushes.html").read_text(encoding="utf-8")
        for code, _, hreflang in [("", "", "en")] + bs.languages(CONFIG):
            self.assertIn(f'href="/TrenchDot/{code + "/" if code else ""}editor/brushes.html" hreflang="{hreflang}"',
                          html)
        missing = (self.SITE / "de" / "404.html").read_text(encoding="utf-8")
        self.assertIn('href="/TrenchDot/de/" hreflang="de"', missing)
        self.assertIn('href="/TrenchDot/ja/" hreflang="ja"', missing)

    def test_social_image_is_the_root_one(self):
        html = (self.SITE / "zh-Hans" / "index.html").read_text(encoding="utf-8")
        self.assertIn('og:image" content="https://paraxdev.github.io/TrenchDot/assets/brand/meta-og-1200x630.png"', html)
        self.assertTrue((self.SITE / "assets" / "brand" / "meta-og-1200x630.png").is_file())

    def test_language_sites_stay_small(self):
        for code, _, _ in bs.languages(CONFIG):
            self.assertLess(bs.compressed_size(self.SITE / code), bs.LANGUAGE_SIZE_LIMIT, code)



@unittest.skipUnless(importlib.util.find_spec("zensical"), "needs zensical, pip install -r docs/requirements.txt")
class FallbackBuildTest(unittest.TestCase):
    """Builds a small German site, so the fallback page is checked whatever the real translations have reached."""

    BRUSHES = "# Brushes\n\nDraw one.\n\n## Reshape { #reshape }\n\nDrag a face.\n"

    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = pathlib.Path(tmp.name)
        self.docs = self.root / "docs"
        real = bs.ROOT / "docs"
        for folder in ("overrides", "javascripts", "stylesheets"):
            shutil.copytree(real / folder, self.docs / folder)
        de = self.docs / "translations" / "de"
        for file in ("assets/logo.png", "translations/de/site.toml"):
            (self.docs / file).parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(real / file, self.docs / file)
        toml = de / "site.toml"
        toml.write_bytes(re.sub(r'(?m)^mcp_switch = .*$', r'mcp_switch = "MCP \\"zeigen\\""',
                                toml.read_text(encoding="utf-8")).encode("utf-8"))
        write(self.docs / "SUMMARY.md", "* [Introduction](README.md)\n* Editor\n    * [Brushes](editor/brushes.md)\n"
              "    * [Faces](editor/faces.md)\n")
        write(self.docs / "README.md", "# Intro\n\nHello.\n")
        write(self.docs / "editor" / "brushes.md", self.BRUSHES + "\nA sentence added later.\n")
        write(self.docs / "editor" / "faces.md", "# Faces\n\n## Pick { #pick }\n")
        write(de / "README.md", stamped("# Intro\n\nHello.\n", "# Einführung\n\nHallo.\n"))
        write(de / "editor" / "brushes.md", stamped(self.BRUSHES, "# Brushes\n\n## Umformen { #reshape }\n"))
        self.strings = bs.load_strings(self.docs, "de")
        with mock.patch.object(bs, "ROOT", self.root), contextlib.redirect_stdout(io.StringIO()):
            bs.build_language(CONFIG, self.docs, "de", self.root / "_site")
        self.site = self.root / "_site" / "de"

    def page(self, name):
        return htmllib.unescape((self.site / name).read_text(encoding="utf-8"))

    def test_outdated_and_missing_pages_show_the_english_text_with_the_note(self):
        for name, english, heading in [("editor/brushes.html", "A sentence added later.", 'id="reshape"'),
                                       ("editor/faces.html", "Pick", 'id="pick"')]:
            html = self.page(name)
            self.assertIn(self.strings["fallback"], html, name)
            self.assertIn(f'<p class="admonition-title">{self.strings["note"]}</p>', html, name)
            self.assertIn(english, html, name)
            self.assertIn(heading, html, name)
            self.assertNotIn("Umformen", html, name)

    def test_a_language_site_over_the_limit_stops_the_build(self):
        def build(size):
            with mock.patch.object(bs, "ROOT", self.root), mock.patch.object(bs, "compressed_size", return_value=size),                     contextlib.redirect_stdout(io.StringIO()) as out:
                bs.build_language(CONFIG, self.docs, "de", self.root / "_site")
            return out.getvalue()

        self.assertIn("de: 1 of 3 pages translated, 6.0 MB compressed", build(bs.LANGUAGE_SIZE_LIMIT))
        with self.assertRaises(SystemExit) as stop:
            build(bs.LANGUAGE_SIZE_LIMIT + 1)
        self.assertIn("The de site is over 6 MB compressed", str(stop.exception))

    def test_a_quote_in_the_mcp_switch_text_stays_inside_the_meta(self):
        html = (self.site / "index.html").read_text(encoding="utf-8")
        meta = re.search(r'<meta name="td-mcp-switch" content="([^"]*)">', html)
        self.assertEqual(htmllib.unescape(meta[1]), 'MCP "zeigen"')

    def test_current_page_is_the_translation_without_the_note(self):
        html = self.page("index.html")
        self.assertIn("Hallo.", html)
        self.assertNotIn(self.strings["fallback"], html)
        self.assertNotIn("Hello.", html)


@unittest.skipUnless(shutil.which("node"), "needs node")
class GlossaryPatternTest(unittest.TestCase):
    """Runs the term pattern of docs/javascripts/td.js, the hover cards link what it matches."""

    def matches(self, names, texts):
        source = (bs.ROOT / "docs" / "javascripts" / "td.js").read_text(encoding="utf-8")
        start = source.index("  var WORD =")
        end = source.index("\n  }\n", source.index("function termPattern")) + 4
        script = source[start:end] + """
var names = %s.sort(function (a, b) { return b.length - a.length; });
var re = new RegExp(names.map(termPattern).join("|"), "giu");
console.log(JSON.stringify(%s.map(function (t) { return t.match(re) || []; })));
""" % (json.dumps(names), json.dumps(texts))
        run = subprocess.run(["node", "-e", script], capture_output=True, encoding="utf-8", check=True)
        return json.loads(run.stdout)

    def test_latin_terms_against_kana_and_kanji(self):
        texts = ["MCPサーバーを起動", "このUVを固定", "用MCP", "ブラシのUV lockを", "FGDファイルとUV"]
        self.assertEqual(self.matches(["MCP", "UV", "UV lock", "FGD", "ブラシ"], texts),
                         [["MCP"], ["UV"], ["MCP"], ["ブラシ", "UV lock"], ["FGD", "UV"]])

    def test_latin_terms_stay_whole_words_next_to_latin_letters_and_digits(self):
        texts = ["The MCP server", "MCPs and EMCP", "FGD2 or _UV", "UV lock, UV", "nothing here"]
        self.assertEqual(self.matches(["MCP", "UV", "UV lock", "FGD"], texts),
                         [["MCP"], [], [], ["UV lock", "UV"], []])

    def test_cjk_terms_match_inside_cjk_text_and_against_latin(self):
        self.assertEqual(self.matches(["ブラシ", "笔刷"], ["大ブラシ箱", "Xブラシ", "这个笔刷很好"]),
                         [["ブラシ"], ["ブラシ"], ["笔刷"]])


ENGLISH_PAGE = """# Maps and Brushes

Intro with `Ctrl+S` and **Inspector**, then *File > Preferences…* in italics, *Just italic* and **Not a label**.

## Draw it { #draw }

1. Click **Open**, see the [glossary](GLOSSARY.md#brush) and the [site](https://example.com/a(b)).

    ![Alt one](../assets/x/one.png)

    ```json
    {"a": 1}
    ```

2. Then ![inline](../assets/x/two.png) stays inline.

> [!NOTE]
> A note about `set_editor`.

<div class="td-mcp" markdown>

## MCP

Call `foo_bar` with `{"x": 1}`.

```json
{"tool": "x"}
```

</div>

> [!TIP]
> ```text
> > [!NOTE]
> ```

## Draw it

Last words.
"""

GERMAN_PAGE = """# Karten und Brushes { #maps-and-brushes }

Einleitung mit `Ctrl+S` und **Inspektor**, dann *Datei > Einstellungen…* kursiv, *Nur kursiv* und **Kein Label**.

## Zeichne es { #draw }

1. Klicke **Öffnen**, sieh dir das [Glossar](GLOSSARY.md#brush) und die [Seite](https://example.com/a(b)) an.

    ![Bild eins](../assets/x/one.png)

    ```json
    {"a": 1}
    ```

2. Dann bleibt ![Bild](../assets/x/two.png) im Text.

> [!NOTE]
> Eine Notiz über `set_editor`.

<div class="td-mcp" markdown>

## MCP { #mcp }

Rufe `foo_bar` mit `{"x": 1}` auf.

```json
{"tool": "x"}
```

</div>

> [!TIP]
> ```text
> > [!NOTE]
> ```

## Zeichne es noch mal { #draw-it }

Letzte Worte.
"""

CATALOGUE_JSON = """{
  "File": "Datei",
  "Preferences…": "Einstellungen…",
  "Open#verb": "Öffnen",
  "Open#adjective": "Offen",
  "Inspector": "Inspektor",
  "Brush": "Brush",
  "{n} brush": { "one": "{n} Brush", "other": "{n} Brushes" }
}"""
BREAK = "join clauses with a comma or a full stop"


def line_of(text, needle):
    return next(i for i, line in enumerate(text.split("\n"), 1) if needle in line)


class CatalogueTest(unittest.TestCase):
    def test_senses_merge_and_plurals_are_skipped(self):
        with tempfile.TemporaryDirectory() as tmp:
            write(pathlib.Path(tmp) / "de.json", CATALOGUE_JSON)
            self.assertEqual(tr.load_catalogue(tmp, "de"), {
                "File": ["Datei"], "Preferences…": ["Einstellungen…"], "Open": ["Öffnen", "Offen"],
                "Inspector": ["Inspektor"], "Brush": ["Brush"]})

    def test_second_language_shares_nothing_with_the_first(self):
        with tempfile.TemporaryDirectory() as tmp:
            write(pathlib.Path(tmp) / "de.json", CATALOGUE_JSON)
            write(pathlib.Path(tmp) / "fr.json", '{"File": "Fichier"}')
            tr.load_catalogue(tmp, "de")
            self.assertEqual(tr.load_catalogue(tmp, "fr"), {"File": ["Fichier"]})

    def test_a_hash_that_is_part_of_the_text_stays(self):
        with tempfile.TemporaryDirectory() as tmp:
            write(pathlib.Path(tmp) / "de.json", '{"C#": "C#", "Open#verb": "Öffnen", "A#B": "x"}')
            self.assertEqual(tr.load_catalogue(tmp, "de"), {"C#": ["C#"], "Open": ["Öffnen"], "A#B": ["x"]})

    def test_real_catalogues_load(self):
        for code in ("de", "ja", "zh-Hans"):
            self.assertIn("Inspector", tr.load_catalogue(tr.LOCALES, code))


class HeadingTest(unittest.TestCase):
    def ids(self, text):
        return [h[3] for h in tr.headings(text)]

    def test_explicit_slug_and_repeats(self):
        text = "# Title { #brushes }\n\n## Reshape { #re }\n\n## A\n\n## A\n\n## a_1 { #x }\n\n## Inputs\n\n## Inputs\n"
        self.assertEqual(self.ids(text), ["brushes", "re", "a", "a_1", "x", "inputs", "inputs_1"])

    def test_explicit_ids_are_reserved_before_the_slugs(self):
        self.assertEqual(self.ids("## Inputs\n\n## Other { #inputs }\n"), ["inputs_1", "inputs"])

    def test_markup_is_not_part_of_the_slug(self):
        self.assertEqual(self.ids("## `Code` *x* [link](u.md) **b**\n\n## Größe ändern\n\n## 日本語\n"),
                         ["code-x-link-b", "groe-andern", "_1"])

    def test_code_and_attribute_values_are_not_headings(self):
        text = '```text\n# not a heading\n```\n\n## Term { #term data-forms="a #b|c" }\n\n    # indented\n'
        found = tr.headings(text)
        self.assertEqual([(h[1], h[2], h[3], h[4]) for h in found], [(2, "Term", "term", ["a #b", "c"])])

    def test_ids_of_the_real_pages_match_python_markdown(self):
        try:
            import markdown
        except ImportError:
            self.skipTest("markdown is not installed")
        for page in ("editor/brushes.md", "gameplay/io.md", "GLOSSARY.md", "getting-started.md"):
            text = bs.read_page(bs.ROOT / "docs" / page)
            html = markdown.Markdown(extensions=["attr_list", "toc", "md_in_html"]).convert(text)
            self.assertEqual(self.ids(text), re.findall(r'<h[1-6][^>]*id="([^"]+)"', html), page)


class CheckPageTest(unittest.TestCase):
    CATALOGUE = {"File": ["Datei"], "Preferences…": ["Einstellungen…"], "Open": ["Öffnen", "Offen"],
                 "Inspector": ["Inspektor"], "Brush": ["Brush"]}

    def problems(self, translated, code="de", english=ENGLISH_PAGE, glossary=False, catalogue=None):
        return tr.check_page(english, translated, code, catalogue or self.CATALOGUE, glossary)

    def test_a_faithful_translation_passes(self):
        self.assertEqual(self.problems(GERMAN_PAGE), [])

    def test_the_english_page_passes_with_itself(self):
        self.assertEqual(self.problems(ENGLISH_PAGE, catalogue={k: [k] for k in self.CATALOGUE}), [])

    def test_the_front_matter_lines_move_the_line_numbers(self):
        text = GERMAN_PAGE.replace("Letzte Worte.", "Letzte \u2014 Worte.")
        found = tr.check_page(ENGLISH_PAGE, text, "de", self.CATALOGUE, offset=3)
        self.assertEqual(found, [f"line {line_of(text, 'Letzte') + 3}: em dash U+2014, {BREAK}"])

    def test_heading_without_the_english_id(self):
        text = GERMAN_PAGE.replace("## Zeichne es { #draw }", "## Zeichne es")
        self.assertEqual(self.problems(text), ["heading ids: number 2 is 'zeichne-es', the English page has 'draw'"])

    def test_umlaut_heading_without_id_fails_and_ascii_slug_equal_to_the_english_id_passes(self):
        self.assertEqual(self.problems(GERMAN_PAGE.replace("## MCP { #mcp }", "## Größe")),
                         ["heading ids: number 3 is 'groe', the English page has 'mcp'"])
        self.assertEqual(self.problems(GERMAN_PAGE.replace("## MCP { #mcp }", "## MCP")), [])

    def test_headings_in_another_order_or_missing(self):
        swapped = GERMAN_PAGE.replace("{ #draw }", "{ #xx }").replace("{ #mcp }", "{ #draw }") \
            .replace("{ #xx }", "{ #mcp }")
        self.assertEqual(self.problems(swapped), ["heading ids: number 2 is 'mcp', the English page has 'draw'"])
        self.assertEqual(self.problems(GERMAN_PAGE.replace("## Zeichne es noch mal { #draw-it }\n", "")),
                         ["heading ids: 3 found, the English page has 4"])

    def test_code_block_changed_indentation_missing_or_extra(self):
        message = "code block 1 differs from the English one, code stays byte for byte"
        self.assertEqual(self.problems(GERMAN_PAGE.replace('{"a": 1}', '{"a": 2}')), [message])
        flat = GERMAN_PAGE.replace('    ```json\n    {"a": 1}\n    ```', '```json\n{"a": 1}\n```')
        self.assertEqual(self.problems(flat), [message])
        self.assertEqual(self.problems(GERMAN_PAGE + "\n```\nx\n```\n"), ["4 code blocks, the English page has 3"])
        self.assertEqual(self.problems(GERMAN_PAGE.replace('{"a": 1}', '{"a": 1}\n    // c')), [message])

    def test_alert_marker_inside_a_fence_is_code(self):
        text = GERMAN_PAGE.replace("> > [!NOTE]", "> > [!TIP]")
        self.assertEqual(self.problems(text), ["code block 3 differs from the English one, code stays byte for byte"])

    def test_inline_code_missing_or_reworded(self):
        self.assertEqual(self.problems(GERMAN_PAGE.replace("`set_editor`", "set_editor")),
                         ["inline code `set_editor` of the English page is missing"])
        self.assertEqual(self.problems(GERMAN_PAGE.replace('`{"x": 1}`', '`{"x":1}`')),
                         ['inline code `{"x": 1}` of the English page is missing'])
        self.assertEqual(self.problems(GERMAN_PAGE + "\nund `extra`\n"), [])

    def test_inline_code_wrapped_over_two_lines_still_counts(self):
        english = ENGLISH_PAGE.replace("`Ctrl+S`", "`two words`")
        self.assertEqual(self.problems(GERMAN_PAGE.replace("`Ctrl+S`", "`two\nwords`"), english=english), [])

    def test_links(self):
        self.assertEqual(self.problems(GERMAN_PAGE.replace("GLOSSARY.md#brush", "GLOSSARY.md#bruch")), [
            "link to GLOSSARY.md#brush of the English page is missing",
            "link to GLOSSARY.md#bruch is not in the English page"])
        self.assertEqual(self.problems(GERMAN_PAGE.replace(" und die [Seite](https://example.com/a(b))", "")),
                         ["link to https://example.com/a(b) of the English page is missing"])

    def test_link_order_and_repeats_do_not_matter(self):
        text = GERMAN_PAGE.replace("das [Glossar](GLOSSARY.md#brush) und die [Seite](https://example.com/a(b))",
                                   "die [Seite](https://example.com/a(b)) und [Glossar](GLOSSARY.md#brush) und "
                                   "[noch mal](GLOSSARY.md#brush)")
        self.assertEqual(self.problems(text), [])

    def test_html_links_count(self):
        english = ENGLISH_PAGE + '\n<a href="x.md">x</a> and <https://example.com/b>\n'
        self.assertEqual(self.problems(GERMAN_PAGE + "\nund\n", english=english), [
            "link to https://example.com/b of the English page is missing",
            "link to x.md of the English page is missing"])

    def test_images_in_order_with_their_indent(self):
        self.assertEqual(self.problems(GERMAN_PAGE.replace("one.png", "uno.png")), [
            "images: number 1 is ('    ', '../assets/x/uno.png'), the English page has "
            "('    ', '../assets/x/one.png')"])
        self.assertEqual(self.problems(GERMAN_PAGE.replace("    ![Bild eins]", "![Bild eins]")), [
            "images: number 1 is ('', '../assets/x/one.png'), the English page has ('    ', '../assets/x/one.png')"])
        self.assertEqual(self.problems(GERMAN_PAGE.replace("![Bild](../assets/x/two.png)", "")),
                         ["images: 1 found, the English page has 2"])

    def test_inline_image_may_move_to_another_indent(self):
        text = GERMAN_PAGE.replace("2. Dann bleibt ![Bild](../assets/x/two.png) im Text.",
                                   "2. Dann bleibt\n   ![Bild](../assets/x/two.png) im Text.")
        self.assertEqual(self.problems(text), [])

    def test_div_blocks(self):
        self.assertEqual(self.problems(GERMAN_PAGE.replace('<div class="td-mcp" markdown>', '<div class="td-mcp">')), [
            "div blocks: number 1 is '<div class=\"td-mcp\">', the English page has '<div class=\"td-mcp\" markdown>'"])
        self.assertEqual(self.problems(GERMAN_PAGE.replace("</div>\n", "")),
                         ["div blocks: 1 found, the English page has 2"])

    def test_alert_kinds(self):
        self.assertEqual(self.problems(GERMAN_PAGE.replace("> [!NOTE]\n> Eine", "> [!WARNING]\n> Eine")),
                         ["alert kinds: number 1 is 'WARNING', the English page has 'NOTE'"])
        text = GERMAN_PAGE.replace("> [!NOTE]\n> Eine Notiz über `set_editor`.\n", "Eine Notiz über `set_editor`.\n")
        self.assertEqual(self.problems(text), ["alert kinds: number 1 is 'TIP', the English page has 'NOTE'"])
        self.assertEqual(self.problems(GERMAN_PAGE + "\n> [!WARNING]\n> Achtung.\n"),
                         ["alert kinds: 3 found, the English page has 2"])

    def test_alert_with_a_title_still_counts_as_its_kind(self):
        self.assertEqual(self.problems(GERMAN_PAGE.replace("> [!NOTE]\n> Eine", "> [!NOTE] Hinweis\n> Eine")), [])

    def test_label_missing_or_in_another_word(self):
        self.assertEqual(self.problems(GERMAN_PAGE.replace("**Inspektor**", "**Inspector**")), [
            "label 'Inspector' is not written as the editor shows it, expected 'Inspektor' in bold or italic"])
        self.assertEqual(self.problems(GERMAN_PAGE.replace("*Datei > Einstellungen…*", "*Datei > Optionen*")), [
            "label 'File > Preferences…' is not written as the editor shows it, expected 'Datei > Einstellungen…' "
            "in bold or italic"])

    def test_label_with_two_senses_takes_either_and_the_message_lists_both(self):
        self.assertEqual(self.problems(GERMAN_PAGE.replace("**Öffnen**", "**Offen**")), [])
        self.assertEqual(self.problems(GERMAN_PAGE.replace("**Öffnen**", "**Auf**")), [
            "label 'Open' is not written as the editor shows it, expected 'Öffnen' or 'Offen' in bold or italic"])

    def test_label_in_plain_text_does_not_count(self):
        self.assertEqual(self.problems(GERMAN_PAGE.replace("**Öffnen**", "Öffnen")), [
            "label 'Open' is not written as the editor shows it, expected 'Öffnen' or 'Offen' in bold or italic"])

    def test_other_bold_and_single_word_italic_are_not_labels(self):
        text = GERMAN_PAGE.replace("**Kein Label**", "**Anderes**").replace("*Nur kursiv*", "*x*")
        self.assertEqual(self.problems(text), [])

    def test_label_between_cjk_letters_without_spaces(self):
        english = "Use **Inspector** and *File > Preferences…* now.\n"
        catalogue = {"Inspector": ["检查器"], "File": ["文件"], "Preferences…": ["首选项…"]}
        self.assertEqual(tr.check_page(english, "现在用**检查器**和*文件 > 首选项…*吧。\n", "zh-Hans", catalogue), [])
        self.assertEqual(tr.check_page(english, "现在用**检查**和*文件 > 首选项…*吧。\n", "zh-Hans", catalogue), [
            "label 'Inspector' is not written as the editor shows it, expected '检查器' in bold or italic"])

    def test_star_in_code_and_arithmetic_is_not_emphasis(self):
        english = "Use `a * b` and 2 * 3 * 4 and **Inspector**.\n"
        self.assertEqual(tr.check_page(english, "Nimm `a * b` und 2 * 3 * 4 und **Inspektor**.\n", "de",
                                       self.CATALOGUE), [])

    def test_glossary_terms_use_the_catalogue_word(self):
        english = "# Glossary\n\n## brush\nA solid.\n\n## Inspector\nThe panel.\n\n## activator\nWho started it.\n"
        good = ("# Glossar { #glossary }\n\n## Brush { #brush data-forms=\"Brushes\" }\nEin Körper.\n\n"
                "## inspektor { #inspector }\nDas Panel.\n\n## Auslöser { #activator }\nWer es startete.\n")
        self.assertEqual(self.problems(good, english=english, glossary=True), [])
        bad = good.replace("## inspektor", "## Prüfer")
        self.assertEqual(self.problems(bad, english=english, glossary=True), [
            "glossary term 'Inspector' is 'Prüfer', the editor calls it 'Inspektor'"])
        self.assertEqual(self.problems(bad, english=english, glossary=False), [])

    def test_dashes(self):
        for char, name in (("\u2014", "em dash"), ("\u2013", "en dash"), ("\u2015", "horizontal bar")):
            text = GERMAN_PAGE.replace("Letzte Worte.", f"Letzte {char} Worte.")
            self.assertEqual(self.problems(text), [f"line {line_of(text, 'Letzte')}: {name} U+{ord(char):04X}, {BREAK}"])
        text = GERMAN_PAGE.replace("Letzte Worte.", "Letzte\u2014Worte\u2013 noch.")
        self.assertEqual(self.problems(text), [f"line {line_of(text, 'Letzte')}: em dash U+2014, {BREAK}",
                                               f"line {line_of(text, 'Letzte')}: en dash U+2013, {BREAK}"])

    def test_spaced_hyphen(self):
        text = GERMAN_PAGE.replace("Letzte Worte.", "Letzte - Worte.")
        self.assertEqual(self.problems(text), [f"line {line_of(text, 'Letzte')}: spaced hyphen as punctuation, {BREAK}"])
        text = GERMAN_PAGE.replace("Letzte Worte.", "Letzte -Worte und Rück- und Vorwärts.")
        self.assertEqual(self.problems(text), [])

    def test_hyphens_that_are_fine(self):
        text = GERMAN_PAGE.replace("Letzte Worte.", "Ein Open-Source-Tool\n\n- Punkt\n    - Unterpunkt\n1. Eins\n\n"
                                   "| a | b |\n| - | --- |\n\n---\n\nund `a - b` im Code")
        self.assertEqual(self.problems(text), [])

    def test_spaced_hyphens_of_the_english_page_are_allowed_once_each(self):
        english = ENGLISH_PAGE.replace("Last words.", "The + and - keys.")
        text = GERMAN_PAGE.replace("Letzte Worte.", "Die + und - Tasten.")
        self.assertEqual(self.problems(text, english=english), [])
        text = GERMAN_PAGE.replace("Letzte Worte.", "Die + und - Tasten - nicht.")
        self.assertEqual(self.problems(text, english=english),
                         [f"line {line_of(text, 'Tasten')}: spaced hyphen as punctuation, {BREAK}"])

    def test_emoji_is_flagged_and_code_is_not(self):
        text = GERMAN_PAGE.replace("Letzte Worte.", "Gut \u2705 so \U0001F600.")
        self.assertEqual(self.problems(text), [f"line {line_of(text, 'Gut')}: emoji"])
        english = ENGLISH_PAGE.replace("Last words.", "In code `a \u2014 b \u2705`.")
        text = GERMAN_PAGE.replace("Letzte Worte.", "Im Code `a \u2014 b \u2705` bleibt es.")
        self.assertEqual(self.problems(text, english=english), [])

    def test_arrows_umlauts_and_symbols_are_not_emoji(self):
        text = GERMAN_PAGE.replace("Letzte Worte.", "Von A \u2192 B, äöüß \u00a9 \u2122 \u2318.")
        self.assertEqual(self.problems(text), [])

    def test_cjk_line_break_only_in_chinese_and_japanese(self):
        english = "# T\n\nLine one\nline two.\n\nMore.\n"
        broken = "# T { #t }\n\n第一行文字\n第二行文字。\n\nMore.\n"
        message = ["line 4: line break between two CJK letters, keep the paragraph on one line"]
        self.assertEqual(tr.check_page(english, broken, "zh-Hans", {}), message)
        self.assertEqual(tr.check_page(english, broken, "ja", {}), message)
        self.assertEqual(tr.check_page(english, broken, "de", {}), [])

    def test_cjk_breaks(self):
        self.assertEqual(tr.cjk_breaks("> あいう\n> えお\n"), [1])
        self.assertEqual(tr.cjk_breaks("한글\n한글\n"), [1])
        self.assertEqual(tr.cjk_breaks("一\n二\n三\n"), [1, 2])

    def test_cjk_line_breaks_that_are_fine(self):
        for text in ("文字、\n文字\n", "文字\n。文字\n", "文字。\n文字\n", "中文\nlatin\n", "latin\n中文\n",
                     "文字\n\n文字\n", "- 项目一\n- 项目二\n", "| 甲 |\n| 乙 |\n", "```text\n文字\n文字\n```\n",
                     "文字\n1. 项目\n"):
            self.assertEqual(tr.cjk_breaks(text), [], text)


class TranslationsToolTest(unittest.TestCase):
    SUMMARY = "# Summary\n\n* [Intro](README.md)\n* Editor\n    * [Brushes](editor/brushes.md)\n    * [Terrain](editor/terrain.md)\n"
    PAGES = {
        "README.md": "# Intro\n\nHello `x` and **Inspector**.\n",
        "GLOSSARY.md": "# Glossary\n\n## brush\nA solid.\n\n## unit\nA step.\n",
        "editor/brushes.md": "# Brushes\n\nUse *File > Preferences…*.\n",
        "editor/terrain.md": "# Terrain\n",
    }

    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = pathlib.Path(tmp.name)
        self.docs = self.root / "docs"
        self.locales = self.root / "locales"
        write(self.docs / "SUMMARY.md", self.SUMMARY)
        for page, text in self.PAGES.items():
            write(self.docs / page, text)
        write(self.locales / "de.json", CATALOGUE_JSON)
        write(self.locales / "fr.json", '{"File": "Fichier", "Preferences…": "Préférences…", "Inspector": "Inspecteur"}')
        self.de = self.docs / "translations" / "de"
        write(self.de / "site.toml", 'theme_language = "de"\n\n[nav]\n"Intro" = "Einführung"\n"Old" = "Alt"\n')
        write(self.de / "README.md", stamped(self.PAGES["README.md"], "# Einführung { #intro }\n\nHallo `x` und **Inspektor**.\n"))
        write(self.de / "GLOSSARY.md", stamped("# Glossary\n\n## brush\nOld.\n", "# Glossar { #glossary }\n\n## Brush { #brush }\nEin Körper.\n"))
        write(self.de / "editor/brushes.md", stamped("# Brushes\n", "# Pinsel { #brushes }\n"))
        write(self.de / "orphan.md", stamped("# Gone\n"))

    def run_main(self, *argv):
        out = io.StringIO()
        with mock.patch.object(tr, "DOCS", self.docs), mock.patch.object(tr, "LOCALES", self.locales), \
                contextlib.redirect_stdout(out):
            code = tr.main(list(argv))
        return code, out.getvalue()

    def test_status_of_german(self):
        self.assertEqual(tr.status(self.docs, ["de"]), """### de: 1 of 4 pages current

Outdated, the English page changed since it was translated (2):

* GLOSSARY.md (the glossary hover cards show English until it is updated)
* editor/brushes.md

Missing (1): editor/terrain.md

Orphans, translated pages whose English page is gone (1): orphan.md

Nav texts with no [nav] entry (3): Editor, Brushes, Terrain

[nav] keys no longer in SUMMARY.md (1): Old
""")

    def test_status_of_a_language_with_nothing(self):
        self.assertEqual(tr.status(self.docs, ["fr"]), """### fr: 0 of 4 pages current

Missing (4): GLOSSARY.md, README.md, editor/brushes.md, editor/terrain.md

Nav texts with no [nav] entry (4): Intro, Editor, Brushes, Terrain
""")

    def test_status_of_a_finished_language(self):
        for page, text in self.PAGES.items():
            write(self.docs / "translations" / "es" / page, stamped(text))
        write(self.docs / "translations" / "es" / "site.toml",
              '[nav]\n"Intro" = "a"\n"Editor" = "b"\n"Brushes" = "c"\n"Terrain" = "d"\n')
        self.assertEqual(tr.status(self.docs, ["es"]), "### es: 4 of 4 pages current\n\nNothing to do.\n")

    def test_status_puts_the_glossary_first_and_lists_languages_in_order(self):
        report = tr.status(self.docs, ["fr", "de"])
        self.assertLess(report.index("### fr"), report.index("### de"))
        german = tr.status(self.docs, ["de"])
        self.assertLess(german.index("* GLOSSARY.md"), german.index("* editor/brushes.md"))

    def test_status_with_diffs(self):
        glossary = "# Glossary\n\n## brush\nOld.\n"
        old = {bs.blob_id(b"# Brushes\n"): "# Brushes\n", bs.blob_id(glossary.encode()): glossary}
        text = tr.status(self.docs, ["de"], True, old.get)
        self.assertIn("* editor/brushes.md\n\n```diff\n--- stamped\n+++ current\n@@ -1 +1,3 @@\n # Brushes\n+\n"
                      "+Use *File > Preferences…*.\n```\n\nMissing (1)", text)
        self.assertIn("```diff\n--- stamped\n+++ current\n@@ -3,2 +3,5 @@\n ## brush\n-Old.\n"
                      "+A solid.\n+\n+## unit\n+A step.\n```\n* editor/brushes.md", text)

    def test_status_diff_when_the_old_page_is_gone(self):
        text = tr.status(self.docs, ["de"], True, lambda blob: None)
        self.assertIn("the English page it followed is not in this clone", text)

    def test_status_with_a_broken_site_toml_still_reports(self):
        write(self.de / "site.toml", "this is not toml")
        self.assertIn("Nav texts with no [nav] entry (4)", tr.status(self.docs, ["de"]))

    def test_status_never_fails(self):
        code, out = self.run_main("status", "--lang", "de")
        self.assertEqual((code, out.startswith("### de: 1 of 4")), (0, True))

    def test_check_of_a_clean_language(self):
        self.assertEqual(tr.check(self.docs, self.locales, ["de"], ["README.md"]), [])

    def test_check_skips_outdated_pages_unless_named(self):
        self.assertEqual(tr.check(self.docs, self.locales, ["de"]), [])
        found = tr.check(self.docs, self.locales, ["de"], ["editor/brushes.md"])
        self.assertEqual(found, [
            "de/editor/brushes.md: label 'File > Preferences…' is not written as the editor shows it, expected "
            "'Datei > Einstellungen…' in bold or italic"])

    def test_check_looks_at_current_pages_and_names_the_page(self):
        write(self.de / "README.md", stamped(self.PAGES["README.md"], "# Einführung { #intro }\n\nHallo und **Inspector**.\n"))
        self.assertEqual(tr.check(self.docs, self.locales, ["de"]), [
            "de/README.md: inline code `x` of the English page is missing",
            "de/README.md: label 'Inspector' is not written as the editor shows it, expected 'Inspektor' in bold or italic"])

    def test_check_front_matter(self):
        good = "# Einführung { #intro }\n\nHallo `x` und **Inspektor**.\n"
        message = "de/README.md: front matter needs source: followed by 40 hex digits, see `prep` and `stamp`"
        for head in ("", "---\nsource: abc\n---\n", "---\nsource: " + "G" * 40 + "\n---\n", "---\n---\n",
                     "---\nsource: " + "A" * 40 + "\n---\n", "---\nsource: " + "a" * 41 + "\n---\n"):
            write(self.de / "README.md", head + good)
            self.assertEqual(tr.check(self.docs, self.locales, ["de"]), [message], head)
        write(self.de / "README.md", f"---\nsource: {'a' * 40}\n---\n" + good)
        self.assertEqual(tr.check(self.docs, self.locales, ["de"], ["README.md"]), [])
        write(self.de / "README.md", f"---\nsource: {'a' * 40}\ntitle: x\n---\n" + good)
        self.assertEqual(tr.check(self.docs, self.locales, ["de"], ["README.md"]), ["de/README.md: front matter holds only source"])

    def test_check_line_numbers_count_the_front_matter(self):
        write(self.de / "README.md", stamped(self.PAGES["README.md"], "# Einführung { #intro }\n\nHallo \u2014 `x` und **Inspektor**.\n"))
        self.assertEqual(tr.check(self.docs, self.locales, ["de"]),
                         [f"de/README.md: line 6: em dash U+2014, {BREAK}"])

    def test_check_glossary_uses_the_catalogue(self):
        write(self.de / "GLOSSARY.md", stamped(self.PAGES["GLOSSARY.md"],
              "# Glossar { #glossary }\n\n## Pinsel { #brush }\nEin Körper.\n\n## Einheit { #unit }\nEin Schritt.\n"))
        self.assertEqual(tr.check(self.docs, self.locales, ["de"]), [
            "de/GLOSSARY.md: glossary term 'brush' is 'Pinsel', the editor calls it 'Brush'"])

    def test_check_named_page_without_translation_or_english_page(self):
        self.assertEqual(tr.check(self.docs, self.locales, ["de"], ["editor/terrain.md", "orphan.md"]), [
            "de/editor/terrain.md: no such translation", "de/orphan.md: no such English page"])

    def test_check_language_without_catalogue(self):
        write(self.docs / "translations" / "ja" / "site.toml", 'theme_language = "ja"\n')
        found = tr.check(self.docs, self.locales, ["ja"])
        self.assertEqual(found, [f"ja: no editor catalogue ja.json in {self.locales}"])

    def test_check_every_language_and_one_language(self):
        write(self.docs / "translations" / "fr" / "README.md", stamped(self.PAGES["README.md"], "# Intro { #intro }\n"))
        both = tr.check(self.docs, self.locales, ["de", "fr"])
        self.assertEqual(len(both), 2)
        self.assertTrue(all(line.startswith("fr/README.md") for line in both))
        self.assertEqual(tr.check(self.docs, self.locales, ["de"]), [])

    def test_site_strings(self):
        def problems(text):
            write(self.de / "site.toml", text)
            return tr.check_site_strings(self.docs, "de")
        self.assertEqual(problems('theme_language = "de"\nnote = "Hinweis"\n[nav]\n"A" = "B"\n'), [])
        self.assertEqual(problems(""), [])
        self.assertEqual(problems('titel = "x"\n'), ["site.toml has an unknown key 'titel'"])
        self.assertEqual(problems("note = 3\n"), ["site.toml note has to be a string"])
        self.assertEqual(problems('nav = "x"\n'), ["site.toml nav has to be a table of strings"])
        self.assertEqual(problems('[nav]\n"A" = 1\n'), ["site.toml nav has to be a table of strings"])
        self.assertEqual(len(problems("nope")), 1)
        self.assertTrue(problems("nope")[0].startswith("site.toml does not parse"))
        self.assertEqual(tr.check_site_strings(self.docs, "pt"), [])

    def test_site_strings_of_the_real_languages_are_fine(self):
        for code in tr.known_languages():
            self.assertEqual(tr.check_site_strings(tr.DOCS, code), [], code)

    def test_check_exit_codes(self):
        code, out = self.run_main("check", "--lang", "de")
        self.assertEqual((code, out), (0, "\ntranslations are fine\n"))
        write(self.de / "README.md", stamped(self.PAGES["README.md"], "# Einführung { #intro }\n"))
        code, out = self.run_main("check", "--lang", "de", "README.md")
        self.assertEqual(code, 1)
        self.assertTrue(out.endswith("2 problems\n"), out)

    def test_unknown_language_and_missing_language(self):
        with mock.patch.object(tr, "DOCS", self.docs), contextlib.redirect_stderr(io.StringIO()):
            for argv in (["status", "--lang", "xx"], ["prep", "README.md"], ["stamp", "README.md"],
                         ["prep", "--lang", "de", "nope.md"], ["stamp", "--lang", "de", "editor/terrain.md"],
                         ["stamp", "--lang", "de"]):
                with self.assertRaises(SystemExit) as raised:
                    tr.main(argv)
                self.assertEqual(raised.exception.code, 2, argv)

    def test_stamp_changes_only_the_source_line(self):
        old = self.de / "editor/brushes.md"
        before = old.read_text(encoding="utf-8")
        self.assertEqual(tr.stamp(self.docs, "de", ["editor/brushes.md", "README.md"]), ["editor/brushes.md"])
        now = bs.blob_id((self.docs / "editor/brushes.md").read_bytes())
        self.assertEqual(old.read_text(encoding="utf-8"), before.replace(before.split("\n")[1], f"source: {now}"))
        self.assertEqual(bs.page_state(self.docs, "de", "editor/brushes.md"), "current")
        self.assertEqual(tr.stamp(self.docs, "de", ["editor/brushes.md"]), [])

    def test_stamp_adds_the_block_or_the_line(self):
        write(self.de / "editor/terrain.md", "# Gelände { #terrain }\n")
        write(self.de / "editor/brushes.md", "---\ntitle: x\n---\n# Pinsel { #brushes }\n")
        tr.stamp(self.docs, "de", ["editor/terrain.md", "editor/brushes.md"])
        now = bs.blob_id((self.docs / "editor/terrain.md").read_bytes())
        self.assertEqual((self.de / "editor/terrain.md").read_text(encoding="utf-8"),
                         f"---\nsource: {now}\n---\n# Gelände {{ #terrain }}\n")
        now = bs.blob_id((self.docs / "editor/brushes.md").read_bytes())
        self.assertEqual((self.de / "editor/brushes.md").read_text(encoding="utf-8"),
                         f"---\ntitle: x\nsource: {now}\n---\n# Pinsel {{ #brushes }}\n")

    def test_stamp_does_not_touch_another_language(self):
        write(self.docs / "translations" / "fr" / "README.md", "---\nsource: old\n---\n# Intro\n")
        tr.stamp(self.docs, "de", ["README.md"])
        self.assertEqual((self.docs / "translations" / "fr" / "README.md").read_text(encoding="utf-8"),
                         "---\nsource: old\n---\n# Intro\n")

    def test_stamp_command(self):
        code, out = self.run_main("stamp", "--lang", "de", "editor/brushes.md")
        self.assertEqual((code, out), (0, "stamped de/editor/brushes.md\n"))

    def test_prep_of_a_page_with_labels_and_glossary_terms(self):
        page = "editor/brushes.md"
        write(self.docs / page, "# Brushes\n\nA brush is made *File > Preferences…*, **Open** and **Inspector** plus `x`.\n")
        write(self.de / page, "---\nsource: " + "0" * 40 + "\n---\n# Pinsel { #brushes }\n")
        write(self.de / "GLOSSARY.md", "---\nsource: x\n---\n# Glossar { #glossary }\n\n## Brush { #brush data-forms=\"Brushes|Brushs\" }\nx\n")
        with mock.patch.object(tr, "git_blob", lambda blob: "# Brushes\n"):
            text = tr.prep(self.docs, self.locales, "de", page)
        self.assertEqual(text, f"""front matter to put at the top of the translation:

---
source: {bs.blob_id((self.docs / page).read_bytes())}
---

heading ids, in order:

# Brushes {{ #brushes }}

labels, written as the editor shows them in bold or italic:

File > Preferences…  =>  Datei > Einstellungen…
Open  =>  Öffnen  or  Offen
Inspector  =>  Inspektor

glossary terms of the page:

brush  =>  Brush  (forms: Brushes, Brushs)

the English page changed since the translation:

--- stamped
+++ current
@@ -1 +1,3 @@
 # Brushes
+
+A brush is made *File > Preferences…*, **Open** and **Inspector** plus `x`.

the English page:

# Brushes

A brush is made *File > Preferences…*, **Open** and **Inspector** plus `x`.
""")

    def test_prep_of_a_current_page_has_no_diff_and_catalogue_words_stand_in_for_a_missing_glossary(self):
        write(self.docs / "README.md", "# Intro\n\nThe Inspector shows it.\n")
        text = tr.prep(self.docs, self.locales, "fr", "README.md")
        self.assertNotIn("changed since", text)
        self.assertIn("glossary terms of the page:\n\n\nthe English page:", text)
        self.assertIn("labels, written as the editor shows them in bold or italic:\n\n\nglossary", text)

    def test_prep_command_prints_the_page(self):
        code, out = self.run_main("prep", "--lang", "de", "README.md")
        self.assertEqual(code, 0)
        self.assertTrue(out.startswith("front matter to put at the top of the translation:"))
        self.assertTrue(out.rstrip().endswith("Hello `x` and **Inspector**."))


class RealTranslationsTest(unittest.TestCase):
    def test_the_repository_translations_pass(self):
        self.assertEqual(tr.check(tr.DOCS, tr.LOCALES, tr.known_languages()), [])

    def test_the_english_docs_pass_as_their_own_translation(self):
        for page in bs.english_pages(tr.DOCS):
            text = bs.read_page(tr.DOCS / page)
            self.assertEqual(tr.check_page(text, text, "de", {}), [], page)

    def test_every_catalogue_is_found_for_every_language(self):
        for code in tr.known_languages():
            self.assertTrue((tr.LOCALES / f"{code}.json").is_file(), code)


class WorkflowTest(unittest.TestCase):
    TEXT = (bs.ROOT / ".github" / "workflows" / "pages.yml").read_text(encoding="utf-8")

    def test_both_triggers_watch_the_tools_and_the_catalogues(self):
        for path in ('"tools/translations.py"', '"crates/td_editor/locales/**"', '"tools/test_docs.py"'):
            self.assertEqual(self.TEXT.count(path), 2, path)

    def test_check_runs_before_the_build_and_the_report_goes_to_the_job_summary(self):
        check = self.TEXT.index("python tools/translations.py check")
        self.assertLess(check, self.TEXT.index("python tools/build_site.py"))
        self.assertIn('python tools/translations.py status >> "$GITHUB_STEP_SUMMARY"', self.TEXT)
        self.assertLess(self.TEXT.index("translations.py status"), self.TEXT.index("upload-pages-artifact"))


if __name__ == "__main__":
    unittest.main()
