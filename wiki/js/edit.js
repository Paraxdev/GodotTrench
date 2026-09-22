// In-place editing for every card type. Text is WYSIWYG: the rendered Markdown becomes
// contenteditable, and htmlToMarkdown turns it back into source on commit, so what is stored in
// the repo is always readable Markdown.
//
// Each editor mutates its card and talks to its host (the document or the canvas) through ctx:
//   markDirty()                 autosave the board draft
//   rerenderContent()           rebuild the card body from the card data
//   uploadAsset(file)           commit a media file to the repo and return its path
//   requestCommit()             ask the host to leave edit state for this card
//   insertBlock(type, extra)    add a new card after this one (a doc splits the text at the caret)
//   pasteFiles(files)           turn pasted images into image cards
//   pages()                     [{ slug, title }] for page links

import { icon } from "./icons.js";
import { el } from "./dom.js";
import { htmlToMarkdown, renderMarkdownInto, buildCallout } from "./markdown.js";

// ----- Block catalogue, shared by the slash menu, the + menus and the canvas add menu -----

export const BLOCKS = [
  { id: "text", label: "Text", icon: "text", hint: "Plain paragraph" },
  { id: "h1", label: "Heading 1", icon: "heading-1", hint: "#" },
  { id: "h2", label: "Heading 2", icon: "heading-2", hint: "##" },
  { id: "h3", label: "Heading 3", icon: "heading-3", hint: "###" },
  { id: "bullet", label: "Bulleted list", icon: "list", hint: "-" },
  { id: "number", label: "Numbered list", icon: "list-ordered", hint: "1." },
  { id: "todo", label: "To-do list", icon: "list-todo", hint: "[]" },
  { id: "quote", label: "Quote", icon: "quote", hint: ">" },
  { id: "note", label: "Note callout", icon: "info" },
  { id: "tip", label: "Tip callout", icon: "lightbulb" },
  { id: "warning", label: "Warning callout", icon: "triangle-alert" },
  { id: "divider", label: "Divider", icon: "separator", hint: "---" },
  { id: "pagelink", label: "Link to page", icon: "file-link", hint: "[[" },
  { id: "code", label: "Code block", icon: "square-code", hint: "```", card: "code" },
  { id: "table", label: "Table", icon: "table", card: "table" },
  { id: "image", label: "Image", icon: "image", card: "image" },
  { id: "video", label: "Video", icon: "video", card: "video" },
  { id: "draw", label: "Drawing", icon: "pen-tool", card: "draw" },
  { id: "shape", label: "Shape", icon: "shapes", card: "shape" },
];

export function blockById(id) {
  return BLOCKS.find((b) => b.id === id);
}

// ----- Floating suggestion list (slash menu, page links, + menus) -----

let suggest = null;

function ensureSuggest() {
  if (suggest) return suggest;
  const box = el("div", { class: "suggest", role: "listbox", hidden: true });
  box.addEventListener("mousedown", (e) => e.preventDefault());
  document.body.appendChild(box);
  suggest = { box, items: [], index: 0, onPick: null, onClose: null };
  return suggest;
}

export function suggestOpen() {
  return !!(suggest && !suggest.box.hidden);
}

// items: [{ label, icon, hint, value }]. Positions under the rect.
export function openSuggest(rect, items, onPick, opts) {
  const s = ensureSuggest();
  s.onPick = onPick;
  s.onClose = (opts && opts.onClose) || null;
  s.title = (opts && opts.title) || "";
  s.empty = (opts && opts.empty) || "No matches";
  setSuggestItems(items);
  s.box.hidden = false;
  placeSuggest(rect);
}

export function setSuggestItems(items) {
  const s = ensureSuggest();
  s.items = items;
  s.index = 0;
  s.box.innerHTML = "";
  if (s.title) s.box.appendChild(el("div", { class: "suggest-title", text: s.title }));
  if (!items.length) {
    s.box.appendChild(el("div", { class: "suggest-empty", text: s.empty }));
    return;
  }
  items.forEach((item, i) => {
    s.box.appendChild(
      el(
        "button",
        {
          type: "button",
          class: "suggest-item" + (i === 0 ? " active" : ""),
          role: "option",
          on: {
            click: () => pickSuggest(i),
            mousemove: () => highlightSuggest(i),
          },
        },
        [
          el("span", { class: "suggest-icon" }, icon(item.icon || "text", { size: 16 })),
          el("span", { class: "suggest-label", text: item.label }),
          item.hint ? el("span", { class: "suggest-hint", text: item.hint }) : null,
        ]
      )
    );
  });
}

function placeSuggest(rect) {
  const box = suggest.box;
  const w = box.offsetWidth || 260;
  const h = box.offsetHeight || 300;
  let left = rect.left;
  let top = rect.bottom + 6;
  if (left + w > window.innerWidth - 8) left = window.innerWidth - w - 8;
  if (top + h > window.innerHeight - 8) top = Math.max(8, rect.top - h - 6);
  box.style.left = Math.max(8, left) + "px";
  box.style.top = Math.max(8, top) + "px";
}

function highlightSuggest(i) {
  const s = suggest;
  s.index = i;
  s.box.querySelectorAll(".suggest-item").forEach((b, j) => b.classList.toggle("active", j === i));
  const active = s.box.querySelectorAll(".suggest-item")[i];
  if (active) active.scrollIntoView({ block: "nearest" });
}

