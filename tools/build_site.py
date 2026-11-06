#!/usr/bin/env python3
"""Builds the docs site in every language: English at the site root, each other language in a folder of its own.

Zensical has no i18n support, so this runs one build per language. A translated page is used only while it is current,
that is while the `source` blob id in its front matter equals the id of the English page. Every other page is the
English text with a short note after its title. See docs/development.md#documentation.

  python3 tools/build_site.py          # English, then every language, into _site
  python3 tools/build_site.py de ja    # only these languages, English is built when _site is missing
"""
import copy
import hashlib
import io
import json
import pathlib
import re
import shutil
import subprocess
import sys
import tarfile
import tomllib
from urllib.parse import unquote, urlsplit

ROOT = pathlib.Path(__file__).resolve().parent.parent
IMAGE_SUFFIXES = {".png", ".jpg", ".jpeg", ".gif", ".webp", ".svg", ".avif"}
# Folders of docs/ that are not pages or their images.
NOT_PAGES = {"translations", "overrides", "javascripts", "stylesheets"}
# A language site reuses the page images of the root, so it is mostly text, about 1 MB compressed. Over this it has
# kept a copy of the images.
LANGUAGE_SIZE_LIMIT = 6 * 1000 * 1000

DEFAULTS = {
    "note": "Note",
    "tip": "Tip",
    "warning": "Warning",
    "fallback": "This page is not translated into this language yet, or the English page changed since it was. "
    "You are reading the English version.",
    "notice": "This translation was written by an AI and no native speaker has checked it yet. If something reads "
    "oddly, the English page is the reference, and corrections are welcome on GitHub.",
}
# The scheme of each palette entry and the site.toml key holding the name of the button that switches away from it.
TOGGLES = {"default": "light_toggle", "slate": "dark_toggle"}

ALERT = re.compile(r"^(?P<quote>[ \t]*(?:>[ \t]?)+)\[!(?P<kind>NOTE|TIP|WARNING)\][ \t]*$")
FENCE = re.compile(r"(?P<mark>`{3,}|~{3,})(?P<rest>.*)$")
TAG = re.compile(r"<(?:img|source|video|a)\b(?:[^>\"']|\"[^\"]*\"|'[^']*')*>", re.IGNORECASE)
ATTRIBUTE = re.compile(r"""(?P<name>\b(?:src|href|poster|srcset))(?P<eq>\s*=\s*)(?P<quote>["'])(?P<value>.*?)(?P=quote)""",
                       re.IGNORECASE | re.DOTALL)


def load_config():
    return tomllib.loads((ROOT / "zensical.toml").read_text(encoding="utf-8"))


def base_path(config):
    return urlsplit(config["project"]["site_url"]).path


def languages(config):
    """The languages besides English as (code, name, hreflang), in the order of extra.alternate. The code is the
    folder of the language's link below the site's base path."""
    base = base_path(config)
    found = []
    for alt in config["project"]["extra"]["alternate"]:
        code = alt["link"].removeprefix(base).strip("/")
        if code:
            found.append((code, alt["name"], alt["lang"]))
    return found


def read_page(path):
    return pathlib.Path(path).read_bytes().decode("utf-8").replace("\r\n", "\n")


def blob_id(data):
    """What `git hash-object` prints for the page. The repository stores LF, so a checkout with CRLF gives the same id."""
    data = data.replace(b"\r\n", b"\n")
    return hashlib.sha1(b"blob %d\0" % len(data) + data).hexdigest()


def split_front_matter(text):
    """The `key: value` lines of a leading `---` block and the text after it. A page with no block, or one that is
    never closed, has no front matter and keeps all its text."""
    lines = text.split("\n")
    if lines[0].rstrip() != "---":
        return {}, text
    for end in range(1, len(lines)):
        if lines[end].rstrip() == "---":
            meta = {}
            for line in lines[1:end]:
                key, colon, value = line.partition(":")
                if colon:
                    meta[key.strip()] = value.strip()
            return meta, "\n".join(lines[end + 1:])
    return {}, text


