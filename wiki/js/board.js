// The board canvas. Renders cards at absolute positions and, in edit mode, wires up
// dragging, resizing, selection, z stacking, and freehand drawing with pointer events.

import { renderContent } from "./cards.js";

const MIN_W = 90;
const MIN_H = 70;

export class BoardView {
  // opts: { getMode, getBoard, getSelectedId, onSelect, onMutate, grid }
  constructor(viewport, opts) {
    this.viewport = viewport;
    this.opts = opts;
    this.grid = opts.grid || 8;
    this.surface = document.createElement("div");
    this.surface.className = "board-surface";
    this.viewport.innerHTML = "";
    this.viewport.appendChild(this.surface);

    // Clicking empty board space clears the selection in edit mode.
    this.surface.addEventListener("pointerdown", (e) => {
      if (e.target === this.surface && this.opts.getMode() === "edit") {
        this.opts.onSelect(null);
      }
    });
  }

  // A point near the top left of the current viewport, for placing new cards.
  spawnPoint() {
    return {
      x: Math.round(this.viewport.scrollLeft + 48),
      y: Math.round(this.viewport.scrollTop + 48),
    };
  }

  snap(value) {
    return Math.round(value / this.grid) * this.grid;
  }

  // Full render from the current board state.
  render() {
    const board = this.opts.getBoard();
    const mode = this.opts.getMode();
    const selId = this.opts.getSelectedId();
    this.surface.classList.toggle("is-edit", mode === "edit");
    this.surface.innerHTML = "";
    const cards = board.cards.slice().sort((a, b) => (a.z || 0) - (b.z || 0));
    if (!cards.length) {
      const hint = document.createElement("div");
      hint.className = "board-empty";
      hint.textContent = mode === "edit" ? "Add cards from the toolbar" : "This page is empty";
      this.surface.appendChild(hint);
      return;
    }
    for (const card of cards) {
      this.surface.appendChild(this._renderCard(card, mode, selId));
    }
  }

  // Toggle the selected outline without a full rebuild, so drags are not interrupted.
  updateSelection() {
    const selId = this.opts.getSelectedId();
    const mode = this.opts.getMode();
    this.surface.querySelectorAll(".card").forEach((el) => {
      el.classList.toggle("selected", mode === "edit" && el.dataset.id === selId);
    });
  }

  // Re-render just one card, used when the inspector edits its content or geometry.
  refreshCard(card) {
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
      if (card.type === "draw" && this.opts.getMode() === "edit") {
        this._attachDraw(body, card);
      }
    }
  }

  _renderCard(card, mode, selId) {
    const el = document.createElement("div");
    el.className = "card";
    el.dataset.id = card.id;
    el.dataset.type = card.type;
    el.style.left = card.x + "px";
    el.style.top = card.y + "px";
    el.style.width = card.w + "px";
    el.style.height = card.h + "px";
    el.style.zIndex = String(card.z || 0);
    if (mode === "edit" && card.id === selId) el.classList.add("selected");

    if (mode === "edit") {
      const header = document.createElement("div");
      header.className = "card-header";
      const tag = document.createElement("span");
      tag.className = "card-tag";
      tag.textContent = card.type;
      header.appendChild(tag);

      const controls = document.createElement("div");
      controls.className = "card-controls";
      controls.appendChild(
        this._ctrlButton("^", "Bring to front", () => this.opts.onBringToFront(card.id))
      );
      controls.appendChild(
        this._ctrlButton("v", "Send to back", () => this.opts.onSendToBack(card.id))
      );
      const del = this._ctrlButton("x", "Delete card", () => this.opts.onDelete(card.id));
      del.classList.add("danger");
      controls.appendChild(del);
      header.appendChild(controls);

      el.appendChild(header);
      this._attachDrag(header, el, card);

      // Selecting on pointerdown makes the inspector and z buttons target this card.
      el.addEventListener("pointerdown", () => this.opts.onSelect(card.id));
    }

    const body = document.createElement("div");
    body.className = "card-body";
    body.appendChild(renderContent(card));
    el.appendChild(body);

    if (mode === "edit") {
      const handle = document.createElement("div");
      handle.className = "card-resize";
      handle.title = "Drag to resize";
      el.appendChild(handle);
      this._attachResize(handle, el, card);
      if (card.type === "draw") this._attachDraw(body, card);
    }
    return el;
  }

  // A small header control button. Its pointerdown is stopped so it does not start a drag.
  _ctrlButton(label, title, onClick) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "card-ctrl";
    btn.textContent = label;
    btn.title = title;
    btn.addEventListener("pointerdown", (e) => e.stopPropagation());
    btn.addEventListener("click", (e) => {
      e.stopPropagation();
      onClick();
    });
    return btn;
  }

  _attachDrag(header, el, card) {
    let startX = 0;
    let startY = 0;
    let originX = 0;
    let originY = 0;
    let dragging = false;

    header.addEventListener("pointerdown", (e) => {
      if (e.button !== 0) return;
      // Selecting here as well, because this handler stops propagation to the card.
      this.opts.onSelect(card.id);
      dragging = true;
      header.setPointerCapture(e.pointerId);
      startX = e.clientX;
      startY = e.clientY;
      originX = card.x;
      originY = card.y;
      el.classList.add("dragging");
      e.preventDefault();
      e.stopPropagation();
    });
    header.addEventListener("pointermove", (e) => {
      if (!dragging) return;
      const nx = Math.max(0, this.snap(originX + (e.clientX - startX)));
      const ny = Math.max(0, this.snap(originY + (e.clientY - startY)));
      card.x = nx;
      card.y = ny;
      el.style.left = nx + "px";
      el.style.top = ny + "px";
    });
    const end = (e) => {
      if (!dragging) return;
      dragging = false;
      try {
        header.releasePointerCapture(e.pointerId);
      } catch (err) {
        // Ignore.
      }
      el.classList.remove("dragging");
      this.opts.onMutate();
    };
    header.addEventListener("pointerup", end);
    header.addEventListener("pointercancel", end);
  }

  _attachResize(handle, el, card) {
    let startX = 0;
    let startY = 0;
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
      originW = card.w;
      originH = card.h;
      el.classList.add("resizing");
      e.preventDefault();
      e.stopPropagation();
    });
    handle.addEventListener("pointermove", (e) => {
      if (!resizing) return;
      const nw = Math.max(MIN_W, this.snap(originW + (e.clientX - startX)));
      const nh = Math.max(MIN_H, this.snap(originH + (e.clientY - startY)));
      card.w = nw;
      card.h = nh;
      el.style.width = nw + "px";
      el.style.height = nh + "px";
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
        x: Math.round((e.clientX - rect.left) * 10) / 10,
        y: Math.round((e.clientY - rect.top) * 10) / 10,
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

// Minimal CSS attribute selector escaping for card ids (ids are alphanumeric plus underscore).
function cssEscape(value) {
  if (window.CSS && window.CSS.escape) return window.CSS.escape(value);
  return String(value).replace(/["\\]/g, "\\$&");
}