// The pick runs before onClose, because onClose resets the state the pick still needs, such as
// where the typed "/query" starts.
function pickSuggest(i) {
  const s = suggest;
  const item = s.items[i];
  const fn = s.onPick;
  const onClose = s.onClose;
  s.box.hidden = true;
  s.onPick = null;
  s.onClose = null;
  if (item && fn) fn(item.value !== undefined ? item.value : item);
  if (onClose) onClose(true);
}

export function closeSuggest(picked) {
  if (!suggest || suggest.box.hidden) return;
  suggest.box.hidden = true;
  const onClose = suggest.onClose;
  suggest.onPick = null;
  suggest.onClose = null;
  if (onClose) onClose(!!picked);
}

// Keyboard navigation while a suggestion list is open. Returns true when the key was used.
export function suggestKey(e) {
  if (!suggestOpen()) return false;
  const s = suggest;
  if (e.key === "ArrowDown") {
    highlightSuggest((s.index + 1) % Math.max(1, s.items.length));
  } else if (e.key === "ArrowUp") {
    highlightSuggest((s.index - 1 + s.items.length) % Math.max(1, s.items.length));
  } else if (e.key === "Enter" || e.key === "Tab") {
    if (!s.items.length) {
      closeSuggest();
      return false;
    }
    pickSuggest(s.index);
  } else if (e.key === "Escape") {
    closeSuggest();
  } else {
    return false;
  }
  e.preventDefault();
  e.stopPropagation();
  return true;
}

document.addEventListener("pointerdown", (e) => {
  if (suggest && !suggest.box.hidden && !suggest.box.contains(e.target)) closeSuggest();
});

export function blockMenuItems(filter) {
  return BLOCKS.filter((b) => !filter || filter(b)).map((b) => ({ label: b.label, icon: b.icon, hint: b.hint, value: b }));
}

function filterBlocks(query) {
  const q = query.toLowerCase();
  return blockMenuItems((b) => !q || b.label.toLowerCase().includes(q) || b.id.startsWith(q));
}

function pageItems(ctx, query) {
  const q = query.toLowerCase();
  return ctx
    .pages()
    .filter((p) => !q || p.title.toLowerCase().includes(q) || p.slug.includes(q))
    .slice(0, 12)
    .map((p) => ({ label: p.title, icon: "file-text", hint: p.slug, value: p }));
}

// ----- Selection toolbar -----

let bubble = null;
let bubbleEditor = null;

function ensureBubble() {
  if (bubble) return bubble;
  bubble = el("div", { class: "bubble-toolbar", hidden: true });
  bubble.addEventListener("mousedown", (e) => {
    if (e.target.tagName !== "INPUT") e.preventDefault();
  });
  document.body.appendChild(bubble);
  return bubble;
}

