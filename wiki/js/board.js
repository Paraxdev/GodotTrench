// The canvas layout: an open, infinite board in the spirit of Excalidraw and Scrintal. One
// transform on the world container pans and zooms everything. Cards move by dragging them,
// double click edits in place, Shift drag on empty space draws a selection box, and connectors
// are arrows bound to card ids so they follow the cards around.

import { renderContent } from "./cards.js";
import { icon } from "./icons.js";
// Imported as h because board.js uses el as a local variable name for card elements.
import { el as h } from "./dom.js";
import {
  mountTextEditor,
  mountCodeEditor,
  mountTableEditor,
  mountSourceEditor,
  mountSelectionControls,
  attachDrawing,
} from "./edit.js";

const SVG_NS = "http://www.w3.org/2000/svg";
const MIN_W = 90;
const MIN_H = 40;
const MIN_SCALE = 0.1;
const MAX_SCALE = 4;
const DOT = 16;

// Types that enter a full inline edit on double click. The rest use on-card controls.
const EDITABLE_TYPES = { text: 1, code: 1, table: 1 };
// These grow with their content; h is their minimum height.
const AUTO_HEIGHT = { text: 1, code: 1, table: 1 };

const RESIZE_DIRS = ["n", "e", "s", "w", "nw", "ne", "se", "sw"];
const CONNECT_SIDES = ["n", "e", "s", "w"];

const TOOLS = [
  { id: "select", icon: "mouse-pointer-2", title: "Select and move", key: "V" },
  { id: "hand", icon: "hand", title: "Pan, or hold Space", key: "H" },
  null,
  { id: "note", icon: "sticky-note", title: "Add a note", key: "N" },
  { id: "text", icon: "type", title: "Add text", key: "T" },
  { id: "connect", icon: "spline", title: "Connect two cards", key: "C" },
];
const EDIT_TOOLS = { note: 1, text: 1, connect: 1 };

function clamp(v, lo, hi) {
  return Math.max(lo, Math.min(hi, v));
}

function svg(tag, attrs) {
  const node = document.createElementNS(SVG_NS, tag);
  if (attrs) Object.keys(attrs).forEach((k) => node.setAttribute(k, attrs[k]));
  return node;
}

function edgeId() {
  return "e_" + Date.now().toString(36) + "_" + Math.random().toString(36).slice(2, 6);
}

export class BoardView {
  // opts: { getMode, getBoard, getSelectedId, onSelect, onMutate, onBringToFront, onSendToBack,
  //         onDelete, onDeleteMany, onToggleCollapse, onSetHidden, onCardMenu, onEditingChange,
  //         onEnsureEdit, onCreate, onAddMenu, uploadAsset, onViewChange, onZoomChange, pages,
  //         pasteFiles, onInsertBelow, grid }
  constructor(viewport, opts) {
    this.viewport = viewport;
    this.opts = opts;
    this.grid = opts.grid || 8;
    this.view = { panX: 0, panY: 0, scale: 1 };
    this.tool = "select";
    this.sel = new Set();
    this.selEdge = null;
    this.editingId = null;
    this._editor = null;
    this._g = null;
    this._touches = new Map();
    this._pinch = null;
    this._space = false;
    this._sizes = new Map();
    this._edgeEls = new Map();
    this._frame = 0;
    this._dirty = { view: false, edges: false };

    this.gridEl = h("div", { class: "board-grid", "aria-hidden": "true" });
    this.surface = h("div", { class: "board-surface" });
    this.edgeSvg = svg("svg", { class: "board-edges", "aria-hidden": "true" });
    this.edgeLayer = h("div", { class: "board-edge-layer" });
    this.overlay = svg("svg", { class: "board-overlay", "aria-hidden": "true" });
    this.marquee = h("div", { class: "board-marquee", hidden: true });
    this.toolbar = this._buildToolbar();
    this.viewport.innerHTML = "";
    this.viewport.append(this.gridEl, this.surface, this.overlay, this.marquee, this.toolbar);

    this._ro = new ResizeObserver((entries) => {
      entries.forEach((entry) => {
        const id = entry.target.dataset.id;
        const box = entry.borderBoxSize && entry.borderBoxSize[0];
        if (id && box) this._sizes.set(id, { w: box.inlineSize, h: box.blockSize });
      });
      this._queue("edges");
    });

    this._wire();
    this.applyTransform();
  }

  // ----- Frame scheduling: every visual update lands in one animation frame -----

  _queue(what) {
    this._dirty[what] = true;
    if (this._frame) return;
    this._frame = requestAnimationFrame(() => {
      this._frame = 0;
      if (this._dirty.view) this.applyTransform();
      if (this._dirty.edges) this._drawEdges();
      this._dirty.view = false;
      this._dirty.edges = false;
    });
  }

  // ----- Pan / zoom -----

  applyTransform() {
    const { scale } = this.view;
    const dpr = window.devicePixelRatio || 1;
    const panX = Math.round(this.view.panX * dpr) / dpr;
    const panY = Math.round(this.view.panY * dpr) / dpr;
    this.surface.style.transform = "translate(" + panX + "px, " + panY + "px) scale(" + scale + ")";
    // A layer that keeps will-change is rasterized once and then stretched, which blurs text at any
    // zoom but the first. Promote it only while the view moves, so it repaints sharp once it settles.
    this.surface.classList.add("moving");
    clearTimeout(this._settle);
    this._settle = setTimeout(() => this.surface.classList.remove("moving"), 160);
    let cell = DOT * scale;
    while (cell < 12) cell *= 2;
    if (cell !== this._cell) {
      this._cell = cell;
      this.gridEl.style.backgroundSize = cell + "px " + cell + "px";
      this.gridEl.style.width = "calc(100% + " + cell * 2 + "px)";
      this.gridEl.style.height = "calc(100% + " + cell * 2 + "px)";
    }
    const ox = (((panX % cell) + cell) % cell) - cell;
    const oy = (((panY % cell) + cell) % cell) - cell;
    this.gridEl.style.transform = "translate(" + ox + "px, " + oy + "px)";
    if (this.opts.onZoomChange) this.opts.onZoomChange(scale);
  }

  getView() {
    return { panX: this.view.panX, panY: this.view.panY, scale: this.view.scale };
  }

  setView(v) {
    if (!v) return;
    this.view.panX = Number(v.panX) || 0;
    this.view.panY = Number(v.panY) || 0;
    this.view.scale = clamp(Number(v.scale) || 1, MIN_SCALE, MAX_SCALE);
    this.applyTransform();
  }

  panBy(dx, dy) {
    this.view.panX += dx;
    this.view.panY += dy;
    this._queue("view");
  }

