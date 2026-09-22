// The board canvas. An infinite pan/zoom surface of absolutely positioned cards. In edit mode
// it wires up dragging, edge resizing, selection, z stacking, freehand drawing, and inline
// on-card editing (see edit.js). Pan and zoom work in both view and edit modes.

import { renderContent } from "./cards.js";
import { icon } from "./icons.js";
// Imported as h because board.js uses el as a local variable name for card elements.
import { el as h } from "./dom.js";
import {
  mountTextEditor,
  mountCodeEditor,
  mountTableEditor,
  mountSelectionControls,
} from "./edit.js";

const MIN_W = 90;
const MIN_H = 70;
const MIN_SCALE = 0.15;
const MAX_SCALE = 4;
const DRAG_THRESHOLD = 4;

// Types that enter a full inline edit on double click. The rest use on-card controls.
const EDITABLE_TYPES = { text: 1, code: 1, table: 1 };

const RESIZE_DIRS = [
  { dir: "n", cursor: "ns-resize" },
  { dir: "e", cursor: "ew-resize" },
  { dir: "s", cursor: "ns-resize" },
  { dir: "w", cursor: "ew-resize" },
  { dir: "nw", cursor: "nwse-resize" },
  { dir: "ne", cursor: "nesw-resize" },
  { dir: "se", cursor: "nwse-resize" },
  { dir: "sw", cursor: "nesw-resize" },
];

function clamp(v, lo, hi) {
  return Math.max(lo, Math.min(hi, v));
}
function dist(a, b) {
  return Math.hypot(a.x - b.x, a.y - b.y);
}
function mid(a, b) {
  return { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 };
}

export class BoardView {
  // opts: { getMode, getBoard, getSelectedId, onSelect, onMutate, onBringToFront, onSendToBack,
  //         onDelete, onToggleCollapse, onSetHidden, onCardMenu, onEditingChange, uploadAsset,
  //         onViewChange, onZoomChange, grid }
  constructor(viewport, opts) {
    this.viewport = viewport;
    this.opts = opts;
    this.grid = opts.grid || 8;
    this.view = { panX: 0, panY: 0, scale: 1 };
    this.editingId = null;
    this._editor = null;
    this._pointers = new Map();
    this._pan = null;
    this._pinch = null;

    this.surface = h("div", { class: "board-surface" });
    this.viewport.innerHTML = "";
    this.viewport.appendChild(this.surface);

    this._wireNavigation();
    this.applyTransform();
  }

  // ----- Pan / zoom -----

  applyTransform() {
    const { panX, panY, scale } = this.view;
    this.surface.style.transform = "translate(" + panX + "px, " + panY + "px) scale(" + scale + ")";
    const gs = 16 * scale;
    this.viewport.style.backgroundSize = gs + "px " + gs + "px";
    this.viewport.style.backgroundPosition = panX + "px " + panY + "px";
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
    this.applyTransform();
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
    this.applyTransform();
  }

  zoomInAt() {
    const r = this.viewport.getBoundingClientRect();
    this.zoomAt(r.left + r.width / 2, r.top + r.height / 2, 1.2);
    this._persist();
  }
  zoomOutAt() {
    const r = this.viewport.getBoundingClientRect();
    this.zoomAt(r.left + r.width / 2, r.top + r.height / 2, 1 / 1.2);
    this._persist();
  }

  resetView() {
    this.view = { panX: 0, panY: 0, scale: 1 };
    this.applyTransform();
    this._persist();
  }

  // Frame all visible cards.
  fit() {
    const board = this.opts.getBoard();
    const mode = this.opts.getMode();
    let cards = board.cards;
    if (mode !== "edit") cards = cards.filter((c) => !c.hidden);
    if (!cards.length) {
      this.resetView();
      return;
    }
    let minX = Infinity;
    let minY = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;
    for (const c of cards) {
      const h = c.collapsed ? 40 : c.h || 0;
      minX = Math.min(minX, c.x || 0);
      minY = Math.min(minY, c.y || 0);
      maxX = Math.max(maxX, (c.x || 0) + (c.w || 0));
      maxY = Math.max(maxY, (c.y || 0) + h);
    }
    const pad = 60;
    const r = this.viewport.getBoundingClientRect();
    const bw = maxX - minX + pad * 2;
    const bh = maxY - minY + pad * 2;
    const scale = clamp(Math.min(r.width / bw, r.height / bh), MIN_SCALE, 1.2);
    this.view.scale = scale;
    this.view.panX = (r.width - (maxX - minX) * scale) / 2 - minX * scale;
    this.view.panY = (r.height - (maxY - minY) * scale) / 2 - minY * scale;
    this.applyTransform();
    this._persist();
  }

