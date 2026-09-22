// The board canvas. Renders cards at absolute positions and, in edit mode, wires up
// dragging, resizing, selection, z stacking, and freehand drawing with pointer events.

import { renderContent } from "./cards.js";
import { icon } from "./icons.js";

const MIN_W = 90;
const MIN_H = 70;

// The eight resize directions, with their cursors. Corners are listed after edges so they
// paint on top and take priority where they overlap.
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

  // Convert a client (viewport) point to a board surface coordinate, accounting for scroll.
  clientToBoard(clientX, clientY) {
    const rect = this.surface.getBoundingClientRect();
    return {
      x: Math.max(0, Math.round(clientX - rect.left)),
      y: Math.max(0, Math.round(clientY - rect.top)),
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
    let cards = board.cards.slice().sort((a, b) => (a.z || 0) - (b.z || 0));
    // Hidden cards are dropped entirely in view mode, and shown faintly in edit mode so they
    // can be brought back with the right click menu.
    if (mode !== "edit") cards = cards.filter((c) => !c.hidden);
    if (!cards.length) {
      const hint = document.createElement("div");
      hint.className = "board-empty";
      hint.textContent =
        mode === "edit" ? "Right click or press Shift A to add a card" : "This page is empty";
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
    // Collapsed or hidden cards change their whole structure, so rebuild rather than patch.
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
      if (card.type === "draw" && this.opts.getMode() === "edit") {
        this._attachDraw(body, card);
      }
    }
  }

  _renderCard(card, mode, selId) {
    const collapsed = !!card.collapsed;
    const el = document.createElement("div");
    el.className = "card";
    if (collapsed) el.classList.add("collapsed");
    if (card.hidden) el.classList.add("is-hidden");
    el.dataset.id = card.id;
    el.dataset.type = card.type;
    el.style.left = card.x + "px";
    el.style.top = card.y + "px";
    el.style.width = card.w + "px";
    // A collapsed card shrinks to just its header, so its height is left to the layout.
    if (!collapsed) el.style.height = card.h + "px";
    el.style.zIndex = String(card.z || 0);
    if (mode === "edit" && card.id === selId) el.classList.add("selected");

    if (mode === "edit") {
      const header = document.createElement("div");
      header.className = "card-header";
      const tag = document.createElement("span");
      tag.className = "card-tag";
      tag.textContent = card.type + (card.hidden ? " · hidden" : "");
      header.appendChild(tag);

      const controls = document.createElement("div");
      controls.className = "card-controls";
      controls.appendChild(
        this._ctrlButton(collapsed ? "chevrons-up-down" : "chevrons-down-up", collapsed ? "Expand" : "Collapse", () =>
          this.opts.onToggleCollapse(card.id)
        )
      );
      controls.appendChild(
        this._ctrlButton("bring-to-front", "Bring to front", () => this.opts.onBringToFront(card.id))
      );
      controls.appendChild(
        this._ctrlButton("send-to-back", "Send to back", () => this.opts.onSendToBack(card.id))
      );
      const del = this._ctrlButton("x", "Delete card", () => this.opts.onDelete(card.id));
      del.classList.add("danger");
      controls.appendChild(del);
      header.appendChild(controls);

      el.appendChild(header);
      this._attachDrag(header, el, card);

      // Selecting on pointerdown makes the inspector and z buttons target this card.
      el.addEventListener("pointerdown", () => this.opts.onSelect(card.id));

      // Right click on a card opens its own menu (hide, collapse, layering, delete).
      el.addEventListener("contextmenu", (e) => {
        e.preventDefault();
        e.stopPropagation();
        this.opts.onSelect(card.id);
        if (this.opts.onCardMenu) this.opts.onCardMenu(card.id, e.clientX, e.clientY);
      });
    }

    if (!collapsed) {
      const body = document.createElement("div");
      body.className = "card-body";
      body.appendChild(renderContent(card));
      el.appendChild(body);

      if (mode === "edit") {
        this._addResizeHandles(el, card);
        if (card.type === "draw") this._attachDraw(body, card);
      } else {
        // A gentle pointer tracking tilt, view mode only.
        this._attachTilt(el);
      }
    }

    // The specular sheen overlay tracks the pointer. It never blocks clicks.
    const sheen = document.createElement("div");
    sheen.className = "card-sheen";
    el.appendChild(sheen);
    this._attachSheen(el);

    return el;
  }

  _addResizeHandles(el, card) {
    RESIZE_DIRS.forEach((spec) => {
      const handle = document.createElement("div");
      handle.className = "card-handle handle-" + spec.dir;
      handle.style.cursor = spec.cursor;
      el.appendChild(handle);
      this._attachResize(handle, el, card, spec.dir);
    });
  }

  _prefersReducedMotion() {
    try {
      return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    } catch (err) {
      return false;
    }
  }

  // A subtle 3D tilt that follows the pointer over a card in view mode. It only sets the
  // element transform, so it never interferes with selection, drag, resize or the inspector,
  // none of which run in view mode.
  _attachTilt(el) {
    if (this._prefersReducedMotion()) return;
    const maxDeg = 5;
    const onMove = (e) => {
      const r = el.getBoundingClientRect();
      if (!r.width || !r.height) return;
      const px = (e.clientX - r.left) / r.width;
      const py = (e.clientY - r.top) / r.height;
      const ry = (px - 0.5) * 2 * maxDeg;
      const rx = -(py - 0.5) * 2 * maxDeg;
      el.style.transform =
        "perspective(900px) rotateX(" + rx.toFixed(2) + "deg) rotateY(" + ry.toFixed(2) + "deg) translate3d(0, -4px, 0)";
    };
    const onLeave = () => {
      el.style.transform = "";
    };
    el.addEventListener("pointerenter", () => {
      el.style.willChange = "transform";
    });
    el.addEventListener("pointermove", onMove);
    el.addEventListener("pointerleave", onLeave);
    el.addEventListener("pointercancel", onLeave);
  }

  // A small header control button with an inline icon. Its pointerdown is stopped so it does
  // not start a drag.
  _ctrlButton(iconName, title, onClick) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "card-ctrl";
    btn.title = title;
    btn.setAttribute("aria-label", title);
    btn.appendChild(icon(iconName, { size: 15 }));
    btn.addEventListener("pointerdown", (e) => e.stopPropagation());
    btn.addEventListener("click", (e) => {
      e.stopPropagation();
      onClick();
    });
    return btn;
  }

  // Cursor tracking specular highlight. Only sets CSS custom properties, so it never
  // interferes with selection, drag, resize or the inspector. Disabled under reduced motion.
  _attachSheen(el) {
    if (this._prefersReducedMotion()) return;
    const sheen = el.querySelector(".card-sheen");
    if (!sheen) return;
    el.addEventListener("pointermove", (e) => {
      const r = el.getBoundingClientRect();
      if (!r.width || !r.height) return;
      const mx = ((e.clientX - r.left) / r.width) * 100;
      const my = ((e.clientY - r.top) / r.height) * 100;
      el.style.setProperty("--mx", mx.toFixed(1) + "%");
      el.style.setProperty("--my", my.toFixed(1) + "%");
    });
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

  // Resize from any of the eight zones. Handles on the top or left move the card origin
  // (x/y) while changing w/h, keeping the opposite edge anchored. Snaps to the grid and
  // enforces the minimum size.
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
      const dx = e.clientX - startX;
      const dy = e.clientY - startY;
      const rightEdge = originX + originW;
      const bottomEdge = originY + originH;
      let x = originX;
      let y = originY;
      let w = originW;
      let h = originH;

      if (dir.indexOf("e") !== -1) {
        const right = this.snap(rightEdge + dx);
        w = Math.max(MIN_W, right - originX);
      }
      if (dir.indexOf("s") !== -1) {
        const bottom = this.snap(bottomEdge + dy);
        h = Math.max(MIN_H, bottom - originY);
      }
      if (dir.indexOf("w") !== -1) {
        let left = Math.max(0, this.snap(originX + dx));
        left = Math.min(left, rightEdge - MIN_W);
        x = left;
        w = rightEdge - left;
      }
      if (dir.indexOf("n") !== -1) {
        let top = Math.max(0, this.snap(originY + dy));
        top = Math.min(top, bottomEdge - MIN_H);
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
