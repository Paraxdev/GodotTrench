// Application entry point. Wires the sidebar, toolbar, board, inspector, autosave,
// and the GitHub Save flow together.

import { GH } from "./config.js";
import {
  loadDraft,
  saveDraft,
  clearDraft,
  loadIndexDraft,
  saveIndexDraft,
  clearIndexDraft,
  fetchJSON,
} from "./storage.js";
import {
  getToken,
  setToken,
  clearToken,
  hasToken,
  commitPage,
  uploadAsset as ghUploadAsset,
} from "./github.js";
import {
  CARD_TYPES,
  TYPE_LABELS,
  newCard,
  normalizeZ,
  bringToFront,
  sendToBack,
} from "./cards.js";
import { BoardView } from "./board.js";
import { renderInspector } from "./inspector.js";
import { registerGdscript } from "./highlight-gdscript.js";

const state = {
  pages: [],
  slug: null,
  board: { title: "", cards: [] },
  mode: "view",
  selectedId: null,
};

let boardView = null;
const el = {};

function $(id) {
  return document.getElementById(id);
}

async function init() {
  if (window.hljs) registerGdscript(window.hljs);

  el.pageList = $("page-list");
  el.pageTitle = $("page-title");
  el.addTools = $("add-tools");
  el.inspector = $("inspector");
  el.status = $("status");
  el.modeToggle = $("mode-toggle");
  el.saveBtn = $("save-btn");

  boardView = new BoardView($("board"), {
    getMode: () => state.mode,
    getBoard: () => state.board,
    getSelectedId: () => state.selectedId,
    onSelect: (id) => selectCard(id),
    onMutate: () => markDirty(),
    grid: 8,
  });

  buildAddTools();
  wireToolbar();
  wireKeyboard();

  await loadIndex();
  renderSidebar();

  const startSlug = pickStartSlug();
  if (startSlug) {
    await openPage(startSlug);
  } else {
    renderBoardChrome();
  }

  applyMode();
}