function bubbleButtons() {
  const bar = ensureBubble();
  bar.innerHTML = "";
  const buttons = [
    ["bold", "Bold  Ctrl+B", () => exec("bold")],
    ["italic", "Italic  Ctrl+I", () => exec("italic")],
    ["strikethrough", "Strikethrough", () => exec("strikeThrough")],
    ["code", "Inline code  Ctrl+E", () => toggleInlineCode()],
    ["link", "Link  Ctrl+K", () => showLinkInput()],
    "sep",
    ["heading-1", "Heading 1", () => toggleBlock("H1")],
    ["heading-2", "Heading 2", () => toggleBlock("H2")],
    ["heading-3", "Heading 3", () => toggleBlock("H3")],
    ["list", "Bulleted list", () => exec("insertUnorderedList")],
    ["list-ordered", "Numbered list", () => exec("insertOrderedList")],
    ["quote", "Quote", () => toggleBlock("BLOCKQUOTE")],
  ];
  buttons.forEach((spec) => {
    if (spec === "sep") {
      bar.appendChild(el("span", { class: "bubble-sep" }));
      return;
    }
    const [name, title, fn] = spec;
    bar.appendChild(
      el(
        "button",
        {
          type: "button",
          class: "bubble-btn",
          title,
          "aria-label": title,
          on: {
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
}

function hideBubble() {
  if (bubble) bubble.hidden = true;
}

function positionBubble(root) {
  const sel = window.getSelection();
  if (!sel || sel.rangeCount === 0 || sel.isCollapsed || !root.contains(sel.anchorNode)) {
    if (!bubble || !bubble.classList.contains("linking")) hideBubble();
    return;
  }
  const rect = sel.getRangeAt(0).getBoundingClientRect();
  if (!rect || (!rect.width && !rect.height)) {
    hideBubble();
    return;
  }
  const bar = ensureBubble();
  if (bar.hidden || bar.classList.contains("linking")) {
    bar.classList.remove("linking");
    bubbleButtons();
  }
  bar.hidden = false;
  const bw = bar.offsetWidth || 360;
  let left = rect.left + rect.width / 2 - bw / 2;
  left = Math.max(8, Math.min(left, window.innerWidth - bw - 8));
  let top = rect.top - (bar.offsetHeight || 40) - 10;
  if (top < 8) top = rect.bottom + 10;
  bar.style.left = left + "px";
  bar.style.top = top + "px";
}

function exec(cmd, value) {
  try {
    document.execCommand(cmd, false, value == null ? null : value);
  } catch (err) {
    // execCommand is deprecated but still the only way to get native undo inside contenteditable.
  }
  if (bubbleEditor) positionBubble(bubbleEditor);
}

function closestIn(root, node, test) {
  let n = node && node.nodeType === 3 ? node.parentNode : node;
  while (n && n !== root) {
    if (test(n)) return n;
    n = n.parentNode;
  }
  return null;
}

function currentBlock(root) {
  const sel = window.getSelection();
  if (!sel || !sel.rangeCount) return null;
  return closestIn(root, sel.getRangeAt(0).startContainer, (n) => /^(P|DIV|H[1-6]|LI|BLOCKQUOTE|PRE)$/.test(n.tagName) && !n.classList.contains("callout-body"));
}

function toggleBlock(tag) {
  const root = bubbleEditor;
  const block = root && currentBlock(root);
  const same = block && (block.tagName === tag || (tag === "BLOCKQUOTE" && closestIn(root, block, (n) => n.tagName === "BLOCKQUOTE")));
  exec("formatBlock", same ? "P" : tag);
}

function toggleInlineCode() {
  const sel = window.getSelection();
  if (!sel || !sel.rangeCount) return;
  const root = bubbleEditor;
  const existing = root && closestIn(root, sel.anchorNode, (n) => n.tagName === "CODE");
  if (existing) {
    existing.replaceWith(document.createTextNode(existing.textContent));
    return;
  }
  if (sel.isCollapsed) return;
  const text = sel.toString();
  exec("insertHTML", "<code>" + escapeHtml(text) + "</code>&#8203;");
}

function escapeHtml(s) {
  return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

// The link field replaces the bubble buttons. Focus moves into the field, so the selection is
// saved first and restored before the link is applied.
function showLinkInput() {
  const sel = window.getSelection();
  if (!sel || !sel.rangeCount) return;
  const saved = sel.getRangeAt(0).cloneRange();
  const root = bubbleEditor;
  const bar = ensureBubble();
  bar.classList.add("linking");
  bar.innerHTML = "";
  const input = el("input", {
    class: "bubble-input",
    type: "text",
    placeholder: "Paste a URL, or type a page name",
    spellcheck: false,
  });
  const apply = (value) => {
    root.focus();
    const s = window.getSelection();
    s.removeAllRanges();
    s.addRange(saved);
    bar.classList.remove("linking");
    hideBubble();
    const v = value.trim();
    if (!v) return;
    const page = ctxOf(root) && ctxOf(root).pages().find((p) => p.slug === v || p.title.toLowerCase() === v.toLowerCase());
    if (page || !/[:/.#]/.test(v)) {
      const target = page ? page.slug : v;
      const label = saved.toString() || (page ? page.title : v);
      exec("insertHTML", wikilinkHtml(target, label) + "&nbsp;");
    } else {
      exec("createLink", /^[a-z]+:|^#|^\//i.test(v) ? v : "https://" + v);
    }
  };
  input.addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      apply(input.value);
    } else if (e.key === "Escape") {
      e.preventDefault();
      apply("");
    }
  });
  input.addEventListener("blur", () => {
    if (bar.classList.contains("linking")) {
      bar.classList.remove("linking");
      hideBubble();
    }
  });
  bar.appendChild(el("span", { class: "bubble-input-icon" }, icon("link", { size: 15 })));
  bar.appendChild(input);
  bar.hidden = false;
  input.focus();
}

function wikilinkHtml(target, label) {
  return (
    '<a class="wikilink" href="#' + escapeHtml(target) + '" data-target="' + escapeHtml(target) + '">' + escapeHtml(label) + "</a>"
  );
}

// ----- Caret helpers -----

function caretRect() {
  const sel = window.getSelection();
  if (!sel || !sel.rangeCount) return null;
  const range = sel.getRangeAt(0).cloneRange();
  range.collapse(true);
  let rect = range.getClientRects()[0];
  if (!rect) {
    const node = range.startContainer.nodeType === 1 ? range.startContainer : range.startContainer.parentNode;
    rect = node.getBoundingClientRect();
  }
  return rect;
}

// Text between the start of the caret's text node and the caret.
function textBeforeCaret() {
  const sel = window.getSelection();
  if (!sel || !sel.rangeCount || !sel.isCollapsed) return null;
  const node = sel.anchorNode;
  if (!node || node.nodeType !== 3) return null;
  return { node, offset: sel.anchorOffset, text: node.data.slice(0, sel.anchorOffset) };
}

function deleteBack(node, offset, count) {
  const r = document.createRange();
  r.setStart(node, offset - count);
  r.setEnd(node, offset);
  const sel = window.getSelection();
  sel.removeAllRanges();
  sel.addRange(r);
  exec("delete");
}

function placeCaretAtEnd(node) {
  const r = document.createRange();
  r.selectNodeContents(node);
  r.collapse(false);
  const sel = window.getSelection();
  sel.removeAllRanges();
  sel.addRange(r);
}

// ----- Text editor -----

const editors = new WeakMap();

function ctxOf(root) {
  const ed = editors.get(root);
  return ed ? ed.ctx : null;
}

// opts.persistent: the document keeps every text block editable and commits on blur. The canvas
// mounts one editor at a time and commits when the card is left.
export function mountTextEditor(root, card, ctx, opts) {
  const persistent = !!(opts && opts.persistent);
  const format = card.format || "markdown";
  root.setAttribute("contenteditable", "true");
  root.classList.add("editing-md");
  // Without a paragraph to type into, the first line becomes a bare text node and block
  // formatting ends up wrapping the whole editor.
  if (!root.firstElementChild && !root.textContent.trim()) root.innerHTML = "<p><br></p>";
  root.dataset.placeholder = "Write something, or type / for blocks";
  root.querySelectorAll("input[type=checkbox]").forEach((box) => box.removeAttribute("disabled"));
  root.querySelectorAll(".callout-icon").forEach((n) => n.setAttribute("contenteditable", "false"));
  try {
    document.execCommand("defaultParagraphSeparator", false, "p");
  } catch (err) {
    // Older engines keep div, which the serializer also understands.
  }

  let slash = null;
  let linkQuery = null;
  let committed = serialize();
  const syncEmpty = () => root.classList.toggle("is-empty", isEmpty(root));
  syncEmpty();

  function serialize() {
    if (format === "html") {
      const clone = root.cloneNode(true);
      clone.querySelectorAll("[contenteditable]").forEach((n) => n.removeAttribute("contenteditable"));
      return window.DOMPurify ? window.DOMPurify.sanitize(clone.innerHTML) : clone.innerHTML;
    }
    return htmlToMarkdown(root);
  }

  function save() {
    const next = serialize();
    if (next === committed) return false;
    committed = next;
    card.md = next;
    if (format === "bbcode") card.format = "markdown";
    delete card.html;
    ctx.markDirty();
    return true;
  }

  // Slash menu: "/" at the start of a line or after a space.
  function updateSlash() {
    const t = textBeforeCaret();
    if (!t) return closeSlash();
    if (!slash) {
      const m = /(^|\s)\/$/.exec(t.text);
      if (!m) return;
      slash = { node: t.node, start: t.offset - 1 };
      openSuggest(caretRect(), filterBlocks(""), (b) => chooseBlock(b), {
        title: "Blocks",
        onClose: () => {
          slash = null;
        },
      });
      return;
    }
    if (t.node !== slash.node || t.offset <= slash.start) return closeSlash();
    const query = t.node.data.slice(slash.start + 1, t.offset);
    if (/\s/.test(query) || query.length > 24) return closeSlash();
    setSuggestItems(filterBlocks(query));
  }

  function closeSlash() {
    if (slash) closeSuggest();
    slash = null;
  }

  function chooseBlock(b) {
    const t = textBeforeCaret();
    if (slash && t && t.node === slash.node) deleteBack(t.node, t.offset, t.offset - slash.start);
    slash = null;
    applyBlock(b);
  }

  // "[[" opens a page picker, Obsidian style.
  function updateLinkQuery() {
    const t = textBeforeCaret();
    const m = t && /\[\[([^[\]\n]*)$/.exec(t.text);
    if (!m) {
      if (linkQuery) closeSuggest();
      linkQuery = null;
      return;
    }
    linkQuery = { node: t.node, offset: t.offset, length: m[0].length };
    const items = pageItems(ctx, m[1]);
    if (!suggestOpen()) {
      openSuggest(caretRect(), items, (p) => insertPageLink(p), {
        title: "Link to page",
        empty: "No page with that name yet",
        onClose: () => {
          linkQuery = null;
        },
      });
    } else {
      setSuggestItems(items);
    }
  }

  function insertPageLink(p) {
    const t = textBeforeCaret();
    if (linkQuery && t && t.node === linkQuery.node) deleteBack(t.node, t.offset, linkQuery.length);
    linkQuery = null;
    exec("insertHTML", wikilinkHtml(p.slug, p.title) + "&nbsp;");
  }

  function applyBlock(b) {
    if (b.card) {
      ctx.insertBlock(b.card, { from: root });
      return;
    }
    switch (b.id) {
      case "text":
        exec("formatBlock", "P");
        break;
      case "h1":
      case "h2":
      case "h3":
        exec("formatBlock", b.id.toUpperCase());
        break;
      case "bullet":
        exec("insertUnorderedList");
        break;
      case "number":
        exec("insertOrderedList");
        break;
      case "todo":
        insertTodo();
        break;
      case "quote":
        exec("formatBlock", "BLOCKQUOTE");
        break;
      case "note":
      case "tip":
      case "warning":
        insertCallout(b.id);
        break;
      case "divider":
        exec("insertHTML", "<hr><p><br></p>");
        break;
      case "pagelink": {
        exec("insertText", "[[");
        updateLinkQuery();
        break;
      }
      default:
        break;
    }
    unwrapBlocksInParagraphs();
    syncEmpty();
  }

  // Chrome sometimes builds a list inside the paragraph it converted, which is invalid HTML.
  function unwrapBlocksInParagraphs() {
    root.querySelectorAll("p > ul, p > ol, p > blockquote, p > h1, p > h2, p > h3, p > hr").forEach((child) => {
      const p = child.parentNode;
      const sel = window.getSelection();
      const range = sel && sel.rangeCount ? sel.getRangeAt(0).cloneRange() : null;
      while (p.firstChild) p.parentNode.insertBefore(p.firstChild, p);
      p.remove();
      if (range) {
        sel.removeAllRanges();
        sel.addRange(range);
      }
    });
  }

  function insertTodo() {
    exec("insertUnorderedList");
    const li = closestIn(root, window.getSelection().anchorNode, (n) => n.tagName === "LI");
    if (li && !li.querySelector(":scope > input[type=checkbox]")) {
      li.classList.add("task");
      li.insertBefore(el("input", { type: "checkbox" }), li.firstChild);
      li.insertBefore(document.createTextNode(" "), li.firstChild.nextSibling);
      placeCaretAtEnd(li);
    }
  }

  function insertCallout(kind) {
    const block = currentBlock(root);
    const box = buildCallout(kind, "");
    const body = box.querySelector(".callout-body");
    const p = el("p");
    if (block && block !== root && block.tagName === "P" && block.parentNode === root) {
      while (block.firstChild) p.appendChild(block.firstChild);
      block.replaceWith(box);
    } else {
      root.appendChild(box);
    }
    if (!p.firstChild) p.appendChild(el("br"));
    body.appendChild(p);
    placeCaretAtEnd(p);
    ctx.markDirty();
  }

  // Markdown shortcuts typed at the start of a block, followed by a space.
  const BLOCK_RULES = { "#": "H1", "##": "H2", "###": "H3", ">": "BLOCKQUOTE" };

  function blockRule(e) {
    const t = textBeforeCaret();
    if (!t) return false;
    const block = currentBlock(root);
    const probe = document.createRange();
    try {
      probe.setStart(block || root, 0);
      probe.setEnd(t.node, t.offset);
    } catch (err) {
      return false;
    }
    const before = probe.toString();
    const kind =
      BLOCK_RULES[before] ||
      (before === "-" || before === "*" ? "bullet" : before === "1." ? "number" : before === "[]" || before === "[ ]" ? "todo" : before === "---" ? "divider" : before === "```" ? "code" : null);
    if (!kind) return false;
    e.preventDefault();
    deleteBack(t.node, t.offset, before.length);
    if (kind === "bullet" || kind === "number" || kind === "todo" || kind === "divider") applyBlock(blockById(kind));
    else if (kind === "code") ctx.insertBlock("code", { from: root });
    else exec("formatBlock", kind);
    return true;
  }

  // Inline shortcuts completed by their closing marker: **bold**, `code`, ~~strike~~, [[page]].
  function inlineRule() {
    const t = textBeforeCaret();
    if (!t) return;
    const rules = [
      [/\*\*([^*\n]+)\*\*$/, (m) => "<strong>" + escapeHtml(m[1]) + "</strong>"],
      [/`([^`\n]+)`$/, (m) => "<code>" + escapeHtml(m[1]) + "</code>"],
      [/~~([^~\n]+)~~$/, (m) => "<s>" + escapeHtml(m[1]) + "</s>"],
      [
        /\[\[([^[\]|\n]+)(?:\|([^[\]\n]+))?\]\]$/,
        (m) => {
          const page = ctx.pages().find((p) => p.slug === m[1].trim() || p.title.toLowerCase() === m[1].trim().toLowerCase());
          return wikilinkHtml(page ? page.slug : m[1].trim(), m[2] ? m[2].trim() : page ? page.title : m[1].trim());
        },
      ],
    ];
    if (closestIn(root, t.node, (n) => n.tagName === "CODE" || n.tagName === "PRE")) return;
    for (const [re, html] of rules) {
      const m = re.exec(t.text);
      if (!m) continue;
      deleteBack(t.node, t.offset, m[0].length);
      exec("insertHTML", html(m) + "&#8203;");
      return;
    }
  }

  function onBeforeInput(e) {
    if (e.inputType === "insertText" && e.data === " " && blockRule(e)) return;
  }

  function onInput(e) {
    syncEmpty();
    if (e.inputType === "insertText" || e.inputType === "deleteContentBackward") {
      updateSlash();
      updateLinkQuery();
    }
    if (e.inputType === "insertText" && /[*`~\]]/.test(e.data || "")) inlineRule();
    if (bubble && !bubble.hidden) positionBubble(root);
  }

  function onKeyDown(e) {
    if (suggestKey(e)) return;
    const mod = e.ctrlKey || e.metaKey;
    if (mod && !e.shiftKey && e.key.toLowerCase() === "k") {
      e.preventDefault();
      if (window.getSelection().isCollapsed) {
        exec("insertText", "[[");
        updateLinkQuery();
      } else {
        bubbleEditor = root;
        positionBubble(root);
        showLinkInput();
      }
      return;
    }
    if (mod && !e.shiftKey && e.key.toLowerCase() === "e") {
      e.preventDefault();
      bubbleEditor = root;
      toggleInlineCode();
      return;
    }
    if (mod && e.shiftKey && e.key.toLowerCase() === "x") {
      e.preventDefault();
      exec("strikeThrough");
      return;
    }
    if (e.key === "Escape") {
      e.preventDefault();
      if (persistent) root.blur();
      else ctx.requestCommit();
      return;
    }
    if (e.key === "Tab") {
      const li = closestIn(root, window.getSelection().anchorNode, (n) => n.tagName === "LI");
      if (li) {
        e.preventDefault();
        exec(e.shiftKey ? "outdent" : "indent");
        return;
      }
    }
    if (e.key === "Enter" && !e.shiftKey) {
      const pre = closestIn(root, window.getSelection().anchorNode, (n) => n.tagName === "PRE");
      if (pre) {
        e.preventDefault();
        exec("insertText", "\n");
        return;
      }
      if (escapeContainer(e)) return;
      if (continueTodo(e)) return;
    }
    if (e.key === "Backspace" && isEmpty(root) && ctx.removeEmpty) {
      e.preventDefault();
      ctx.removeEmpty();
    }
  }

  // Enter on an empty last line of a quote or callout steps out of it.
  function escapeContainer(e) {
    const block = currentBlock(root);
    if (!block || block.textContent.trim()) return false;
    const container = closestIn(root, block, (n) => n.tagName === "BLOCKQUOTE" || (n.classList && n.classList.contains("callout")));
    if (!container) return false;
    const inner = container.classList.contains("callout") ? container.querySelector(".callout-body") : container;
    if (inner.lastElementChild !== block) return false;
    e.preventDefault();
    block.remove();
    const p = el("p", {}, el("br"));
    container.after(p);
    if (inner.children.length === 0) container.remove();
    placeCaretAtEnd(p);
    ctx.markDirty();
    return true;
  }

  // New items in a to-do list start unchecked, like the previous item had a box.
  function continueTodo(e) {
    const li = closestIn(root, window.getSelection().anchorNode, (n) => n.tagName === "LI");
    if (!li || !li.querySelector(":scope > input[type=checkbox]")) return false;
    const text = li.textContent.trim();
    e.preventDefault();
    if (!text) {
      exec("insertUnorderedList");
      li.querySelectorAll(":scope > input").forEach((n) => n.remove());
      return true;
    }
    const next = el("li", { class: "task" }, [el("input", { type: "checkbox" }), " "]);
    li.after(next);
    placeCaretAtEnd(next);
    return true;
  }

  function onPaste(e) {
    const dt = e.clipboardData;
    if (!dt) return;
    const files = Array.from(dt.files || []).filter((f) => f.type.startsWith("image/"));
    if (files.length && ctx.pasteFiles) {
      e.preventDefault();
      ctx.pasteFiles(files, root);
      return;
    }
    if (closestIn(root, window.getSelection().anchorNode, (n) => n.tagName === "PRE" || n.tagName === "CODE")) {
      e.preventDefault();
      exec("insertText", dt.getData("text/plain"));
      return;
    }
    const html = dt.getData("text/html");
    const text = dt.getData("text/plain");
    e.preventDefault();
    if (html && window.DOMPurify) {
      // Normalize foreign markup through Markdown so pasted styles and classes never leak in.
      const tmp = document.createElement("div");
      tmp.innerHTML = window.DOMPurify.sanitize(html, {
        ALLOWED_TAGS: ["p", "br", "b", "strong", "i", "em", "s", "del", "code", "pre", "a", "ul", "ol", "li", "blockquote", "h1", "h2", "h3", "h4", "table", "thead", "tbody", "tr", "th", "td", "img", "hr", "div", "span"],
        ALLOWED_ATTR: ["href", "src", "alt"],
      });
      insertMarkdown(htmlToMarkdown(tmp));
    } else if (text && /(^|\n)\s*(#{1,3} |[-*] |\d+\. |> |```)|\*\*|\[\[/.test(text)) {
      insertMarkdown(text);
    } else {
      exec("insertText", text);
    }
  }

  function insertMarkdown(md) {
    const tmp = document.createElement("div");
    if (!renderMarkdownInto(tmp, md)) {
      exec("insertText", md);
      return;
    }
    // A single paragraph goes in inline, so pasting a phrase does not break the line.
    const only = tmp.children.length === 1 && tmp.firstElementChild.tagName === "P" ? tmp.firstElementChild : null;
    exec("insertHTML", only ? only.innerHTML : tmp.innerHTML);
  }

  const onSelect = () => {
    bubbleEditor = root;
    positionBubble(root);
  };
  const onFocus = () => {
    bubbleEditor = root;
    root.classList.add("focused");
    if (ctx.onFocus) ctx.onFocus();
  };
  const onBlur = () => {
    root.classList.remove("focused");
    closeSlash();
    if (persistent) {
      save();
      setTimeout(() => {
        if (document.activeElement !== root && !(bubble && bubble.contains(document.activeElement))) hideBubble();
      }, 0);
    }
  };
  // Links navigate only with Ctrl or Cmd held, so a click can place the caret inside them.
  const onClick = (e) => {
    const a = e.target.closest && e.target.closest("a[href]");
    if (a && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      save();
      const href = a.getAttribute("href");
      if (href.startsWith("#")) location.hash = href.slice(1);
      else window.open(href, "_blank", "noopener");
    } else if (a) {
      e.preventDefault();
    }
    if (e.target.matches && e.target.matches("input[type=checkbox]")) {
      setTimeout(() => {
        if (e.target.checked) e.target.setAttribute("checked", "");
        else e.target.removeAttribute("checked");
        save();
      }, 0);
    }
  };

  root.addEventListener("beforeinput", onBeforeInput);
  root.addEventListener("input", onInput);
  root.addEventListener("keydown", onKeyDown);
  root.addEventListener("keyup", onSelect);
  root.addEventListener("mouseup", onSelect);
  root.addEventListener("paste", onPaste);
  root.addEventListener("focus", onFocus);
  root.addEventListener("blur", onBlur);
  root.addEventListener("click", onClick);

  const api = {
    ctx,
    root,
    save,
    // newLine starts a fresh paragraph at the end, so typing under a heading does not extend it.
    focus(atEnd, newLine) {
      root.focus();
      if (newLine && root.lastElementChild && root.lastElementChild.tagName !== "P") {
        const p = el("p", {}, el("br"));
        root.appendChild(p);
        placeCaretAtEnd(p);
        return;
      }
      if (atEnd !== false) placeCaretAtEnd(root);
    },
    applyBlock,
    // Split the text at the caret: the text before stays in this card, the returned Markdown
    // is what followed the caret. Used when a code block or image is inserted mid-text.
    splitAtCaret() {
      const sel = window.getSelection();
      if (!sel || !sel.rangeCount || !root.contains(sel.anchorNode)) return "";
      const r = sel.getRangeAt(0).cloneRange();
      r.setEndAfter(root.lastChild || root);
      const tail = document.createElement("div");
      tail.appendChild(r.extractContents());
      const tailMd = htmlToMarkdown(tail);
      root.querySelectorAll("p, h1, h2, h3, li").forEach((n) => {
        if (!n.textContent.trim() && !n.querySelector("img, input") && n.parentNode) n.remove();
      });
      save();
      syncEmpty();
      return tailMd;
    },
    commit() {
      save();
      teardown();
    },
    cancel() {
      teardown();
    },
  };
  editors.set(root, api);
  if (!persistent) root.focus();
  return api;

  function teardown() {
    closeSlash();
    root.removeEventListener("beforeinput", onBeforeInput);
    root.removeEventListener("input", onInput);
    root.removeEventListener("keydown", onKeyDown);
    root.removeEventListener("keyup", onSelect);
    root.removeEventListener("mouseup", onSelect);
    root.removeEventListener("paste", onPaste);
    root.removeEventListener("focus", onFocus);
    root.removeEventListener("blur", onBlur);
    root.removeEventListener("click", onClick);
    root.removeAttribute("contenteditable");
    root.classList.remove("editing-md", "is-empty", "focused");
    editors.delete(root);
    hideBubble();
  }
}

function isEmpty(root) {
  return !root.textContent.replace(/​/g, "").trim() && !root.querySelector("img, hr, table, input, .callout");
}

export function editorFor(root) {
  return editors.get(root) || null;
}

// ----- Source editor (raw Markdown, BBCode or HTML), Obsidian's source mode -----

export function mountSourceEditor(bodyEl, card, ctx) {
  bodyEl.innerHTML = "";
  const ta = el("textarea", {
    class: "source-edit",
    spellcheck: false,
    value: card.md || "",
    on: { input: () => autoGrow(ta) },
  });
  const fmt = el(
    "select",
    { class: "source-format", title: "Source format" },
    ["markdown", "bbcode", "html"].map((f) => el("option", { value: f, text: f === "bbcode" ? "BBCode" : f === "html" ? "HTML" : "Markdown" }))
  );
  fmt.value = card.format || "markdown";
  const done = el("button", { type: "button", class: "inline-btn", text: "Done", on: { click: () => ctx.requestCommit() } });
  bodyEl.appendChild(el("div", { class: "source-wrap" }, [ta, el("div", { class: "source-bar" }, [el("span", { class: "source-tag", text: "Source" }), fmt, done])]));
  ta.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      e.preventDefault();
      ctx.requestCommit();
    }
  });
  requestAnimationFrame(() => autoGrow(ta));
  ta.focus();
  return {
    commit() {
      const changed = ta.value !== (card.md || "") || fmt.value !== (card.format || "markdown");
      card.md = ta.value;
      card.format = fmt.value;
      delete card.html;
      ctx.rerenderContent();
      if (changed) ctx.markDirty();
    },
    cancel() {
      ctx.rerenderContent();
    },
  };
}

function autoGrow(ta) {
  ta.style.height = "auto";
  ta.style.height = ta.scrollHeight + 2 + "px";
}

// ----- Code editor -----

export const LANGUAGES = ["gdscript", "csharp", "json", "javascript", "python", "bash", "glsl", "xml", "plaintext"];

export function mountCodeEditor(bodyEl, card, ctx) {
  bodyEl.innerHTML = "";
  const before = { code: card.code || "", lang: card.lang || "" };
  const ta = el("textarea", {
    class: "code-edit",
    spellcheck: false,
    value: card.code || "",
    on: {
      input: () => {
        card.code = ta.value;
        autoGrow(ta);
      },
      keydown: (e) => {
        if (e.key === "Escape") {
          e.preventDefault();
          ctx.requestCommit();
          return;
        }
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
  const langs = LANGUAGES.includes(card.lang || "") || !card.lang ? LANGUAGES : [card.lang].concat(LANGUAGES);
  const select = el(
    "select",
    { class: "code-lang", title: "Language", on: { change: () => (card.lang = select.value) } },
    langs.map((l) => el("option", { value: l, text: l }))
  );
  select.value = card.lang || "plaintext";
  bodyEl.appendChild(el("div", { class: "code-edit-wrap" }, [ta, select]));
  requestAnimationFrame(() => autoGrow(ta));
  ta.focus();

  return {
    commit() {
      card.code = ta.value;
      card.lang = select.value === "plaintext" && !before.lang ? "" : select.value;
      ctx.rerenderContent();
      if (card.code !== before.code || card.lang !== before.lang) ctx.markDirty();
    },
    cancel() {
      card.code = before.code;
      card.lang = before.lang;
      ctx.rerenderContent();
    },
  };
}

// ----- Table editor -----

export function mountTableEditor(bodyEl, card, ctx) {
  if (!Array.isArray(card.rows) || !card.rows.length) card.rows = [[""]];
  const rows = card.rows;
  const table = bodyEl.querySelector("table.card-table");
  if (table) {
    table.querySelectorAll("th, td").forEach((cell) => {
      const tr = cell.parentNode;
      const ri = Array.prototype.indexOf.call(tr.parentNode.children, tr);
      const ci = Array.prototype.indexOf.call(tr.children, cell);
      cell.setAttribute("contenteditable", "true");
      cell.addEventListener("input", () => {
        if (rows[ri]) rows[ri][ci] = cell.textContent;
      });
      cell.addEventListener("keydown", (e) => {
        if (e.key === "Escape") {
          e.preventDefault();
          ctx.requestCommit();
        } else if (e.key === "Enter" && !e.shiftKey) {
          e.preventDefault();
          const below = tr.nextElementSibling && tr.nextElementSibling.children[ci];
          if (below) below.focus();
        }
      });
    });
    const first = table.querySelector("th, td");
    if (first) first.focus();
  }

  const change = (fn) => () => {
    fn();
    ctx.markDirty();
    ctx.requestCommit();
    ctx.reopen && ctx.reopen();
  };
  const cols = () => (rows[0] || [""]).length;
  const bar = el("div", { class: "tbl-adders" }, [
    adder("plus", "Row", change(() => rows.push(new Array(cols()).fill("")))),
    adder("plus", "Column", change(() => rows.forEach((r) => r.push("")))),
    adder("minus", "Row", change(() => rows.length > 1 && rows.pop())),
    adder("minus", "Column", change(() => cols() > 1 && rows.forEach((r) => r.pop()))),
  ]);
  bodyEl.appendChild(bar);

  function adder(iconName, label, fn) {
    return el(
      "button",
      { type: "button", class: "inline-btn", on: { mousedown: (e) => e.preventDefault(), click: fn } },
      [icon(iconName, { size: 13 }), label]
    );
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

// ----- On-card controls for image / video / shape / draw -----
// Added in edit mode and revealed on hover or selection. They mutate the card and re-render
// its content live, so there is no separate commit step.

export function mountSelectionControls(cardEl, card, ctx) {
  let controls = null;
  if (card.type === "image") controls = imageControls(card, ctx);
  else if (card.type === "video") controls = videoControls(card, ctx);
  else if (card.type === "shape") controls = shapeControls(card, ctx);
  else if (card.type === "draw") controls = drawControls(card, ctx);
  if (!controls) return;
  controls.classList.add("inline-controls");
  controls.addEventListener("pointerdown", (e) => e.stopPropagation());
  controls.addEventListener("dblclick", (e) => e.stopPropagation());
  cardEl.appendChild(controls);
}

function textInput(value, placeholder, onChange) {
  const input = el("input", {
    type: "text",
    value,
    placeholder,
    spellcheck: false,
    on: { change: () => onChange(input.value), keydown: (e) => e.key === "Enter" && input.blur() },
  });
  return input;
}

function ctrlButton(label, onClick, iconName) {
  return el("button", { type: "button", class: "inline-btn", on: { click: onClick } }, [iconName ? icon(iconName, { size: 13 }) : null, label]);
}

function colorInput(value, fallback, onPick) {
  return el("input", {
    type: "color",
    class: "inline-color",
    value: toHex(value, fallback),
    title: "Colour",
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
    ctrlButton("Upload", () => file.click(), "upload"),
    file,
    textInput(card.src || "", "Image URL or assets/…", (v) => {
      card.src = v.trim();
      if (v.trim()) card.svg = "";
      ctx.rerenderContent();
      ctx.markDirty();
    }),
    textInput(card.alt || "", "Caption", (v) => {
      card.alt = v.trim();
      ctx.rerenderContent();
      ctx.markDirty();
    }),
  ]);
}

function videoControls(card, ctx) {
  return el("div", {}, [
    textInput(card.src || "", "mp4 URL, or a YouTube or Vimeo link", (v) => {
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

// Freehand drawing on a draw card body. getScale converts screen pixels to card units, which
// matters on a zoomed canvas.
export function attachDrawing(body, card, getScale, onDone) {
  const svg = body.querySelector("svg.card-draw");
  const path = svg ? svg.querySelector("path.draw-line") : null;
  if (!path) return;
  let drawing = false;
  const toLocal = (e) => {
    const rect = body.getBoundingClientRect();
    const s = getScale() || 1;
    return {
      x: Math.round(((e.clientX - rect.left) / s) * 10) / 10,
      y: Math.round(((e.clientY - rect.top) / s) * 10) / 10,
    };
  };
  body.addEventListener("pointerdown", (e) => {
    if (e.button !== 0 || e.target.closest(".inline-controls")) return;
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
      // Capture may already be gone.
    }
    onDone();
  };
  body.addEventListener("pointerup", end);
  body.addEventListener("pointercancel", end);
}

function toHex(value, fallback) {
  if (typeof value === "string" && /^#[0-9a-fA-F]{6}$/.test(value)) return value;
  return fallback;
}

