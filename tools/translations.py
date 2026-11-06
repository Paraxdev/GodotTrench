#!/usr/bin/env python3
"""Checks the translated docs pages against their English source and reports which ones fell behind.

  python3 tools/translations.py check [--lang de] [page ...]   # exit 1 on problems
  python3 tools/translations.py status [--lang de] [--diff]    # markdown report, always exit 0
  python3 tools/translations.py prep --lang de editor/brushes.md
  python3 tools/translations.py stamp --lang de editor/brushes.md ...

`check` looks at every translation that is current, and at the pages it is given by name whatever their state, so a
page can be checked before it is stamped. A translation whose English page changed is only reported by `status`, it
never fails a check. `prep` prints what a translator needs for one page, `stamp` records that a translation now
follows the English page as it is. See docs/development.md#documentation.
"""
import argparse
import difflib
import html
import itertools
import json
import pathlib
import re
import subprocess
import sys
import tomllib
import unicodedata

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

import build_site as bs  # noqa: E402

DOCS = bs.ROOT / "docs"
LOCALES = bs.ROOT / "crates" / "td_editor" / "locales"
SITE_KEYS = {"theme_language", "description", "light_toggle", "dark_toggle", "mcp_switch", "notice", "fallback",
             "note", "tip", "warning", "nav"}
SOURCE = re.compile(r"[0-9a-f]{40}")
SENSE = re.compile(r"#[a-z_]+$")
HEADING = re.compile(r"^(?P<hashes>#{1,6})[ \t]+(?P<text>.*?)[ \t]*$")
ATTRS = re.compile(r"[ \t]*\{:?[ \t]*(?P<attrs>[^{}]*)\}$")
QUOTED = re.compile(r'"[^"]*"|\'[^\']*\'')
CODE_SPAN = re.compile(r"(?<!`)(`+)(?!`)(.+?)(?<!`)\1(?!`)", re.DOTALL)
BOLD = re.compile(r"\*\*(?=\S)(.+?)(?<=\S)\*\*", re.DOTALL)
ITALIC = re.compile(r"(?<!\*)\*(?!\*)(?=\S)(.+?)(?<=\S)(?<!\*)\*(?!\*)", re.DOTALL)
DESTINATION = r"(?P<target><[^>\n]*>|[^()\s]*(?:\([^()\s]*\)[^()\s]*)*)(?:\s+(?:\"[^\"]*\"|'[^']*'))?"
IMAGE = re.compile(r"!\[[^\]]*\]\(\s*" + DESTINATION + r"\s*\)")
LINK = re.compile(r"(?<!!)\[[^\]]*\]\(\s*" + DESTINATION + r"\s*\)")
HTML_LINK = re.compile(r"""<a\b[^>]*?\bhref\s*=\s*["']([^"']*)["']""", re.IGNORECASE)
AUTOLINK = re.compile(r"<(https?://[^>\s]+)>")
DIV = re.compile(r"^\s*(?:>\s*)*(<div\b.*|</div>)\s*$")
ALERT_KIND = re.compile(r"^\s*(?:>\s*)+\[!(NOTE|TIP|WARNING)\]")
DASHES = {"\u2014": "em dash", "\u2013": "en dash", "\u2015": "horizontal bar"}
SPACED_HYPHEN = re.compile(r"(?<=\s)-(?=\s)")
LIST_MARKER = re.compile(r"^\s*(?:>\s*)*(?:[-*+]|\d+[.)])(?:\s+|$)")
TABLE_RULE = re.compile(r"^[\s|:\-]+$")
EMOJI = re.compile("[\U0001F000-\U0001FAFF\u2600-\u27bf\u2b05-\u2b07\u2b50\u2b55\u231a\u231b\u23e9-\u23f3"
                   "\u23f8-\u23fa\u25b6\u25c0\u25fb-\u25fe\ufe0f\u20e3]")
# Han, kana and Hangul letters. The full width punctuation of U+3000 to U+303F and U+FF00 to U+FFEF is left out on
# purpose, a line may end or begin with it.
CJK = re.compile("[\u3040-\u30ff\u31f0-\u31ff\u3400-\u4dbf\u4e00-\u9fff\uf900-\ufaff\uac00-\ud7af]")
CJK_LANGUAGES = {"zh-Hans", "ja"}
NOT_PRESENT = "\0"


