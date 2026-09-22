// Inline, on-card editing. Replaces the old side inspector.
// Text cards are edited WYSIWYG in place (contenteditable + a floating selection toolbar and
// a few markdown input rules). Code and tables are edited inline too. Image, video, shape and
// draw cards get small controls baked into the card, revealed when the card is selected.
//
// Each editor mutates the card object directly. ctx provides:
//   markDirty()        autosave the board draft
//   rerenderContent()  rebuild the card body from the card data
//   uploadAsset(file)  commit a media file to the repo and return its path
//   requestCommit()    ask the board to leave edit state for this card

import { icon } from "./icons.js";
import { el } from "./dom.js";

// ----- Shared WYSIWYG selection toolbar -----

let bubble = null;

function ensureBubble() {
  if (bubble) return bubble;
  bubble = el("div", { class: "bubble-toolbar", hidden: true });
  const buttons = [
    ["bold", "Bold", () => applyFormat("bold")],
    ["italic", "Italic", () => applyFormat("italic")],
    ["strikethrough", "Strikethrough", () => applyFormat("strikeThrough")],
    ["heading-1", "Heading 1", () => applyBlock("H1")],
    ["heading-2", "Heading 2", () => applyBlock("H2")],
    ["list", "Bullet list", () => applyFormat("insertUnorderedList")],
    ["list-ordered", "Numbered list", () => applyFormat("insertOrderedList")],
    ["quote", "Quote", () => applyBlock("BLOCKQUOTE")],
    ["code", "Inline code", () => wrapInlineCode()],
    ["link", "Link", () => addLink()],
  ];
  buttons.forEach(([name, title, fn]) => {
    bubble.appendChild(
      el(
        "button",
        {
          type: "button",
          class: "bubble-btn",
          title,
          "aria-label": title,
          on: {
            // Keep the selection: don't let the button take focus.
            mousedown: (e) => e.preventDefault(),
            click: (e) => {
              e.preventDefault();
              fn();
            },
          },
        },
        icon(name, { size: 16 })
      )
    );
  });
  document.body.appendChild(bubble);
  return bubble;
}

function hideBubble() {
  if (bubble) bubble.hidden = true;
}

function positionBubble() {
  const sel = window.getSelection();
  if (!sel || sel.rangeCount === 0 || sel.isCollapsed) {
    hideBubble();
    return;
  }
  const rect = sel.getRangeAt(0).getBoundingClientRect();
  if (!rect || (!rect.width && !rect.height)) {
    hideBubble();
    return;
  }
  const bar = ensureBubble();
  bar.hidden = false;
  const bw = bar.offsetWidth || 300;
  let left = rect.left + rect.width / 2 - bw / 2;
  left = Math.max(8, Math.min(left, window.innerWidth - bw - 8));
  let top = rect.top - (bar.offsetHeight || 40) - 8;
  if (top < 8) top = rect.bottom + 8;
  bar.style.left = left + "px";
  bar.style.top = top + "px";
}

function applyFormat(cmd) {
  try {
    document.execCommand(cmd, false, null);
  } catch (err) {
    // execCommand is deprecated but universally supported; ignore failures.
  }
  positionBubble();
}

function applyBlock(tag) {
  try {
    document.execCommand("formatBlock", false, tag);
  } catch (err) {
    // Ignore.
  }
  positionBubble();
}

function wrapInlineCode() {
  const sel = window.getSelection();
  if (!sel || sel.rangeCount === 0 || sel.isCollapsed) return;
  const range = sel.getRangeAt(0);
  const code = document.createElement("code");
  try {
    code.appendChild(range.extractContents());
    range.insertNode(code);
    sel.removeAllRanges();
    const r = document.createRange();
    r.selectNodeContents(code);
    sel.addRange(r);
  } catch (err) {
    // Ignore.
  }
  positionBubble();
}

function addLink() {
  const url = window.prompt("Link URL");
  if (!url) return;
  try {
    document.execCommand("createLink", false, url);
  } catch (err) {
    // Ignore.
  }
  positionBubble();
}

// ----- Markdown input rules (type "# " etc. at the start of a block) -----

const BLOCK_RULES = {
  "#": "H1",
  "##": "H2",
  "###": "H3",
  ">": "BLOCKQUOTE",
};

function currentBlock(editor, node) {
  let block = node.nodeType === 1 ? node : node.parentNode;
  while (block && block !== editor && !/^(P|DIV|H1|H2|H3|H4|LI|BLOCKQUOTE|PRE)$/.test(block.tagName)) {
    block = block.parentNode;
  }
  return block && block !== editor ? block : editor;
}

