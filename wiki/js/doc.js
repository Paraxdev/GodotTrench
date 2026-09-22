// The document layout: a single readable column of blocks, Notion and Obsidian style. Cards
// render in array order and their canvas positions are ignored. In edit mode every text block
// is live contenteditable, blocks get a gutter with + and a drag handle, and code, tables and
// media edit in place.

import { renderContent, newCard } from "./cards.js";
import { icon } from "./icons.js";
import { el } from "./dom.js";
import {
  mountTextEditor,
  mountCodeEditor,
  mountTableEditor,
  mountSourceEditor,
  mountSelectionControls,
  attachDrawing,
  openSuggest,
  blockMenuItems,
  editorFor,
} from "./edit.js";

const DRAG_THRESHOLD = 4;
const RESIZABLE = { draw: 1, shape: 1 };

export class DocView {
  // opts: { getMode, getBoard, getPageInfo, onMutate, uploadAsset, pages, backlinks, pasteFiles,
  //         onCardMenuAction }
  constructor(container, opts) {
    this.container = container;
    this.opts = opts;
    this.scroller = el("div", { class: "doc-scroll" });
    this.container.appendChild(this.scroller);
    this.active = null;
    this.scroller.addEventListener("scroll", () => this._spy(), { passive: true });
  }

  cards() {
    return this.opts.getBoard().cards;
  }

  // Save every mounted editor before the DOM is thrown away.
  commitAll() {
    this.scroller.querySelectorAll(".md.editing-md").forEach((root) => {
      const ed = editorFor(root);
      if (ed) ed.save();
    });
    if (this.active) {
      const a = this.active;
      this.active = null;
      try {
        a.editor.commit();
      } catch (err) {
        // The editor's DOM may already be gone.
      }
    }
  }

  render(opts) {
    const keepScroll = opts && opts.keepScroll;
    const top = this.scroller.scrollTop;
    this.commitAll();
    const mode = this.opts.getMode();
    const editing = mode === "edit";
    const info = this.opts.getPageInfo() || {};
    this.scroller.innerHTML = "";

    const sheet = el("article", { class: "doc-sheet" + (editing ? " is-edit" : "") });
    if (info.category) sheet.appendChild(el("div", { class: "doc-kicker", text: info.category }));

    const list = el("div", { class: "doc-blocks" });
    const cards = this.cards().filter((c) => editing || !c.hidden);
    cards.forEach((card) => list.appendChild(this._block(card, editing)));
    sheet.appendChild(list);

    if (!cards.length) {
      sheet.appendChild(
        el("div", { class: "doc-empty" }, [
          el("p", { text: editing ? "This page is empty." : "This page is empty. Press Edit to start writing." }),
        ])
      );
    }
    if (editing) {
      sheet.appendChild(
        el(
          "button",
          {
            type: "button",
            class: "doc-add",
            on: {
              click: (e) => {
                const last = this.cards()[this.cards().length - 1];
                if (!last || last.type !== "text" || (last.md || "").trim()) {
                  this.insertAfter(last ? last.id : null, { id: "text" });
                } else {
                  this.focusCard(last.id, true);
                }
                e.stopPropagation();
              },
            },
          },
          [icon("plus", { size: 16 }), el("span", { text: "Click to add a block, or type / in any text for more" })]
        )
      );
    }

    const back = el("footer", { class: "doc-backlinks", hidden: true });
    sheet.appendChild(back);
    Promise.resolve(this.opts.backlinks ? this.opts.backlinks() : []).then((links) => {
      if (!links || !links.length) return;
      back.hidden = false;
      back.appendChild(el("div", { class: "doc-backlinks-title" }, [icon("corner-down-left", { size: 14 }), "Linked from"]));
      back.appendChild(
        el(
          "div",
          { class: "doc-backlinks-list" },
          links.map((p) => el("a", { class: "chip", href: "#" + p.slug }, [icon(p.layout === "canvas" ? "layout-dashboard" : "file-text", { size: 14 }), p.title]))
        )
      );
    });

    this.outline = el("nav", { class: "doc-outline", "aria-label": "On this page" });
    this.scroller.appendChild(el("div", { class: "doc-layout" }, [sheet, this.outline]));
    this._buildOutline();
    if (keepScroll) this.scroller.scrollTop = top;
  }

  // ----- Blocks -----