function pickStartSlug() {
  const hash = decodeURIComponent((location.hash || "").replace(/^#/, "")).trim();
  if (hash && state.pages.some((p) => p.slug === hash)) return hash;
  return state.pages.length ? state.pages[0].slug : null;
}

async function loadIndex() {
  const draft = loadIndexDraft();
  if (draft && draft.pages.length) {
    state.pages = draft.pages;
    return;
  }
  try {
    const fromServer = await fetchJSON("pages/index.json");
    state.pages = Array.isArray(fromServer) ? fromServer : [];
  } catch (err) {
    state.pages = [];
    setStatus("Could not load the page index: " + err.message, "error");
  }
}

// ----- Sidebar -----

function renderSidebar() {
  el.pageList.innerHTML = "";
  state.pages.forEach((page) => {
    const item = document.createElement("button");
    item.type = "button";
    item.className = "page-item";
    if (page.slug === state.slug) item.classList.add("active");
    item.textContent = page.title || page.slug;
    item.title = page.slug;
    item.addEventListener("click", () => openPage(page.slug));
    el.pageList.appendChild(item);
  });
  if (!state.pages.length) {
    const empty = document.createElement("div");
    empty.className = "page-empty";
    empty.textContent = "No pages yet. Add one to begin.";
    el.pageList.appendChild(empty);
  }
}

// ----- Page loading -----

async function openPage(slug) {
  state.slug = slug;
  state.selectedId = null;
  location.hash = slug;

  const indexEntry = state.pages.find((p) => p.slug === slug);
  const fallbackTitle = indexEntry ? indexEntry.title : slug;

  const draft = loadDraft(slug);
  if (draft) {
    state.board = normalizeBoard(draft.board, fallbackTitle);
  } else {
    let loaded = null;
    try {
      loaded = await fetchJSON("pages/" + slug + ".json");
    } catch (err) {
      setStatus("Could not load page " + slug + ": " + err.message, "error");
    }
    state.board = loaded ? normalizeBoard(loaded, fallbackTitle) : { title: fallbackTitle, cards: [] };
  }

  renderSidebar();
  renderBoardChrome();
  boardView.render();
  refreshInspector();
}

function normalizeBoard(board, fallbackTitle) {
  const cards = Array.isArray(board.cards) ? board.cards : [];
  cards.forEach((c) => {
    c.x = Number(c.x) || 0;
    c.y = Number(c.y) || 0;
    c.w = Number(c.w) || 200;
    c.h = Number(c.h) || 140;
    c.z = Number.isFinite(c.z) ? c.z : 0;
  });
  return { title: board.title || fallbackTitle, cards };
}

function renderBoardChrome() {
  el.pageTitle.textContent = state.board.title || state.slug || "";
}

// ----- Mode -----

function applyMode() {
  const editing = state.mode === "edit";
  document.body.classList.toggle("mode-edit", editing);
  el.modeToggle.textContent = editing ? "Done" : "Edit";
  el.modeToggle.classList.toggle("btn-primary", editing);
  boardView.render();
  refreshInspector();
}

function setMode(mode) {
  state.mode = mode;
  if (mode !== "edit") state.selectedId = null;
  applyMode();
}

// ----- Toolbar -----

function buildAddTools() {
  el.addTools.innerHTML = "";
  CARD_TYPES.forEach((type) => {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "btn btn-small add-card-btn";
    btn.dataset.type = type;
    btn.textContent = "+ " + TYPE_LABELS[type];
    btn.addEventListener("click", () => addCard(type));
    el.addTools.appendChild(btn);
  });
}

function wireToolbar() {
  el.modeToggle.addEventListener("click", () => setMode(state.mode === "edit" ? "view" : "edit"));
  el.saveBtn.addEventListener("click", () => save());
  $("add-page").addEventListener("click", addPage);
  $("rename-page").addEventListener("click", renamePage);
  $("delete-page").addEventListener("click", deletePage);
  $("token-btn").addEventListener("click", () => openTokenModal());
  window.addEventListener("hashchange", () => {
    const slug = decodeURIComponent((location.hash || "").replace(/^#/, "")).trim();
    if (slug && slug !== state.slug && state.pages.some((p) => p.slug === slug)) {
      openPage(slug);
    }
  });
}

function wireKeyboard() {
  document.addEventListener("keydown", (e) => {
    if (state.mode !== "edit" || !state.selectedId) return;
    const tag = (document.activeElement && document.activeElement.tagName) || "";
    if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;
    if (e.key === "Delete") {
      e.preventDefault();
      removeSelected();
    }
  });
}

// ----- Card actions -----

function addCard(type) {
  if (state.mode !== "edit") setMode("edit");
  const card = newCard(type, boardView.spawnPoint(), state.board.cards);
  state.board.cards.push(card);
  markDirty();
  boardView.render();
  selectCard(card.id);
}

function selectCard(id) {
  state.selectedId = id;
  boardView.updateSelection();
  refreshInspector();
}

function removeSelected() {
  const id = state.selectedId;
  if (!id) return;
  state.board.cards = state.board.cards.filter((c) => c.id !== id);
  state.selectedId = null;
  markDirty();
  boardView.render();
  refreshInspector();
}

function refreshInspector() {
  if (state.mode !== "edit") {
    el.inspector.innerHTML = "";
    return;
  }
  const card = state.board.cards.find((c) => c.id === state.selectedId) || null;
  renderInspector(el.inspector, {
    card,
    update: () => {
      if (card) boardView.refreshCard(card);
      markDirty();
    },
    rerender: () => {
      if (card) boardView.refreshCard(card);
    },
    bringToFront: () => {
      if (!card) return;
      bringToFront(state.board.cards, card.id);
      markDirty();
      boardView.render();
      boardView.updateSelection();
    },
    sendToBack: () => {
      if (!card) return;
      sendToBack(state.board.cards, card.id);
      markDirty();
      boardView.render();
      boardView.updateSelection();
    },
    remove: () => removeSelected(),
    uploadAsset: (file) => uploadAsset(file),
  });
}

// ----- Autosave -----

function markDirty() {
  if (state.slug) saveDraft(state.slug, state.board);
  saveIndexDraft(state.pages);
}

// ----- Page management -----

function deriveSlug(title) {
  let slug = String(title)
    .toLowerCase()
    .replace(/[^\w]+/g, "-")
    .replace(/^-+|-+$/g, "");
  if (!slug) slug = "page";
  let unique = slug;
  let n = 2;
  while (state.pages.some((p) => p.slug === unique)) {
    unique = slug + "-" + n;
    n += 1;
  }
  return unique;
}

function addPage() {
  const title = window.prompt("New page title");
  if (title == null) return;
  const clean = title.trim();
  if (!clean) return;
  const slug = deriveSlug(clean);
  state.pages.push({ slug, title: clean });
  saveIndexDraft(state.pages);
  state.board = { title: clean, cards: [] };
  saveDraft(slug, state.board);
  renderSidebar();
  openPage(slug);
  setStatus('Page "' + clean + '" added. Save to publish it.', "info");
}

function renamePage() {
  if (!state.slug) return;
  const entry = state.pages.find((p) => p.slug === state.slug);
  const current = entry ? entry.title : state.slug;
  const title = window.prompt("Rename page", current);
  if (title == null) return;
  const clean = title.trim();
  if (!clean) return;
  if (entry) entry.title = clean;
  state.board.title = clean;
  markDirty();
  renderSidebar();
  renderBoardChrome();
  setStatus("Renamed. Save to publish the change.", "info");
}

function deletePage() {
  if (!state.slug) return;
  const entry = state.pages.find((p) => p.slug === state.slug);
  const label = entry ? entry.title : state.slug;
  const ok = window.confirm('Delete page "' + label + '"? This also removes it from the index on the next Save.');
  if (!ok) return;
  const removed = state.slug;
  state.pages = state.pages.filter((p) => p.slug !== removed);
  clearDraft(removed);
  saveIndexDraft(state.pages);
  const nextSlug = state.pages.length ? state.pages[0].slug : null;
  renderSidebar();
  if (nextSlug) {
    openPage(nextSlug);
  } else {
    state.slug = null;
    state.board = { title: "", cards: [] };
    renderBoardChrome();
    boardView.render();
    refreshInspector();
  }
  setStatus('Deleted "' + label + '". Save to publish, and remove wiki/pages/' + removed + '.json in the repo if needed.', "info");
}

// ----- GitHub Save -----

async function ensureToken() {
  if (hasToken()) return getToken();
  await openTokenModal();
  return hasToken() ? getToken() : null;
}

async function save() {
  if (!state.slug) {
    setStatus("There is no page to save. Add a page first.", "error");
    return;
  }
  const token = await ensureToken();
  if (!token) {
    setStatus("Save cancelled, no token provided.", "info");
    return;
  }
  normalizeZ(state.board.cards);
  boardView.render();
  boardView.updateSelection();

  el.saveBtn.disabled = true;
  el.saveBtn.textContent = "Saving...";
  setStatus("Committing to " + GH.owner + "/" + GH.repo + " ...", "info");
  try {
    await commitPage(state.slug, state.board, state.pages, token);
    clearDraft(state.slug);
    clearIndexDraft();
    setStatus("Saved to GitHub. Pages will redeploy shortly.", "success");
  } catch (err) {
    setStatus("Save failed: " + err.message, "error");
  } finally {
    el.saveBtn.disabled = false;
    el.saveBtn.textContent = "Save";
  }
}

async function uploadAsset(file) {
  const token = await ensureToken();
  if (!token) {
    setStatus("Upload cancelled, no token provided.", "info");
    return null;
  }
  setStatus("Uploading " + file.name + " ...", "info");
  try {
    const relPath = await ghUploadAsset(file, token);
    setStatus("Uploaded " + relPath + ".", "success");
    return relPath;
  } catch (err) {
    setStatus("Upload failed: " + err.message, "error");
    return null;
  }
}

// ----- Token modal -----

function openTokenModal() {
  return new Promise((resolve) => {
    const modal = $("token-modal");
    const input = $("token-input");
    const saveBtn = $("token-save");
    const clearBtn = $("token-clear");
    const cancelBtn = $("token-cancel");

    input.value = "";
    modal.hidden = false;
    input.focus();

    const close = () => {
      modal.hidden = true;
      saveBtn.removeEventListener("click", onSave);
      clearBtn.removeEventListener("click", onClear);
      cancelBtn.removeEventListener("click", onCancel);
      modal.removeEventListener("click", onBackdrop);
      resolve();
    };
    const onSave = () => {
      const value = input.value.trim();
      if (!value) {
        setStatus("Token was empty.", "error");
        return;
      }
      setToken(value);
      setStatus("Token stored in this browser only.", "success");
      close();
    };
    const onClear = () => {
      clearToken();
      setStatus("Token cleared from this browser.", "info");
      close();
    };
    const onCancel = () => close();
    const onBackdrop = (e) => {
      if (e.target === modal) close();
    };

    saveBtn.addEventListener("click", onSave);
    clearBtn.addEventListener("click", onClear);
    cancelBtn.addEventListener("click", onCancel);
    modal.addEventListener("click", onBackdrop);
  });
}

// ----- Status line -----

let statusTimer = null;
function setStatus(message, kind) {
  el.status.textContent = message;
  el.status.className = "status show " + (kind || "info");
  if (statusTimer) clearTimeout(statusTimer);
  if (kind !== "error") {
    statusTimer = setTimeout(() => {
      el.status.className = "status";
    }, 4000);
  }
}

// Expose a tiny hook for headless verification. This reads state only, it never writes.
window.__wiki = {
  getState: () => ({
    slug: state.slug,
    mode: state.mode,
    selectedId: state.selectedId,
    cardCount: state.board.cards.length,
    cards: state.board.cards.map((c) => ({ id: c.id, type: c.type, x: c.x, y: c.y, w: c.w, h: c.h, z: c.z })),
  }),
  setMode,
  addCard,
  selectCard,
};

init();
