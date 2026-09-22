// GitHub REST Contents API integration.
// This module is the only place that talks to the network for writing.
// The token is read from localStorage and only ever sent to https://api.github.com.
// The token is never logged.

import { GH, API_BASE, TOKEN_KEY } from "./config.js";

export function getToken() {
  try {
    return localStorage.getItem(TOKEN_KEY) || "";
  } catch (err) {
    return "";
  }
}

export function setToken(token) {
  try {
    localStorage.setItem(TOKEN_KEY, token);
  } catch (err) {
    // Ignore, the token then only lives for this session in memory via callers.
  }
}

export function clearToken() {
  try {
    localStorage.removeItem(TOKEN_KEY);
  } catch (err) {
    // Ignore.
  }
}

export function hasToken() {
  return getToken().length > 0;
}

function headers(token) {
  return {
    Authorization: "Bearer " + token,
    Accept: "application/vnd.github+json",
    "X-GitHub-Api-Version": "2022-11-28",
  };
}

// Encode a UTF-8 string as base64 in a way that handles non-ASCII characters.
export function base64FromString(str) {
  const bytes = new TextEncoder().encode(str);
  return base64FromBytes(bytes);
}

// Encode raw bytes as base64. Chunked so large files do not overflow the call stack.
export function base64FromBytes(bytes) {
  let binary = "";
  const chunk = 0x8000;
  for (let i = 0; i < bytes.length; i += chunk) {
    binary += String.fromCharCode.apply(null, bytes.subarray(i, i + chunk));
  }
  return btoa(binary);
}

// Read a File or Blob and return its base64 content, without the data URL prefix.
export function base64FromFile(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const bytes = new Uint8Array(reader.result);
      resolve(base64FromBytes(bytes));
    };
    reader.onerror = () => reject(new Error("Could not read the selected file."));
    reader.readAsArrayBuffer(file);
  });
}

// Turn an API failure into a readable message.
async function describeError(res) {
  let detail = "";
  try {
    const body = await res.json();
    if (body && body.message) detail = body.message;
  } catch (err) {
    // No JSON body.
  }
  if (res.status === 401) {
    return "Unauthorized (401). The token is missing, expired, or lacks Contents write access.";
  }
  if (res.status === 403) {
    const remaining = res.headers.get("x-ratelimit-remaining");
    if (remaining === "0") {
      return "Rate limited (403). GitHub API rate limit reached, please wait and try again.";
    }
    return "Forbidden (403). " + (detail || "The token may lack write access to this repository.");
  }
  if (res.status === 404) {
    return "Not found (404). Check the owner, repo, and that the token can see this repository.";
  }
  if (res.status === 409) {
    return "Conflict (409). The file changed on the server, reload and try again.";
  }
  if (res.status === 422) {
    return "Unprocessable (422). " + (detail || "The request was rejected by GitHub.");
  }
  return "GitHub API error (" + res.status + "). " + detail;
}

// Full repo path for a path that is relative to the wiki root (for example "pages/home.json").
function repoPath(relPath) {
  return GH.basePath + relPath;
}

function contentsUrl(relPath) {
  return API_BASE + "/repos/" + GH.owner + "/" + GH.repo + "/contents/" + encodeURI(repoPath(relPath));
}

// GET the current file to obtain its sha. Returns null when the file does not exist yet.
export async function getFileSha(relPath, token) {
  const res = await fetch(contentsUrl(relPath) + "?ref=" + encodeURIComponent(GH.branch), {
    headers: headers(token),
    cache: "no-store",
  });
  if (res.status === 404) return null;
  if (!res.ok) throw new Error(await describeError(res));
  const body = await res.json();
  return body.sha || null;
}

// PUT a file. Passes sha only when updating an existing file, so a missing file is created.
export async function putFile(relPath, contentBase64, message, token) {
  const sha = await getFileSha(relPath, token);
  const payload = {
    message,
    content: contentBase64,
    branch: GH.branch,
  };
  if (sha) payload.sha = sha;

  const res = await fetch(contentsUrl(relPath), {
    method: "PUT",
    headers: headers(token),
    body: JSON.stringify(payload),
  });
  if (!res.ok) throw new Error(await describeError(res));
  return res.json();
}