  _persist() {
    if (this.opts.onViewChange) this.opts.onViewChange(this.getView());
  }

  _wireNavigation() {
    this.viewport.addEventListener(
      "wheel",
      (e) => {
        e.preventDefault();
        if (e.ctrlKey || e.metaKey) {
          const factor = Math.exp(-e.deltaY * 0.0015);
          this.zoomAt(e.clientX, e.clientY, factor);
        } else {
          this.panBy(-e.deltaX, -e.deltaY);
        }
        this._persist();
      },
      { passive: false }
    );

    this.viewport.addEventListener("pointerdown", (e) => this._onViewportPointerDown(e));
    this.viewport.addEventListener("pointermove", (e) => this._onViewportPointerMove(e));
    this.viewport.addEventListener("pointerup", (e) => this._onViewportPointerUp(e));
    this.viewport.addEventListener("pointercancel", (e) => this._onViewportPointerUp(e));
  }

  _onViewportPointerDown(e) {
    this._pointers.set(e.pointerId, { x: e.clientX, y: e.clientY });
    if (this._pointers.size === 2) {
      const pts = Array.from(this._pointers.values());
      this._pinch = { dist: dist(pts[0], pts[1]), mid: mid(pts[0], pts[1]) };
      this._pan = null;
      return;
    }
    if (e.button !== 0) return;
    const onCard = e.target.closest && e.target.closest(".card");
    const onCtrl = e.target.closest && e.target.closest(".canvas-controls, .bubble-toolbar");
    if (onCard || onCtrl) return;
    this._pan = { id: e.pointerId, sx: e.clientX, sy: e.clientY, px: this.view.panX, py: this.view.panY, moved: false };
    try {
      this.viewport.setPointerCapture(e.pointerId);
    } catch (err) {
      // Ignore.
    }
    this.viewport.classList.add("panning");
  }

  _onViewportPointerMove(e) {
    if (this._pointers.has(e.pointerId)) this._pointers.set(e.pointerId, { x: e.clientX, y: e.clientY });
    if (this._pinch) {
      const pts = Array.from(this._pointers.values());
      if (pts.length < 2) return;
      const d = dist(pts[0], pts[1]);
      const m = mid(pts[0], pts[1]);
      if (this._pinch.dist > 0) this.zoomAt(m.x, m.y, d / this._pinch.dist);
      this.view.panX += m.x - this._pinch.mid.x;
      this.view.panY += m.y - this._pinch.mid.y;
      this.applyTransform();
      this._pinch = { dist: d, mid: m };
      return;
    }
    if (this._pan && this._pan.id === e.pointerId) {
      const dx = e.clientX - this._pan.sx;
      const dy = e.clientY - this._pan.sy;
      if (Math.abs(dx) + Math.abs(dy) > DRAG_THRESHOLD) this._pan.moved = true;
      this.view.panX = this._pan.px + dx;
      this.view.panY = this._pan.py + dy;
      this.applyTransform();
    }
  }

  _onViewportPointerUp(e) {
    this._pointers.delete(e.pointerId);
    if (this._pinch && this._pointers.size < 2) {
      this._pinch = null;
      this._persist();
    }
    if (this._pan && this._pan.id === e.pointerId) {
      const moved = this._pan.moved;
      this._pan = null;
      this.viewport.classList.remove("panning");
      try {
        this.viewport.releasePointerCapture(e.pointerId);
      } catch (err) {
        // Ignore.
      }
      // A click on empty canvas commits any open editor and clears the selection.
      if (!moved && this.opts.getMode() === "edit") {
        this.commitEdit();
        this.opts.onSelect(null);
      }
      this._persist();
    }
  }

  // ----- Coordinates -----

  clientToBoard(clientX, clientY) {
    const r = this.viewport.getBoundingClientRect();
    return {
      x: Math.round((clientX - r.left - this.view.panX) / this.view.scale),
      y: Math.round((clientY - r.top - this.view.panY) / this.view.scale),
    };
  }

  // The center of the current view, in board coordinates, for placing new cards.
  spawnPoint() {
    const r = this.viewport.getBoundingClientRect();
    return this.clientToBoard(r.left + r.width / 2, r.top + r.height / 2);
  }

  snap(value) {
    return Math.round(value / this.grid) * this.grid;
  }