function handleMarkdownSpace(editor, e) {
  const sel = window.getSelection();
  if (!sel || sel.rangeCount === 0) return;
  const range = sel.getRangeAt(0);
  const block = currentBlock(editor, range.endContainer);
  const probe = document.createRange();
  try {
    probe.setStart(block, 0);
    probe.setEnd(range.endContainer, range.endOffset);
  } catch (err) {
    return;
  }
  const before = probe.toString();
  if (before.indexOf(" ") !== -1 || before.indexOf("\n") !== -1) return;
  const token = before;
  const isList = token === "-" || token === "*";
  const isOrdered = token === "1.";
  const block2 = BLOCK_RULES[token];
  if (!isList && !isOrdered && !block2) return;

  e.preventDefault();
  // Remove the marker text.
  const del = document.createRange();
  del.setStart(block, 0);
  del.setEnd(range.endContainer, range.endOffset);
  del.deleteContents();

  try {
    if (isList) document.execCommand("insertUnorderedList", false, null);
    else if (isOrdered) document.execCommand("insertOrderedList", false, null);
    else document.execCommand("formatBlock", false, block2);
  } catch (err) {
    // Ignore.
  }
}

// ----- Text editor -----

export function mountTextEditor(cardEl, bodyEl, card, ctx) {
  const md = bodyEl.querySelector(".md") || bodyEl;
  md.setAttribute("contenteditable", "true");
  md.classList.add("editing-md");
  md.focus();

  const onBeforeInput = (e) => {
    if (e.inputType === "insertText" && e.data === " ") handleMarkdownSpace(md, e);
  };
  const onSelect = () => positionBubble();
  md.addEventListener("beforeinput", onBeforeInput);
  md.addEventListener("keyup", onSelect);
  md.addEventListener("mouseup", onSelect);

  return {
    commit() {
      const clean = window.DOMPurify
        ? window.DOMPurify.sanitize(md.innerHTML, { ADD_ATTR: ["target"] })
        : md.innerHTML;
      card.html = clean;
      teardown();
      ctx.markDirty();
    },
    cancel() {
      teardown();
    },
  };

  function teardown() {
    md.removeEventListener("beforeinput", onBeforeInput);
    md.removeEventListener("keyup", onSelect);
    md.removeEventListener("mouseup", onSelect);
    md.removeAttribute("contenteditable");
    md.classList.remove("editing-md");
    hideBubble();
  }
}

// ----- Code editor -----

export function mountCodeEditor(cardEl, bodyEl, card, ctx) {
  bodyEl.innerHTML = "";

  const ta = el("textarea", {
    class: "code-edit",
    spellcheck: false,
    value: card.code || "",
    on: {
      input: () => {
        card.code = ta.value;
      },
      // Tab inserts a tab instead of leaving the field.
      keydown: (e) => {
        if (e.key !== "Tab") return;
        e.preventDefault();
        const s = ta.selectionStart;
        const en = ta.selectionEnd;
        ta.value = ta.value.slice(0, s) + "\t" + ta.value.slice(en);
        ta.selectionStart = ta.selectionEnd = s + 1;
        card.code = ta.value;
      },
    },
  });

  const chip = el("button", {
    type: "button",
    class: "code-lang",
    text: card.lang || "text",
    title: "Set language",
    on: {
      mousedown: (e) => e.stopPropagation(),
      click: () => {
        const input = el("input", {
          type: "text",
          class: "code-lang-input",
          value: card.lang || "",
          placeholder: "gdscript, js, ...",
          on: {
            blur: () => {
              card.lang = input.value.trim();
              chip.textContent = card.lang || "text";
              input.replaceWith(chip);
            },
            keydown: (e) => {
              if (e.key === "Enter") input.blur();
            },
          },
        });
        chip.replaceWith(input);
        input.focus();
      },
    },
  });

  bodyEl.appendChild(el("div", { class: "code-edit-wrap" }, [ta, chip]));
  ta.focus();

  return {
    commit() {
      card.code = ta.value;
      ctx.rerenderContent();
      ctx.markDirty();
    },
    cancel() {
      ctx.rerenderContent();
    },
  };
}

// ----- Table editor -----

export function mountTableEditor(cardEl, bodyEl, card, ctx) {
  const rows = Array.isArray(card.rows) ? card.rows : (card.rows = [[""]]);
  const table = bodyEl.querySelector("table.card-table");
  if (table) {
    const cells = table.querySelectorAll("th, td");
    cells.forEach((cell) => {
      const tr = cell.parentNode;
      const ri = Array.prototype.indexOf.call(tr.parentNode.children, tr);
      const ci = Array.prototype.indexOf.call(tr.children, cell);
      cell.setAttribute("contenteditable", "true");
      cell.addEventListener("input", () => {
        if (rows[ri]) rows[ri][ci] = cell.textContent;
      });
    });
    if (cells.length) cells[0].focus();
  }

  const bar = el("div", { class: "tbl-adders" });
  bar.appendChild(
    adder("+ row", () => {
      const cols = (rows[0] || [""]).length;
      rows.push(new Array(cols).fill(""));
      ctx.markDirty();
      ctx.requestCommit();
      ctx.rerenderContent();
    })
  );
  bar.appendChild(
    adder("+ col", () => {
      rows.forEach((r) => r.push(""));
      ctx.markDirty();
      ctx.requestCommit();
      ctx.rerenderContent();
    })
  );
  bar.appendChild(
    adder("− col", () => {
      if ((rows[0] || []).length <= 1) return;
      rows.forEach((r) => r.pop());
      ctx.markDirty();
      ctx.requestCommit();
      ctx.rerenderContent();
    })
  );
  bodyEl.appendChild(bar);

  function adder(label, fn) {
    return el("button", {
      type: "button",
      class: "tbl-add",
      text: label,
      on: { mousedown: (e) => e.stopPropagation(), click: fn },
    });
  }

  return {
    commit() {
      ctx.rerenderContent();
      ctx.markDirty();
    },
    cancel() {
      ctx.rerenderContent();
    },
  };
}

