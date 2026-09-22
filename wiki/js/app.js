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
  clearAllDrafts,
  loadView,
  saveView,
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
import { registerGdscript } from "./highlight-gdscript.js";
import { icon, TYPE_ICON } from "./icons.js";

const state = {
  pages: [],
  slug: null,
  board: { title: "", cards: [] },
  mode: "view",
  selectedId: null,
  editingId: null,
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
  el.status = $("status");
  el.modeToggle = $("mode-toggle");
  el.saveBtn = $("save-btn");
  el.board = $("board");
  el.zoomLabel = $("zoom-label");

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
    onSetHidden: (id, hidden) => doSetHidden(id, hidden),
    onCardMenu: (id, x, y) => openCardContextMenu(id, x, y),
    onEditingChange: (id) => {
      state.editingId = id;
    },
    uploadAsset: (file) => uploadAsset(file),
    onViewChange: (view) => {
      if (state.slug) saveView(state.slug, view);
    },
    onZoomChange: (scale) => {
      if (el.zoomLabel) el.zoomLabel.textContent = Math.round(scale * 100) + "%";
    },
    grid: 8,
  });

  setupToolbarIcons();
  setupCardMenu();
  setupCardContextMenu();
  setupCanvasControls();
  setupNav();
  wireToolbar();
  wireKeyboard();
  wirePaste();

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
  // The repo index is the source of truth for which pages exist and their titles and
  // categories, so freshly published pages always appear. A local draft only contributes
  // pages the user created but has not saved yet (slugs the server does not have), so unsaved
  // new pages are not lost. This prevents a stale draft from hiding published content.
  let serverPages = [];
  let serverOk = false;
  try {
    const fromServer = await fetchJSON("pages/index.json");
    serverPages = Array.isArray(fromServer) ? fromServer : [];
    serverOk = true;
  } catch (err) {
    setStatus("Could not load the page index: " + err.message, "error");
  }

  const draft = loadIndexDraft();
  if (draft && draft.pages.length) {
    if (serverOk) {
      const serverSlugs = new Set(serverPages.map((p) => p.slug));
      const draftOnly = draft.pages.filter((p) => !serverSlugs.has(p.slug));
      state.pages = serverPages.concat(draftOnly);
      // The index draft has served its purpose; drop it unless it carries unsaved new pages.
      if (!draftOnly.length) clearIndexDraft();
    } else {
      // Offline: fall back to the draft so the app still works.
      state.pages = draft.pages;
    }
  } else {
    state.pages = serverPages;
  }
}

// ----- Sidebar -----

const DEFAULT_CATEGORY = "General";

function pageCategory(page) {
  return (page.category && String(page.category).trim()) || DEFAULT_CATEGORY;
}

// Categories the user has collapsed in the sidebar, remembered per browser.
function loadCollapsedCats() {
  try {
    return new Set(JSON.parse(localStorage.getItem("wiki_collapsed_cats") || "[]"));
  } catch (err) {
    return new Set();
  }
}
function saveCollapsedCats(set) {
  try {
    localStorage.setItem("wiki_collapsed_cats", JSON.stringify(Array.from(set)));
  } catch (err) {
    // Ignore.
  }
}
const collapsedCats = loadCollapsedCats();

