# GodotTrench Wiki

The documentation site for GodotTrench, published from this folder by GitHub Pages. Any commit that changes this
folder redeploys the site.

Pages live in `pages/` as JSON, one file per page plus `index.json` for the sidebar. A page is a title, a layout
(`doc` for a column of blocks, `canvas` for free cards) and a list of cards. Text cards store Markdown, with `[[slug]]`
page links and `> [!NOTE]` callouts (also TIP, WARNING, DANGER and EXAMPLE), so pages are easy to edit by hand. When editing JSON by hand,
note that the wiki renders with marked option `breaks: true`, so a single newline in page text becomes a hard line break.

To preview locally, serve this folder with any static server, for example `python -m http.server` from inside
`wiki/`, and open `http://localhost:8000`.

## Libraries

- marked 12.0.2 (Markdown): Loaded from cdnjs, with a local fallback under `js/vendor/` if unavailable
- DOMPurify 3.1.7 (HTML sanitising): Loaded from cdnjs, with a local fallback
- highlight.js 11.10.0 (code highlighting): A trimmed self-hosted build in `js/vendor/`, plus the github-dark theme CSS from cdnjs

The wiki also includes `js/config.js` (GitHub API configuration) and `js/highlight-gdscript.js` (GDScript syntax highlighting registration).