  _block(card, editing) {
    const block = el("div", { class: "block", dataset: { id: card.id, type: card.type } });
    if (card.hidden) block.classList.add("is-hidden");
    const body = el("div", { class: "block-body" });
    body.appendChild(renderContent(card));
    if (RESIZABLE[card.type]) body.style.height = (card.h || 200) + "px";
    block.appendChild(body);

    if (!editing) return block;

    if (card.hidden) block.appendChild(el("span", { class: "block-badge", text: "Hidden from readers" }));
    block.appendChild(this._gutter(card, block));
    const ctx = this._ctx(card, block, body);

    if (card.type === "text") {
      if ((card.format || "markdown") === "bbcode") {
        body.addEventListener("click", () => this._openSource(card, block, body));
      } else {
        const root = body.querySelector(".md");
        if (root) mountTextEditor(root, card, ctx, { persistent: true });
      }
    } else if (card.type === "code" || card.type === "table") {
      body.addEventListener("click", (e) => {
        if (this.active && this.active.id === card.id) return;
        if (e.target.closest(".code-copy")) return;
        this._openEditor(card, block, body, card.type === "code" ? mountCodeEditor : mountTableEditor);
      });
    } else {
      mountSelectionControls(block, card, ctx);
      if (card.type === "draw") attachDrawing(body, card, () => 1, () => this.opts.onMutate());
      if (RESIZABLE[card.type]) block.appendChild(this._resizer(card, body));
    }
    return block;
  }

  _ctx(card, block, body) {
    return {
      markDirty: () => this.opts.onMutate(),
      rerenderContent: () => {
        body.innerHTML = "";
        body.appendChild(renderContent(card));
        if (card.type === "draw") attachDrawing(body, card, () => 1, () => this.opts.onMutate());
      },
      uploadAsset: (f) => this.opts.uploadAsset(f),
      requestCommit: () => {
        if (this.active && this.active.id === card.id) this._closeEditor();
        else {
          const root = body.querySelector(".md");
          if (root) root.blur();
        }
      },
      reopen: () => {
        const fn = card.type === "table" ? mountTableEditor : mountCodeEditor;
        this._openEditor(card, block, body, fn);
      },
      insertBlock: (type, extra) => this._insertFromText(card, type, extra),
      pasteFiles: (files) => this.opts.pasteFiles(files, card.id),
      pages: () => this.opts.pages(),
      removeEmpty: () => this._removeEmpty(card),
    };
  }

  // Code, tables and source editing mount one at a time and commit when focus leaves them.
  _openEditor(card, block, body, mount) {
    if (this.active) this._closeEditor();
    block.classList.add("editing");
    const ctx = this._ctx(card, block, body);
    const editor = mount(body, card, ctx);
    const onOut = (e) => {
      if (e.relatedTarget && body.contains(e.relatedTarget)) return;
      setTimeout(() => {
        if (this.active && this.active.id === card.id && !body.contains(document.activeElement)) this._closeEditor();
      }, 0);
    };
    body.addEventListener("focusout", onOut);
    this.active = { id: card.id, editor, block, off: () => body.removeEventListener("focusout", onOut) };
  }

  _closeEditor() {
    const a = this.active;
    if (!a) return;
    this.active = null;
    a.off();
    a.block.classList.remove("editing");
    a.editor.commit();
    this._buildOutline();
  }

  _openSource(card, block, body) {
    this._openEditor(card, block, body, (b, c, ctx) => {
      const ed = mountSourceEditor(b, c, ctx);
      return {
        commit: () => {
          ed.commit();
          this.render({ keepScroll: true });
        },
        cancel: () => ed.cancel(),
      };
    });
  }

  _gutter(card, block) {
    const add = el(
      "button",
      {
        type: "button",
        class: "block-btn",
        title: "Add a block below",
        "aria-label": "Add a block below",
        on: {
          click: (e) => {
            e.stopPropagation();
            openSuggest(add.getBoundingClientRect(), blockMenuItems(), (b) => this.insertAfter(card.id, b), { title: "Add below" });
          },
        },
      },
      icon("plus", { size: 16 })
    );
    const handle = el("button", { type: "button", class: "block-btn block-handle", title: "Drag to move, click for options", "aria-label": "Block options" }, icon("grip-vertical", { size: 16 }));
    this._wireHandle(handle, card, block);
    return el("div", { class: "block-gutter", contenteditable: "false" }, [add, handle]);
  }