// ----- Persistent on-card controls for image / video / shape / draw -----
// These are added to the card in edit mode and revealed when the card is selected. They
// mutate the card and re-render its content live, so there is no separate commit step.

export function mountSelectionControls(cardEl, bodyEl, card, ctx) {
  let controls = null;
  if (card.type === "image") controls = imageControls(card, ctx);
  else if (card.type === "video") controls = videoControls(card, ctx);
  else if (card.type === "shape") controls = shapeControls(card, ctx);
  else if (card.type === "draw") controls = drawControls(card, ctx);
  if (controls) {
    controls.className = "inline-controls";
    // Controls must not start a card drag.
    controls.addEventListener("pointerdown", (e) => e.stopPropagation());
    controls.addEventListener("dblclick", (e) => e.stopPropagation());
    cardEl.appendChild(controls);
  }
}

function labeledInput(type, value, placeholder, oninput) {
  const input = el("input", {
    type,
    value,
    placeholder,
    on: { input: () => oninput(input.value) },
  });
  return input;
}

function ctrlButton(label, onClick) {
  return el("button", { type: "button", class: "inline-btn", text: label, on: { click: onClick } });
}

function colorInput(value, fallback, onPick) {
  return el("input", {
    type: "color",
    class: "inline-color",
    value: toHex(value, fallback),
    title: "Stroke",
    on: { input: (e) => onPick(e.target.value) },
  });
}

function imageControls(card, ctx) {
  const file = el("input", {
    type: "file",
    accept: "image/*",
    style: { display: "none" },
    on: {
      change: async () => {
        const f = file.files && file.files[0];
        if (!f) return;
        const rel = await ctx.uploadAsset(f);
        if (rel) {
          card.src = rel;
          card.svg = "";
          ctx.rerenderContent();
          ctx.markDirty();
        }
        file.value = "";
      },
    },
  });
  return el("div", {}, [
    ctrlButton("Upload", () => file.click()),
    file,
    labeledInput("text", card.src || "", "image URL or assets/...", (v) => {
      card.src = v.trim();
      if (v.trim()) card.svg = "";
      ctx.rerenderContent();
      ctx.markDirty();
    }),
  ]);
}

function videoControls(card, ctx) {
  const toggle = ctrlButton(card.embed ? "Embed" : "File", () => {
    card.embed = !card.embed;
    toggle.textContent = card.embed ? "Embed" : "File";
    ctx.rerenderContent();
    ctx.markDirty();
  });
  return el("div", {}, [
    toggle,
    labeledInput("text", card.src || "", "mp4 URL or YouTube/Vimeo link", (v) => {
      card.src = v.trim();
      ctx.rerenderContent();
      ctx.markDirty();
    }),
  ]);
}

function shapeControls(card, ctx) {
  const wrap = el("div");
  const kinds = [
    ["rect", "square"],
    ["ellipse", "circle"],
    ["line", "minus"],
    ["arrow", "move-right"],
  ];
  kinds.forEach(([kind, iconName]) => {
    const b = el(
      "button",
      {
        type: "button",
        class: "inline-btn icon-btn" + ((card.shape || "rect") === kind ? " active" : ""),
        title: kind,
        on: {
          click: () => {
            card.shape = kind;
            wrap.querySelectorAll(".icon-btn").forEach((x) => x.classList.remove("active"));
            b.classList.add("active");
            ctx.rerenderContent();
            ctx.markDirty();
          },
        },
      },
      icon(iconName, { size: 15 })
    );
    wrap.appendChild(b);
  });
  wrap.appendChild(
    colorInput(card.stroke, "#58b3e6", (v) => {
      card.stroke = v;
      ctx.rerenderContent();
      ctx.markDirty();
    })
  );
  return wrap;
}

function drawControls(card, ctx) {
  return el("div", {}, [
    ctrlButton("Clear", () => {
      card.path = "";
      ctx.rerenderContent();
      ctx.markDirty();
    }),
    colorInput(card.stroke, "#f0d24a", (v) => {
      card.stroke = v;
      ctx.rerenderContent();
      ctx.markDirty();
    }),
  ]);
}

function toHex(value, fallback) {
  if (typeof value === "string" && /^#[0-9a-fA-F]{6}$/.test(value)) return value;
  return fallback;
}