  // Zoom by factor, keeping the board point under (clientX, clientY) fixed on screen.
  zoomAt(clientX, clientY, factor) {
    const r = this.viewport.getBoundingClientRect();
    const cx = clientX - r.left;
    const cy = clientY - r.top;
    const s2 = clamp(this.view.scale * factor, MIN_SCALE, MAX_SCALE);
    if (s2 === this.view.scale) return;
    this.view.panX = cx - (cx - this.view.panX) * (s2 / this.view.scale);
    this.view.panY = cy - (cy - this.view.panY) * (s2 / this.view.scale);
    this.view.scale = s2;
    this._queue("view");
  }

  _zoomCenter(factor) {
    const r = this.viewport.getBoundingClientRect();
    this.zoomAt(r.left + r.width / 2, r.top + r.height / 2, factor);
    this._persist();
  }

  zoomInAt() {
    this._zoomCenter(1.2);
  }

  zoomOutAt() {
    this._zoomCenter(1 / 1.2);
  }

  // 100% around the center of the view, the way Excalidraw's zoom reset works.
  resetView() {
    this._zoomCenter(1 / this.view.scale);
  }

  // Frame all visible cards. With readable set, as when a page opens, a board too big to read
  // when framed whole (a phone) opens at a legible zoom on its top left corner instead.
  fit(readable) {
    const cards = this._visibleCards();
    if (!cards.length) {
      this.view = { panX: 0, panY: 0, scale: 1 };
      this.applyTransform();
      this._persist();
      return;
    }
    let minX = Infinity;
    let minY = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;
    for (const c of cards) {
      const r = this._rect(c);
      minX = Math.min(minX, r.x);
      minY = Math.min(minY, r.y);
      maxX = Math.max(maxX, r.x + r.w);
      maxY = Math.max(maxY, r.y + r.h);
    }
    const pad = 56;
    const vr = this.viewport.getBoundingClientRect();
    const topInset = 64;
    const scale = clamp(Math.min(vr.width / (maxX - minX + pad * 2), (vr.height - topInset) / (maxY - minY + pad * 2)), MIN_SCALE, 1.2);
    if (readable === true && scale < 0.5) {
      const s2 = Math.min(0.62, (vr.width - 24) / Math.max(1, Math.min(maxX - minX, 620)));
      this.view.scale = s2;
      this.view.panX = 12 - minX * s2;
      this.view.panY = topInset + 8 - minY * s2;
      this.applyTransform();
      this._persist();
      return;
    }
    this.view.scale = scale;
    this.view.panX = (vr.width - (maxX - minX) * scale) / 2 - minX * scale;
    // Centered across, pinned near the top: on a tall phone screen the page starts where
    // reading starts instead of floating in the middle.
    this.view.panY = topInset + Math.min((vr.height - topInset - (maxY - minY) * scale) / 2, 28) - minY * scale;
    this.applyTransform();
    this._persist();
  }

  _persist() {
    if (this.opts.onViewChange) this.opts.onViewChange(this.getView());
  }

  // ----- Coordinates -----

  clientToBoard(clientX, clientY) {
    const r = this.viewport.getBoundingClientRect();
    return {
      x: Math.round((clientX - r.left - this.view.panX) / this.view.scale),
      y: Math.round((clientY - r.top - this.view.panY) / this.view.scale),
    };
  }

  boardToClient(x, y) {
    const r = this.viewport.getBoundingClientRect();
    return { x: r.left + this.view.panX + x * this.view.scale, y: r.top + this.view.panY + y * this.view.scale };
  }

  // The center of the current view, in board coordinates, for placing new cards.
  spawnPoint() {
    const r = this.viewport.getBoundingClientRect();
    return this.clientToBoard(r.left + r.width / 2, r.top + r.height / 2);
  }

  snap(value) {
    return Math.round(value / this.grid) * this.grid;
  }

  // ----- Model helpers -----

  _board() {
    return this.opts.getBoard();
  }

  _edit() {
    return this.opts.getMode() === "edit";
  }

  _card(id) {
    return this._board().cards.find((c) => c.id === id) || null;
  }

  _cardEl(id) {
    return this.surface.querySelector('.card[data-id="' + cssEscape(id) + '"]');
  }

  _visibleCards() {
    const cards = this._board().cards;
    return this._edit() ? cards : cards.filter((c) => !c.hidden);
  }

  _edges() {
    return Array.isArray(this._board().edges) ? this._board().edges : [];
  }

  // A card's box in board units. Auto height cards are as tall as their content, so the
  // measured size wins over the stored minimum.
  _rect(card) {
    let size = this._sizes.get(card.id);
    if (!size) {
      const node = this._cardEl(card.id);
      if (node) {
        size = { w: node.offsetWidth, h: node.offsetHeight };
        this._sizes.set(card.id, size);
      }
    }
    return { x: card.x || 0, y: card.y || 0, w: size ? size.w : card.w || 0, h: size ? size.h : card.h || 0 };
  }

  // ----- Selection -----

  setSelection(ids) {
    this.sel = new Set((ids || []).filter((id) => this._card(id)));
    if (this.sel.size) this.selEdge = null;
    this._applySelection();
    this._emitSelection();
  }

  _emitSelection() {
    const ids = Array.from(this.sel);
    this.opts.onSelect(ids.length ? ids[ids.length - 1] : null, ids);
  }

  // Kept for the app, which tracks one primary selected card.
  updateSelection() {
    const id = this.opts.getSelectedId();
    if (id == null) this.sel.clear();
    else if (!this.sel.has(id)) this.sel = new Set([id]);
    this._applySelection();
  }

  getSelection() {
    return { cards: Array.from(this.sel), edge: this.selEdge };
  }

  _applySelection() {
    const edit = this._edit();
    this.surface.querySelectorAll(".card").forEach((node) => {
      node.classList.toggle("selected", edit && this.sel.has(node.dataset.id));
    });
    this.surface.classList.toggle("single-sel", this.sel.size === 1);
    this._edgeEls.forEach((parts, id) => {
      const on = edit && id === this.selEdge;
      parts.g.classList.toggle("selected", on);
      parts.label.classList.toggle("selected", on);
    });
    this._placeEdgeMenu();
  }

  selectEdge(id) {
    this.selEdge = id;
    this.sel.clear();
    this._applySelection();
    this._emitSelection();
  }

  selectAll() {
    this.setSelection(this._visibleCards().map((c) => c.id));
  }

  // ----- Tools -----

  setTool(tool) {
    if (!TOOLS.some((t) => t && t.id === tool)) return;
    if (EDIT_TOOLS[tool] && !this._edit() && this.opts.onEnsureEdit) this.opts.onEnsureEdit();
    this.tool = tool;
    this.viewport.dataset.tool = tool;
    this.toolbar.querySelectorAll("[data-tool]").forEach((b) => {
      const on = b.dataset.tool === tool;
      b.classList.toggle("active", on);
      b.setAttribute("aria-pressed", String(on));
    });
  }