  _wireHandle(handle, card, block) {
    let start = null;
    let dragging = false;
    let marker = null;
    let targetIndex = -1;

    handle.addEventListener("pointerdown", (e) => {
      if (e.button !== 0) return;
      e.preventDefault();
      start = { x: e.clientX, y: e.clientY };
      dragging = false;
      handle.setPointerCapture(e.pointerId);
    });
    handle.addEventListener("pointermove", (e) => {
      if (!start) return;
      if (!dragging) {
        if (Math.abs(e.clientY - start.y) + Math.abs(e.clientX - start.x) < DRAG_THRESHOLD) return;
        dragging = true;
        this.commitAll();
        block.classList.add("dragging");
        marker = el("div", { class: "drop-marker" });
        this.scroller.querySelector(".doc-blocks").appendChild(marker);
      }
      const blocks = Array.from(this.scroller.querySelectorAll(".doc-blocks > .block"));
      let idx = blocks.length;
      for (let i = 0; i < blocks.length; i += 1) {
        const r = blocks[i].getBoundingClientRect();
        if (e.clientY < r.top + r.height / 2) {
          idx = i;
          break;
        }
      }
      targetIndex = idx;
      const list = this.scroller.querySelector(".doc-blocks");
      const lr = list.getBoundingClientRect();
      const y = idx < blocks.length ? blocks[idx].getBoundingClientRect().top - lr.top - 3 : lr.height + 1;
      marker.style.top = y + "px";
      this._autoScroll(e.clientY);
    });
    const end = (e) => {
      if (!start) return;
      const wasDragging = dragging;
      start = null;
      dragging = false;
      try {
        handle.releasePointerCapture(e.pointerId);
      } catch (err) {
        // Capture may already be gone.
      }
      if (marker) marker.remove();
      marker = null;
      block.classList.remove("dragging");
      if (wasDragging) {
        this._moveTo(card.id, targetIndex);
      } else if (e.type === "pointerup") {
        this._blockMenu(card, handle.getBoundingClientRect());
      }
    };
    handle.addEventListener("pointerup", end);
    handle.addEventListener("pointercancel", end);
  }

  _autoScroll(clientY) {
    const r = this.scroller.getBoundingClientRect();
    if (clientY < r.top + 60) this.scroller.scrollTop -= 14;
    else if (clientY > r.bottom - 60) this.scroller.scrollTop += 14;
  }

  // Move a card so it lands before the visible block at `index`, counting hidden cards in view
  // mode is unnecessary because dragging only happens in edit mode, where every card is shown.
  _moveTo(id, index) {
    const cards = this.cards();
    const from = cards.findIndex((c) => c.id === id);
    if (from < 0) return;
    let to = index;
    if (to > from) to -= 1;
    if (to === from) return;
    const [card] = cards.splice(from, 1);
    cards.splice(to, 0, card);
    this.opts.onMutate();
    this.render({ keepScroll: true });
  }

  _blockMenu(card, rect) {
    const cards = this.cards();
    const i = cards.findIndex((c) => c.id === card.id);
    const items = [];
    if (card.type === "text") items.push({ label: "Edit Markdown source", icon: "hash", value: () => this._sourceFor(card) });
    items.push({ label: "Duplicate", icon: "copy", value: () => this.opts.onCardMenuAction("duplicate", card.id) });
    if (i > 0) items.push({ label: "Move up", icon: "arrow-up", value: () => this._moveTo(card.id, i - 1) });
    if (i < cards.length - 1) items.push({ label: "Move down", icon: "arrow-down", value: () => this._moveTo(card.id, i + 2) });
    items.push(
      card.hidden
        ? { label: "Show to readers", icon: "eye", value: () => this.opts.onCardMenuAction("show", card.id) }
        : { label: "Hide from readers", icon: "eye-off", value: () => this.opts.onCardMenuAction("hide", card.id) }
    );
    items.push({ label: "Delete", icon: "trash-2", hint: "Del", value: () => this.opts.onCardMenuAction("delete", card.id) });
    openSuggest(rect, items, (fn) => fn(), { title: "Block" });
  }

  _sourceFor(card) {
    const block = this.scroller.querySelector('.block[data-id="' + cssEscape(card.id) + '"]');
    if (!block) return;
    const body = block.querySelector(".block-body");
    const root = body.querySelector(".md");
    if (root) {
      const ed = editorFor(root);
      if (ed) {
        ed.save();
        ed.cancel();
      }
    }
    this._openSource(card, block, body);
  }

  _resizer(card, body) {
    const grip = el("div", { class: "block-resize", title: "Drag to resize" });
    let startY = 0;
    let startH = 0;
    grip.addEventListener("pointerdown", (e) => {
      e.preventDefault();
      e.stopPropagation();
      startY = e.clientY;
      startH = body.offsetHeight;
      grip.setPointerCapture(e.pointerId);
      grip.classList.add("active");
    });
    grip.addEventListener("pointermove", (e) => {
      if (!grip.classList.contains("active")) return;
      card.h = Math.max(80, Math.round(startH + e.clientY - startY));
      body.style.height = card.h + "px";
    });
    const end = () => {
      if (!grip.classList.contains("active")) return;
      grip.classList.remove("active");
      this.opts.onMutate();
    };
    grip.addEventListener("pointerup", end);
    grip.addEventListener("pointercancel", end);
    return grip;
  }

  // ----- Inserting and removing -----

