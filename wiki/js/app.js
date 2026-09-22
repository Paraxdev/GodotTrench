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
  deletePageFiles,
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
import { icon, TYPE_ICON } from "./icons.js";

const state = {
  pages: [],
  slug: null,
  board: { title: "", cards: [] },
  mode: "view",
  selectedId: null,
  // Slugs whose page files should be removed from the repo on the next Save.
  deletedPages: [],
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
  el.inspector = $("inspector");
  el.status = $("status");
  el.modeToggle = $("mode-toggle");
  el.saveBtn = $("save-btn");
  el.board = $("board");

  boardView = new BoardView(el.board, {
    getMode: () => state.mode,
    getBoard: () => state.board,
    getSelectedId: () => state.selectedId,
    onSelect: (id) => selectCard(id),
    onMutate: () => markDirty(),
    onBringToFront: (id) => doBringToFront(id),
    onSendToBack: (id) => doSendToBack(id),
    onDelete: (id) => doDeleteCard(id),
    onToggleCollapse: (id) => doToggleCollapse(id),
    onCardMenu: (id, x, y) => openCardContextMenu(id, x, y),
    grid: 8,
  });

  setupToolbarIcons();
  setupCardMenu();
  setupCardContextMenu();
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
  if (editing) {
    setButton(el.modeToggle, "check", "Done", "Switch to view mode");
  } else {
    setButton(el.modeToggle, "pencil", "Edit", "Switch to edit mode");
  }
  el.modeToggle.classList.toggle("is-active", editing);
  if (!editing) closeMenus();
  boardView.render();
  refreshInspector();
}

function setMode(mode) {
  state.mode = mode;
  if (mode !== "edit") state.selectedId = null;
  applyMode();
}

// ----- Toolbar -----

// Set a button's content to an icon plus an optional text label.
function setButton(btn, iconName, label, ariaLabel) {
  btn.innerHTML = "";
  btn.appendChild(icon(iconName));
  if (label) {
    const span = document.createElement("span");
    span.className = "btn-label";
    span.textContent = label;
    btn.appendChild(span);
  }
  if (ariaLabel) btn.setAttribute("aria-label", ariaLabel);
}

function setupToolbarIcons() {
  setButton($("add-page"), "plus", "Page", "Add page");
  setButton($("rename-page"), "pencil-line", "Rename", "Rename page");
  setButton($("delete-page"), "trash-2", "Delete page", "Delete page");
  setButton(el.saveBtn, "save", "Save", "Save to GitHub");
}

// ----- Add card context menu (edit mode only) -----

let menuEl = null;
let menuBoardPoint = { x: 0, y: 0 };
const lastPointer = { x: 0, y: 0, overBoard: false };

function setupCardMenu() {
  menuEl = document.createElement("div");
  menuEl.className = "card-menu";
  menuEl.setAttribute("role", "menu");
  menuEl.setAttribute("aria-label", "Add a card");
  menuEl.hidden = true;

  CARD_TYPES.forEach((type) => {
    const item = document.createElement("button");
    item.type = "button";
    item.className = "menu-item";
    item.dataset.type = type;
    item.setAttribute("role", "menuitem");
    item.setAttribute("aria-label", "Add " + TYPE_LABELS[type] + " card");
    item.appendChild(icon(TYPE_ICON[type]));
    const label = document.createElement("span");
    label.textContent = TYPE_LABELS[type];
    item.appendChild(label);
    item.addEventListener("click", () => {
      const at = menuBoardPoint;
      closeCardMenu();
      addCardAt(type, at);
    });
    menuEl.appendChild(item);
  });
  document.body.appendChild(menuEl);

  // Track the pointer over the board so Shift+A can place a card under the cursor.
  el.board.addEventListener("pointermove", (e) => {
    lastPointer.x = e.clientX;
    lastPointer.y = e.clientY;
    lastPointer.overBoard = true;
  });
  el.board.addEventListener("pointerleave", () => {
    lastPointer.overBoard = false;
  });

  // Right click opens the menu in edit mode and suppresses the browser menu. View mode is
  // left untouched.
  el.board.addEventListener("contextmenu", (e) => {
    if (state.mode !== "edit") return;
    e.preventDefault();
    openCardMenu(e.clientX, e.clientY);
  });

  // Close on an outside pointerdown or Escape.
  document.addEventListener("pointerdown", (e) => {
    if (!menuEl.hidden && !menuEl.contains(e.target)) closeCardMenu();
    if (cardCtxEl && !cardCtxEl.hidden && !cardCtxEl.contains(e.target)) closeCardContextMenu();
  });
  document.addEventListener("keydown", (e) => {
    const ctxOpen = cardCtxEl && !cardCtxEl.hidden;
    if ((!menuEl.hidden || ctxOpen) && e.key === "Escape") {
      e.preventDefault();
      closeMenus();
      return;
    }
    if (state.mode !== "edit") return;
    const tag = (document.activeElement && document.activeElement.tagName) || "";
    if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;
    // Shift+A opens the add menu at the cursor, Blender style.
    if (e.shiftKey && (e.code === "KeyA" || e.key === "A" || e.key === "a")) {
      e.preventDefault();
      let cx;
      let cy;
      if (lastPointer.overBoard) {
        cx = lastPointer.x;
        cy = lastPointer.y;
      } else {
        const r = el.board.getBoundingClientRect();
        cx = r.left + r.width / 2;
        cy = r.top + r.height / 2;
      }
      openCardMenu(cx, cy);
    }
  });
}