def load_catalogue(locales, code):
    """The translations of the editor in `code` by English text, without the `#name` suffix of a sense. A text that
    has several senses lists every value. Plural entries are left out."""
    data = json.loads((pathlib.Path(locales) / f"{code}.json").read_text(encoding="utf-8"))
    catalogue = {}
    for key, value in data.items():
        if isinstance(value, str):
            values = catalogue.setdefault(SENSE.sub("", key), [])
            if value not in values:
                values.append(value)
    return catalogue


def split_attrs(text):
    """A heading's text without its trailing `{ #id data-forms="a|b" }` and the id and forms from it."""
    match = ATTRS.search(text)
    if not match:
        return text, None, []
    attrs = match["attrs"]
    forms = re.search(r'data-forms\s*=\s*"([^"]*)"', attrs)
    bare = QUOTED.sub("", attrs)
    ident = re.search(r"(?:^|\s)#([^\s]+)", bare)
    return text[:match.start()].rstrip(), ident[1] if ident else None, forms[1].split("|") if forms else []


def plain(text):
    """What a heading's markup reads as, the way the table of contents sees it."""
    text = re.sub(r"!?\[([^\]]*)\]\([^)]*\)", r"\1", text)
    text = CODE_SPAN.sub(r"\2", text)
    text = re.sub(r"<[^>]+>", "", text)
    text = re.sub(r"\*+|(?<!\w)_+|_+(?!\w)", "", text)
    return html.unescape(text).strip()


def slugify(text):
    """The id Python-Markdown's table of contents gives a heading text, the ASCII letters and digits of it."""
    text = unicodedata.normalize("NFKD", text).encode("ascii", "ignore").decode("ascii")
    text = re.sub(r"[^\w\s-]", "", text).strip().lower()
    return re.sub(r"[-\s]+", "-", text)


def unique_id(ident, used):
    while ident in used or not ident:
        match = re.match(r"^(.*)_([0-9]+)$", ident)
        ident = f"{match[1]}_{int(match[2]) + 1}" if match else f"{ident}_1"
    used.add(ident)
    return ident


def headings(text):
    """Every heading outside code as (line index, level, text, id, forms). An explicit `{ #id }` is the id, else it
    is made from the text, with `_1`, `_2` after a repeat, as Python-Markdown does."""
    lines = text.split("\n")
    skip = bs.fenced(lines)
    found = []
    for i, line in enumerate(lines):
        match = None if skip[i] else HEADING.match(line)
        if match:
            title, ident, forms = split_attrs(match["text"])
            found.append((i, len(match["hashes"]), title, ident, forms))
    used = {ident for _, _, _, ident, _ in found if ident}
    result = []
    for i, level, title, ident, forms in found:
        result.append((i, level, title, ident or unique_id(slugify(plain(title)), used), forms))
    return result


def paragraphs(text):
    """The text outside fenced code split at blank lines, each as its lines joined with a newline."""
    lines = text.split("\n")
    skip = bs.fenced(lines)
    found, current = [], []
    for line, inside in zip(lines, skip):
        if inside or not line.strip():
            if current:
                found.append("\n".join(current))
            current = []
        else:
            current.append(line)
    if current:
        found.append("\n".join(current))
    return found


def code_spans(text):
    """The inline code spans of the text outside fences, whitespace inside a span collapsed."""
    return [re.sub(r"\s+", " ", m[2]).strip() for p in paragraphs(text) for m in CODE_SPAN.finditer(p)]


def blank(match):
    return NOT_PRESENT * len(match[0])


def emphasis(text):
    """The bold spans and the italic spans of the text outside code as (marker, content), in reading order."""
    found = []
    for paragraph in paragraphs(text):
        paragraph = CODE_SPAN.sub(blank, paragraph)
        spans = [(m.start(), "**", m[1]) for m in BOLD.finditer(paragraph)]
        spans += [(m.start(), "*", m[1]) for m in ITALIC.finditer(BOLD.sub(blank, paragraph))]
        found += [(marker, re.sub(r"\s+", " ", content)) for _, marker, content in sorted(spans)]
    return found


def labels(text, catalogue):
    """The menu and panel labels an English page names, each as the list of its parts. Every bold span and every
    italic span holding ` > ` is split at ` > `, and it is a label when each part is a catalogue key."""
    found = []
    for marker, content in emphasis(text):
        if marker == "*" and " > " not in content:
            continue
        parts = content.split(" > ")
        if all(part in catalogue for part in parts) and parts not in found:
            found.append(parts)
    return found