def page_state(docs, code, page):
    """`current` when the translation follows the English page as it is now, `outdated` when it follows another
    version or has no usable front matter, `missing` when there is no translation, `orphan` when the English page is
    gone. The build only ever uses a current page."""
    english = docs / page
    translated = docs / "translations" / code / page
    if not english.is_file():
        return "orphan" if translated.is_file() else "missing"
    if not translated.is_file():
        return "missing"
    meta, _ = split_front_matter(read_page(translated))
    return "current" if meta.get("source") == blob_id(english.read_bytes()) else "outdated"


def fenced(lines):
    """For every line whether it is inside a fenced code block, the fence lines included. Fences may be indented
    inside a list or quoted."""
    inside = []
    fence = None
    for line in lines:
        match = FENCE.match(line.lstrip(" \t>"))
        if fence is None:
            opens = match is not None and not (match["mark"][0] == "`" and "`" in match["rest"])
            if opens:
                fence = (match["mark"][0], len(match["mark"]))
            inside.append(opens)
        else:
            inside.append(True)
            if match and match["mark"][0] == fence[0] and len(match["mark"]) >= fence[1] and not match["rest"].strip():
                fence = None
    return inside


def with_note(text, title, note):
    """The page with a NOTE alert after its first h1, or at the top when it has none. The rest stays as it is."""
    lines = text.split("\n")
    alert = [f"> [!NOTE] {title}", f"> {note}"]
    skip = fenced(lines)
    for i, line in enumerate(lines):
        if line.startswith("# ") and not skip[i]:
            rest = lines[i + 1:]
            gap = [""] if rest and rest[0].strip() else []
            return "\n".join(lines[:i + 1] + [""] + alert + gap + rest)
    return "\n".join(alert + [""] + lines)


def localise_alerts(text, titles):
    """`> [!NOTE]` alone on its line becomes `> [!NOTE] Hinweis`. A marker inside a code fence or with a title
    already is left alone. `titles` maps NOTE, TIP and WARNING to the words."""
    lines = text.split("\n")
    skip = fenced(lines)
    for i, line in enumerate(lines):
        match = None if skip[i] else ALERT.match(line)
        if match:
            lines[i] = f"{line.rstrip()} {titles[match['kind']]}"
    return "\n".join(lines)


def translate_nav(summary, nav):
    """SUMMARY.md with the text of every link and section label looked up in `nav`, the English text being the key.
    A text with no entry stays English, link targets are never touched."""
    link = re.compile(r"^(?P<head>\s*[*+-]\s+)\[(?P<text>.*)\]\((?P<url>[^)]*)\)(?P<tail>\s*)$")
    section = re.compile(r"^(?P<head>\s*[*+-]\s+)(?P<text>[^\[\s].*?)(?P<tail>\s*)$")
    out = []
    for line in summary.split("\n"):
        if match := link.match(line):
            text = nav.get(match["text"], match["text"])
            line = f"{match['head']}[{text}]({match['url']}){match['tail']}"
        elif match := section.match(line):
            line = f"{match['head']}{nav.get(match['text'], match['text'])}{match['tail']}"
        out.append(line)
    return "\n".join(out)


def toml_key(key):
    return key if re.fullmatch(r"[A-Za-z0-9_-]+", key) else json.dumps(key, ensure_ascii=False)


def toml_value(value):
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (int, float)):
        return repr(value)
    if isinstance(value, str):
        return json.dumps(value, ensure_ascii=False)
    if isinstance(value, list):
        return "[" + ", ".join(toml_value(v) for v in value) + "]"
    if isinstance(value, dict):
        return "{ " + ", ".join(f"{toml_key(k)} = {toml_value(v)}" for k, v in value.items()) + " }" if value else "{}"
    raise TypeError(f"cannot write {value!r} as TOML")


def toml_dump(config):
    """TOML text for a dict of tables. Everything below a table header is written as inline tables and arrays, so
    nested tables and arrays of tables need no headers of their own."""
    out = []
    for name, table in config.items():
        out.append(f"[{toml_key(name)}]")
        out.extend(f"{toml_key(k)} = {toml_value(v)}" for k, v in table.items())
        out.append("")
    return "\n".join(out)


