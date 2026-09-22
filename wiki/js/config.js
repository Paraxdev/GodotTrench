// Central configuration for the canvas wiki.
// These constants point the Save feature at the GitHub repository.
// The site is served from the same repo under the wiki/ folder on GitHub Pages.

export const GH = {
  owner: "Paraxdev",
  repo: "GodotTrench",
  branch: "main",
  // Everything the wiki reads and writes lives under this path in the repo.
  basePath: "wiki/",
};

// localStorage keys. The token is only ever stored here and only ever sent to api.github.com.
export const TOKEN_KEY = "wiki_gh_token";
export const DRAFT_PREFIX = "wiki_draft_";
export const INDEX_DRAFT_KEY = "wiki_index_draft";

// The GitHub REST API base. Requests only ever go here.
export const API_BASE = "https://api.github.com";