def label_options(parts, catalogue):
    """Every way the label reads in the language, a part with several senses giving several."""
    return [" > ".join(choice) for choice in itertools.product(*(catalogue[part] for part in parts))]


def targets(text, pattern):
    """The targets of the links or images matching `pattern` in the lines outside code."""
    found = []
    lines = text.split("\n")
    for line, inside in zip(lines, bs.fenced(lines)):
        if not inside:
            for match in pattern.finditer(line):
                found.append(match["target"].strip("<>"))
    return found


def image_lines(text):
    """Every image as (indent, target), the indent only for an image alone on its line."""
    found = []
    lines = text.split("\n")
    for line, inside in zip(lines, bs.fenced(lines)):
        if inside:
            continue
        for match in IMAGE.finditer(line):
            alone = line.strip() == match[0]
            found.append((line[:len(line) - len(line.lstrip())] if alone else None, match["target"].strip("<>")))
    return found


def link_targets(text):
    found = set(targets(text, LINK))
    lines = text.split("\n")
    for line, inside in zip(lines, bs.fenced(lines)):
        if not inside:
            found.update(HTML_LINK.findall(line))
            found.update(AUTOLINK.findall(line))
    return found


def blocks(text):
    """The lines of every fenced code block, indentation and fence lines as they are."""
    lines = text.split("\n")
    return [lines[start:end + 1] for start, end in bs.fence_spans(lines)]


def div_lines(text):
    found = []
    lines = text.split("\n")
    for line, inside in zip(lines, bs.fenced(lines)):
        match = None if inside else DIV.match(line)
        if match:
            found.append(re.sub(r"\s+", " ", match[1]))
    return found


def alert_kinds(text):
    found = []
    lines = text.split("\n")
    for line, inside in zip(lines, bs.fenced(lines)):
        match = None if inside else ALERT_KIND.match(line)
        if match:
            found.append(match[1])
    return found


def prose_lines(text):
    """(line index, text) of every line outside fenced code, inline code removed."""
    lines = text.split("\n")
    return [(i, CODE_SPAN.sub("", line)) for i, (line, inside) in enumerate(zip(lines, bs.fenced(lines))) if not inside]


def spaced_hyphens(text):
    """How many spaced hyphens the text has outside code, list markers and table rules."""
    return sum(len(SPACED_HYPHEN.findall(LIST_MARKER.sub("", line))) for _, line in prose_lines(text)
               if not TABLE_RULE.match(line))


def style_problems(text, allowed=0):
    """(line index, message) of dashes used as punctuation and emoji, outside code. `allowed` spaced hyphens are let
    through, the ones of the English page, where one is a key name like the + and - keys."""
    found = []
    for i, line in prose_lines(text):
        for char, name in DASHES.items():
            if char in line:
                found.append((i, f"{name} U+{ord(char):04X}, join clauses with a comma or a full stop"))
        if not TABLE_RULE.match(line):
            for _ in SPACED_HYPHEN.findall(LIST_MARKER.sub("", line)):
                if allowed > 0:
                    allowed -= 1
                else:
                    found.append((i, "spaced hyphen as punctuation, join clauses with a comma or a full stop"))
        if EMOJI.search(line):
            found.append((i, "emoji"))
    return found


def cjk_breaks(text):
    """Line indexes where a paragraph breaks between two CJK letters, which browsers show as a space."""
    lines = text.split("\n")
    skip = bs.fenced(lines)
    found = []
    for i in range(len(lines) - 1):
        if skip[i] or skip[i + 1] or not lines[i].strip() or not lines[i + 1].strip():
            continue
        end = lines[i].rstrip()[-1]
        start = lines[i + 1].lstrip(" \t>")[:1]
        if CJK.match(end) and CJK.match(start):
            found.append(i + 1)
    return found


def compare_lists(what, expected, got):
    if expected == got:
        return []
    for i, (a, b) in enumerate(zip(expected, got)):
        if a != b:
            return [f"{what}: number {i + 1} is {b!r}, the English page has {a!r}"]
    return [f"{what}: {len(got)} found, the English page has {len(expected)}"]


