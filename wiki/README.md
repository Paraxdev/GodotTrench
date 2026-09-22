# Canvas Wiki

A static, dependency light canvas wiki for the GodotTrench repository. Each page is a board
where cards (text, code, image, video, table, shape and freehand draw) are placed freely,
dragged, resized and layered. You edit on the live site and save changes back to the repo.

The site is served from this `wiki/` folder by GitHub Pages. It is a personal wiki for the
repo owner, but note that the deployed site is public.

## Enabling GitHub Pages

1. Push this `wiki/` folder and the workflow at `.github/workflows/pages.yml` to `main`.
2. In the repository, open Settings, then Pages.
3. Under Build and deployment, set Source to GitHub Actions.
4. The `Deploy wiki to GitHub Pages` workflow publishes the `wiki/` folder on every push to
   `main` that touches `wiki/**`. You can also run it manually from the Actions tab.
5. The site appears at `https://paraxdev.github.io/GodotTrench/`.

## Editing and saving

1. Open the site and press Edit in the toolbar.
2. Use the add buttons to place cards, drag them by their header, resize from the bottom right
   corner, and select a card to edit its content in the inspector on the right.
3. Press Save to commit the current page and the page index back to the repo.

Edits are also autosaved to your browser localStorage per page, so unsaved work survives a
reload. A GitHub Save clears that draft for the page once the commit succeeds.

## Minting a Personal Access Token

Saving writes to the repo through the GitHub REST Contents API, which needs a token.

1. Go to GitHub, then Settings, then Developer settings, then Personal access tokens, then
   Fine grained tokens, then Generate new token.
2. Resource owner: Paraxdev. Repository access: Only select repositories, and choose
   Paraxdev/GodotTrench.
3. Permissions: under Repository permissions set Contents to Read and write.
4. Generate the token and copy it.
5. On the site, press Save (or the GitHub token button). Paste the token into the Save dialog.

### Where the token lives

The token is stored only in your browser localStorage under the key `wiki_gh_token`, and is
only ever sent to `https://api.github.com`. It is never committed, never logged, and never sent
anywhere else. You can clear it any time from the token dialog. Because the deployed site is
public, only paste a token on a device and browser you trust, and prefer a fine grained token
scoped to this single repository with only Contents write access.

## How saving works

- The current page is written to `wiki/pages/<slug>.json`.
- The page list is written to `wiki/pages/index.json`.
- Uploaded images, gifs and videos are written to `wiki/assets/<filename>`, and the card `src`
  is set to a path relative to the site root, for example `assets/<filename>`.
- Each write uses `PUT /repos/Paraxdev/GodotTrench/contents/<path>` with base64 content, a
  commit message, the branch, and the existing file `sha` when updating (fetched first with a
  GET, and a 404 is treated as a new file).

## Layout

```
wiki/
  index.html            app shell
  css/style.css         dark theme
  js/
    app.js              entry, sidebar, toolbar, save flow
    board.js            canvas, drag, resize, select, z, draw
    cards.js            card model and per type renderers
    inspector.js        selected card editor
    storage.js          localStorage drafts and page loading
    github.js           GitHub Contents API integration
    config.js           owner, repo, branch, base path
    highlight-gdscript.js  small GDScript grammar for highlight.js
    vendor/             pinned local fallbacks for the CDN libraries
  pages/
    index.json          page list
    home.json           seed page
    godottrench.json    seed page
  assets/               uploaded media committed here
```

## Libraries

Loaded from cdnjs with pinned versions, with a local fallback under `js/vendor/` if the CDN is
unavailable:

- marked 12.0.2 (Markdown)
- DOMPurify 3.1.7 (HTML sanitising)
- highlight.js 11.10.0 with the github-dark theme (code highlighting)

There is no build step and no bundler.