  _buildToolbar() {
    const bar = h("div", { class: "board-toolbar", role: "toolbar", "aria-label": "Canvas tools" });
    TOOLS.forEach((t) => {
      if (!t) {
        bar.appendChild(h("span", { class: "board-tool-sep", "aria-hidden": "true" }));
        return;
      }
      bar.appendChild(
        h(
          "button",
          {
            type: "button",
            class: "board-tool" + (t.id === this.tool ? " active" : ""),
            title: t.title + " (" + t.key + ")",
            "aria-label": t.title,
            "aria-pressed": String(t.id === this.tool),
            dataset: { tool: t.id },
            on: { click: () => this.setTool(t.id) },
          },
          [icon(t.icon, { size: 17 }), h("kbd", { class: "board-tool-key", text: t.key })]
        )
      );
    });
    bar.appendChild(h("span", { class: "board-tool-sep", "aria-hidden": "true" }));
    const more = h(
      "button",
      {
        type: "button",
        class: "board-tool",
        title: "Add a card (Shift A)",
        "aria-label": "Add a card",
        on: {
          click: () => {
            const r = more.getBoundingClientRect();
            if (this.opts.onAddMenu) this.opts.onAddMenu(r.left, r.bottom + 8);
          },
        },
      },
      icon("plus", { size: 17 })
    );
    bar.appendChild(more);
    return bar;
  }

  // ----- Rendering -----

  render() {
    const board = this._board();
    const mode = this.opts.getMode();
    const edit = mode === "edit";
    this.editingId = null;
    this._editor = null;
    this._g = null;
    this._ro.disconnect();
    this._sizes.clear();
    this.viewport.classList.toggle("is-edit", edit);
    this.surface.classList.toggle("is-edit", edit);
    if (!edit && EDIT_TOOLS[this.tool]) this.setTool("select");
    this.surface.innerHTML = "";
    this.surface.appendChild(this.edgeSvg);
    const cards = this._visibleCards()
      .slice()
      .sort((a, b) => (a.z || 0) - (b.z || 0));
    if (!cards.length) {
      this.surface.appendChild(
        h("div", {
          class: "board-empty",
          text: edit ? "Pick a tool above, right click, or press Shift A to add a card" : "This page is empty",
        })
      );
    }
    cards.forEach((card) => {
      const node = this._renderCard(card, edit);
      this.surface.appendChild(node);
      this._ro.observe(node);
    });
    this.surface.appendChild(this.edgeLayer);
    this.sel.forEach((id) => {
      if (!this._card(id)) this.sel.delete(id);
    });
    if (this.selEdge && !this._edges().some((e) => e.id === this.selEdge)) this.selEdge = null;
    this._buildEdges();
    this._applySelection();
  }

  refreshCard(card) {
    const node = this._cardEl(card.id);
    if (!node || (!!card.collapsed !== node.classList.contains("collapsed")) || (!!card.hidden !== node.classList.contains("is-hidden"))) {
      this.render();
      return;
    }
    this._place(node, card);
    applyHeight(node, card);
    node.style.zIndex = String((card.z || 0) + 1);
    if (card.variant) node.dataset.variant = card.variant;
    else delete node.dataset.variant;
    const body = node.querySelector(".card-body");
    if (body) {
      body.innerHTML = "";
      body.appendChild(this._content(card));
      if (card.type === "draw" && this._edit()) this._attachDraw(body, card);
    }
    this._queue("edges");
  }

  _content(card) {
    const node = renderContent(card, { rich: true });
    node.querySelectorAll("img, a").forEach((n) => n.setAttribute("draggable", "false"));
    return node;
  }

  _place(node, card) {
    node.style.left = (card.x || 0) + "px";
    node.style.top = (card.y || 0) + "px";
    node.style.width = (card.w || 200) + "px";
  }

  _renderCard(card, edit) {
    const collapsed = !!card.collapsed;
    const cardEl = h("div", {
      class: "card",
      dataset: { id: card.id, type: card.type },
      style: { zIndex: String((card.z || 0) + 1) },
    });
    this._place(cardEl, card);
    if (card.variant) cardEl.dataset.variant = card.variant;
    if (collapsed) cardEl.classList.add("collapsed");
    if (card.hidden) cardEl.classList.add("is-hidden");
    if (!collapsed) applyHeight(cardEl, card);

    if (collapsed) {
      cardEl.appendChild(h("div", { class: "card-collapsed-label", text: card.type + (card.hidden ? " · hidden" : "") }));
    }

    const body = h("div", { class: "card-body" });
    if (!collapsed) body.appendChild(this._content(card));
    cardEl.appendChild(body);

    if (!edit) return cardEl;

    cardEl.appendChild(this._actionsBar(card));
    cardEl.addEventListener("contextmenu", (e) => {
      e.preventDefault();
      e.stopPropagation();
      this.commitEdit();
      if (!this.sel.has(card.id)) this.setSelection([card.id]);
      if (this.opts.onCardMenu) this.opts.onCardMenu(card.id, e.clientX, e.clientY);
    });
    if (!collapsed) {
      RESIZE_DIRS.forEach((dir) => cardEl.appendChild(h("div", { class: "card-handle handle-" + dir, dataset: { dir } })));
      CONNECT_SIDES.forEach((side) =>
        cardEl.appendChild(h("div", { class: "connect-dot dot-" + side, title: "Drag to another card to connect", dataset: { side } }))
      );
      if (card.type === "draw") this._attachDraw(body, card);
      mountSelectionControls(cardEl, card, this._ctx(card, cardEl, body));
    }
    return cardEl;
  }

  _ctx(card, node, body) {
    return {
      markDirty: () => this.opts.onMutate(),
      rerenderContent: () => {
        body.innerHTML = "";
        body.appendChild(this._content(card));
        if (card.type === "draw") this._attachDraw(body, card);
      },
      uploadAsset: (f) => this.opts.uploadAsset(f),
      requestCommit: () => this.commitEdit(),
      reopen: () => this.beginEdit(card),
      insertBlock: (type, extra) => {
        this.commitEdit();
        if (this.opts.onInsertBelow) this.opts.onInsertBelow(card, type, extra && extra.block);
      },
      pasteFiles: (files) => this.opts.pasteFiles && this.opts.pasteFiles(files, card.id),
      pages: () => (this.opts.pages ? this.opts.pages() : []),
    };
  }

