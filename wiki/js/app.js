// Application entry point. Owns the state and wires the sidebar, toolbar, the two page layouts
// (document and canvas), drafts, undo, search and the GitHub save flow together.

import { GH } from "./config.js";
// Imported as h because app.js uses el as the cache of toolbar and sidebar element references.
import { el as h } from "./dom.js";
import {
  loadDraft,
  saveDraft,
  clearDraft,
  loadIndexDraft,
  saveIndexDraft,
  clearIndexDraft,
  clearAllDrafts,
  listDraftSlugs,
  savePending,
  loadPending,
  clearPending,
  hashString,
  loadView,
  saveView,
  fetchJSON,
} from "./storage.js";
import { getToken, setToken, clearToken, hasToken, commitFiles, uploadAsset as ghUploadAsset } from "./github.js";
import { newCard, normalizeZ, bringToFront, sendToBack } from "./cards.js";
import { BoardView } from "./board.js";
import { DocView } from "./doc.js";
import { closeSuggest, blockById, STARTERS } from "./edit.js";
import { setPageResolver, slugify } from "./markdown.js";
import { SearchIndex, openSearchDialog } from "./search.js";
import { registerGdscript } from "./highlight-gdscript.js";
import { icon, TYPE_ICON } from "./icons.js";

const state = {
  pages: [],
  slug: null,
  board: { title: "", layout: "doc", cards: [] },
  mode: "view",
  selectedId: null,
  editingId: null,
  // Slugs whose page files should be removed from the repo on the next Save.
  deletedPages: [],
  // The repo's pages/index.json as last fetched, before any local edits. Save diffs against
  // this to tell "we changed this entry" from "we just carried it over", so a page published
  // by someone else since is merged in rather than dropped.
  indexBase: [],
  // Hash of the repo version the open page was loaded from, stored with its draft.
  baseHash: null,
  // Set when a draft was started from an older repo version: { board, hash }.
  conflict: null,
  publishing: false,
};

let boardView = null;
let docView = null;
let searchIndex = null;
const el = {};

function $(id) {
  return document.getElementById(id);
}

function layout() {
  return state.board.layout === "canvas" ? "canvas" : "doc";
}

function pageInfo(slug) {
  return state.pages.find((p) => p.slug === (slug || state.slug)) || null;
}

async function init() {
  if (window.hljs) registerGdscript(window.hljs);

  el.pageList = $("page-list");
  el.pageTitle = $("page-title");
  el.status = $("status");
  el.modeToggle = $("mode-toggle");
  el.saveBtn = $("save-btn");
  el.board = $("board");
  el.doc = $("doc");
  el.banner = $("banner");
  el.zoomLabel = $("zoom-label");

  setPageResolver(resolvePage);
  searchIndex = new SearchIndex(loadBoardForIndex, resolvePage);

  boardView = new BoardView(el.board, {
    getMode: () => state.mode,
    getBoard: () => state.board,
    getSelectedId: () => state.selectedId,
    onSelect: (id) => {
      state.selectedId = id;
    },
    onMutate: () => markDirty(),
    onBringToFront: (id) => doBringToFront(id),
    onSendToBack: (id) => doSendToBack(id),
    onDelete: (id) => doDeleteCard(id),
    onDeleteMany: (ids, edgeIds) => doDeleteMany(ids, edgeIds),
    onEnsureEdit: () => setMode("edit"),
    onCreate: (variant, at, size) => addCardAt("text", at, Object.assign({ variant }, size)),
    onAddMenu: (x, y) => {
      if (state.mode !== "edit") setMode("edit");
      openCardMenu(x, y);
    },
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
    pages: () => state.pages,
    pasteFiles: (files, afterId) => pasteFiles(files, afterId),
    onInsertBelow: (card, type, block) => insertBelowOnCanvas(card, type, block),
    grid: 8,
  });

  docView = new DocView(el.doc, {
    getMode: () => state.mode,
    getBoard: () => state.board,
    getPageInfo: () => pageInfo(),
    onMutate: () => markDirty(),
    uploadAsset: (file) => uploadAsset(file),
    pages: () => state.pages,
    backlinks: () => (state.slug ? searchIndex.backlinks(state.slug) : []),
    pasteFiles: (files, afterId) => pasteFiles(files, afterId),
    onCardMenuAction: (action, id) => cardAction(action, id),
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

  const start = parseHash();
  const startSlug = start && state.pages.some((p) => p.slug === start) ? start : state.pages.length ? state.pages[0].slug : null;
  if (startSlug) await openPage(startSlug);
  else renderChrome();
  applyMode();
}

function parseHash() {
  return decodeURIComponent((location.hash || "").replace(/^#/, "")).trim();
}

// Wikilinks resolve by slug, by title, or by the slug a title would get.
function resolvePage(target) {
  const t = String(target || "").trim();
  if (!t) return null;
  const lower = t.toLowerCase();
  return (
    state.pages.find((p) => p.slug === t) ||
    state.pages.find((p) => (p.title || "").toLowerCase() === lower) ||
    state.pages.find((p) => p.slug === slugify(t)) ||
    null
  );
}

// ----- Page index -----

async function loadIndex() {
  // The repo index is the source of truth for which pages exist, so freshly published pages
  // always appear. A local draft only adds pages created but not saved yet, and hides pages
  // deleted but not saved yet.
  let serverPages = null;
  try {
    const fromServer = await fetchJSON("pages/index.json");
    serverPages = Array.isArray(fromServer) ? fromServer : [];
  } catch (err) {
    setStatus("Could not load the page index: " + err.message, "error");
  }
  if (serverPages) state.indexBase = serverPages.map((p) => Object.assign({}, p));

  const pending = loadPending("index");
  if (pending && serverPages) {
    if (JSON.stringify(pending.data) === JSON.stringify(serverPages)) clearPending("index");
    else serverPages = pending.data;
  }

  const draft = loadIndexDraft();
  if (draft && draft.pages.length) {
    if (serverPages) {
      const serverSlugs = new Set(serverPages.map((p) => p.slug));
      const draftBySlug = new Map(draft.pages.map((p) => [p.slug, p]));
      const deleted = new Set(draft.deleted || []);
      const merged = serverPages.filter((p) => !deleted.has(p.slug)).map((p) => draftBySlug.get(p.slug) || p);
      const draftOnly = draft.pages.filter((p) => !serverSlugs.has(p.slug));
      state.pages = merged.concat(draftOnly);
      state.deletedPages = Array.from(deleted);
      if (!draftOnly.length && !deleted.size && JSON.stringify(merged) === JSON.stringify(serverPages)) clearIndexDraft();
    } else {
      state.pages = draft.pages;
      state.deletedPages = draft.deleted || [];
    }
  } else {
    state.pages = serverPages || [];
  }
  searchIndex.setPages(state.pages);
}

function saveIndex() {
  saveIndexDraft(state.pages, state.deletedPages);
  searchIndex.setPages(state.pages);
  updateDirtyUi();
}

// Build the index to publish from a freshly fetched repo copy, so a page someone else added or
// edited since this tab loaded is kept: an entry only wins locally when it actually differs from
// state.indexBase (the repo version this session started from), meaning it was renamed, moved to
// another category, or newly created here. Untouched entries and pages this tab never learned
// about both come from the fresh copy.
function mergeIndexForSave(freshServer) {
  const baseBySlug = new Map(state.indexBase.map((p) => [p.slug, p]));
  const localBySlug = new Map(state.pages.map((p) => [p.slug, p]));
  const freshSlugs = new Set(freshServer.map((p) => p.slug));
  const deleted = new Set(state.deletedPages);

  const merged = freshServer
    .filter((p) => !deleted.has(p.slug))
    .map((p) => {
      const local = localBySlug.get(p.slug);
      if (!local) return p;
      const based = baseBySlug.get(p.slug);
      if (based && JSON.stringify(local) === JSON.stringify(based)) return p;
      return local;
    });

  const additions = state.pages.filter((p) => !freshSlugs.has(p.slug) && !deleted.has(p.slug));
  return merged.concat(additions);
}

// ----- Sidebar -----

const DEFAULT_CATEGORY = "General";

function pageCategory(page) {
  return (page.category && String(page.category).trim()) || DEFAULT_CATEGORY;
}

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
    el.pageList.appendChild(h("div", { class: "page-empty", text: "No pages yet. Add one to begin." }));
    return;
  }
  const dirty = new Set(listDraftSlugs());

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
    const group = h("div", { class: "page-group" });
    const isCollapsed = collapsedCats.has(cat);
    group.appendChild(
      h(
        "button",
        {
          type: "button",
          class: "page-cat" + (isCollapsed ? " collapsed" : ""),
          "aria-expanded": String(!isCollapsed),
          on: {
            click: () => {
              if (collapsedCats.has(cat)) collapsedCats.delete(cat);
              else collapsedCats.add(cat);
              saveCollapsedCats(collapsedCats);
              renderSidebar();
            },
          },
        },
        [icon("chevron-down", { size: 14, cls: "page-cat-chevron" }), h("span", { class: "page-cat-label", text: cat })]
      )
    );

    if (!isCollapsed) {
      groups.get(cat).forEach((page) => {
        group.appendChild(
          h(
            "button",
            {
              type: "button",
              class: "page-item" + (page.slug === state.slug ? " active" : ""),
              title: page.title || page.slug,
              "aria-current": page.slug === state.slug ? "page" : null,
              on: {
                click: () => openPage(page.slug),
                contextmenu: (e) => {
                  e.preventDefault();
                  openPageMenu(page.slug, e.clientX, e.clientY);
                },
              },
            },
            [
              icon(page.layout === "canvas" ? "layout-dashboard" : "file-text", { size: 16, cls: "page-item-icon" }),
              h("span", { class: "page-item-label", text: page.title || page.slug }),
              dirty.has(page.slug) ? h("span", { class: "dirty-dot", title: "Unsaved changes" }) : null,
            ]
          )
        );
      });
    }
    el.pageList.appendChild(group);
  });
}