function renderSidebar() {
  el.pageList.innerHTML = "";
  if (!state.pages.length) {
    const empty = document.createElement("div");
    empty.className = "page-empty";
    empty.textContent = "No pages yet. Add one to begin.";
    el.pageList.appendChild(empty);
    return;
  }

  // Group pages by category, preserving first-seen order for both categories and pages.
  const order = [];
  const groups = new Map();
  state.pages.forEach((page) => {
    const cat = pageCategory(page);
    if (!groups.has(cat)) {
      groups.set(cat, []);
      order.push(cat);
    }
    groups.get(cat).push(page);
  });

  order.forEach((cat) => {
    const group = document.createElement("div");
    group.className = "page-group";
    const isCollapsed = collapsedCats.has(cat);

    const header = document.createElement("button");
    header.type = "button";
    header.className = "page-cat" + (isCollapsed ? " collapsed" : "");
    header.appendChild(icon(isCollapsed ? "chevron-right" : "chevron-down", { size: 14 }));
    const label = document.createElement("span");
    label.className = "page-cat-label";
    label.textContent = cat;
    header.appendChild(label);
    const count = document.createElement("span");
    count.className = "page-cat-count";
    count.textContent = String(groups.get(cat).length);
    header.appendChild(count);
    header.addEventListener("click", () => {
      if (collapsedCats.has(cat)) collapsedCats.delete(cat);
      else collapsedCats.add(cat);
      saveCollapsedCats(collapsedCats);
      renderSidebar();
    });
    group.appendChild(header);

    if (!isCollapsed) {
      groups.get(cat).forEach((page) => {
        const item = document.createElement("button");
        item.type = "button";
        item.className = "page-item";
        if (page.slug === state.slug) item.classList.add("active");
        item.textContent = page.title || page.slug;
        item.title = page.slug;
        item.addEventListener("click", () => openPage(page.slug));
        item.addEventListener("contextmenu", (e) => {
          e.preventDefault();
          openPageMenu(page.slug, e.clientX, e.clientY);
        });
        group.appendChild(item);
      });
    }
    el.pageList.appendChild(group);
  });
}

// ----- Page loading -----

async function openPage(slug) {
  state.slug = slug;
  state.selectedId = null;
  location.hash = slug;
  closeNav();

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
  // Restore the per-viewer pan/zoom for this page, otherwise frame the content.
  const savedView = loadView(slug);
  if (savedView) boardView.setView(savedView);
  else boardView.fit();
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
  setButton($("nav-toggle"), "menu", "", "Pages");
  setButton($("add-page"), "plus", "Page", "Add page");
  setButton($("rename-page"), "pencil-line", "Rename", "Rename page");
  setButton($("delete-page"), "trash-2", "Delete page", "Delete page");
  setButton(el.saveBtn, "save", "Save", "Save to GitHub");
  setButton($("refresh-btn"), "refresh", "Refresh", "Discard local drafts and reload from the repo");
  setButton($("mobile-add"), "plus", "", "Add a card");
}

// ----- Mobile navigation drawer -----

function openNav() {
  document.body.classList.add("nav-open");
  const bd = $("sidebar-backdrop");
  if (bd) bd.hidden = false;
}

function closeNav() {
  document.body.classList.remove("nav-open");
  const bd = $("sidebar-backdrop");
  if (bd) bd.hidden = true;
}

