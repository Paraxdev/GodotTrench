#!/usr/bin/env python3
"""Checks the built docs site for what `zensical build --strict` lets through: a mistyped page in SUMMARY.md, which
still renders as a dead nav link, an image or file that does not exist, a link to a missing heading, and a page that
is built but has no place in the navigation. Links to other sites are not checked.

  zensical build --clean --strict
  python3 tools/check_site.py   # exit 1 on any problem
"""
import os
import pathlib
import sys
import tomllib
from html.parser import HTMLParser
from urllib.parse import unquote, urlsplit

ROOT = pathlib.Path(__file__).resolve().parent.parent


class Page(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.ids = set()
        self.links = []
        self.nav = []

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if a.get("id"):
            self.ids.add(a["id"])
        if tag == "a" and a.get("name"):
            self.ids.add(a["name"])
        for key in ("href", "src", "poster"):
            if a.get(key):
                self.links.append(a[key])
        for part in (a.get("srcset") or "").split(","):
            if part.strip():
                self.links.append(part.split()[0])
        if tag == "a" and "md-nav__link" in (a.get("class") or "").split() and a.get("href"):
            self.nav.append(a["href"])


def main():
    config = tomllib.loads((ROOT / "zensical.toml").read_text(encoding="utf-8"))["project"]
    site = (ROOT / config.get("site_dir", "site")).resolve()
    base = urlsplit(config["site_url"]).path
    if not (site / "index.html").is_file():
        sys.exit(f"{site} has no index.html, build the site first")

    pages = {}
    for path in site.rglob("*.html"):
        page = Page()
        page.feed(path.read_text(encoding="utf-8"))
        pages[path.resolve()] = page

    def name(path):
        return pathlib.Path(os.path.relpath(path, site)).as_posix()

    # The file a link points at and its fragment, None for a link to another site. The 404 page links by absolute path,
    # which has to start with the site's base path.
    def target(path, url):
        parts = urlsplit(url)
        if parts.scheme or parts.netloc:
            return None, ""
        if not parts.path:
            return path, unquote(parts.fragment)
        if parts.path.startswith("/"):
            file = site / unquote(parts.path.removeprefix(base).lstrip("/"))
        else:
            file = path.parent / unquote(parts.path)
        file = file.resolve()
        if file.is_dir():
            file = file / "index.html"
        return file, unquote(parts.fragment)

    # Each broken target once, with the pages linking to it, since a dead nav entry is on every page.
    broken = {}
    for path, page in pages.items():
        for url in page.links:
            file, fragment = target(path, url)
            if file is None:
                continue
            if url.startswith("/") and not url.startswith(base):
                problem = f"{url} is outside {base}"
            elif not file.is_file():
                problem = f"{name(file)} does not exist"
            elif fragment and file in pages and fragment not in pages[file].ids:
                problem = f"{name(file)}#{fragment} does not exist"
            else:
                continue
            broken.setdefault(problem, set()).add(name(path))

    problems = []
    for problem, linked in sorted(broken.items()):
        first, *rest = sorted(linked)
        others = f" and {len(rest)} other page{'s' if len(rest) > 1 else ''}" if rest else ""
        problems.append(f"{problem}, linked from {first}{others}")

    index = site / "index.html"
    in_nav = {index} | {target(index, url)[0] for url in pages[index].nav}
    for path in sorted(set(pages) - in_nav):
        if name(path) != "404.html":
            problems.append(f"{name(path)} is not in the navigation, add its page to docs/SUMMARY.md")

    for problem in problems:
        print(problem)
    print(f"{len(pages)} pages checked, {len(problems)} problem{'' if len(problems) == 1 else 's'}")
    sys.exit(1 if problems else 0)


if __name__ == "__main__":
    main()