// ----- Page loading -----

function normalizeBoard(board, fallbackTitle, fallbackLayout) {
  const cards = Array.isArray(board && board.cards) ? board.cards : [];
  const lay = (board && board.layout) || fallbackLayout || "canvas";
  cards.forEach((c, i) => {
    if (!c.id) c.id = "card_" + i;
    if (lay === "canvas") {
      c.x = Number(c.x) || 0;
      c.y = Number(c.y) || 0;
      c.w = Number(c.w) || 200;
      c.h = Number(c.h) || 140;
      c.z = Number.isFinite(c.z) ? c.z : i;
    }
  });
  const ids = new Set(cards.map((c) => c.id));
  const edges = (Array.isArray(board && board.edges) ? board.edges : [])
    .filter((e) => e && ids.has(e.from) && ids.has(e.to))
    .map((e, i) => (e.id ? e : Object.assign({ id: "edge_" + i }, e)));
  return { title: (board && board.title) || fallbackTitle, layout: lay, cards, edges };
}

// The on-disk form of a board. Documents carry no positions, only a height for the few
// card types that have one.
function serializeBoard(board) {
  const lay = board.layout === "canvas" ? "canvas" : "doc";
  const cards = board.cards.map((c) => {
    const out = {};
    Object.keys(c).forEach((k) => {
      const v = c[k];
      if (v === undefined || v === false) return;
      if (lay === "doc" && (k === "x" || k === "y" || k === "w" || k === "z" || k === "collapsed")) return;
      if (lay === "doc" && k === "h" && c.type !== "draw" && c.type !== "shape") return;
      if (k === "html" && !v) return;
      out[k] = v;
    });
    return out;
  });
  const out = { title: board.title, layout: lay, cards };
  // Connectors are an optional list next to the cards, bound to card ids, so older pages and
  // readers that do not know them keep working.
  const edges = lay === "canvas" && Array.isArray(board.edges) ? board.edges.map(serializeEdge) : [];
  if (edges.length) out.edges = edges;
  return out;
}

function serializeEdge(e) {
  const out = { id: e.id, from: e.from, to: e.to };
  if (e.label) out.label = e.label;
  return out;
}

function boardHash(board) {
  return hashString(JSON.stringify(serializeBoard(board)));
}

async function fetchServerBoard(slug) {
  const info = pageInfo(slug);
  const fallbackTitle = info ? info.title : slug;
  let server = null;
  let ok = true;
  try {
    const loaded = await fetchJSON("pages/" + slug + ".json");
    if (loaded) server = normalizeBoard(loaded, fallbackTitle, info && info.layout);
  } catch (err) {
    ok = false;
  }
  const pending = loadPending("page_" + slug);
  if (pending) {
    const served = server ? boardHash(server) : null;
    if (served === pending.hash) clearPending("page_" + slug);
    else return { board: normalizeBoard(pending.data, fallbackTitle), ok, publishing: true };
  }
  return { board: server, ok, publishing: false };
}

// Used by the search index: the draft if there is one, else the repo copy.
async function loadBoardForIndex(slug) {
  if (slug === state.slug) return state.board;
  const draft = loadDraft(slug);
  if (draft) return draft.board;
  const res = await fetchServerBoard(slug);
  return res.board || { title: slug, cards: [] };
}