def language_config(config, code, strings):
    """The config of one language's build, which runs in `.site-build/<code>` on a copy of docs/."""
    config = copy.deepcopy(config)
    project = config["project"]
    site_url = project["site_url"].rstrip("/") + "/"
    project["site_url"] = f"{site_url}{code}/"
    project["docs_dir"] = "docs"
    project["site_dir"] = "site"
    project.pop("edit_uri", None)
    # The social image lives in the English root, a language site does not keep a copy of the page images.
    extra = project.setdefault("extra", {})
    extra["root_url"] = site_url
    strings = {**DEFAULTS, **strings}
    project["site_description"] = strings.get("description", project.get("site_description"))
    extra["mcp_switch"] = strings.get("mcp_switch", extra.get("mcp_switch"))
    extra["notice"] = strings["notice"]
    theme = project["theme"]
    for palette in theme.get("palette", []):
        name = strings.get(TOGGLES.get(palette.get("scheme")))
        if name:
            palette["toggle"]["name"] = name
    theme["custom_dir"] = "docs/overrides"
    theme["language"] = strings.get("theme_language", code)
    theme["features"] = [f for f in theme.get("features", []) if f != "content.action.edit"]
    return config


def load_strings(docs, code):
    file = docs / "translations" / code / "site.toml"
    strings = dict(DEFAULTS)
    if file.is_file():
        strings.update(tomllib.loads(file.read_text(encoding="utf-8")))
    return strings


def english_pages(docs):
    """Paths of the English pages in docs/, posix style, relative to it, SUMMARY.md left out."""
    pages = []
    for path in sorted(docs.rglob("*.md")):
        rel = path.relative_to(docs)
        if rel.parts[0] != "translations" and rel.as_posix() != "SUMMARY.md":
            pages.append(rel.as_posix())
    return pages


def render_pages(docs, code, strings):
    """The text of every page of the language's site by path, and how many of them are translated. Only pages with an
    English counterpart are built, an orphan translation is ignored."""
    titles = {kind: strings[kind.lower()] for kind in ("NOTE", "TIP", "WARNING")}
    pages = {}
    translated = 0
    for page in english_pages(docs):
        if page_state(docs, code, page) == "current":
            _, body = split_front_matter(read_page(docs / "translations" / code / page))
            pages[page] = localise_alerts(body, titles)
            translated += 1
        else:
            pages[page] = with_note(read_page(docs / page), titles["NOTE"], strings["fallback"])
    pages["SUMMARY.md"] = translate_nav(read_page(docs / "SUMMARY.md"), strings.get("nav", {}))
    return pages, translated


def image_folders(docs):
    """The folders of docs/ that hold page images, as paths relative to it. The images of `assets/` itself (the logo)
    stay in every language, and nested folders are covered by the folder that holds them."""
    holders = set()
    for path in docs.rglob("*"):
        rel = path.relative_to(docs)
        if rel.parts[0] in NOT_PAGES or not path.is_file() or path.suffix.lower() not in IMAGE_SUFFIXES:
            continue
        if rel.parent.as_posix() not in (".", "assets"):
            holders.add(rel.parent.as_posix())
    return sorted(h for h in holders if not any(h.startswith(other + "/") for other in holders))


def point_up(url, page_dir, folders, base):
    """The URL one level up when it points into an image folder, else unchanged. `page_dir` is the folder of the page
    inside the language site, as parts. `base` is the language site's own path for absolute URLs, `/TrenchDot/de/`."""
    parts = urlsplit(url)
    if parts.scheme or parts.netloc or not parts.path:
        return url
    if parts.path.startswith("/"):
        if not parts.path.startswith(base):
            return url
        target = unquote(parts.path.removeprefix(base))
        if any(target.startswith(f + "/") for f in folders):
            parent = base.rstrip("/").rpartition("/")[0] + "/"
            return parent + url[len(base):]
        return url
    stack = list(page_dir)
    for segment in unquote(parts.path).split("/"):
        if segment == "..":
            if not stack:
                return url
            stack.pop()
        elif segment != ".":
            stack.append(segment)
    if any("/".join(stack).startswith(f + "/") for f in folders):
        return "../" + url
    return url