def check_page(english, translated, code, catalogue, glossary=False, offset=0):
    """The problems of one translated page (its text after the front matter, which took `offset` lines) against the
    English page."""
    problems = []
    problems += compare_lists("heading ids", [h[3] for h in headings(english)], [h[3] for h in headings(translated)])
    if len(blocks(english)) != len(blocks(translated)):
        problems.append(f"{len(blocks(translated))} code blocks, the English page has {len(blocks(english))}")
    else:
        for i, (a, b) in enumerate(zip(blocks(english), blocks(translated)), 1):
            if a != b:
                problems.append(f"code block {i} differs from the English one, code stays byte for byte")
                break
    have = set(code_spans(translated))
    for span in dict.fromkeys(code_spans(english)):
        if span not in have:
            problems.append(f"inline code `{span}` of the English page is missing")
    for target in sorted(link_targets(english) - link_targets(translated)):
        problems.append(f"link to {target} of the English page is missing")
    for target in sorted(link_targets(translated) - link_targets(english)):
        problems.append(f"link to {target} is not in the English page")
    problems += compare_lists("images", image_lines(english), image_lines(translated))
    problems += compare_lists("div blocks", div_lines(english), div_lines(translated))
    problems += compare_lists("alert kinds", alert_kinds(english), alert_kinds(translated))
    spans = [content for _, content in emphasis(translated)]
    for parts in labels(english, catalogue):
        options = label_options(parts, catalogue)
        if not any(option in span for option in options for span in spans):
            problems.append(f"label {' > '.join(parts)!r} is not written as the editor shows it, expected "
                            + " or ".join(repr(option) for option in options) + " in bold or italic")
    if glossary:
        names = {c.lower(): v for c, v in catalogue.items()}
        mine = {h[3]: h for h in headings(translated)}
        for _, _, title, ident, _ in headings(english):
            options = names.get(title.lower())
            if options and ident in mine and mine[ident][2].lower() not in {o.lower() for o in options}:
                problems.append(f"glossary term {title!r} is {mine[ident][2]!r}, the editor calls it "
                                + " or ".join(repr(o) for o in options))
    for line, message in style_problems(translated, spaced_hyphens(english)):
        problems.append(f"line {line + 1 + offset}: {message}")
    if code in CJK_LANGUAGES:
        for line in cjk_breaks(translated):
            problems.append(f"line {line + 1 + offset}: line break between two CJK letters, keep the paragraph on one line")
    return problems