async function openPage(slug, focusCardId) {
  if (state.slug) commitEditors();
  const info = pageInfo(slug);
  const fallbackTitle = info ? info.title : slug;
  state.slug = slug;
  state.selectedId = null;
  state.conflict = null;
  if (location.hash.replace(/^#/, "") !== slug) setHash(slug);
  closeNav();

  const res = await fetchServerBoard(slug);
  if (state.slug !== slug) return;
  const server = res.board;
  state.publishing = res.publishing;
  const serverHash = server ? boardHash(server) : null;
  const draft = loadDraft(slug);

  if (!draft) {
    state.board = server || { title: fallbackTitle, layout: (info && info.layout) || "doc", cards: [] };
  } else {
    const draftBoard = normalizeBoard(draft.board, fallbackTitle, info && info.layout);
    if (server && boardHash(draftBoard) === serverHash) {
      clearDraft(slug);
      state.board = server;
    } else {
      state.board = draftBoard;
      if (server && draft.base !== serverHash) state.conflict = { board: server, hash: serverHash };
    }
  }
  if (!res.ok && !draft) setStatus("Could not load page " + slug + ".", "error");
  // A draft in conflict keeps its own base until "Keep my draft" is chosen, so Save still asks
  // before overwriting the newer repo version.
  state.baseHash = state.conflict ? draft.base || null : serverHash;
  resetHistory();
  searchIndex.update(slug, state.board);

  renderSidebar();
  renderView();
  if (layout() === "canvas") {
    const savedView = loadView(slug);
    if (savedView) boardView.setView(savedView);
    else boardView.fit(true);
  } else {
    el.doc.querySelector(".doc-scroll").scrollTop = 0;
  }
  if (focusCardId) revealCard(focusCardId);
}

// pushState rather than location.hash, so opening a page does not fire hashchange and load it
// twice. The first page replaces the entry, so Back leaves the wiki instead of stalling on it.
function setHash(slug) {
  try {
    if (location.hash) history.pushState(null, "", "#" + slug);
    else history.replaceState(null, "", "#" + slug);
  } catch (err) {
    location.hash = slug;
  }
}

function revealCard(id) {
  if (layout() === "doc") docView.scrollToCard(id, true);
  else {
    const card = state.board.cards.find((c) => c.id === id);
    if (card) {
      const r = el.board.getBoundingClientRect();
      boardView.setView({ panX: r.width / 2 - (card.x + card.w / 2), panY: 80 - card.y, scale: 1 });
      state.selectedId = id;
      boardView.updateSelection();
    }
  }
}

function renderChrome() {
  const info = pageInfo();
  el.pageTitle.innerHTML = "";
  if (info && info.category) el.pageTitle.appendChild(h("span", { class: "crumb", text: info.category }));
  el.pageTitle.appendChild(h("span", { class: "crumb-page", text: state.board.title || state.slug || "" }));
  if (state.publishing) {
    el.pageTitle.appendChild(h("span", { class: "chip chip-accent", title: "Saved. GitHub Pages is still deploying, this is your saved version.", text: "Publishing" }));
  }
  document.title = (state.board.title ? state.board.title + " · " : "") + "GodotTrench Wiki";
  document.body.classList.toggle("layout-canvas", layout() === "canvas");
  document.body.classList.toggle("layout-doc", layout() === "doc");
  renderBanner();
  updateDirtyUi();
}

function renderView() {
  renderChrome();
  if (layout() === "canvas") {
    el.doc.hidden = true;
    el.board.hidden = false;
    boardView.render();
  } else {
    el.board.hidden = true;
    el.doc.hidden = false;
    docView.render();
  }
}

function commitEditors() {
  if (layout() === "canvas") boardView.commitEdit();
  else docView.commitAll();
}

// ----- Conflict banner -----

function renderBanner() {
  el.banner.innerHTML = "";
  el.banner.hidden = !state.conflict;
  if (!state.conflict) return;
  el.banner.appendChild(icon("triangle-alert", { size: 18 }));
  el.banner.appendChild(
    h("span", { class: "banner-text", text: "This page changed in the repo after your local draft was started. Keeping the draft will overwrite those changes on the next save." })
  );
  el.banner.appendChild(
    h("button", {
      type: "button",
      class: "btn btn-small",
      text: "Keep my draft",
      on: {
        click: () => {
          state.baseHash = state.conflict.hash;
          state.conflict = null;
          saveDraft(state.slug, serializeBoard(state.board), state.baseHash);
          renderChrome();
        },
      },
    })
  );
  el.banner.appendChild(
    h("button", {
      type: "button",
      class: "btn btn-small btn-primary",
      text: "Use the repo version",
      on: {
        click: () => {
          state.board = state.conflict.board;
          state.baseHash = state.conflict.hash;
          state.conflict = null;
          clearDraft(state.slug);
          resetHistory();
          searchIndex.update(state.slug, state.board);
          renderSidebar();
          renderView();
        },
      },
    })
  );
}

// ----- Mode -----

function applyMode() {
  const editing = state.mode === "edit";
  document.body.classList.toggle("mode-edit", editing);
  if (editing) setButton(el.modeToggle, "check", "Done", "Finish editing (E)");
  else setButton(el.modeToggle, "pencil", "Edit", "Edit this page (E)");
  el.modeToggle.classList.toggle("is-active", editing);
  if (!editing) closeMenus();
  renderView();
}

function setMode(mode) {
  if (mode === state.mode) return;
  commitEditors();
  state.mode = mode;
  if (mode !== "edit") state.selectedId = null;
  applyMode();
}

// ----- Toolbar -----

function setButton(btn, iconName, label, ariaLabel) {
  btn.innerHTML = "";
  btn.appendChild(icon(iconName));
  if (label) btn.appendChild(h("span", { class: "btn-label", text: label }));
  if (ariaLabel) {
    btn.setAttribute("aria-label", ariaLabel);
    btn.setAttribute("title", ariaLabel);
  }
}

function setupToolbarIcons() {
  setButton($("nav-toggle"), "menu", "", "Pages");
  setButton($("add-page"), "plus", "", "New page");
  setButton($("search-btn"), "search", "Search", "Search pages (Ctrl+K)");
  $("search-btn").appendChild(h("kbd", { text: "Ctrl K" }));
  setButton($("undo-btn"), "undo-2", "", "Undo (Ctrl+Z)");
  setButton($("redo-btn"), "redo-2", "", "Redo (Ctrl+Shift+Z)");
  setButton($("settings-page"), "settings", "", "Page settings");
  setButton($("delete-page"), "trash-2", "", "Delete page");
  setButton(el.saveBtn, "save", "Save", "Publish all changes to GitHub (Ctrl+S)");
  setButton($("token-btn"), "key-round", "", "GitHub token");
  setButton($("refresh-btn"), "refresh", "", "Discard local drafts and reload from the repo");
  setButton($("mobile-add"), "plus", "", "Add a card");
}

// ----- Mobile navigation drawer -----

function openNav() {
  document.body.classList.add("nav-open");
  $("sidebar-backdrop").hidden = false;
}

function closeNav() {
  document.body.classList.remove("nav-open");
  $("sidebar-backdrop").hidden = true;
}

function setupNav() {
  $("nav-toggle").addEventListener("click", () => {
    if (document.body.classList.contains("nav-open")) closeNav();
    else openNav();
  });
  $("sidebar-backdrop").addEventListener("click", closeNav);
  $("mobile-add").addEventListener("click", () => {
    if (state.mode !== "edit") setMode("edit");
    if (layout() === "doc") {
      const last = state.board.cards[state.board.cards.length - 1];
      docView.insertAfter(last ? last.id : null, { id: "text" });
      return;
    }
    const r = el.board.getBoundingClientRect();
    openCardMenu(r.left + r.width / 2, r.top + r.height / 2);
  });
}

// ----- Canvas add card menu (edit mode only) -----

let menuEl = null;
let menuBoardPoint = { x: 0, y: 0 };
const lastPointer = { x: 0, y: 0, overBoard: false };

// What the canvas add menu offers. Tables and definition lists are text cards with starter
// Markdown, the same elements the text editor inserts, so they look and edit the same everywhere.
const CANVAS_ADD = [
  { id: "text", label: "Text card", icon: "type", type: "text" },
  { id: "note", label: "Note", icon: "sticky-note", type: "text", extra: { variant: "note", w: 224, h: 160 }, desc: "A warm tinted card, also the N tool" },
  { id: "plain", label: "Plain text", icon: "text", type: "text", extra: { variant: "plain", w: 280, h: 44 }, desc: "Words on the board without a panel, also the T tool" },
  { id: "entry", label: "Entry panel", icon: "square", type: "text", extra: { variant: "entry" }, block: "entry" },
  { id: "table", label: "Table", icon: "table", type: "text", extra: { md: STARTERS.table, w: 360 }, block: "table" },
  { id: "deflist", label: "Definition list", icon: "table-properties", type: "text", extra: { md: STARTERS.deflist, w: 360 }, block: "deflist" },
  { id: "code", label: "Code", icon: TYPE_ICON.code, type: "code" },
  { id: "image", label: "Image", icon: TYPE_ICON.image, type: "image", block: "image" },
  { id: "video", label: "Video", icon: TYPE_ICON.video, type: "video" },
  { id: "shape", label: "Shape", icon: TYPE_ICON.shape, type: "shape" },
  { id: "draw", label: "Draw", icon: TYPE_ICON.draw, type: "draw" },
];

function addFromMenu(entry, at) {
  const card = addCardAt(entry.type, at, entry.extra ? Object.assign({}, entry.extra) : null);
  const block = entry.block && blockById(entry.block);
  if (block && block.then) boardView.applyBlock(blockById(block.then));
  return card;
}

function setupCardMenu() {
  menuEl = h("div", { class: "card-menu", role: "menu", "aria-label": "Add a card", hidden: true });
  menuEl.appendChild(h("div", { class: "suggest-title", text: "Add card" }));
  CANVAS_ADD.forEach((entry) => {
    const block = entry.block && blockById(entry.block);
    menuEl.appendChild(
      h(
        "button",
        {
          type: "button",
          class: "menu-item",
          dataset: { type: entry.id },
          role: "menuitem",
          title: entry.desc || (block && block.desc) || null,
          on: {
            click: () => {
              const at = menuBoardPoint;
              closeCardMenu();
              addFromMenu(entry, at);
            },
          },
        },
        [icon(entry.icon), h("span", { text: entry.label })]
      )
    );
  });
  document.body.appendChild(menuEl);

  el.board.addEventListener("pointermove", (e) => {
    lastPointer.x = e.clientX;
    lastPointer.y = e.clientY;
    lastPointer.overBoard = true;
  });
  el.board.addEventListener("pointerleave", () => {
    lastPointer.overBoard = false;
  });
  el.board.addEventListener("contextmenu", (e) => {
    if (state.mode !== "edit") return;
    e.preventDefault();
    openCardMenu(e.clientX, e.clientY);
  });

  document.addEventListener("pointerdown", (e) => {
    if (!menuEl.hidden && !menuEl.contains(e.target)) closeCardMenu();
    if (cardCtxEl && !cardCtxEl.hidden && !cardCtxEl.contains(e.target)) closeCardContextMenu();
    if (pageMenuEl && !pageMenuEl.hidden && !pageMenuEl.contains(e.target)) closePageMenu();
  });
}

function placeMenu(menu, clientX, clientY) {
  menu.hidden = false;
  const mw = menu.offsetWidth || 190;
  const mh = menu.offsetHeight || 240;
  let left = clientX;
  let top = clientY;
  if (left + mw > window.innerWidth - 8) left = window.innerWidth - mw - 8;
  if (top + mh > window.innerHeight - 8) top = window.innerHeight - mh - 8;
  menu.style.left = Math.max(8, left) + "px";
  menu.style.top = Math.max(8, top) + "px";
}

function openCardMenu(clientX, clientY) {
  closeCardContextMenu();
  menuBoardPoint = boardView.clientToBoard(clientX, clientY);
  placeMenu(menuEl, clientX, clientY);
}

function closeCardMenu() {
  if (menuEl) menuEl.hidden = true;
}

// ----- Canvas card context menu -----

let cardCtxEl = null;

function setupCardContextMenu() {
  cardCtxEl = h("div", { class: "card-menu card-ctx-menu", role: "menu", "aria-label": "Card actions", hidden: true });
  document.body.appendChild(cardCtxEl);
}

function ctxItem(iconName, label, onClick, danger) {
  return h(
    "button",
    {
      type: "button",
      class: "menu-item" + (danger ? " danger" : ""),
      role: "menuitem",
      on: {
        click: () => {
          closeMenus();
          onClick();
        },
      },
    },
    [icon(iconName), h("span", { text: label })]
  );
}

function openCardContextMenu(cardId, clientX, clientY) {
  const card = state.board.cards.find((c) => c.id === cardId);
  if (!card || !cardCtxEl) return;
  closeCardMenu();
  cardCtxEl.innerHTML = "";
  if (card.type === "text") {
    cardCtxEl.appendChild(ctxItem("hash", "Edit source", () => boardView.beginEdit(card, true)));
    const styles = [
      ["", "square", "Card"],
      ["note", "sticky-note", "Note"],
      ["plain", "type", "Plain text"],
      ["entry", "square", "Entry panel"],
    ];
    styles.forEach(([variant, iconName, label]) => {
      if ((card.variant || "") === variant) return;
      cardCtxEl.appendChild(ctxItem(iconName, "Style: " + label, () => doSetVariant(cardId, variant)));
    });
  }
  cardCtxEl.appendChild(ctxItem("copy", "Duplicate", () => cardAction("duplicate", cardId)));
  cardCtxEl.appendChild(
    card.collapsed
      ? ctxItem("chevrons-up-down", "Expand", () => doToggleCollapse(cardId))
      : ctxItem("chevrons-down-up", "Collapse", () => doToggleCollapse(cardId))
  );
  cardCtxEl.appendChild(
    card.hidden ? ctxItem("eye", "Show to readers", () => doSetHidden(cardId, false)) : ctxItem("eye-off", "Hide from readers", () => doSetHidden(cardId, true))
  );
  cardCtxEl.appendChild(ctxItem("bring-to-front", "Bring to front", () => doBringToFront(cardId)));
  cardCtxEl.appendChild(ctxItem("send-to-back", "Send to back", () => doSendToBack(cardId)));
  cardCtxEl.appendChild(ctxItem("trash-2", "Delete", () => doDeleteCard(cardId), true));
  placeMenu(cardCtxEl, clientX, clientY);
}

function closeCardContextMenu() {
  if (cardCtxEl) cardCtxEl.hidden = true;
}

function closeMenus() {
  closeCardMenu();
  closeCardContextMenu();
  closePageMenu();
  closeSuggest();
}

// ----- Sidebar page menu -----

let pageMenuEl = null;

function openPageMenu(slug, clientX, clientY) {
  if (!pageMenuEl) {
    pageMenuEl = h("div", { class: "card-menu page-ctx-menu", role: "menu", "aria-label": "Page actions", hidden: true });
    document.body.appendChild(pageMenuEl);
  }
  closeCardMenu();
  closeCardContextMenu();
  pageMenuEl.innerHTML = "";
  pageMenuEl.appendChild(ctxItem("settings", "Page settings", () => openPageSettings(slug)));
  pageMenuEl.appendChild(ctxItem("trash-2", "Delete page", () => deletePageSlug(slug), true));
  placeMenu(pageMenuEl, clientX, clientY);
}

function closePageMenu() {
  if (pageMenuEl) pageMenuEl.hidden = true;
}

// ----- Canvas zoom controls -----

function setupCanvasControls() {
  setButton($("zoom-out"), "minus", "", "Zoom out");
  setButton($("zoom-in"), "plus", "", "Zoom in");
  setButton($("zoom-fit"), "maximize", "", "Fit to content");
  setButton($("zoom-reset"), "rotate-ccw", "", "Zoom to 100% (Shift 0)");
  $("zoom-fit").setAttribute("title", "Fit to content (Shift 1)");
  $("zoom-label").setAttribute("title", "Zoom to 100%");
  $("zoom-label").addEventListener("click", () => boardView.resetView());
  $("zoom-out").addEventListener("click", () => boardView.zoomOutAt());
  $("zoom-in").addEventListener("click", () => boardView.zoomInAt());
  $("zoom-fit").addEventListener("click", () => boardView.fit());
  $("zoom-reset").addEventListener("click", () => boardView.resetView());
}

function wireToolbar() {
  el.modeToggle.addEventListener("click", () => setMode(state.mode === "edit" ? "view" : "edit"));
  el.saveBtn.addEventListener("click", () => save());
  $("add-page").addEventListener("click", () => openPageSettings(null));
  $("settings-page").addEventListener("click", () => state.slug && openPageSettings(state.slug));
  $("delete-page").addEventListener("click", () => state.slug && deletePageSlug(state.slug));
  $("undo-btn").addEventListener("click", () => undo());
  $("redo-btn").addEventListener("click", () => redo());
  $("search-btn").addEventListener("click", () => openSearch());
  $("token-btn").addEventListener("click", () => openTokenModal());
  $("refresh-btn").addEventListener("click", refreshFromRepo);
  const onNav = () => {
    const slug = parseHash();
    if (slug && slug !== state.slug && state.pages.some((p) => p.slug === slug)) openPage(slug);
  };
  window.addEventListener("hashchange", onNav);
  window.addEventListener("popstate", onNav);
}

function refreshFromRepo() {
  const ok = window.confirm("Discard every local draft and reload the wiki from the repo? Unsaved edits on this device will be lost.");
  if (!ok) return;
  clearAllDrafts();
  location.reload();
}

function isTyping() {
  const ae = document.activeElement;
  const tag = (ae && ae.tagName) || "";
  return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || (ae && ae.isContentEditable);
}

function wireKeyboard() {
  document.addEventListener("keydown", (e) => {
    const mod = e.ctrlKey || e.metaKey;
    const key = e.key.toLowerCase();
    if (mod && key === "s") {
      e.preventDefault();
      save();
      return;
    }
    if (document.querySelector(".search-modal, .modal:not([hidden])")) return;
    if (e.key === "Escape") {
      const menuOpen = (menuEl && !menuEl.hidden) || (cardCtxEl && !cardCtxEl.hidden) || (pageMenuEl && !pageMenuEl.hidden);
      if (menuOpen) {
        e.preventDefault();
        closeMenus();
        return;
      }
      if (layout() === "canvas" && !e.defaultPrevented && !isTyping() && boardView.escape()) e.preventDefault();
      return;
    }
    if (isTyping()) return;
    if (mod && key === "k") {
      e.preventDefault();
      openSearch();
      return;
    }
    if (mod && key === "z") {
      e.preventDefault();
      if (e.shiftKey) redo();
      else undo();
      return;
    }
    if (mod && key === "y") {
      e.preventDefault();
      redo();
      return;
    }
    if (!mod && !e.altKey && key === "e") {
      e.preventDefault();
      setMode(state.mode === "edit" ? "view" : "edit");
      return;
    }
    if (layout() === "canvas" && boardView.handleKey(e)) return;
    if (state.mode !== "edit") return;
    if (layout() === "canvas" && e.shiftKey && key === "a") {
      e.preventDefault();
      const r = el.board.getBoundingClientRect();
      if (lastPointer.overBoard) openCardMenu(lastPointer.x, lastPointer.y);
      else openCardMenu(r.left + r.width / 2, r.top + r.height / 2);
      return;
    }
    if ((e.key === "Delete" || e.key === "Backspace") && state.selectedId && !state.editingId) {
      e.preventDefault();
      doDeleteCard(state.selectedId);
    }
  });
}

function openSearch() {
  commitEditors();
  if (state.slug) searchIndex.update(state.slug, state.board);
  openSearchDialog(searchIndex, (slug, cardId) => {
    if (slug === state.slug) {
      if (cardId) revealCard(cardId);
    } else openPage(slug, cardId);
  });
}

// ----- Paste images, gifs and SVG -----

function pastePoint() {
  if (lastPointer.overBoard) return boardView.clientToBoard(lastPointer.x, lastPointer.y);
  return boardView.spawnPoint();
}

function isImageUrl(text) {
  if (/^data:image\//i.test(text)) return true;
  if (!/^https?:\/\//i.test(text)) return false;
  return /\.(png|jpe?g|gif|webp|svg|avif|bmp)(\?.*)?$/i.test(text);
}

function readAsDataUrl(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = () => reject(new Error("Could not read the pasted image."));
    reader.readAsDataURL(file);
  });
}

// With a token, pasted images are committed as assets. Without one they are embedded as data
// URLs, which works but makes the page file large.
async function imageSource(file) {
  if (hasToken()) {
    const rel = await uploadAsset(file);
    if (rel) return rel;
  }
  return readAsDataUrl(file);
}

async function pasteFiles(files, afterId) {
  if (state.mode !== "edit") setMode("edit");
  for (let i = 0; i < files.length; i += 1) {
    const file = files[i];
    let src;
    try {
      src = await imageSource(file);
    } catch (err) {
      setStatus(err.message, "error");
      continue;
    }
    if (layout() === "doc") {
      const card = docView.insertAfter(afterId || lastCardId(), { card: "image" });
      card.src = src;
      card.alt = "";
      markDirty();
      docView.render({ keepScroll: true });
      afterId = card.id;
    } else {
      const at = pastePoint();
      const card = addCardAt("image", { x: at.x + i * 24, y: at.y + i * 24 });
      card.src = src;
      markDirty();
      boardView.refreshCard(card);
    }
  }
  setStatus(files.length > 1 ? "Pasted " + files.length + " images." : "Pasted an image.", "success");
}

function lastCardId() {
  const c = state.board.cards[state.board.cards.length - 1];
  return c ? c.id : null;
}

function placeImage(src, extra) {
  if (layout() === "doc") {
    const card = docView.insertAfter(lastCardId(), { card: "image" });
    Object.assign(card, extra, src != null ? { src } : {});
    if (!card.svg) delete card.svg;
    markDirty();
    docView.render({ keepScroll: true });
    return card;
  }
  const card = addCardAt("image", pastePoint());
  Object.assign(card, extra, src != null ? { src } : {});
  if (!card.svg) delete card.svg;
  markDirty();
  boardView.refreshCard(card);
  return card;
}

function handlePastedText(text) {
  const t = (text || "").trim();
  if (!t) return false;
  if (/^<svg[\s>]/i.test(t) && /<\/svg>/i.test(t)) {
    placeImage(null, { svg: t });
    setStatus("Pasted SVG.", "success");
    return true;
  }
  if (isImageUrl(t)) {
    placeImage(t, {});
    setStatus("Pasted image link.", "success");
    return true;
  }
  return false;
}

function wirePaste() {
  document.addEventListener("paste", (e) => {
    if (state.mode !== "edit") return;
    // A focused field or text block handles its own paste.
    if (isTyping()) return;
    const dt = e.clipboardData;
    if (!dt) return;
    const files = Array.from(dt.files || []).filter((f) => f.type.startsWith("image/"));
    if (files.length) {
      e.preventDefault();
      pasteFiles(files, null);
      return;
    }
    if (handlePastedText(dt.getData("text/plain") || "")) e.preventDefault();
  });
}

// ----- Card actions -----

function addCardAt(type, at, extra) {
  if (state.mode !== "edit") setMode("edit");
  const card = Object.assign(newCard(type, at, state.board.cards), extra || {});
  state.board.cards.push(card);
  markDirty();
  boardView.render();
  selectCard(card.id);
  if (type === "text" || type === "code" || type === "table") boardView.beginEdit(card);
  return card;
}

function insertBelowOnCanvas(card, type, block) {
  const node = el.board.querySelector('.card[data-id="' + CSS.escape(card.id) + '"]');
  const height = node ? node.offsetHeight : card.h;
  addCardAt(type, { x: card.x, y: card.y + height + 24 }, block && block.variant ? { variant: block.variant } : null);
  if (block && block.then) boardView.applyBlock(blockById(block.then));
}

function selectCard(id) {
  state.selectedId = id;
  if (layout() === "canvas") boardView.setSelection(id ? [id] : []);
}

function cardAction(action, id) {
  if (action === "delete") doDeleteCard(id);
  else if (action === "hide") doSetHidden(id, true);
  else if (action === "show") doSetHidden(id, false);
  else if (action === "duplicate") doDuplicate(id);
}

function rerender() {
  if (layout() === "doc") docView.render({ keepScroll: true });
  else {
    boardView.render();
    boardView.updateSelection();
  }
}

function doDeleteCard(id) {
  if (!id) return;
  doDeleteMany([id], []);
}

// Removing a card also removes the connectors attached to it.
function doDeleteMany(ids, edgeIds) {
  const gone = new Set(ids || []);
  const goneEdges = new Set(edgeIds || []);
  if (!gone.size && !goneEdges.size) return;
  commitEditors();
  state.board.cards = state.board.cards.filter((c) => !gone.has(c.id));
  if (Array.isArray(state.board.edges)) {
    state.board.edges = state.board.edges.filter((e) => !goneEdges.has(e.id) && !gone.has(e.from) && !gone.has(e.to));
  }
  if (gone.has(state.selectedId)) state.selectedId = null;
  markDirty();
  rerender();
  const what = gone.size > 1 ? gone.size + " cards deleted." : gone.size ? "Block deleted." : "Connector deleted.";
  setStatus(what + " Ctrl+Z brings it back.", "info");
}

function doDuplicate(id) {
  commitEditors();
  const i = state.board.cards.findIndex((c) => c.id === id);
  if (i < 0) return;
  const copy = JSON.parse(JSON.stringify(state.board.cards[i]));
  copy.id = newCard(copy.type, { x: 0, y: 0 }, []).id;
  if (layout() === "canvas") {
    copy.x += 24;
    copy.y += 24;
    copy.z = state.board.cards.length;
  }
  state.board.cards.splice(i + 1, 0, copy);
  markDirty();
  rerender();
}

function doBringToFront(id) {
  bringToFront(state.board.cards, id);
  state.selectedId = id;
  markDirty();
  rerender();
}

function doSendToBack(id) {
  sendToBack(state.board.cards, id);
  state.selectedId = id;
  markDirty();
  rerender();
}

function doToggleCollapse(id) {
  const card = state.board.cards.find((c) => c.id === id);
  if (!card) return;
  card.collapsed = !card.collapsed;
  markDirty();
  rerender();
}

function doSetVariant(id, variant) {
  const card = state.board.cards.find((c) => c.id === id);
  if (!card) return;
  commitEditors();
  if (variant) card.variant = variant;
  else delete card.variant;
  markDirty();
  boardView.refreshCard(card);
}

function doSetHidden(id, hidden) {
  const card = state.board.cards.find((c) => c.id === id);
  if (!card) return;
  commitEditors();
  card.hidden = hidden;
  markDirty();
  rerender();
}

// ----- Drafts and undo -----

const undoStack = { back: [], forward: [], last: null };

function snapshot() {
  return JSON.stringify(state.board);
}

function resetHistory() {
  undoStack.back = [];
  undoStack.forward = [];
  undoStack.last = snapshot();
  updateDirtyUi();
}

// Every mutation ends up here, so it is also where undo steps are recorded.
function markDirty() {
  if (!state.slug) return;
  const now = snapshot();
  if (now !== undoStack.last) {
    if (undoStack.last != null) undoStack.back.push(undoStack.last);
    if (undoStack.back.length > 200) undoStack.back.shift();
    undoStack.forward = [];
    undoStack.last = now;
  }
  saveDraft(state.slug, serializeBoard(state.board), state.baseHash);
  searchIndex.update(state.slug, state.board);
  updateDirtyUi();
}

function restore(json) {
  state.board = JSON.parse(json);
  undoStack.last = json;
  saveDraft(state.slug, serializeBoard(state.board), state.baseHash);
  searchIndex.update(state.slug, state.board);
  rerender();
  renderChrome();
}

function undo() {
  commitEditors();
  if (!undoStack.back.length) return;
  undoStack.forward.push(undoStack.last);
  restore(undoStack.back.pop());
}

function redo() {
  commitEditors();
  if (!undoStack.forward.length) return;
  undoStack.back.push(undoStack.last);
  restore(undoStack.forward.pop());
}

function hasUnsaved() {
  return listDraftSlugs().length > 0 || !!loadIndexDraft() || state.deletedPages.length > 0;
}

let lastDirtyKey = "";
function updateDirtyUi() {
  el.saveBtn.classList.toggle("has-changes", hasUnsaved());
  $("undo-btn").disabled = !undoStack.back.length;
  $("redo-btn").disabled = !undoStack.forward.length;
  const key = listDraftSlugs().sort().join(",");
  if (key !== lastDirtyKey) {
    lastDirtyKey = key;
    renderSidebar();
  }
}

// ----- Page management -----

function uniqueSlug(title, except) {
  const base = slugify(title);
  let slug = base;
  let n = 2;
  while (state.pages.some((p) => p.slug === slug && p.slug !== except)) {
    slug = base + "-" + n;
    n += 1;
  }
  return slug;
}

function openPageSettings(slug) {
  const entry = slug ? pageInfo(slug) : null;
  const modal = $("page-modal");
  const title = $("page-title-input");
  const cat = $("page-cat-input");
  const cats = $("page-cats");
  const heading = $("page-modal-title");
  const submit = $("page-save");
  heading.textContent = entry ? "Page settings" : "New page";
  submit.textContent = entry ? "Apply" : "Create page";
  title.value = entry ? entry.title || "" : "";
  cat.value = entry ? entry.category || "" : lastCategory;
  cats.innerHTML = "";
  Array.from(new Set(state.pages.map(pageCategory))).forEach((c) => cats.appendChild(h("option", { value: c })));
  const current = entry ? entry.layout || (slug === state.slug ? layout() : "doc") : "doc";
  modal.querySelectorAll("input[name=layout]").forEach((r) => {
    r.checked = r.value === current;
  });
  modal.hidden = false;
  title.focus();
  title.select();

  const close = () => {
    modal.hidden = true;
    submit.removeEventListener("click", onSubmit);
    $("page-cancel").removeEventListener("click", close);
    modal.removeEventListener("keydown", onKey);
  };
  const onKey = (e) => {
    if (e.key === "Escape") close();
    if (e.key === "Enter" && e.target.tagName === "INPUT" && e.target.type === "text") onSubmit();
  };
  const onSubmit = () => {
    const t = title.value.trim();
    if (!t) {
      title.focus();
      return;
    }
    const category = cat.value.trim() || DEFAULT_CATEGORY;
    const chosen = (modal.querySelector("input[name=layout]:checked") || {}).value || "doc";
    lastCategory = category === DEFAULT_CATEGORY ? "" : category;
    close();
    if (entry) applyPageSettings(entry, t, category, chosen);
    else createPage(t, category, chosen);
  };
  submit.addEventListener("click", onSubmit);
  $("page-cancel").addEventListener("click", close);
  modal.addEventListener("keydown", onKey);
}

let lastCategory = "";

function createPage(title, category, lay) {
  const slug = uniqueSlug(title);
  state.deletedPages = state.deletedPages.filter((s) => s !== slug);
  state.pages.push({ slug, title, category, layout: lay });
  saveIndex();
  const board = {
    title,
    layout: lay,
    cards: lay === "doc" ? [Object.assign(newCard("text", { x: 0, y: 0 }, []), { md: "# " + title })] : [],
  };
  saveDraft(slug, board, null);
  renderSidebar();
  state.mode = "edit";
  openPage(slug).then(() => {
    applyMode();
    if (lay === "doc" && state.board.cards[0]) docView.focusCard(state.board.cards[0].id, true, true);
  });
  setStatus('Created "' + title + '". Save to publish it.', "info");
}

function applyPageSettings(entry, title, category, lay) {
  entry.title = title;
  entry.category = category;
  const layoutChanged = (entry.layout || "canvas") !== lay;
  entry.layout = lay;
  saveIndex();
  if (entry.slug === state.slug) {
    commitEditors();
    state.board.title = title;
    if (layoutChanged || state.board.layout !== lay) convertLayout(lay);
    markDirty();
    renderView();
    if (lay === "canvas") boardView.fit();
  } else if (layoutChanged) {
    const draft = loadDraft(entry.slug);
    setStatus("Open the page to finish switching its layout.", "info");
    if (draft) {
      draft.board.layout = lay;
      saveDraft(entry.slug, draft.board, draft.base);
    }
  }
  renderSidebar();
  renderChrome();
  setStatus("Page settings updated. Save to publish.", "info");
}

// Documents keep cards in reading order; a canvas keeps positions. Switching stacks the cards
// in a column one way, and reads them top to bottom, left to right the other way.
function convertLayout(lay) {
  const cards = state.board.cards;
  if (lay === "canvas") {
    let y = 40;
    cards.forEach((c, i) => {
      const block = el.doc.querySelector('.block[data-id="' + CSS.escape(c.id) + '"]');
      const height = block ? block.offsetHeight : c.h || 160;
      c.x = 40;
      c.y = y;
      c.w = c.type === "image" || c.type === "video" || c.type === "shape" || c.type === "draw" ? 480 : 760;
      c.h = c.type === "text" || c.type === "code" || c.type === "table" ? 60 : Math.max(80, height);
      c.z = i;
      y += height + 24;
    });
  } else {
    cards.sort((a, b) => (a.y || 0) - (b.y || 0) || (a.x || 0) - (b.x || 0));
  }
  state.board.layout = lay;
}

function deletePageSlug(slug) {
  const entry = pageInfo(slug);
  if (!entry) return;
  const label = entry.title || slug;
  if (!window.confirm('Delete "' + label + '"? The page is removed from the site and the repo on the next save.')) return;
  state.pages = state.pages.filter((p) => p.slug !== slug);
  clearDraft(slug);
  if (!state.deletedPages.includes(slug)) state.deletedPages.push(slug);
  saveIndex();
  renderSidebar();
  if (slug === state.slug) {
    const next = state.pages.length ? state.pages[0].slug : null;
    if (next) openPage(next);
    else {
      state.slug = null;
      state.board = { title: "", layout: "doc", cards: [] };
      renderView();
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

let saving = false;

async function save() {
  if (saving) return;
  commitEditors();
  if (!hasUnsaved()) {
    setStatus("Everything is already published.", "info");
    return;
  }
  const token = await ensureToken();
  if (!token) {
    setStatus("Save cancelled, no token provided.", "info");
    return;
  }

  saving = true;
  el.saveBtn.disabled = true;
  setButton(el.saveBtn, "save", "Saving", "Saving");
  setStatus("Checking the repo for changes...", "info");

  try {
    const allDraftSlugs = listDraftSlugs().filter((s) => state.pages.some((p) => p.slug === s));

    // Every drafted page is checked against what's actually on the repo right now, not just the
    // page that happens to be open, so a draft for a page nobody reopened this session still gets
    // a chance to conflict instead of silently overwriting someone else's change.
    const remote = new Map();
    await Promise.all(
      allDraftSlugs.map(async (s) => {
        try {
          const loaded = await fetchJSON("pages/" + s + ".json");
          if (!loaded) {
            remote.set(s, { hash: null, board: null });
            return;
          }
          const info = pageInfo(s);
          const board = normalizeBoard(loaded, info ? info.title : s, info && info.layout);
          remote.set(s, { hash: boardHash(board), board });
        } catch (err) {
          remote.set(s, { hash: undefined, board: null }); // Could not check; save rather than block on a network blip.
        }
      })
    );

    const toSave = [];
    const conflicted = [];
    allDraftSlugs.forEach((s) => {
      const base = s === state.slug ? state.baseHash : (loadDraft(s) || {}).base;
      const r = remote.get(s);
      if (!r || r.hash === undefined || (base || null) === (r.hash || null)) toSave.push(s);
      else conflicted.push(s);
    });

    if (conflicted.length) {
      conflicted.forEach((s) => {
        const title = (pageInfo(s) || {}).title || s;
        const keepMine = window.confirm(
          '"' + title + '" changed on the repo since your local draft started.\n\n' +
            "OK keeps your local version and overwrites the repo.\nCancel discards your local draft and takes the repo version."
        );
        if (keepMine) {
          toSave.push(s);
          return;
        }
        clearDraft(s);
        const r = remote.get(s);
        if (s === state.slug && r && r.board) {
          state.board = r.board;
          state.baseHash = r.hash;
          state.conflict = null;
          resetHistory();
          searchIndex.update(s, state.board);
          renderView();
        }
      });
      renderSidebar();
    }

    let freshIndex = null;
    try {
      const fetched = await fetchJSON("pages/index.json");
      freshIndex = Array.isArray(fetched) ? fetched : [];
    } catch (err) {
      // Could not verify the live index; fall back to publishing the local one as-is.
    }
    const mergedIndex = freshIndex ? mergeIndexForSave(freshIndex) : state.pages;

    const boards = new Map();
    toSave.forEach((s) => {
      if (s === state.slug) {
        if (layout() === "canvas") normalizeZ(state.board.cards);
        boards.set(s, serializeBoard(state.board));
      } else {
        const d = loadDraft(s);
        if (d) boards.set(s, serializeBoard(normalizeBoard(d.board, s)));
      }
    });
    const files = Array.from(boards.entries()).map(([s, b]) => ({ path: "pages/" + s + ".json", content: JSON.stringify(b, null, 2) + "\n" }));
    files.push({ path: "pages/index.json", content: JSON.stringify(mergedIndex, null, 2) + "\n" });
    const deletes = state.deletedPages.filter((s) => !state.pages.some((p) => p.slug === s)).map((s) => "pages/" + s + ".json");

    const names = Array.from(boards.keys()).map((s) => (pageInfo(s) || {}).title || s);
    const parts = [];
    if (names.length === 1) parts.push("update " + names[0]);
    else if (names.length > 1 && names.length <= 3) parts.push("update " + names.join(", "));
    else if (names.length) parts.push("update " + names.length + " pages");
    if (deletes.length) parts.push("delete " + deletes.length + (deletes.length === 1 ? " page" : " pages"));
    const message = "wiki: " + (parts.length ? parts.join(", ") : "update the page index");

    setStatus("Committing to " + GH.owner + "/" + GH.repo + "...", "info");
    const res = await commitFiles(files, deletes, message, token);
    boards.forEach((b, s) => {
      savePending("page_" + s, b, hashString(JSON.stringify(b)));
      // Someone may have kept typing while the commit was in flight; only clear the draft if it
      // still matches what was actually published, so those in-flight edits are not dropped.
      const draftNow = loadDraft(s);
      if (draftNow && JSON.stringify(draftNow.board) === JSON.stringify(b)) clearDraft(s);
      if (s === state.slug) {
        state.baseHash = hashString(JSON.stringify(b));
        state.conflict = null;
        state.publishing = true;
      }
    });
    savePending("index", mergedIndex, null);
    deletes.forEach((d) => clearPending("page_" + d.replace(/^pages\/|\.json$/g, "")));
    state.pages = mergedIndex;
    state.indexBase = mergedIndex.map((p) => Object.assign({}, p));
    clearIndexDraft();
    state.deletedPages = [];
    searchIndex.setPages(state.pages);
    renderChrome();
    renderSidebar();
    const base = res.unchanged ? "The repo already had these changes." : "Published. GitHub Pages redeploys in a minute or so.";
    const skipped = conflicted.filter((s) => !toSave.includes(s)).map((s) => (pageInfo(s) || {}).title || s);
    setStatus(skipped.length ? base + " Took the repo version for: " + skipped.join(", ") + "." : base, "success");
  } catch (err) {
    setStatus("Save failed: " + err.message, "error");
  } finally {
    saving = false;
    el.saveBtn.disabled = false;
    setButton(el.saveBtn, "save", "Save", "Publish all changes to GitHub (Ctrl+S)");
    updateDirtyUi();
  }
}

async function uploadAsset(file) {
  const token = await ensureToken();
  if (!token) {
    setStatus("Upload cancelled, no token provided.", "info");
    return null;
  }
  setStatus("Uploading " + (file.name || "image") + "...", "info");
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
      input.removeEventListener("keydown", onKey);
      resolve();
    };
    const onSave = () => {
      const value = input.value.trim();
      if (!value) {
        setStatus("The token was empty.", "error");
        return;
      }
      setToken(value);
      setStatus("Token stored in this browser only.", "success");
      close();
    };
    const onClear = () => {
      clearToken();
      setStatus("Token removed from this browser.", "info");
      close();
    };
    const onCancel = () => close();
    const onBackdrop = (e) => {
      if (e.target === modal) close();
    };
    const onKey = (e) => {
      if (e.key === "Enter") onSave();
      if (e.key === "Escape") close();
    };

    saveBtn.addEventListener("click", onSave);
    clearBtn.addEventListener("click", onClear);
    cancelBtn.addEventListener("click", onCancel);
    modal.addEventListener("click", onBackdrop);
    input.addEventListener("keydown", onKey);
  });
}

// ----- Status line -----

let statusTimer = null;
function setStatus(message, kind) {
  el.status.textContent = message;
  el.status.className = "status show " + (kind || "info");
  if (statusTimer) clearTimeout(statusTimer);
  statusTimer = setTimeout(
    () => {
      el.status.className = "status";
    },
    kind === "error" ? 9000 : 4000
  );
}

// A small hook for headless verification. It reads state and drives the UI the way a user would.
window.__wiki = {
  getState: () => ({
    slug: state.slug,
    mode: state.mode,
    layout: layout(),
    selectedId: state.selectedId,
    editingId: state.editingId,
    cardCount: state.board.cards.length,
    pageCount: state.pages.length,
    deletedPages: state.deletedPages.slice(),
    conflict: !!state.conflict,
    publishing: state.publishing,
    undo: undoStack.back.length,
    redo: undoStack.forward.length,
    unsaved: hasUnsaved(),
    view: layout() === "canvas" ? boardView.getView() : null,
    cards: state.board.cards.map((c) => Object.assign({}, c)),
    edges: (state.board.edges || []).map((e) => Object.assign({}, e)),
    selection: layout() === "canvas" ? boardView.getSelection() : null,
    tool: boardView.tool,
  }),
  serialize: () => serializeBoard(state.board),
  setMode,
  openPage,
  addCardAt,
  selectCard,
  undo,
  redo,
  toggleCollapse: doToggleCollapse,
  setHidden: doSetHidden,
  pasteText: handlePastedText,
  categories: () => Array.from(new Set(state.pages.map((p) => pageCategory(p)))),
  getView: () => boardView.getView(),
  addEdge: (from, to) => boardView.addEdge(from, to),
  setTool: (t) => boardView.setTool(t),
  save: () => save(),
  setView: (v) => boardView.setView(v),
  fit: () => boardView.fit(),
  clientToBoard: (x, y) => boardView.clientToBoard(x, y),
  search: (q) => searchIndex.ensure().then(() => searchIndex.search(q).map((r) => ({ slug: r.page.slug, cardId: r.cardId }))),
  backlinks: (slug) => searchIndex.backlinks(slug).then((ps) => ps.map((p) => p.slug)),
};

init();
