# GodotTrench Wiki

The documentation site for GodotTrench, published from this folder by GitHub Pages. Any commit that changes this
folder redeploys the site.

Pages live in `pages/` as JSON, one file per page plus `index.json` for the sidebar. A page is a title, a layout
(`doc` for a column of blocks, `canvas` for free cards) and a list of cards. Text cards store Markdown, with `[[slug]]`
page links and `> [!NOTE]` callouts (also TIP, WARNING, DANGER and EXAMPLE), so pages are easy to edit by hand.

To preview locally, serve this folder with any static server, for example `python -m http.server` from inside
`wiki/`, and open `http://localhost:8000`.

## Libraries

Loaded from cdnjs with pinned versions, with a local fallback under `js/vendor/` if the CDN is
unavailable:

- marked 12.0.2 (Markdown)
- DOMPurify 3.1.7 (HTML sanitising)
- highlight.js 11.10.0 with the github-dark theme (code highlighting), a trimmed self-hosted build