async function api(path, token, init) {
  const opts = init || {};
  const res = await fetch(API_BASE + "/repos/" + GH.owner + "/" + GH.repo + path, {
    method: opts.method || "GET",
    headers: headers(token),
    body: opts.body ? JSON.stringify(opts.body) : undefined,
    cache: "no-store",
  });
  if (!res.ok) {
    const err = new Error(await describeError(res));
    err.status = res.status;
    throw err;
  }
  return res.status === 204 ? null : res.json();
}

// File names in a wiki folder on the branch, so deletions only name files that exist.
async function listNames(relDir, token) {
  try {
    const items = await api("/contents/" + encodeURI(repoPath(relDir)) + "?ref=" + encodeURIComponent(GH.branch), token);
    return new Set((Array.isArray(items) ? items : []).map((i) => i.name));
  } catch (err) {
    if (err.status === 404) return new Set();
    throw err;
  }
}

// Write several text files and delete others in a single commit, through the Git Data API:
// read the branch head, build a tree on top of it, commit, and fast forward the branch. If the
// branch moved in the meantime the ref update is rejected, and the whole thing is retried once
// on top of the new head.
//   files: [{ path, content }] with paths relative to the wiki root
//   deletes: [path]
export async function commitFiles(files, deletes, message, token) {
  let removable = [];
  if (deletes.length) {
    const byDir = new Map();
    for (const d of deletes) {
      const dir = d.split("/").slice(0, -1).join("/");
      if (!byDir.has(dir)) byDir.set(dir, await listNames(dir, token));
    }
    removable = deletes.filter((d) => byDir.get(d.split("/").slice(0, -1).join("/")).has(d.split("/").pop()));
  }
  for (let attempt = 0; attempt < 2; attempt += 1) {
    const ref = await api("/git/ref/heads/" + encodeURIComponent(GH.branch), token);
    const headSha = ref.object.sha;
    const head = await api("/git/commits/" + headSha, token);
    const entries = files
      .map((f) => ({ path: repoPath(f.path), mode: "100644", type: "blob", content: f.content }))
      .concat(removable.map((d) => ({ path: repoPath(d), mode: "100644", type: "blob", sha: null })));
    const tree = await api("/git/trees", token, { method: "POST", body: { base_tree: head.tree.sha, tree: entries } });
    if (tree.sha === head.tree.sha) return { unchanged: true };
    const commit = await api("/git/commits", token, { method: "POST", body: { message, tree: tree.sha, parents: [headSha] } });
    try {
      await api("/git/refs/heads/" + encodeURIComponent(GH.branch), token, { method: "PATCH", body: { sha: commit.sha } });
      return { sha: commit.sha };
    } catch (err) {
      if (err.status !== 422 || attempt === 1) throw err;
    }
  }
  return { unchanged: true };
}

// Upload a media file to wiki/assets and return the site-root relative path for a card src.
export async function uploadAsset(file, token) {
  // Pasted clipboard images are all called image.png, so every upload gets a unique suffix.
  const clean = sanitizeFilename(file.name || "image.png");
  const dot = clean.lastIndexOf(".");
  const stamp = "-" + Date.now().toString(36);
  const safeName = dot > 0 ? clean.slice(0, dot) + stamp + clean.slice(dot) : clean + stamp;
  const relPath = "assets/" + safeName;
  const contentBase64 = await base64FromFile(file);
  await putFile(relPath, contentBase64, "wiki: upload asset " + safeName, token);
  // Card src is stored relative to the site root, which is the wiki/ folder on Pages.
  return relPath;
}

function sanitizeFilename(name) {
  const cleaned = String(name)
    .replace(/[^\w.\-]+/g, "_")
    .replace(/^_+|_+$/g, "");
  return cleaned || "file";
}