function setupNav() {
  $("nav-toggle").addEventListener("click", () => {
    if (document.body.classList.contains("nav-open")) closeNav();
    else openNav();
  });
  const bd = $("sidebar-backdrop");
  if (bd) bd.addEventListener("click", closeNav);
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && document.body.classList.contains("nav-open")) closeNav();
  });
  // The mobile add button opens the add-card menu at the center of the board.
  $("mobile-add").addEventListener("click", () => {
    if (state.mode !== "edit") setMode("edit");
    const r = el.board.getBoundingClientRect();
    openCardMenu(r.left + r.width / 2, r.top + r.height / 2);
  });
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
    if (pageMenuEl && !pageMenuEl.hidden && !pageMenuEl.contains(e.target)) closePageMenu();
  });
  document.addEventListener("keydown", (e) => {
    const ctxOpen = (cardCtxEl && !cardCtxEl.hidden) || (pageMenuEl && !pageMenuEl.hidden);
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
  closePageMenu();
}

// ----- Page (sidebar) context menu: rename, category, delete -----

let pageMenuEl = null;

function ensurePageMenu() {
  if (pageMenuEl) return;
  pageMenuEl = document.createElement("div");
  pageMenuEl.className = "card-menu page-ctx-menu";
  pageMenuEl.setAttribute("role", "menu");
  pageMenuEl.setAttribute("aria-label", "Page actions");
  pageMenuEl.hidden = true;
  document.body.appendChild(pageMenuEl);
}

function openPageMenu(slug, clientX, clientY) {
  ensurePageMenu();
  closeCardMenu();
  closeCardContextMenu();
  pageMenuEl.innerHTML = "";
  pageMenuEl.appendChild(ctxItem("pencil-line", "Rename", () => renamePageSlug(slug)));
  pageMenuEl.appendChild(ctxItem("folder-input", "Set category", () => setCategorySlug(slug)));
  pageMenuEl.appendChild(ctxItem("trash-2", "Delete page", () => deletePageSlug(slug), true));

  pageMenuEl.hidden = false;
  const mw = pageMenuEl.offsetWidth || 190;
  const mh = pageMenuEl.offsetHeight || 140;
  let left = clientX;
  let top = clientY;
  if (left + mw > window.innerWidth - 8) left = window.innerWidth - mw - 8;
  if (top + mh > window.innerHeight - 8) top = window.innerHeight - mh - 8;
  pageMenuEl.style.left = Math.max(8, left) + "px";
  pageMenuEl.style.top = Math.max(8, top) + "px";
}

function closePageMenu() {
  if (pageMenuEl) pageMenuEl.hidden = true;
}

// ----- Canvas zoom controls -----

function setupCanvasControls() {
  setButton($("zoom-out"), "minus", "", "Zoom out");
  setButton($("zoom-in"), "plus", "", "Zoom in");
  setButton($("zoom-fit"), "maximize", "", "Fit to content");
  setButton($("zoom-reset"), "rotate-ccw", "", "Reset zoom");
  $("zoom-out").addEventListener("click", () => boardView.zoomOutAt());
  $("zoom-in").addEventListener("click", () => boardView.zoomInAt());
  $("zoom-fit").addEventListener("click", () => boardView.fit());
  $("zoom-reset").addEventListener("click", () => boardView.resetView());
}

function wireToolbar() {
  el.modeToggle.addEventListener("click", () => setMode(state.mode === "edit" ? "view" : "edit"));
  el.saveBtn.addEventListener("click", () => save());
  $("add-page").addEventListener("click", addPage);
  $("rename-page").addEventListener("click", renamePage);
  $("delete-page").addEventListener("click", deletePage);
  $("token-btn").addEventListener("click", () => openTokenModal());
  $("refresh-btn").addEventListener("click", refreshFromRepo);
  window.addEventListener("hashchange", () => {
    const slug = decodeURIComponent((location.hash || "").replace(/^#/, "")).trim();
    if (slug && slug !== state.slug && state.pages.some((p) => p.slug === slug)) {
      openPage(slug);
    }
  });
}

// Discard every local draft and reload straight from the repo. This is the escape hatch when
// a browser is showing stale local content instead of what was published.
function refreshFromRepo() {
  const ok = window.confirm(
    "Discard local drafts and reload the wiki from the repo? Unsaved edits on this device will be lost."
  );
  if (!ok) return;
  clearAllDrafts();
  location.reload();
}

function wireKeyboard() {
  document.addEventListener("keydown", (e) => {
    if (state.mode !== "edit") return;
    // Escape commits an open inline editor.
    if (e.key === "Escape") {
      boardView.commitEdit();
      return;
    }
    if (!state.selectedId) return;
    const ae = document.activeElement;
    const tag = (ae && ae.tagName) || "";
    if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || (ae && ae.isContentEditable)) return;
    if (state.editingId) return;
    if (e.key === "Delete" || e.key === "Backspace") {
      e.preventDefault();
      removeSelected();
    }
  });
}

// ----- Paste images, gifs and SVG onto the board -----

// The point where a pasted or newly placed item should land.
function pastePoint() {
  if (lastPointer.overBoard) return boardView.clientToBoard(lastPointer.x, lastPointer.y);
  return boardView.spawnPoint();
}

function isImageUrl(text) {
  if (/^data:image\//i.test(text)) return true;
  if (!/^https?:\/\//i.test(text)) return false;
  return /\.(png|jpe?g|gif|webp|svg|avif|bmp)(\?.*)?$/i.test(text);
}

// Place an image card from a resolved src (data URL or link).
function placeImageCard(src, name, offset) {
  const at = pastePoint();
  const n = offset || 0;
  at.x += n * 24;
  at.y += n * 24;
  const card = addCardAt("image", at);
  card.src = src;
  card.svg = "";
  if (name) card.alt = name;
  markDirty();
  boardView.refreshCard(card);
  return card;
}

// Place an SVG card from inline markup.
function placeSvgCard(svg) {
  const at = pastePoint();
  const card = addCardAt("image", at);
  card.svg = svg;
  card.src = "";
  markDirty();
  boardView.refreshCard(card);
  return card;
}

// Handle pasted text: inline SVG, or a link to an image. Returns true when it was handled.
function handlePastedText(text) {
  const t = (text || "").trim();
  if (!t) return false;
  if (/^<svg[\s>]/i.test(t) && /<\/svg>/i.test(t)) {
    placeSvgCard(t);
    setStatus("Pasted SVG.", "success");
    return true;
  }
  if (isImageUrl(t)) {
    placeImageCard(t, "");
    setStatus("Pasted image link.", "success");
    return true;
  }
  return false;
}

function addImageFromFile(file, index) {
  const reader = new FileReader();
  reader.onload = () => {
    placeImageCard(String(reader.result), file.name || "", index);
    setStatus("Pasted " + (file.name || "image") + ".", "success");
  };
  reader.onerror = () => setStatus("Could not read the pasted image.", "error");
  reader.readAsDataURL(file);
}

function wirePaste() {
  document.addEventListener("paste", (e) => {
    if (state.mode !== "edit") return;
    // Let a focused field or an open inline editor handle its own paste.
    const ae = document.activeElement;
    const tag = (ae && ae.tagName) || "";
    if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || (ae && ae.isContentEditable)) return;
    const dt = e.clipboardData;
    if (!dt) return;

    const files = Array.from(dt.files || []).filter((f) => f.type.startsWith("image/"));
    if (files.length) {
      e.preventDefault();
      files.forEach((file, i) => addImageFromFile(file, i));
      return;
    }

    const text = dt.getData("text/plain") || "";
    if (handlePastedText(text)) e.preventDefault();
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
}

// Shared card actions, used by the card action bar and the right click menu.

function doDeleteCard(id) {
  if (!id) return;
  state.board.cards = state.board.cards.filter((c) => c.id !== id);
  if (state.selectedId === id) state.selectedId = null;
  markDirty();
  boardView.render();
}

function doBringToFront(id) {
  if (!id) return;
  bringToFront(state.board.cards, id);
  state.selectedId = id;
  markDirty();
  boardView.render();
  boardView.updateSelection();
}

function doSendToBack(id) {
  if (!id) return;
  sendToBack(state.board.cards, id);
  state.selectedId = id;
  markDirty();
  boardView.render();
  boardView.updateSelection();
}

function doToggleCollapse(id) {
  const card = state.board.cards.find((c) => c.id === id);
  if (!card) return;
  card.collapsed = !card.collapsed;
  markDirty();
  boardView.render();
  boardView.updateSelection();
}

function doSetHidden(id, hidden) {
  const card = state.board.cards.find((c) => c.id === id);
  if (!card) return;
  card.hidden = hidden;
  markDirty();
  boardView.render();
  boardView.updateSelection();
}

function removeSelected() {
  doDeleteCard(state.selectedId);
}

// ----- Autosave -----

// Card edits only change the current board, not the page index, so only the board draft is
// written here. Page add/rename/category/delete save the index draft themselves.
function markDirty() {
  if (state.slug) saveDraft(state.slug, state.board);
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

let lastCategory = "";

function addPage() {
  const title = window.prompt("New page title");
  if (title == null) return;
  const clean = title.trim();
  if (!clean) return;
  const cat = window.prompt("Category (blank for General)", lastCategory);
  if (cat == null) return;
  const category = cat.trim() || DEFAULT_CATEGORY;
  lastCategory = category === DEFAULT_CATEGORY ? "" : category;
  const slug = deriveSlug(clean);
  // If this reuses a slug queued for deletion, keep its file.
  state.deletedPages = state.deletedPages.filter((s) => s !== slug);
  state.pages.push({ slug, title: clean, category });
  saveIndexDraft(state.pages);
  state.board = { title: clean, cards: [] };
  saveDraft(slug, state.board);
  renderSidebar();
  openPage(slug);
  setStatus('Page "' + clean + '" added to ' + category + ". Save to publish it.", "info");
}

function renamePage() {
  if (state.slug) renamePageSlug(state.slug);
}

function deletePage() {
  if (state.slug) deletePageSlug(state.slug);
}

function renamePageSlug(slug) {
  const entry = state.pages.find((p) => p.slug === slug);
  if (!entry) return;
  const title = window.prompt("Rename page", entry.title || slug);
  if (title == null) return;
  const clean = title.trim();
  if (!clean) return;
  entry.title = clean;
  if (slug === state.slug) {
    state.board.title = clean;
    renderBoardChrome();
    markDirty();
  }
  saveIndexDraft(state.pages);
  renderSidebar();
  setStatus("Renamed. Save to publish the change.", "info");
}

function setCategorySlug(slug) {
  const entry = state.pages.find((p) => p.slug === slug);
  if (!entry) return;
  const cat = window.prompt("Category for this page (blank for General)", entry.category || "");
  if (cat == null) return;
  entry.category = cat.trim() || DEFAULT_CATEGORY;
  lastCategory = entry.category === DEFAULT_CATEGORY ? "" : entry.category;
  saveIndexDraft(state.pages);
  renderSidebar();
  setStatus('Moved to ' + entry.category + ". Save to publish the change.", "info");
}

function deletePageSlug(slug) {
  const entry = state.pages.find((p) => p.slug === slug);
  if (!entry) return;
  const label = entry.title || slug;
  const ok = window.confirm('Delete page "' + label + '"? Save removes it from the site and the repo.');
  if (!ok) return;
  state.pages = state.pages.filter((p) => p.slug !== slug);
  clearDraft(slug);
  saveIndexDraft(state.pages);
  // Queue the repo file for deletion on the next Save.
  if (!state.deletedPages.includes(slug)) state.deletedPages.push(slug);
  renderSidebar();
  if (slug === state.slug) {
    const nextSlug = state.pages.length ? state.pages[0].slug : null;
    if (nextSlug) {
      openPage(nextSlug);
    } else {
      state.slug = null;
      state.board = { title: "", cards: [] };
      renderBoardChrome();
      boardView.render();
    }
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
  boardView.commitEdit();
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
    editingId: state.editingId,
    cardCount: state.board.cards.length,
    pageCount: state.pages.length,
    deletedPages: state.deletedPages.slice(),
    view: boardView ? boardView.getView() : null,
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
      md: c.md,
      html: c.html,
      code: c.code,
      lang: c.lang,
      rows: c.rows,
      src: c.src,
      svg: c.svg,
      shape: c.shape,
      stroke: c.stroke,
    })),
  }),
  setMode,
  addCard,
  addCardAt,
  selectCard,
  toggleCollapse: doToggleCollapse,
  setHidden: doSetHidden,
  pasteText: handlePastedText,
  placeImageSrc: (src, name) => placeImageCard(src, name || ""),
  categories: () => Array.from(new Set(state.pages.map((p) => pageCategory(p)))),
  // Canvas + inline-edit hooks for headless verification.
  getView: () => boardView.getView(),
  setView: (v) => boardView.setView(v),
  fit: () => boardView.fit(),
  clientToBoard: (x, y) => boardView.clientToBoard(x, y),
  beginEdit: (id) => {
    const card = state.board.cards.find((c) => c.id === id);
    if (card) boardView.beginEdit(card);
  },
  commitEdit: () => boardView.commitEdit(),
  setContent: (id, patch) => {
    const card = state.board.cards.find((c) => c.id === id);
    if (!card) return;
    Object.assign(card, patch);
    boardView.refreshCard(card);
    markDirty();
  },
};

init();