def rewrite_images(site, folders, base):
    """Points the image links of a language site at the root copy and deletes its own copy of the image folders.
    Only attribute values of real <img>, <source>, <video> and <a> tags are touched, so a code sample that shows
    one stays as written. Returns how many attribute values changed."""
    changed = 0

    def attributes(page_dir):
        def tag(match):
            def attribute(found):
                nonlocal changed
                value = found["value"]
                if found["name"].lower() == "srcset":
                    items = []
                    for item in value.split(","):
                        url, space, descriptor = item.strip().partition(" ")
                        items.append(point_up(url, page_dir, folders, base) + space + descriptor)
                    new = ", ".join(items)
                else:
                    new = point_up(value, page_dir, folders, base)
                changed += new != value
                return f"{found['name']}{found['eq']}{found['quote']}{new}{found['quote']}"

            return ATTRIBUTE.sub(attribute, match[0])

        return tag

    for path in site.rglob("*.html"):
        page_dir = path.relative_to(site).parent.parts
        html = path.read_text(encoding="utf-8")
        new = TAG.sub(attributes(page_dir), html)
        if new != html:
            path.write_bytes(new.encode("utf-8"))
    for folder in folders:
        shutil.rmtree(site / folder, ignore_errors=True)
    return changed


def zensical(config_file, cwd):
    subprocess.run([sys.executable, "-m", "zensical", "build", "-f", str(config_file), "--clean", "--strict"],
                   cwd=cwd, check=True)


class Counter(io.RawIOBase):
    size = 0

    def writable(self):
        return True

    def write(self, data):
        self.size += len(data)
        return len(data)


def compressed_size(path):
    """Bytes of the folder as a .tar.gz, counted without writing it."""
    counter = Counter()
    with tarfile.open(fileobj=counter, mode="w|gz") as archive:
        archive.add(path, arcname=path.name)
    return counter.size


def build_language(config, docs, code, out):
    strings = load_strings(docs, code)
    work = ROOT / ".site-build" / code
    shutil.rmtree(work, ignore_errors=True)
    shutil.copytree(docs, work / "docs", ignore=lambda folder, names: [
        n for n in names if n.endswith(".md") or (pathlib.Path(folder) == docs and n == "translations")])
    pages, translated = render_pages(docs, code, strings)
    for page, text in pages.items():
        target = work / "docs" / page
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(text.encode("utf-8"))
    (work / "zensical.toml").write_bytes(toml_dump(language_config(config, code, strings)).encode("utf-8"))
    zensical(work / "zensical.toml", work)
    target = out / code
    shutil.rmtree(target, ignore_errors=True)
    shutil.copytree(work / "site", target)
    rewrite_images(target, image_folders(docs), base_path(config) + code + "/")
    size = compressed_size(target)
    print(f"{code}: {translated} of {len(pages) - 1} pages translated, {size / 1e6:.1f} MB compressed")
    if size > LANGUAGE_SIZE_LIMIT:
        sys.exit(f"The {code} site is over {LANGUAGE_SIZE_LIMIT // 1000000} MB compressed, a language site is meant to "
                 "reuse the images of the root, see rewrite_images")


def main(codes):
    config = load_config()
    known = [code for code, _, _ in languages(config)]
    for code in codes:
        if code not in known:
            sys.exit(f"{code} is not a language in zensical.toml, the languages are {', '.join(known)}")
    docs = ROOT / config["project"]["docs_dir"]
    out = ROOT / config["project"].get("site_dir", "site")
    if not codes or not (out / "index.html").is_file():
        shutil.rmtree(out, ignore_errors=True)
        zensical(ROOT / "zensical.toml", ROOT)
    for code in codes or known:
        build_language(config, docs, code, out)
    print(f"{out.name} is {compressed_size(out) / 1e6:.1f} MB compressed")


if __name__ == "__main__":
    main(sys.argv[1:])