  // blockSpec is an entry of BLOCKS: either a card type, or a text format applied to a new
  // text block.
  insertAfter(afterId, blockSpec) {
    const cards = this.cards();
    const type = blockSpec.card || "text";
    const card = newCard(type, { x: 0, y: 0 }, []);
    if (type === "draw" || type === "shape") card.h = 220;
    const i = afterId ? cards.findIndex((c) => c.id === afterId) : cards.length - 1;
    cards.splice(i + 1, 0, card);
    this.opts.onMutate();
    this.render({ keepScroll: true });
    this._afterInsert(card, blockSpec);
    return card;
  }

  _insertFromText(card, type, extra) {
    const root = extra && extra.from;
    const ed = root && editorFor(root);
    const tail = ed ? ed.splitAtCaret() : "";
    const cards = this.cards();
    const i = cards.findIndex((c) => c.id === card.id);
    const fresh = newCard(type, { x: 0, y: 0 }, []);
    if (type === "draw" || type === "shape") fresh.h = 220;
    const add = [fresh];
    if (tail.trim()) {
      const rest = newCard("text", { x: 0, y: 0 }, []);
      rest.md = tail;
      add.push(rest);
    }
    cards.splice(i + 1, 0, ...add);
    // An empty text block left behind by the split is just noise.
    if (!(card.md || "").trim() && card.type === "text") cards.splice(i, 1);
    this.opts.onMutate();
    this.render({ keepScroll: true });
    this._afterInsert(fresh, { card: type });
  }

  _afterInsert(card, blockSpec) {
    const block = this.scroller.querySelector('.block[data-id="' + cssEscape(card.id) + '"]');
    if (!block) return;
    block.scrollIntoView({ block: "nearest" });
    const body = block.querySelector(".block-body");
    if (card.type === "text") {
      const root = body.querySelector(".md");
      const ed = root && editorFor(root);
      if (ed) {
        ed.focus();
        if (blockSpec && !blockSpec.card && blockSpec.id !== "text") ed.applyBlock(blockSpec);
      }
    } else if (card.type === "code") {
      this._openEditor(card, block, body, mountCodeEditor);
    } else if (card.type === "table") {
      this._openEditor(card, block, body, mountTableEditor);
    } else {
      const input = block.querySelector(".inline-controls input[type=text]");
      block.classList.add("selected");
      if (input) input.focus();
    }
  }

  _removeEmpty(card) {
    const cards = this.cards();
    const i = cards.findIndex((c) => c.id === card.id);
    if (i <= 0 || cards.length < 2) return;
    const prev = cards[i - 1];
    cards.splice(i, 1);
    this.opts.onMutate();
    this.render({ keepScroll: true });
    if (prev.type === "text") this.focusCard(prev.id, true);
  }

  focusCard(id, atEnd, newLine) {
    const block = this.scroller.querySelector('.block[data-id="' + cssEscape(id) + '"]');
    if (!block) return;
    const root = block.querySelector(".md");
    const ed = root && editorFor(root);
    if (ed) ed.focus(atEnd, newLine);
  }

  scrollToCard(id, flash) {
    const block = this.scroller.querySelector('.block[data-id="' + cssEscape(id) + '"]');
    if (!block) return;
    block.scrollIntoView({ block: "center", behavior: "smooth" });
    if (flash) {
      block.classList.add("flash");
      setTimeout(() => block.classList.remove("flash"), 1600);
    }
  }

  // ----- Outline -----

  _buildOutline() {
    const nav = this.outline;
    if (!nav) return;
    nav.innerHTML = "";
    const heads = Array.from(this.scroller.querySelectorAll(".doc-blocks .md h1, .doc-blocks .md h2, .doc-blocks .md h3"));
    this._heads = heads;
    if (heads.length < 3) {
      nav.hidden = true;
      return;
    }
    nav.hidden = false;
    nav.appendChild(el("div", { class: "doc-outline-title", text: "On this page" }));
    heads.forEach((h, i) => {
      nav.appendChild(
        el("button", {
          type: "button",
          class: "outline-item lvl-" + h.tagName[1],
          text: h.textContent,
          on: {
            click: () => h.scrollIntoView({ block: "start", behavior: "smooth" }),
          },
          dataset: { i: String(i) },
        })
      );
    });
    this._spy();
  }

  _spy() {
    if (!this._heads || !this._heads.length || !this.outline || this.outline.hidden) return;
    const top = this.scroller.getBoundingClientRect().top + 90;
    let current = 0;
    this._heads.forEach((h, i) => {
      if (h.getBoundingClientRect().top <= top) current = i;
    });
    this.outline.querySelectorAll(".outline-item").forEach((b, i) => b.classList.toggle("active", i === current));
  }
}

function cssEscape(value) {
  if (window.CSS && window.CSS.escape) return window.CSS.escape(value);
  return String(value).replace(/["\\]/g, "\\$&");
}
