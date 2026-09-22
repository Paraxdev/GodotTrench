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

// Commit a single page board and the page index in one Save action.
export async function commitPage(slug, board, pages, token) {
  const pageJson = JSON.stringify(board, null, 2) + "\n";
  const indexJson = JSON.stringify(pages, null, 2) + "\n";

  await putFile(
    "pages/" + slug + ".json",
    base64FromString(pageJson),
    "wiki: update page " + slug,
    token
  );
  await putFile(
    "pages/index.json",
    base64FromString(indexJson),
    "wiki: update page index",
    token
  );
  return true;
}

// Upload a media file to wiki/assets and return the site-root relative path for a card src.
export async function uploadAsset(file, token) {
  const safeName = sanitizeFilename(file.name);
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