  // The floating action bar: move grip, collapse, layer, hide, delete.
  _actionsBar(card) {
    const grip = h("div", { class: "card-grip", title: "Drag to move" }, icon("grip-vertical", { size: 15 }));
    const bar = h("div", { class: "card-actions" }, grip);
    const mkBtn = (name, title, fn, danger) =>
      h(
        "button",
        {
          type: "button",
          class: "card-act" + (danger ? " danger" : ""),
          title,
          "aria-label": title,
          on: {
            click: (e) => {
              e.stopPropagation();
              this.commitEdit();
              fn();
            },
          },
        },
        icon(name, { size: 15 })
      );
    bar.appendChild(
      mkBtn(card.collapsed ? "chevrons-up-down" : "chevrons-down-up", card.collapsed ? "Expand" : "Collapse", () =>
        this.opts.onToggleCollapse(card.id)
      )
    );
    bar.appendChild(mkBtn("bring-to-front", "Bring to front", () => this.opts.onBringToFront(card.id)));
    bar.appendChild(mkBtn("send-to-back", "Send to back", () => this.opts.onSendToBack(card.id)));
    bar.appendChild(
      mkBtn(card.hidden ? "eye" : "eye-off", card.hidden ? "Show" : "Hide", () => this.opts.onSetHidden(card.id, !card.hidden))
    );
    bar.appendChild(mkBtn("x", "Delete card", () => this.opts.onDelete(card.id), true));
    return bar;
  }

  // ----- Connectors -----

  _buildEdges() {
    this.edgeSvg.innerHTML = "";
    this.edgeLayer.innerHTML = "";
    this._edgeEls.clear();
    const edges = this._edges();
    edges.forEach((edge) => {
      const g = svg("g", { class: "edge", "data-edge": edge.id });
      const hit = svg("path", { class: "edge-hit" });
      const line = svg("path", { class: "edge-line" });
      const head = svg("path", { class: "edge-head" });
      g.append(hit, line, head);
      this.edgeSvg.appendChild(g);
      const label = h("div", { class: "edge-label", dataset: { edge: edge.id } });
      this.edgeLayer.appendChild(label);
      this._edgeEls.set(edge.id, { g, hit, line, head, label });
    });
    this.edgeMenu = this._buildEdgeMenu();
    this.edgeLayer.appendChild(this.edgeMenu);
    this._drawEdges();
  }

  _buildEdgeMenu() {
    const btn = (name, title, fn, danger) =>
      h(
        "button",
        {
          type: "button",
          class: "card-act" + (danger ? " danger" : ""),
          title,
          "aria-label": title,
          on: {
            click: (e) => {
              e.stopPropagation();
              fn();
            },
          },
        },
        icon(name, { size: 15 })
      );
    return h("div", { class: "edge-menu card-actions", hidden: true }, [
      btn("pencil-line", "Label", () => this.selEdge && this.editEdgeLabel(this.selEdge)),
      btn("arrow-left-right", "Reverse direction", () => this._reverseEdge(this.selEdge)),
      btn("trash-2", "Delete connector", () => this.deleteSelection(), true),
    ]);
  }

  _placeEdgeMenu() {
    if (!this.edgeMenu) return;
    const parts = this.selEdge && this._edgeEls.get(this.selEdge);
    const show = !!(parts && parts.mid && this._edit());
    this.edgeMenu.hidden = !show;
    if (!show) return;
    this.edgeMenu.style.left = parts.mid.x + "px";
    this.edgeMenu.style.top = parts.mid.y + "px";
  }

  _drawEdges() {
    const byId = new Map(this._visibleCards().map((c) => [c.id, c]));
    let minX = Infinity;
    let minY = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;
    this._edges().forEach((edge) => {
      const parts = this._edgeEls.get(edge.id);
      if (!parts) return;
      const a = byId.get(edge.from);
      const b = byId.get(edge.to);
      const ok = a && b && a !== b;
      parts.g.style.display = ok ? "" : "none";
      parts.label.hidden = !ok || !edge.label;
      if (!ok) {
        parts.mid = null;
        return;
      }
      const r = route(this._rect(a), this._rect(b));
      const d = "M" + r.p1.x + " " + r.p1.y + " C" + r.c1.x + " " + r.c1.y + " " + r.c2.x + " " + r.c2.y + " " + r.end.x + " " + r.end.y;
      parts.hit.setAttribute("d", d);
      parts.line.setAttribute("d", d);
      parts.head.setAttribute("d", r.head);
      parts.mid = r.mid;
      parts.label.textContent = edge.label || "";
      parts.label.style.left = r.mid.x + "px";
      parts.label.style.top = r.mid.y + "px";
      [r.p1, r.c1, r.c2, r.p2].forEach((p) => {
        minX = Math.min(minX, p.x);
        minY = Math.min(minY, p.y);
        maxX = Math.max(maxX, p.x);
        maxY = Math.max(maxY, p.y);
      });
    });
    // The SVG only spans the connectors, so it never becomes a huge layer.
    if (minX === Infinity) {
      this.edgeSvg.style.display = "none";
    } else {
      const pad = 24;
      const x = Math.floor(minX - pad);
      const y = Math.floor(minY - pad);
      const w = Math.ceil(maxX - minX + pad * 2);
      const hh = Math.ceil(maxY - minY + pad * 2);
      this.edgeSvg.style.display = "";
      this.edgeSvg.style.left = x + "px";
      this.edgeSvg.style.top = y + "px";
      this.edgeSvg.style.width = w + "px";
      this.edgeSvg.style.height = hh + "px";
      this.edgeSvg.setAttribute("viewBox", x + " " + y + " " + w + " " + hh);
    }
    this._placeEdgeMenu();
  }

  addEdge(from, to) {
    if (!from || !to || from === to) return null;
    const board = this._board();
    if (!Array.isArray(board.edges)) board.edges = [];
    const existing = board.edges.find((e) => e.from === from && e.to === to);
    if (existing) {
      this._buildEdges();
      this.selectEdge(existing.id);
      return existing;
    }
    const edge = { id: edgeId(), from, to };
    board.edges.push(edge);
    this.opts.onMutate();
    this._buildEdges();
    this.selectEdge(edge.id);
    return edge;
  }

  _reverseEdge(id) {
    const edge = this._edges().find((e) => e.id === id);
    if (!edge) return;
    const t = edge.from;
    edge.from = edge.to;
    edge.to = t;
    this.opts.onMutate();
    this._drawEdges();
  }

