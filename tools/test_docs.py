#!/usr/bin/env python3
"""Tests of the docs tools, `python3 tools/test_docs.py`. The tests that look at a built site are skipped until
`python3 tools/build_site.py` has produced _site."""
import copy
import io
import pathlib
import subprocess
import sys
import tarfile
import tempfile
import tomllib
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

import build_site as bs  # noqa: E402
import check_site  # noqa: E402

CONFIG = bs.load_config()
ZH = {"theme_language": "zh"}


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
        self.assertEqual(project, expected)

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

    def test_fallback_page_has_the_note_and_the_english_ids(self):
        html = (self.SITE / "de" / "editor" / "brushes.html").read_text(encoding="utf-8")
        self.assertIn(bs.DEFAULTS["fallback"], html)
        self.assertIn('id="reshape"', html)

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


if __name__ == "__main__":
    unittest.main()
