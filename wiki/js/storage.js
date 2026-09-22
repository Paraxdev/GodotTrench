// Local persistence and page loading.
// Drafts are autosaved to localStorage so edits survive reloads even before a GitHub Save.
// Every localStorage access is wrapped in try/catch because storage can be unavailable
// (private windows, disabled site data) and must never break the app.

import { DRAFT_PREFIX, INDEX_DRAFT_KEY } from "./config.js";

export function loadDraft(slug) {
  try {
    const raw = localStorage.getItem(DRAFT_PREFIX + slug);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (!parsed || !parsed.board) return null;
    return parsed; // { savedAt, board, base }
  } catch (err) {
    return null;
  }
}

// base is a hash of the repo version the draft started from, so a draft that is older than
// what was published since can be recognised instead of silently winning.
export function saveDraft(slug, board, base) {
  try {
    const payload = JSON.stringify({ savedAt: Date.now(), board, base: base || null });
    localStorage.setItem(DRAFT_PREFIX + slug, payload);
  } catch (err) {
    // Storage may be full or blocked. The app keeps working in memory.
  }
}

export function clearDraft(slug) {
  try {
    localStorage.removeItem(DRAFT_PREFIX + slug);
  } catch (err) {
    // Ignore.
  }
}

export function loadIndexDraft() {
  try {
    const raw = localStorage.getItem(INDEX_DRAFT_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw);
    if (!parsed || !Array.isArray(parsed.pages)) return null;
    return parsed; // { savedAt, pages, deleted }
  } catch (err) {
    return null;
  }
}

export function saveIndexDraft(pages, deleted) {
  try {
    localStorage.setItem(INDEX_DRAFT_KEY, JSON.stringify({ savedAt: Date.now(), pages, deleted: deleted || [] }));
  } catch (err) {
    // Ignore.
  }
}

export function clearIndexDraft() {
  try {
    localStorage.removeItem(INDEX_DRAFT_KEY);
  } catch (err) {
    // Ignore.
  }
}

// Per-viewer pan/zoom for a page. Kept separate from the page JSON so it never affects
// published content and never reaches other viewers.
export function saveView(slug, view) {
  try {
    localStorage.setItem("wiki_view_" + slug, JSON.stringify(view));
  } catch (err) {
    // Ignore.
  }
}

export function loadView(slug) {
  try {
    const raw = localStorage.getItem("wiki_view_" + slug);
    if (!raw) return null;
    const v = JSON.parse(raw);
    if (v && typeof v.scale === "number") return v;
    return null;
  } catch (err) {
    return null;
  }
}

export function listDraftSlugs() {
  const slugs = [];
  try {
    for (let i = 0; i < localStorage.length; i += 1) {
      const k = localStorage.key(i);
      if (k && k.indexOf(DRAFT_PREFIX) === 0) slugs.push(k.slice(DRAFT_PREFIX.length));
    }
  } catch (err) {
    // Storage unavailable, so there are no drafts.
  }
  return slugs;
}

// Published but possibly not deployed yet. GitHub Pages takes a minute to redeploy, and during
// that time the site still serves the old files. Until the served copy matches, the saved version
// is shown from here. Entries expire so a failed deploy cannot pin stale content forever.
const PENDING_PREFIX = "wiki_pending_";
const PENDING_TTL = 15 * 60 * 1000;

export function savePending(key, data, hash) {
  try {
    localStorage.setItem(PENDING_PREFIX + key, JSON.stringify({ at: Date.now(), data, hash }));
  } catch (err) {
    // Ignore, the only cost is a stale view until Pages redeploys.
  }
}

export function loadPending(key) {
  try {
    const raw = localStorage.getItem(PENDING_PREFIX + key);
    if (!raw) return null;
    const p = JSON.parse(raw);
    if (!p || Date.now() - p.at > PENDING_TTL) {
      localStorage.removeItem(PENDING_PREFIX + key);
      return null;
    }
    return p;
  } catch (err) {
    return null;
  }
}

export function clearPending(key) {
  try {
    localStorage.removeItem(PENDING_PREFIX + key);
  } catch (err) {
    // Ignore.
  }
}

// FNV-1a, enough to tell two versions of a page apart.
export function hashString(str) {
  let h = 0x811c9dc5;
  for (let i = 0; i < str.length; i += 1) {
    h ^= str.charCodeAt(i);
    h = Math.imul(h, 0x01000193);
  }
  return (h >>> 0).toString(16);
}

// Remove every local draft (all page boards and the index), so the next load comes entirely
// from the repo. Used by the sidebar Refresh action.
export function clearAllDrafts() {
  try {
    const keys = [];
    for (let i = 0; i < localStorage.length; i += 1) {
      const k = localStorage.key(i);
      if (k && (k.indexOf(DRAFT_PREFIX) === 0 || k === INDEX_DRAFT_KEY || k.indexOf(PENDING_PREFIX) === 0)) keys.push(k);
    }
    keys.forEach((k) => localStorage.removeItem(k));
  } catch (err) {
    // Ignore.
  }
}

// Fetch a JSON file relative to the site root. Works on GitHub Pages and on a local server.
// Returns null on 404 so callers can start an empty board or an empty index.
export async function fetchJSON(path) {
  const res = await fetch(path, { cache: "no-store" });
  if (res.status === 404) return null;
  if (!res.ok) throw new Error("Failed to load " + path + " (HTTP " + res.status + ")");
  return res.json();
}