  editEdgeLabel(id) {
    const edge = this._edges().find((e) => e.id === id);
    const parts = this._edgeEls.get(id);
    if (!edge || !parts || !parts.mid) return;
    if (!this._edit() && this.opts.onEnsureEdit) {
      this.opts.onEnsureEdit();
      this.editEdgeLabel(id);
      return;
    }
    this.selectEdge(id);
    const p = this.boardToClient(parts.mid.x, parts.mid.y);
    const vr = this.viewport.getBoundingClientRect();
    const input = h("input", {
      type: "text",
      class: "edge-label-input",
      value: edge.label || "",
      placeholder: "Label",
      spellcheck: false,
      style: { left: p.x - vr.left + "px", top: p.y - vr.top + "px" },
    });
    let done = false;
    const finish = (keep) => {
      if (done) return;
      done = true;
      const value = input.value.trim();
      input.remove();
      if (!keep || value === (edge.label || "")) return;
      if (value) edge.label = value;
      else delete edge.label;
      this.opts.onMutate();
      this._drawEdges();
    };
    input.addEventListener("keydown", (e) => {
      e.stopPropagation();
      if (e.key === "Enter") finish(true);
      else if (e.key === "Escape") {
        e.preventDefault();
        finish(false);
      }
    });
    input.addEventListener("blur", () => finish(true));
    this.viewport.appendChild(input);
    input.focus();
    input.select();
  }

  deleteSelection() {
    const cards = Array.from(this.sel);
    const edges = this.selEdge ? [this.selEdge] : [];
    if (!cards.length && !edges.length) return false;
    this.commitEdit();
    this.sel.clear();
    this.selEdge = null;
    if (this.opts.onDeleteMany) this.opts.onDeleteMany(cards, edges);
    return true;
  }

  // ----- Inline editing -----

  beginEdit(card, source) {
    if (!card || this.editingId === card.id) return;
    if (!this._edit() && this.opts.onEnsureEdit) this.opts.onEnsureEdit();
    this.commitEdit();
    const node = this._cardEl(card.id);
    if (!node) return;
    const body = node.querySelector(".card-body");
    if (!body) return;
    if (!this.sel.has(card.id) || this.sel.size > 1) this.setSelection([card.id]);
    this.editingId = card.id;
    node.classList.add("editing");
    const ctx = this._ctx(card, node, body);
    if (card.type === "code") this._editor = mountCodeEditor(body, card, ctx);
    else if (card.type === "table") this._editor = mountTableEditor(body, card, ctx);
    else if (source || (card.format || "markdown") === "bbcode") this._editor = mountSourceEditor(body, card, ctx);
    else {
      // The editor works on plain rendered Markdown, without the reading presentation.
      body.innerHTML = "";
      body.appendChild(renderContent(card));
      this._editor = mountTextEditor(body.querySelector(".md") || body, card, ctx);
    }
    if (this.opts.onEditingChange) this.opts.onEditingChange(card.id);
  }

  // Apply a block from the catalogue to the text being edited, as the slash menu would.
  applyBlock(block) {
    if (block && this._editor && this._editor.applyBlock) this._editor.applyBlock(block);
  }

  commitEdit() {
    this._finishEdit("commit");
  }

  cancelEdit() {
    this._finishEdit("cancel");
  }

  _finishEdit(how) {
    if (!this.editingId) return;
    const id = this.editingId;
    const card = this._card(id);
    const editor = this._editor;
    this._editor = null;
    this.editingId = null;
    if (editor) {
      try {
        editor[how]();
      } catch (err) {
        // The editor's DOM may already be gone.
      }
    }
    const node = this._cardEl(id);
    if (node) node.classList.remove("editing");
    if (card) this.refreshCard(card);
    if (this.opts.onEditingChange) this.opts.onEditingChange(null);
  }

  // ----- Input -----

  _wire() {
    const vp = this.viewport;
    vp.addEventListener("wheel", (e) => this._onWheel(e), { passive: false });
    vp.addEventListener("pointerdown", (e) => this._onDown(e));
    vp.addEventListener("pointermove", (e) => this._onMove(e));
    vp.addEventListener("pointerup", (e) => this._onUp(e));
    vp.addEventListener("pointercancel", (e) => this._onUp(e, true));
    // Touch pointers start out implicitly captured by the element under the finger, and moving
    // that capture to the viewport fires a bubbling lostpointercapture from the child first.
    vp.addEventListener("lostpointercapture", (e) => {
      if (e.target !== vp) return;
      if (this._g && this._g.pid === e.pointerId && this._g.captured) this._onUp(e, true);
    });
    vp.addEventListener("dblclick", (e) => this._onDoubleClick(e));
    // Native drag of a link, an image or selected text is what left a ghost copy behind a card.
    vp.addEventListener("dragstart", (e) => e.preventDefault());
    // Safari pinch gestures, handled through the ctrl+wheel path instead.
    vp.addEventListener("gesturestart", (e) => e.preventDefault());

    window.addEventListener("keydown", (e) => {
      if (e.key !== " " || this.viewport.hidden || isTyping() || e.repeat) return;
      if (document.activeElement && document.activeElement !== document.body && !this.viewport.contains(document.activeElement)) return;
      e.preventDefault();
      this._space = true;
      this.viewport.classList.add("space-pan");
    });
    window.addEventListener("keyup", (e) => {
      if (e.key !== " ") return;
      this._space = false;
      this.viewport.classList.remove("space-pan");
    });
    window.addEventListener("blur", () => {
      this._space = false;
      this.viewport.classList.remove("space-pan");
    });
  }

  _onWheel(e) {
    if (e.target.closest && e.target.closest(".edge-label-input")) return;
    e.preventDefault();
    const unit = e.deltaMode === 1 ? 16 : e.deltaMode === 2 ? 400 : 1;
    const dx = e.deltaX * unit;
    const dy = e.deltaY * unit;
    if (e.ctrlKey || e.metaKey) {
      // Trackpad pinches arrive as small ctrl+wheel deltas, a mouse wheel as large steps.
      const factor = Math.abs(dy) < 40 ? Math.exp(-dy * 0.012) : dy > 0 ? 1 / 1.18 : 1.18;
      this.zoomAt(e.clientX, e.clientY, factor);
    } else if (e.shiftKey && !dx) {
      this.panBy(-dy, 0);
    } else {
      this.panBy(-dx, -dy);
    }
    clearTimeout(this._wheelTimer);
    this._wheelTimer = setTimeout(() => this._persist(), 200);
  }