  // ----- Rendering -----

  render() {
    const board = this.opts.getBoard();
    const mode = this.opts.getMode();
    const selId = this.opts.getSelectedId();
    this.editingId = null;
    this._editor = null;
    this.viewport.classList.toggle("is-edit", mode === "edit");
    this.surface.classList.toggle("is-edit", mode === "edit");
    this.surface.innerHTML = "";
    let cards = board.cards.slice().sort((a, b) => (a.z || 0) - (b.z || 0));
    if (mode !== "edit") cards = cards.filter((c) => !c.hidden);
    if (!cards.length) {
      this.surface.appendChild(
        h("div", {
          class: "board-empty",
          text: mode === "edit" ? "Right click or press Shift A to add a card" : "This page is empty",
        })
      );
      return;
    }
    for (const card of cards) {
      this.surface.appendChild(this._renderCard(card, mode, selId));
    }
  }

  updateSelection() {
    const selId = this.opts.getSelectedId();
    const mode = this.opts.getMode();
    this.surface.querySelectorAll(".card").forEach((el) => {
      el.classList.toggle("selected", mode === "edit" && el.dataset.id === selId);
    });
  }

  refreshCard(card) {
    if (card.collapsed || card.hidden) {
      this.render();
      return;
    }
    const el = this.surface.querySelector('.card[data-id="' + cssEscape(card.id) + '"]');
    if (!el) {
      this.render();
      return;
    }
    el.style.left = card.x + "px";
    el.style.top = card.y + "px";
    el.style.width = card.w + "px";
    el.style.height = card.h + "px";
    el.style.zIndex = String(card.z || 0);
    const body = el.querySelector(".card-body");
    if (body) {
      body.innerHTML = "";
      body.appendChild(renderContent(card));
      if (card.type === "draw" && this.opts.getMode() === "edit") this._attachDraw(body, card);
    }
  }

  _renderCard(card, mode, selId) {
    const collapsed = !!card.collapsed;
    const cardEl = h("div", {
      class: "card",
      dataset: { id: card.id, type: card.type },
      style: { left: card.x + "px", top: card.y + "px", width: card.w + "px", zIndex: String(card.z || 0) },
    });
    if (collapsed) cardEl.classList.add("collapsed");
    if (card.hidden) cardEl.classList.add("is-hidden");
    if (!collapsed) cardEl.style.height = card.h + "px";
    if (mode === "edit" && card.id === selId) cardEl.classList.add("selected");

    if (collapsed) {
      cardEl.appendChild(
        h("div", { class: "card-collapsed-label", text: card.type + (card.hidden ? " · hidden" : "") })
      );
    }

    const body = h("div", { class: "card-body" });
    if (!collapsed) body.appendChild(renderContent(card));
    cardEl.appendChild(body);

    if (mode === "edit") {
      cardEl.appendChild(this._actionsBar(card));

      // Select on any pointerdown within the card (capture phase, before children stop it).
      cardEl.addEventListener(
        "pointerdown",
        (e) => {
          if (e.button !== 0) return;
          if (this.editingId && this.editingId !== card.id) this.commitEdit();
          if (this.editingId !== card.id) this.opts.onSelect(card.id);
        },
        true
      );

      // Double click enters inline editing for text-like cards.
      cardEl.addEventListener("dblclick", (e) => {
        if (!EDITABLE_TYPES[card.type]) return;
        if (e.target.closest(".card-actions, .inline-controls, .card-handle")) return;
        e.preventDefault();
        this.beginEdit(card);
      });

      cardEl.addEventListener("contextmenu", (e) => {
        e.preventDefault();
        e.stopPropagation();
        this.commitEdit();
        this.opts.onSelect(card.id);
        if (this.opts.onCardMenu) this.opts.onCardMenu(card.id, e.clientX, e.clientY);
      });

      if (!collapsed) {
        this._addResizeHandles(cardEl, card);
        // The whole body is the drag surface, except draw cards where the body draws.
        if (card.type === "draw") this._attachDraw(body, card);
        else this._attachDrag(body, cardEl, card, true);
        mountSelectionControls(cardEl, body, card, this._ctx(card, cardEl, body));
      }
    }

    return cardEl;
  }

  _ctx(card, el, body) {
    return {
      markDirty: () => this.opts.onMutate(),
      rerenderContent: () => {
        body.innerHTML = "";
        body.appendChild(renderContent(card));
        if (card.type === "draw") this._attachDraw(body, card);
      },
      uploadAsset: (f) => this.opts.uploadAsset(f),
      requestCommit: () => this.commitEdit(),
    };
  }