def check_site_strings(docs, code):
    file = docs / "translations" / code / "site.toml"
    if not file.is_file():
        return []
    try:
        data = tomllib.loads(file.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as error:
        return [f"site.toml does not parse: {error}"]
    problems = [f"site.toml has an unknown key {key!r}" for key in data if key not in SITE_KEYS]
    for key, value in data.items():
        if key == "nav":
            if not isinstance(value, dict) or not all(isinstance(v, str) for v in value.values()):
                problems.append("site.toml nav has to be a table of strings")
        elif key in SITE_KEYS and not isinstance(value, str):
            problems.append(f"site.toml {key} has to be a string")
    return problems


def page_problems(docs, code, page, catalogue):
    english = docs / page
    translated = docs / "translations" / code / page
    meta, body = bs.split_front_matter(bs.read_page(translated))
    problems = []
    if not SOURCE.fullmatch(meta.get("source", "")):
        problems.append("front matter needs source: followed by 40 hex digits, see `prep` and `stamp`")
    if set(meta) - {"source"}:
        problems.append("front matter holds only source")
    offset = bs.read_page(translated).count("\n") - body.count("\n")
    problems += check_page(bs.read_page(english), body, code, catalogue, page == "GLOSSARY.md", offset)
    return problems


def check(docs, locales, codes, pages=None):
    """The problems of the translations of `codes` as `code/page: message` lines. Without `pages` it looks at every
    current translation, a translation with no valid source also counts, since the build would take it for outdated."""
    problems = []
    for code in codes:
        problems += [f"{code}/site.toml: {m}" for m in check_site_strings(docs, code)]
        folder = docs / "translations" / code
        names = pages
        if names is None:
            names = [p for p in bs.english_pages(docs) if (folder / p).is_file()]
        try:
            catalogue = load_catalogue(locales, code)
        except OSError:
            problems.append(f"{code}: no editor catalogue {code}.json in {locales}")
            continue
        for page in names:
            if not (folder / page).is_file():
                problems.append(f"{code}/{page}: no such translation")
                continue
            if not (docs / page).is_file():
                problems.append(f"{code}/{page}: no such English page")
                continue
            meta, _ = bs.split_front_matter(bs.read_page(folder / page))
            if pages is None and SOURCE.fullmatch(meta.get("source", "")) and \
                    bs.page_state(docs, code, page) != "current":
                continue
            problems += [f"{code}/{page}: {m}" for m in page_problems(docs, code, page, catalogue)]
    return problems


def status(docs, codes, diffs=False, fetch=None):
    """A markdown report of every language, translations that fell behind first. `fetch(blob id)` gives an old
    English page for the diffs."""
    summary = bs.read_page(docs / "SUMMARY.md")
    english = sorted(bs.english_pages(docs))
    out = []
    for code in codes:
        states = {page: bs.page_state(docs, code, page) for page in english}
        folder = docs / "translations" / code
        orphans = sorted(p.relative_to(folder).as_posix() for p in folder.rglob("*.md")
                         if p.relative_to(folder).as_posix() == "SUMMARY.md"
                         or not (docs / p.relative_to(folder)).is_file())
        done = sum(1 for s in states.values() if s == "current")
        outdated = [p for p, s in states.items() if s == "outdated"]
        outdated.sort(key=lambda p: (p != "GLOSSARY.md", p))
        missing = [p for p, s in states.items() if s == "missing"]
        out.append(f"### {code}: {done} of {len(english)} pages current")
        out.append("")
        if outdated:
            out.append(f"Outdated, the English page changed since it was translated ({len(outdated)}):")
            out.append("")
            for page in outdated:
                out.append(f"* {page}" + (" (the glossary hover cards show English until it is updated)"
                                          if page == "GLOSSARY.md" else ""))
                if diffs:
                    out += ["", "```diff", *english_diff(docs, code, page, fetch), "```"]
            out.append("")
        if missing:
            out.append(f"Missing ({len(missing)}): " + ", ".join(missing))
            out.append("")
        if orphans:
            out.append(f"Orphans, translated pages whose English page is gone ({len(orphans)}): " + ", ".join(orphans))
            out.append("")
        nav = {}
        file = folder / "site.toml"
        if file.is_file():
            try:
                nav = tomllib.loads(file.read_text(encoding="utf-8")).get("nav", {})
            except tomllib.TOMLDecodeError:
                pass
        titles = bs.nav_titles(summary)
        untranslated = [t for t in dict.fromkeys(titles) if t not in nav]
        stale = [k for k in nav if k not in titles]
        if untranslated:
            out.append(f"Nav texts with no [nav] entry ({len(untranslated)}): " + ", ".join(untranslated))
            out.append("")
        if stale:
            out.append(f"[nav] keys no longer in SUMMARY.md ({len(stale)}): " + ", ".join(stale))
            out.append("")
        if not (outdated or missing or orphans or untranslated or stale):
            out.append("Nothing to do.")
            out.append("")
    return "\n".join(out).rstrip() + "\n"


def git_blob(blob):
    """The text of a blob of this repository, or None when the clone does not have it."""
    done = subprocess.run(["git", "cat-file", "blob", blob], cwd=bs.ROOT, capture_output=True)
    return done.stdout.decode("utf-8").replace("\r\n", "\n") if done.returncode == 0 else None


def english_diff(docs, code, page, fetch=None):
    """Unified diff lines of the English page from the version the translation follows to the current one."""
    meta, _ = bs.split_front_matter(bs.read_page(docs / "translations" / code / page))
    old = (fetch or git_blob)(meta.get("source", ""))
    if old is None:
        return ["the English page it followed is not in this clone"]
    new = bs.read_page(docs / page)
    return [line.rstrip("\n") for line in difflib.unified_diff(old.splitlines(), new.splitlines(), "stamped", "current",
                                                               lineterm="", n=1)]


def glossary_terms(docs, code, catalogue):
    """The glossary of the language by English heading text as (translation, forms). Terms the translated glossary
    lacks fall back to the catalogue word when the term is a UI name."""
    english = {h[3]: h[2] for h in headings(bs.read_page(docs / "GLOSSARY.md"))}
    found = {}
    mine = docs / "translations" / code / "GLOSSARY.md"
    translated = {h[3]: h for h in headings(bs.split_front_matter(bs.read_page(mine))[1])} if mine.is_file() else {}
    names = {c.lower(): v for c, v in catalogue.items()}
    for ident, term in english.items():
        if ident in translated:
            found[term] = (translated[ident][2], translated[ident][4])
        elif term.lower() in names:
            found[term] = (names[term.lower()][0], [])
    return found


def prep(docs, locales, code, page):
    """What to read before translating `page` into `code`."""
    english_text = bs.read_page(docs / page)
    catalogue = load_catalogue(locales, code)
    out = ["front matter to put at the top of the translation:", "", "---",
           f"source: {bs.blob_id((docs / page).read_bytes())}", "---", "", "heading ids, in order:", ""]
    for _, level, title, ident, _ in headings(english_text):
        out.append(f"{'#' * level} {title} {{ #{ident} }}")
    out += ["", "labels, written as the editor shows them in bold or italic:", ""]
    for parts in labels(english_text, catalogue):
        out.append(f"{' > '.join(parts)}  =>  " + "  or  ".join(label_options(parts, catalogue)))
    out += ["", "glossary terms of the page:", ""]
    lowered = english_text.lower()
    for term, (word, forms) in glossary_terms(docs, code, catalogue).items():
        if re.search(r"(?<![\w-])" + re.escape(term.lower()) + r"(?![\w-])", lowered):
            out.append(f"{term}  =>  {word}" + (f"  (forms: {', '.join(forms)})" if forms else ""))
    if bs.page_state(docs, code, page) == "outdated":
        out += ["", "the English page changed since the translation:", "", *english_diff(docs, code, page)]
    out += ["", "the English page:", "", english_text]
    return "\n".join(out)


def stamp(docs, code, pages):
    """Sets `source` in the front matter of each translation to the id of its English page and touches nothing else.
    Returns the pages changed."""
    changed = []
    for page in pages:
        file = docs / "translations" / code / page
        text = bs.read_page(file)
        line = f"source: {bs.blob_id((docs / page).read_bytes())}"
        lines = text.split("\n")
        meta, _ = bs.split_front_matter(text)
        if not meta:
            new = ["---", line, "---", *lines]
        else:
            end = next(i for i in range(1, len(lines)) if lines[i].rstrip() == "---")
            at = next((i for i in range(1, end) if lines[i].partition(":")[0].strip() == "source"), None)
            new = lines[:at] + [line] + lines[at + 1:] if at is not None else lines[:end] + [line] + lines[end:]
        if new != lines:
            file.write_bytes("\n".join(new).encode("utf-8"))
            changed.append(page)
    return changed


def known_languages():
    return [code for code, _, _ in bs.languages(bs.load_config())]


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    sub = parser.add_subparsers(dest="command", required=True)
    for name in ("check", "status", "prep", "stamp"):
        command = sub.add_parser(name)
        command.add_argument("--lang", help="only this language code, like de or zh-Hans")
        if name == "status":
            command.add_argument("--diff", action="store_true", help="show what changed in the outdated English pages")
        if name in ("check", "stamp"):
            command.add_argument("pages", nargs="*", help="docs paths like editor/brushes.md")
        if name == "prep":
            command.add_argument("page")
    args = parser.parse_args(argv)
    codes = known_languages()
    if args.lang and args.lang not in codes:
        parser.error(f"unknown language {args.lang}, the languages are {', '.join(codes)}")
    if args.command in ("prep", "stamp") and not args.lang:
        parser.error(f"{args.command} needs --lang")
    chosen = [args.lang] if args.lang else codes
    if args.command == "check":
        problems = check(DOCS, LOCALES, chosen, args.pages or None)
        print("\n".join(problems))
        print(f"{len(problems)} problems" if problems else "translations are fine")
        return 1 if problems else 0
    if args.command == "status":
        print(status(DOCS, chosen, args.diff))
        return 0
    page = args.page if args.command == "prep" else None
    pages = [page] if page else args.pages
    missing = [p for p in pages if not (DOCS / p).is_file() or
               (args.command == "stamp" and not (DOCS / "translations" / args.lang / p).is_file())]
    if missing or not pages:
        parser.error("no such page: " + ", ".join(missing or ["(none given)"]))
    if args.command == "prep":
        print(prep(DOCS, LOCALES, args.lang, page))
    else:
        for changed in stamp(DOCS, args.lang, pages):
            print(f"stamped {args.lang}/{changed}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
