# GodotTrench Wiki

The documentation site for GodotTrench, published from this folder by GitHub Pages. Pages are
edited in the browser, either as documents (a column of blocks) or canvases (free cards), and
Save commits them back to this repo. The site itself explains how: see the "Using this wiki" and
"How this wiki works" pages.

Pages live in `pages/` as JSON, one file per page plus `index.json` for the sidebar. Text is
stored as Markdown, with `[[slug]]` page links and `> [!NOTE]` callouts, so the files can also be
edited by hand.

To run it locally, serve this folder with any static server, for example
`python -m http.server` from inside `wiki/`, and open `http://localhost:8000`.

## Libraries

Loaded from cdnjs with pinned versions, with a local fallback under `js/vendor/` if the CDN is
unavailable:

- marked 12.0.2 (Markdown)
- DOMPurify 3.1.7 (HTML sanitising)
- highlight.js 11.10.0 with the github-dark theme (code highlighting), a trimmed self-hosted build