  _onDown(e) {
    const t = e.target;
    if (t.closest(".board-toolbar, .edge-label-input, .inline-controls, .card-act, .edge-menu")) return;

    if (e.pointerType === "touch") {
      this._touches.set(e.pointerId, { x: e.clientX, y: e.clientY });
      if (this._touches.size === 2) {
        this._abortGesture();
        const pts = Array.from(this._touches.values());
        this._pinch = { d: Math.hypot(pts[0].x - pts[1].x, pts[0].y - pts[1].y), m: midpoint(pts[0], pts[1]) };
        return;
      }
      if (this._touches.size > 2) return;
    }
    if (this._g || this._pinch) return;

    const edit = this._edit();
    const cardEl = t.closest(".card");
    const card = cardEl && this.surface.contains(cardEl) ? this._card(cardEl.dataset.id) : null;
    const onGrip = !!t.closest(".card-grip");

    // Text inside the card being edited belongs to the editor: selection works normally there.
    if (card && this.editingId === card.id && !onGrip && !t.closest(".card-handle, .connect-dot")) return;

    const touchReading = e.pointerType === "touch" && !edit;
    if (e.button === 1 || (e.button === 0 && (this._space || this.tool === "hand" || touchReading))) {
      e.preventDefault();
      this._startPan(e, false);
      return;
    }
    if (e.button !== 0) return;
    if (this.editingId && (!card || card.id !== this.editingId)) this.commitEdit();

    if (edit && card) {
      const handle = t.closest(".card-handle");
      if (handle) return this._startResize(e, card, cardEl, handle.dataset.dir);
      if (t.closest(".connect-dot")) return this._startConnect(e, card);
    }
    const edgeNode = t.closest("[data-edge]");
    if (edit && edgeNode && !card) {
      e.preventDefault();
      this.selectEdge(edgeNode.dataset.edge);
      return;
    }
    if (this.tool === "note" || this.tool === "text") {
      e.preventDefault();
      this._createAt(e);
      return;
    }
    if (card) {
      if (this.tool === "connect" && edit) return this._startConnect(e, card);
      return this._startMove(e, card);
    }
    if (edit && (e.shiftKey || this.tool === "connect")) {
      e.preventDefault();
      if (this.tool === "connect") return this._startPan(e, true);
      return this._startMarquee(e);
    }
    e.preventDefault();
    this._startPan(e, true);
  }

  _onMove(e) {
    if (this._touches.has(e.pointerId)) this._touches.set(e.pointerId, { x: e.clientX, y: e.clientY });
    if (this._pinch) {
      const pts = Array.from(this._touches.values());
      if (pts.length < 2) return;
      const d = Math.hypot(pts[0].x - pts[1].x, pts[0].y - pts[1].y);
      const m = midpoint(pts[0], pts[1]);
      if (this._pinch.d > 0) this.zoomAt(m.x, m.y, d / this._pinch.d);
      this.panBy(m.x - this._pinch.m.x, m.y - this._pinch.m.y);
      this._pinch = { d, m };
      return;
    }
    const g = this._g;
    if (!g || g.pid !== e.pointerId) return;
    const dx = e.clientX - g.sx;
    const dy = e.clientY - g.sy;
    if (!g.started) {
      const threshold = e.pointerType === "touch" ? 8 : 4;
      if (Math.abs(dx) + Math.abs(dy) < threshold) return;
      g.started = true;
      this._capture(g, e);
      if (g.onStart) g.onStart();
    }
    g.onMove(e, dx, dy);
  }

  _onUp(e, cancelled) {
    this._touches.delete(e.pointerId);
    if (this._pinch) {
      if (this._touches.size < 2) {
        this._pinch = null;
        this._persist();
      }
      return;
    }
    const g = this._g;
    if (!g || g.pid !== e.pointerId) return;
    this._g = null;
    if (g.captured) {
      try {
        this.viewport.releasePointerCapture(e.pointerId);
      } catch (err) {
        // Capture may already be gone.
      }
    }
    g.onEnd(e, !!cancelled);
  }

  _capture(g, e) {
    try {
      this.viewport.setPointerCapture(e.pointerId);
      g.captured = true;
    } catch (err) {
      // Synthetic or already released pointers cannot be captured.
    }
  }

  _abortGesture() {
    const g = this._g;
    if (!g) return;
    this._g = null;
    g.onEnd(null, true);
  }

  _onDoubleClick(e) {
    const t = e.target;
    if (t.closest(".board-toolbar, .inline-controls, .card-actions, .edge-menu, .edge-label-input")) return;
    const edgeNode = t.closest("[data-edge]");
    const cardEl = t.closest(".card");
    if (edgeNode && !cardEl) {
      e.preventDefault();
      this.editEdgeLabel(edgeNode.dataset.edge);
      return;
    }
    if (cardEl) {
      const card = this._card(cardEl.dataset.id);
      if (!card || this.editingId === card.id || !EDITABLE_TYPES[card.type] || card.collapsed) return;
      e.preventDefault();
      this.beginEdit(card);
      return;
    }
    if (this._edit() && this.tool === "select") {
      e.preventDefault();
      this._createAt(e, "text");
    }
  }

  // ----- Gestures -----

  _startPan(e, clickClears) {
    const g = {
      pid: e.pointerId,
      sx: e.clientX,
      sy: e.clientY,
      px: this.view.panX,
      py: this.view.panY,
      started: false,
      onStart: () => this.viewport.classList.add("panning"),
      onMove: (ev, dx, dy) => {
        this.view.panX = g.px + dx;
        this.view.panY = g.py + dy;
        this._queue("view");
      },
      onEnd: () => {
        this.viewport.classList.remove("panning");
        if (g.started) this._persist();
        else if (clickClears && this._edit()) {
          this.commitEdit();
          this.selEdge = null;
          this.setSelection([]);
        }
      },
    };
    this._g = g;
    if (e.button === 1 || this._space || this.tool === "hand") {
      g.started = true;
      this._capture(g, e);
      g.onStart();
    }
  }

