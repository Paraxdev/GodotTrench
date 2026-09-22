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
    return parsed; // { savedAt, board }
  } catch (err) {
    return null;
  }
}

export function saveDraft(slug, board) {
  try {
    const payload = JSON.stringify({ savedAt: Date.now(), board });
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
    return parsed; // { savedAt, pages }
  } catch (err) {
    return null;
  }
}

export function saveIndexDraft(pages) {
  try {
    localStorage.setItem(INDEX_DRAFT_KEY, JSON.stringify({ savedAt: Date.now(), pages }));
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

// Remove every local draft (all page boards and the index), so the next load comes entirely
// from the repo. Used by the sidebar Refresh action.
export function clearAllDrafts() {
  try {
    const keys = [];
    for (let i = 0; i < localStorage.length; i += 1) {
      const k = localStorage.key(i);
      if (k && (k.indexOf(DRAFT_PREFIX) === 0 || k === INDEX_DRAFT_KEY)) keys.push(k);
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