  // The floating action bar: move grip, collapse, layer, hide, delete. Shown on hover/selected.
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
            pointerdown: (e) => e.stopPropagation(),
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
      mkBtn(card.hidden ? "eye" : "eye-off", card.hidden ? "Show" : "Hide", () =>
        this.opts.onSetHidden(card.id, !card.hidden)
      )
    );
    bar.appendChild(mkBtn("x", "Delete card", () => this.opts.onDelete(card.id), true));

    // Wire the grip as a drag handle after the card element exists.
    setTimeout(() => {
      const cardEl = bar.closest(".card");
      if (cardEl) this._attachDrag(grip, cardEl, card, false);
    }, 0);
    return bar;
  }

  _addResizeHandles(cardEl, card) {
    RESIZE_DIRS.forEach((spec) => {
      const handle = h("div", { class: "card-handle handle-" + spec.dir, style: { cursor: spec.cursor } });
      cardEl.appendChild(handle);
      this._attachResize(handle, cardEl, card, spec.dir);
    });
  }

  // ----- Inline editing -----

  beginEdit(card) {
    if (this.editingId === card.id) return;
    this.commitEdit();
    const el = this.surface.querySelector('.card[data-id="' + cssEscape(card.id) + '"]');
    if (!el) return;
    const body = el.querySelector(".card-body");
    if (!body) return;
    this.editingId = card.id;
    el.classList.add("editing");
    const ctx = this._ctx(card, el, body);
    if (card.type === "code") this._editor = mountCodeEditor(el, body, card, ctx);
    else if (card.type === "table") this._editor = mountTableEditor(el, body, card, ctx);
    else this._editor = mountTextEditor(el, body, card, ctx);
    if (this.opts.onEditingChange) this.opts.onEditingChange(card.id);
  }

  commitEdit() {
    if (!this.editingId) return;
    const id = this.editingId;
    const card = this.opts.getBoard().cards.find((c) => c.id === id);
    if (this._editor) {
      try {
        this._editor.commit();
      } catch (err) {
        // Ignore.
      }
      this._editor = null;
    }
    this.editingId = null;
    const el = this.surface.querySelector('.card[data-id="' + cssEscape(id) + '"]');
    if (el) el.classList.remove("editing");
    if (card) this.refreshCard(card);
    if (this.opts.onEditingChange) this.opts.onEditingChange(null);
  }

  cancelEdit() {
    if (!this.editingId) return;
    const id = this.editingId;
    const card = this.opts.getBoard().cards.find((c) => c.id === id);
    if (this._editor) {
      try {
        this._editor.cancel();
      } catch (err) {
        // Ignore.
      }
      this._editor = null;
    }
    this.editingId = null;
    const el = this.surface.querySelector('.card[data-id="' + cssEscape(id) + '"]');
    if (el) el.classList.remove("editing");
    if (card) this.refreshCard(card);
    if (this.opts.onEditingChange) this.opts.onEditingChange(null);
  }

  // ----- Drag / resize / draw (all scale aware) -----

  _attachDrag(handle, el, card, threshold) {
    let startX = 0;
    let startY = 0;
    let originX = 0;
    let originY = 0;
    let active = false;
    let dragging = false;

    handle.addEventListener("pointerdown", (e) => {
      if (e.button !== 0) return;
      if (this.editingId === card.id) return;
      if (e.target.closest(".card-handle, .inline-controls, .card-act")) return;
      active = true;
      dragging = !threshold;
      startX = e.clientX;
      startY = e.clientY;
      originX = card.x;
      originY = card.y;
      if (!threshold) {
        try {
          handle.setPointerCapture(e.pointerId);
        } catch (err) {
          // Ignore.
        }
        el.classList.add("dragging");
        this.opts.onSelect(card.id);
      }
      e.stopPropagation();
    });
    handle.addEventListener("pointermove", (e) => {
      if (!active) return;
      const dx = e.clientX - startX;
      const dy = e.clientY - startY;
      if (!dragging) {
        if (Math.abs(dx) + Math.abs(dy) < DRAG_THRESHOLD) return;
        dragging = true;
        try {
          handle.setPointerCapture(e.pointerId);
        } catch (err) {
          // Ignore.
        }
        el.classList.add("dragging");
        this.opts.onSelect(card.id);
      }
      const nx = this.snap(originX + dx / this.view.scale);
      const ny = this.snap(originY + dy / this.view.scale);
      card.x = nx;
      card.y = ny;
      el.style.left = nx + "px";
      el.style.top = ny + "px";
    });
    const end = (e) => {
      if (!active) return;
      const wasDragging = dragging;
      active = false;
      dragging = false;
      try {
        handle.releasePointerCapture(e.pointerId);
      } catch (err) {
        // Ignore.
      }
      el.classList.remove("dragging");
      if (wasDragging) this.opts.onMutate();
    };
    handle.addEventListener("pointerup", end);
    handle.addEventListener("pointercancel", end);
  }

  _attachResize(handle, el, card, dir) {
    let startX = 0;
    let startY = 0;
    let originX = 0;
    let originY = 0;
    let originW = 0;
    let originH = 0;
    let resizing = false;

    handle.addEventListener("pointerdown", (e) => {
      if (e.button !== 0) return;
      this.opts.onSelect(card.id);
      resizing = true;
      handle.setPointerCapture(e.pointerId);
      startX = e.clientX;
      startY = e.clientY;
      originX = card.x;
      originY = card.y;
      originW = card.w;
      originH = card.h;
      el.classList.add("resizing");
      e.preventDefault();
      e.stopPropagation();
    });
    handle.addEventListener("pointermove", (e) => {
      if (!resizing) return;
      const dx = (e.clientX - startX) / this.view.scale;
      const dy = (e.clientY - startY) / this.view.scale;
      const rightEdge = originX + originW;
      const bottomEdge = originY + originH;
      let x = originX;
      let y = originY;
      let w = originW;
      let h = originH;

      if (dir.indexOf("e") !== -1) w = Math.max(MIN_W, this.snap(rightEdge + dx) - originX);
      if (dir.indexOf("s") !== -1) h = Math.max(MIN_H, this.snap(bottomEdge + dy) - originY);
      if (dir.indexOf("w") !== -1) {
        let left = Math.min(this.snap(originX + dx), rightEdge - MIN_W);
        x = left;
        w = rightEdge - left;
      }
      if (dir.indexOf("n") !== -1) {
        let top = Math.min(this.snap(originY + dy), bottomEdge - MIN_H);
        y = top;
        h = bottomEdge - top;
      }

      card.x = x;
      card.y = y;
      card.w = w;
      card.h = h;
      el.style.left = x + "px";
      el.style.top = y + "px";
      el.style.width = w + "px";
      el.style.height = h + "px";
    });
    const end = (e) => {
      if (!resizing) return;
      resizing = false;
      try {
        handle.releasePointerCapture(e.pointerId);
      } catch (err) {
        // Ignore.
      }
      el.classList.remove("resizing");
      this.opts.onMutate();
    };
    handle.addEventListener("pointerup", end);
    handle.addEventListener("pointercancel", end);
  }

  _attachDraw(body, card) {
    const svg = body.querySelector("svg.card-draw");
    const path = svg ? svg.querySelector("path.draw-line") : null;
    if (!path) return;
    let drawing = false;

    const toLocal = (e) => {
      const rect = body.getBoundingClientRect();
      return {
        x: Math.round(((e.clientX - rect.left) / this.view.scale) * 10) / 10,
        y: Math.round(((e.clientY - rect.top) / this.view.scale) * 10) / 10,
      };
    };
    body.addEventListener("pointerdown", (e) => {
      if (this.opts.getMode() !== "edit") return;
      drawing = true;
      body.setPointerCapture(e.pointerId);
      const p = toLocal(e);
      card.path = (card.path ? card.path + " " : "") + "M " + p.x + " " + p.y;
      path.setAttribute("d", card.path);
      const hint = body.querySelector(".draw-hint");
      if (hint) hint.remove();
      e.stopPropagation();
    });
    body.addEventListener("pointermove", (e) => {
      if (!drawing) return;
      const p = toLocal(e);
      card.path += " L " + p.x + " " + p.y;
      path.setAttribute("d", card.path);
    });
    const end = (e) => {
      if (!drawing) return;
      drawing = false;
      try {
        body.releasePointerCapture(e.pointerId);
      } catch (err) {
        // Ignore.
      }
      this.opts.onMutate();
    };
    body.addEventListener("pointerup", end);
    body.addEventListener("pointercancel", end);
  }
}

function cssEscape(value) {
  if (window.CSS && window.CSS.escape) return window.CSS.escape(value);
  return String(value).replace(/["\\]/g, "\\$&");
}