  _startMove(e, card) {
    const edit = this._edit();
    let collapseTo = null;
    if (edit) {
      if (e.shiftKey) {
        const next = new Set(this.sel);
        if (next.has(card.id)) next.delete(card.id);
        else next.add(card.id);
        this.setSelection(Array.from(next));
        if (!next.has(card.id)) return;
      } else if (!this.sel.has(card.id)) {
        this.setSelection([card.id]);
      } else if (this.sel.size > 1) {
        collapseTo = card.id;
      }
    }
    const ids = edit && this.sel.has(card.id) ? Array.from(this.sel) : [card.id];
    const items = ids
      .map((id) => this._card(id))
      .filter(Boolean)
      .map((c) => ({ card: c, x: c.x || 0, y: c.y || 0, node: this._cardEl(c.id) }));
    const scale = this.view.scale;
    const g = {
      pid: e.pointerId,
      sx: e.clientX,
      sy: e.clientY,
      started: false,
      onStart: () => items.forEach((it) => it.node && it.node.classList.add("dragging")),
      onMove: (ev, dx, dy) => {
        let nx = card.x;
        let ny = card.y;
        const lead = items.find((it) => it.card === card) || items[0];
        nx = lead.x + dx / scale;
        ny = lead.y + dy / scale;
        if (!ev.altKey) {
          nx = this.snap(nx);
          ny = this.snap(ny);
        } else {
          nx = Math.round(nx);
          ny = Math.round(ny);
        }
        const ddx = nx - lead.x;
        const ddy = ny - lead.y;
        items.forEach((it) => {
          it.card.x = it.x + ddx;
          it.card.y = it.y + ddy;
          if (it.node) {
            it.node.style.left = it.card.x + "px";
            it.node.style.top = it.card.y + "px";
          }
        });
        this._queue("edges");
      },
      onEnd: (ev, cancelled) => {
        items.forEach((it) => it.node && it.node.classList.remove("dragging"));
        if (!g.started) {
          if (collapseTo && !cancelled) this.setSelection([collapseTo]);
          return;
        }
        const moved = items.some((it) => it.card.x !== it.x || it.card.y !== it.y);
        if (!moved) return;
        if (!edit) {
          this.sel = new Set(ids);
          this._emitSelection();
        }
        this.opts.onMutate();
        if (!edit && this.opts.onEnsureEdit) this.opts.onEnsureEdit();
      },
    };
    this._g = g;
  }

  _startMarquee(e) {
    const base = new Set(this.sel);
    const vr = this.viewport.getBoundingClientRect();
    const cards = this._visibleCards().map((c) => ({ id: c.id, r: this._rect(c) }));
    const g = {
      pid: e.pointerId,
      sx: e.clientX,
      sy: e.clientY,
      started: false,
      onStart: () => {
        this.marquee.hidden = false;
      },
      onMove: (ev) => {
        const x1 = Math.min(g.sx, ev.clientX);
        const y1 = Math.min(g.sy, ev.clientY);
        const x2 = Math.max(g.sx, ev.clientX);
        const y2 = Math.max(g.sy, ev.clientY);
        Object.assign(this.marquee.style, {
          left: x1 - vr.left + "px",
          top: y1 - vr.top + "px",
          width: x2 - x1 + "px",
          height: y2 - y1 + "px",
        });
        const a = this.clientToBoard(x1, y1);
        const b = this.clientToBoard(x2, y2);
        const next = new Set(base);
        cards.forEach(({ id, r }) => {
          if (r.x < b.x && r.x + r.w > a.x && r.y < b.y && r.y + r.h > a.y) next.add(id);
        });
        this.sel = next;
        this.selEdge = null;
        this._applySelection();
      },
      onEnd: () => {
        this.marquee.hidden = true;
        this._emitSelection();
      },
    };
    this._g = g;
  }

  _startResize(e, card, cardEl, dir) {
    e.preventDefault();
    e.stopPropagation();
    this.setSelection([card.id]);
    const r = this._rect(card);
    const origin = { x: card.x, y: card.y, w: card.w, h: AUTO_HEIGHT[card.type] ? r.h : card.h };
    const scale = this.view.scale;
    const g = {
      pid: e.pointerId,
      sx: e.clientX,
      sy: e.clientY,
      started: true,
      onMove: (ev, dx0, dy0) => {
        const snap = ev.altKey ? (v) => Math.round(v) : (v) => this.snap(v);
        const dx = dx0 / scale;
        const dy = dy0 / scale;
        const right = origin.x + origin.w;
        const bottom = origin.y + origin.h;
        let { x, y, w } = origin;
        let hh = origin.h;
        if (dir.includes("e")) w = Math.max(MIN_W, snap(right + dx) - origin.x);
        if (dir.includes("s")) hh = Math.max(MIN_H, snap(bottom + dy) - origin.y);
        if (dir.includes("w")) {
          x = Math.min(snap(origin.x + dx), right - MIN_W);
          w = right - x;
        }
        if (dir.includes("n")) {
          y = Math.min(snap(origin.y + dy), bottom - MIN_H);
          hh = bottom - y;
        }
        Object.assign(card, { x, y, w, h: hh });
        this._place(cardEl, card);
        applyHeight(cardEl, card);
        this._queue("edges");
      },
      onEnd: () => {
        cardEl.classList.remove("resizing");
        this.opts.onMutate();
      },
    };
    cardEl.classList.add("resizing");
    this._g = g;
    this._capture(g, e);
  }

  _startConnect(e, card) {
    e.preventDefault();
    e.stopPropagation();
    const vr = this.viewport.getBoundingClientRect();
    const path = svg("path", { class: "connect-draft" });
    this.overlay.appendChild(path);
    let target = null;
    const setTarget = (id) => {
      if (id === target) return;
      if (target) {
        const old = this._cardEl(target);
        if (old) old.classList.remove("connect-target");
      }
      target = id;
      if (target) {
        const n = this._cardEl(target);
        if (n) n.classList.add("connect-target");
      }
    };
    const g = {
      pid: e.pointerId,
      sx: e.clientX,
      sy: e.clientY,
      started: false,
      onStart: () => this.viewport.classList.add("connecting"),
      onMove: (ev) => {
        const under = document.elementFromPoint(ev.clientX, ev.clientY);
        const n = under && under.closest(".card");
        setTarget(n && this.surface.contains(n) && n.dataset.id !== card.id ? n.dataset.id : null);
        const a = this._rect(card);
        const p = this.clientToBoard(ev.clientX, ev.clientY);
        const b = target ? this._rect(this._card(target)) : { x: p.x, y: p.y, w: 0, h: 0 };
        const r = route(a, b);
        const toClient = (q) => {
          const c = this.boardToClient(q.x, q.y);
          return c.x - vr.left + " " + (c.y - vr.top);
        };
        path.setAttribute("d", "M" + toClient(r.p1) + " C" + toClient(r.c1) + " " + toClient(r.c2) + " " + toClient(target ? r.end : p));
      },
      onEnd: (ev, cancelled) => {
        path.remove();
        this.viewport.classList.remove("connecting");
        const to = target;
        setTarget(null);
        if (!g.started) {
          if (this.tool === "connect" || !this.sel.has(card.id)) this.setSelection([card.id]);
          return;
        }
        if (!cancelled && to) this.addEdge(card.id, to);
      },
    };
    this._g = g;
  }

  _createAt(e, forced) {
    const kind = forced || this.tool;
    const at = this.clientToBoard(e.clientX, e.clientY);
    const note = kind === "note";
    const w = note ? 224 : 280;
    const hh = note ? 160 : 44;
    const pos = { x: this.snap(at.x - (note ? w / 2 : 12)), y: this.snap(at.y - (note ? hh / 2 : 22)) };
    if (!forced) this.setTool("select");
    if (this.opts.onCreate) this.opts.onCreate(note ? "note" : "plain", pos, { w, h: hh });
  }