function openCardMenu(clientX, clientY) {
  closeCardContextMenu();
  menuBoardPoint = boardView.clientToBoard(clientX, clientY);
  menuEl.hidden = false;
  const mw = menuEl.offsetWidth || 190;
  const mh = menuEl.offsetHeight || 280;
  let left = clientX;
  let top = clientY;
  if (left + mw > window.innerWidth - 8) left = window.innerWidth - mw - 8;
  if (top + mh > window.innerHeight - 8) top = window.innerHeight - mh - 8;
  menuEl.style.left = Math.max(8, left) + "px";
  menuEl.style.top = Math.max(8, top) + "px";
}

function closeCardMenu() {
  if (menuEl) menuEl.hidden = true;
}

// ----- Per card context menu: hide, collapse, layering, delete (edit mode) -----

let cardCtxEl = null;

function setupCardContextMenu() {
  cardCtxEl = document.createElement("div");
  cardCtxEl.className = "card-menu card-ctx-menu";
  cardCtxEl.setAttribute("role", "menu");
  cardCtxEl.setAttribute("aria-label", "Card actions");
  cardCtxEl.hidden = true;
  document.body.appendChild(cardCtxEl);
}

function ctxItem(iconName, label, onClick, danger) {
  const item = document.createElement("button");
  item.type = "button";
  item.className = "menu-item" + (danger ? " danger" : "");
  item.setAttribute("role", "menuitem");
  item.appendChild(icon(iconName));
  const span = document.createElement("span");
  span.textContent = label;
  item.appendChild(span);
  item.addEventListener("click", () => {
    closeCardContextMenu();
    onClick();
  });
  return item;
}

// Build the menu fresh for the target card, so labels reflect its collapsed and hidden state.
function openCardContextMenu(cardId, clientX, clientY) {
  const card = state.board.cards.find((c) => c.id === cardId);
  if (!card || !cardCtxEl) return;
  closeCardMenu();
  cardCtxEl.innerHTML = "";

  cardCtxEl.appendChild(
    card.collapsed
      ? ctxItem("chevrons-up-down", "Expand", () => doToggleCollapse(cardId))
      : ctxItem("chevrons-down-up", "Collapse", () => doToggleCollapse(cardId))
  );
  cardCtxEl.appendChild(
    card.hidden
      ? ctxItem("eye", "Show", () => doSetHidden(cardId, false))
      : ctxItem("eye-off", "Hide", () => doSetHidden(cardId, true))
  );
  cardCtxEl.appendChild(ctxItem("bring-to-front", "Bring to front", () => doBringToFront(cardId)));
  cardCtxEl.appendChild(ctxItem("send-to-back", "Send to back", () => doSendToBack(cardId)));
  cardCtxEl.appendChild(ctxItem("trash-2", "Delete", () => doDeleteCard(cardId), true));

  cardCtxEl.hidden = false;
  const mw = cardCtxEl.offsetWidth || 190;
  const mh = cardCtxEl.offsetHeight || 220;
  let left = clientX;
  let top = clientY;
  if (left + mw > window.innerWidth - 8) left = window.innerWidth - mw - 8;
  if (top + mh > window.innerHeight - 8) top = window.innerHeight - mh - 8;
  cardCtxEl.style.left = Math.max(8, left) + "px";
  cardCtxEl.style.top = Math.max(8, top) + "px";
}