  // Keyboard shortcuts that only make sense on the canvas. Returns true when handled.
  handleKey(e) {
    const mod = e.ctrlKey || e.metaKey;
    const key = e.key.toLowerCase();
    const edit = this._edit();
    if (mod && (key === "=" || key === "+")) {
      e.preventDefault();
      this.zoomInAt();
      return true;
    }
    if (mod && key === "-") {
      e.preventDefault();
      this.zoomOutAt();
      return true;
    }
    if (mod && key === "0") {
      e.preventDefault();
      this.resetView();
      return true;
    }
    if (mod && key === "a" && edit) {
      e.preventDefault();
      this.selectAll();
      return true;
    }
    if (mod || e.altKey) return false;
    if (e.shiftKey && e.code === "Digit1") {
      e.preventDefault();
      this.fit();
      return true;
    }
    if (e.shiftKey && e.code === "Digit0") {
      e.preventDefault();
      this.resetView();
      return true;
    }
    if (e.shiftKey) return false;
    const tool = { v: "select", h: "hand", n: "note", t: "text", c: "connect" }[key];
    if (tool) {
      e.preventDefault();
      this.setTool(tool);
      return true;
    }
    if (!edit) return false;
    if (e.key === "Enter" && this.sel.size === 1) {
      const card = this._card(Array.from(this.sel)[0]);
      if (card && EDITABLE_TYPES[card.type]) {
        e.preventDefault();
        this.beginEdit(card);
        return true;
      }
    }
    if (e.key === "Delete" || e.key === "Backspace") {
      if (this.deleteSelection()) {
        e.preventDefault();
        return true;
      }
    }
    const arrows = { arrowleft: [-1, 0], arrowright: [1, 0], arrowup: [0, -1], arrowdown: [0, 1] };
    if (arrows[key] && this.sel.size) {
      e.preventDefault();
      const step = this.grid;
      this.sel.forEach((id) => {
        const c = this._card(id);
        if (!c) return;
        c.x += arrows[key][0] * step;
        c.y += arrows[key][1] * step;
        const n = this._cardEl(id);
        if (n) this._place(n, c);
      });
      this._queue("edges");
      this.opts.onMutate();
      return true;
    }
    return false;
  }

  // Esc steps back one level: text editing, then the active tool, then the selection.
  escape() {
    if (this._g) {
      this._abortGesture();
      return true;
    }
    if (this.editingId) {
      this.commitEdit();
      return true;
    }
    if (this.tool !== "select") {
      this.setTool("select");
      return true;
    }
    if (this.sel.size || this.selEdge) {
      this.selEdge = null;
      this.setSelection([]);
      return true;
    }
    return false;
  }

  _attachDraw(body, card) {
    attachDrawing(body, card, () => this.view.scale, () => this.opts.onMutate());
  }
}

// A connector leaves the side of one card that faces the other and arrives square on the
// facing side of the target. When the cards overlap on the cross axis the ends line up, so
// cards in a row get straight arrows.
function route(a, b) {
  const ac = { x: a.x + a.w / 2, y: a.y + a.h / 2 };
  const bc = { x: b.x + b.w / 2, y: b.y + b.h / 2 };
  const gapX = Math.max(b.x - (a.x + a.w), a.x - (b.x + b.w));
  const gapY = Math.max(b.y - (a.y + a.h), a.y - (b.y + b.h));
  const horizontal = gapX >= gapY;
  let p1;
  let p2;
  let d1;
  let d2;
  if (horizontal) {
    const right = bc.x >= ac.x;
    const top = Math.max(a.y, b.y);
    const bottom = Math.min(a.y + a.h, b.y + b.h);
    const shared = bottom - top > 24 ? (top + bottom) / 2 : null;
    p1 = { x: right ? a.x + a.w : a.x, y: shared != null ? shared : ac.y };
    p2 = { x: right ? b.x : b.x + b.w, y: shared != null ? shared : bc.y };
    d1 = { x: right ? 1 : -1, y: 0 };
    d2 = { x: right ? -1 : 1, y: 0 };
  } else {
    const down = bc.y >= ac.y;
    const left = Math.max(a.x, b.x);
    const right = Math.min(a.x + a.w, b.x + b.w);
    const shared = right - left > 24 ? (left + right) / 2 : null;
    p1 = { x: shared != null ? shared : ac.x, y: down ? a.y + a.h : a.y };
    p2 = { x: shared != null ? shared : bc.x, y: down ? b.y : b.y + b.h };
    d1 = { x: 0, y: down ? 1 : -1 };
    d2 = { x: 0, y: down ? -1 : 1 };
  }
  const gap = 5;
  p1 = { x: p1.x + d1.x * gap, y: p1.y + d1.y * gap };
  p2 = { x: p2.x + d2.x * gap, y: p2.y + d2.y * gap };
  const k = clamp(Math.hypot(p2.x - p1.x, p2.y - p1.y) * 0.45, 16, 140);
  const c1 = { x: p1.x + d1.x * k, y: p1.y + d1.y * k };
  const c2 = { x: p2.x + d2.x * k, y: p2.y + d2.y * k };
  // The arrowhead points along the final tangent; the line stops at its base.
  const u = { x: -d2.x, y: -d2.y };
  const size = 11;
  const base = { x: p2.x - u.x * size, y: p2.y - u.y * size };
  const perp = { x: -u.y, y: u.x };
  const head =
    "M" + p2.x + " " + p2.y +
    " L" + (base.x + perp.x * 6) + " " + (base.y + perp.y * 6) +
    " L" + (base.x - perp.x * 6) + " " + (base.y - perp.y * 6) + " Z";
  const end = { x: p2.x - u.x * (size - 2), y: p2.y - u.y * (size - 2) };
  const mid = { x: (p1.x + 3 * c1.x + 3 * c2.x + end.x) / 8, y: (p1.y + 3 * c1.y + 3 * c2.y + end.y) / 8 };
  return { p1, c1, c2, p2, end, head, mid };
}

function midpoint(a, b) {
  return { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 };
}

function applyHeight(node, card) {
  if (AUTO_HEIGHT[card.type]) {
    node.style.height = "auto";
    node.style.minHeight = (card.h || 40) + "px";
  } else {
    node.style.minHeight = "";
    node.style.height = card.h + "px";
  }
}

function isTyping() {
  const ae = document.activeElement;
  const tag = (ae && ae.tagName) || "";
  return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || (ae && ae.isContentEditable);
}

function cssEscape(value) {
  if (window.CSS && window.CSS.escape) return window.CSS.escape(value);
  return String(value).replace(/["\\]/g, "\\$&");
}