function closeCardContextMenu() {
  if (cardCtxEl) cardCtxEl.hidden = true;
}

function closeMenus() {
  closeCardMenu();
  closeCardContextMenu();
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
    if (e.key === "Delete" || e.key === "Backspace") {
      e.preventDefault();
      removeSelected();
    }
  });
}

// ----- Card actions -----

// Add a card at a specific board coordinate (from the context menu or Shift+A).
function addCardAt(type, at) {
  if (state.mode !== "edit") setMode("edit");
  const card = newCard(type, at, state.board.cards);
  state.board.cards.push(card);
  markDirty();
  boardView.render();
  selectCard(card.id);
  return card;
}

// Add a card near the top left of the viewport (used by the test hook).
function addCard(type) {
  return addCardAt(type, boardView.spawnPoint());
}

function selectCard(id) {
  state.selectedId = id;
  boardView.updateSelection();
  refreshInspector();
}

// Shared card actions, used by both the inspector and the card header controls.

function doDeleteCard(id) {
  if (!id) return;
  state.board.cards = state.board.cards.filter((c) => c.id !== id);
  if (state.selectedId === id) state.selectedId = null;
  markDirty();
  boardView.render();
  refreshInspector();
}

function doBringToFront(id) {
  if (!id) return;
  bringToFront(state.board.cards, id);
  state.selectedId = id;
  markDirty();
  boardView.render();
  boardView.updateSelection();
  refreshInspector();
}

function doSendToBack(id) {
  if (!id) return;
  sendToBack(state.board.cards, id);
  state.selectedId = id;
  markDirty();
  boardView.render();
  boardView.updateSelection();
  refreshInspector();
}

function doToggleCollapse(id) {
  const card = state.board.cards.find((c) => c.id === id);
  if (!card) return;
  card.collapsed = !card.collapsed;
  markDirty();
  boardView.render();
  boardView.updateSelection();
  refreshInspector();
}

function doSetHidden(id, hidden) {
  const card = state.board.cards.find((c) => c.id === id);
  if (!card) return;
  card.hidden = hidden;
  markDirty();
  boardView.render();
  boardView.updateSelection();
  refreshInspector();
}

function removeSelected() {
  doDeleteCard(state.selectedId);
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
    bringToFront: () => card && doBringToFront(card.id),
    sendToBack: () => card && doSendToBack(card.id),
    remove: () => card && doDeleteCard(card.id),
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
  // If this reuses a slug queued for deletion, keep its file.
  state.deletedPages = state.deletedPages.filter((s) => s !== slug);
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
  const ok = window.confirm('Delete page "' + label + '"? Save removes it from the site and the repo.');
  if (!ok) return;
  const removed = state.slug;
  state.pages = state.pages.filter((p) => p.slug !== removed);
  clearDraft(removed);
  saveIndexDraft(state.pages);
  // Queue the repo file for deletion on the next Save.
  if (!state.deletedPages.includes(removed)) state.deletedPages.push(removed);
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
  setStatus('Deleted "' + label + '". Save to publish the removal.', "info");
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
  setButton(el.saveBtn, "save", "Saving...", "Saving");
  setStatus("Committing to " + GH.owner + "/" + GH.repo + " ...", "info");
  try {
    await commitPage(state.slug, state.board, state.pages, token);
    // Remove the files of any deleted pages, unless a slug was recreated in the meantime.
    const toDelete = state.deletedPages.filter((s) => !state.pages.some((p) => p.slug === s));
    if (toDelete.length) await deletePageFiles(toDelete, token);
    state.deletedPages = [];
    clearDraft(state.slug);
    clearIndexDraft();
    setStatus("Saved to GitHub. Pages will redeploy shortly.", "success");
  } catch (err) {
    setStatus("Save failed: " + err.message, "error");
  } finally {
    el.saveBtn.disabled = false;
    setButton(el.saveBtn, "save", "Save", "Save to GitHub");
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
    pageCount: state.pages.length,
    deletedPages: state.deletedPages.slice(),
    cards: state.board.cards.map((c) => ({
      id: c.id,
      type: c.type,
      x: c.x,
      y: c.y,
      w: c.w,
      h: c.h,
      z: c.z,
      collapsed: !!c.collapsed,
      hidden: !!c.hidden,
      format: c.format,
    })),
  }),
  setMode,
  addCard,
  addCardAt,
  selectCard,
  toggleCollapse: doToggleCollapse,
  setHidden: doSetHidden,
};

init();
