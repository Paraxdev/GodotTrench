// Markdown in both directions. Pages are stored as Markdown so the repo diffs stay readable,
// the editor works on rendered HTML, and htmlToMarkdown turns the edited DOM back into source.
//
// Extensions on top of GitHub flavored Markdown:
//   [[slug]] or [[Page title|label]]   wikilinks between pages, Obsidian style
//   > [!NOTE] optional title           callouts (NOTE, TIP, WARNING, DANGER, EXAMPLE)

import { el } from "./dom.js";
import { icon } from "./icons.js";

export const CALLOUTS = {
  note: { label: "Note", icon: "info" },
  tip: { label: "Tip", icon: "lightbulb" },
  warning: { label: "Warning", icon: "triangle-alert" },
  danger: { label: "Danger", icon: "octagon-alert" },
  example: { label: "Example", icon: "flask" },
};
// Aliases GitHub and Obsidian accept, mapped onto the five styles above.
const CALLOUT_ALIASES = { info: "note", important: "note", hint: "tip", caution: "warning", error: "danger", bug: "danger" };

function calloutKind(name) {
  const k = String(name || "").toLowerCase();
  if (CALLOUTS[k]) return k;
  return CALLOUT_ALIASES[k] || "note";
}

// ----- Wikilink resolution, supplied by the app so rendering knows which pages exist -----

let resolvePage = () => null;

export function setPageResolver(fn) {
  resolvePage = fn;
}

export function slugify(text) {
  return (
    String(text)
      .toLowerCase()
      .replace(/[^\w]+/g, "-")
      .replace(/^-+|-+$/g, "") || "page"
  );
}

// ----- Rendering -----

let configured = false;

function configure() {
  if (configured || !window.marked) return;
  configured = true;
  window.marked.use({
    gfm: true,
    breaks: true,
    extensions: [
      {
        name: "wikilink",
        level: "inline",
        start(src) {
          const i = src.indexOf("[[");
          return i < 0 ? undefined : i;
        },
        tokenizer(src) {
          const m = /^\[\[([^\]|\n]+)(?:\|([^\]\n]+))?\]\]/.exec(src);
          if (!m) return undefined;
          return { type: "wikilink", raw: m[0], target: m[1].trim(), label: (m[2] || "").trim() };
        },
        renderer(tok) {
          const page = resolvePage(tok.target);
          const label = tok.label || (page ? page.title : tok.target);
          const href = "#" + (page ? page.slug : slugify(tok.target));
          const cls = "wikilink" + (page ? "" : " missing");
          return (
            '<a class="' + cls + '" href="' + escapeAttr(href) + '" data-target="' + escapeAttr(tok.target) + '">' +
            escapeHtml(label) +
            "</a>"
          );
        },
      },
    ],
  });
}

function escapeHtml(s) {
  return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function escapeAttr(s) {
  return escapeHtml(s).replace(/"/g, "&quot;");
}

const SANITIZE = { ADD_ATTR: ["target", "contenteditable"] };

// Render Markdown into a sanitized element. Returns null when a library is missing so the
// caller can fall back to plain text.
export function renderMarkdownInto(div, source) {
  configure();
  if (!window.marked || !window.DOMPurify) return false;
  div.innerHTML = window.DOMPurify.sanitize(window.marked.parse(source || ""), SANITIZE);
  enhance(div);
  return true;
}

export function renderHtmlInto(div, html) {
  if (!window.DOMPurify) return false;
  div.innerHTML = window.DOMPurify.sanitize(html || "", SANITIZE);
  enhance(div);
  return true;
}

// Post-processing shared by every rendered text: callouts, highlighted fenced code, link
// targets and heading ids for the outline.
export function enhance(root) {
  root.querySelectorAll("blockquote").forEach(upgradeCallout);
  root.querySelectorAll("pre > code").forEach((code) => {
    const hljs = window.hljs;
    const m = /language-([\w-]+)/.exec(code.className);
    if (!hljs) return;
    try {
      if (m && hljs.getLanguage(m[1])) hljs.highlightElement(code);
    } catch (err) {
      // Leave the code unhighlighted.
    }
  });
  root.querySelectorAll("a[href]").forEach((a) => {
    const href = a.getAttribute("href") || "";
    if (href.startsWith("#")) {
      a.removeAttribute("target");
    } else {
      a.target = "_blank";
      a.rel = "noopener noreferrer";
    }
  });
  root.querySelectorAll("li > input[type=checkbox]").forEach((box) => {
    box.parentElement.classList.add("task");
  });
  const used = new Set();
  root.querySelectorAll("h1, h2, h3").forEach((h) => {
    let id = "h-" + slugify(h.textContent);
    while (used.has(id)) id += "-";
    used.add(id);
    h.dataset.anchor = id;
  });
}

// A blockquote whose first line is [!TYPE] becomes a callout box.
function upgradeCallout(bq) {
  const first = bq.firstElementChild;
  if (!first || first.tagName !== "P") return;
  const head = first.firstChild;
  if (!head || head.nodeType !== 3) return;
  const m = /^\s*\[!(\w+)\][+-]? ?/.exec(head.data);
  if (!m) return;
  const kind = calloutKind(m[1]);

  // The title is everything up to the first line break of the first paragraph, formatting kept.
  head.data = head.data.slice(m[0].length);
  const title = document.createDocumentFragment();
  let node = head;
  while (node && node.nodeName !== "BR") {
    const next = node.nextSibling;
    title.appendChild(node);
    node = next;
  }
  if (node) node.remove();
  if (!first.textContent.trim() && !first.querySelector("img")) first.remove();

  const box = buildCallout(kind, title.textContent.trim() ? title : "");
  const body = box.querySelector(".callout-body");
  while (bq.firstChild) body.appendChild(bq.firstChild);
  if (!body.firstChild) body.appendChild(el("p", {}, el("br")));
  bq.replaceWith(box);
}

// title is a string or a fragment of inline nodes; empty means the kind's default label.
export function buildCallout(kind, title) {
  const info = CALLOUTS[kind] || CALLOUTS.note;
  const label = el("span", { class: "callout-label" });
  if (title && typeof title !== "string") {
    label.appendChild(title);
    while (label.firstChild && label.firstChild.nodeType === 3 && !label.firstChild.data.trim()) label.firstChild.remove();
  } else {
    label.textContent = title || info.label;
  }
  return el("div", { class: "callout", dataset: { callout: kind } }, [
    el("div", { class: "callout-title" }, [el("span", { class: "callout-icon", contenteditable: "false" }, icon(info.icon, { size: 16 })), label]),
    el("div", { class: "callout-body" }),
  ]);
}

// ----- Plain text, for search and snippets -----

export function markdownToText(md) {
  return String(md || "")
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/\[\[([^\]|]+)\|([^\]]+)\]\]/g, "$2")
    .replace(/\[\[([^\]]+)\]\]/g, "$1")
    .replace(/!\[[^\]]*\]\([^)]*\)/g, " ")
    .replace(/\[([^\]]+)\]\([^)]*\)/g, "$1")
    .replace(/^>\s*\[![^\]]+\]/gm, "")
    .replace(/[#>*`~|]/g, " ")
    .replace(/(^|\s)_+|_+(?=\s|$)/g, "$1")
    .replace(/\s+/g, " ")
    .trim();
}

// ----- HTML back to Markdown -----

const BLOCK_TAGS = new Set([
  "P", "DIV", "H1", "H2", "H3", "H4", "H5", "H6", "UL", "OL", "LI", "BLOCKQUOTE", "PRE", "HR", "TABLE",
  "SECTION", "ARTICLE", "HEADER", "FOOTER", "FIGURE",
]);

function isBlock(node) {
  return node.nodeType === 1 && BLOCK_TAGS.has(node.tagName);
}

export function htmlToMarkdown(root) {
  return tidy(blocks(root).join("\n\n"));
}

function tidy(s) {
  return s
    .replace(/[ \t]+$/gm, "")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

// Serialize a container's children as a list of block strings. Runs of inline nodes form an
// implicit paragraph, which is what contenteditable produces when text sits directly in a div.
function blocks(parent) {
  const out = [];
  let run = [];
  const flush = () => {
    if (!run.length) return;
    const text = paragraph(run);
    if (text.trim()) out.push(text);
    run = [];
  };
  parent.childNodes.forEach((node) => {
    if (isBlock(node)) {
      flush();
      const b = block(node);
      if (b != null && b.trim() !== "") out.push(b);
    } else {
      run.push(node);
    }
  });
  flush();
  return out;
}

function paragraph(nodes) {
  const text = inline(nodes).replace(/^\n+|\n+$/g, "");
  return text
    .split("\n")
    .map((line) => escapeLineStart(line.replace(/^ +/, "")))
    .join("\n");
}

function block(node) {
  const tag = node.tagName;
  if (/^H[1-6]$/.test(tag)) {
    // Contenteditable can nest the next line inside a heading; only the inline start is the heading.
    const kids = Array.from(node.childNodes);
    const cut = kids.findIndex(isBlock);
    const text = inline(cut < 0 ? kids : kids.slice(0, cut)).replace(/\n+/g, " ").trim();
    const out = text ? "#".repeat(Number(tag[1])) + " " + text : "";
    if (cut < 0) return out;
    const rest = document.createElement("div");
    kids.slice(cut).forEach((k) => rest.appendChild(k.cloneNode(true)));
    return [out].concat(blocks(rest)).filter(Boolean).join("\n\n");
  }
  if (tag === "P") {
    const kids = Array.from(node.childNodes);
    return kids.some(isBlock) ? blocks(node).join("\n\n") : paragraph(kids);
  }
  if (tag === "HR") return "---";
  if (tag === "PRE") return fencedCode(node);
  if (tag === "UL" || tag === "OL") return list(node);
  if (tag === "TABLE") return table(node);
  if (tag === "BLOCKQUOTE") return quote(blocks(node).join("\n\n"));
  if (tag === "DIV" && node.classList.contains("callout")) return callout(node);
  if (tag === "LI") return list({ tagName: "UL", children: [node], getAttribute: () => null });
  return blocks(node).join("\n\n");
}

function quote(text) {
  return text
    .split("\n")
    .map((line) => (line ? "> " + line : ">"))
    .join("\n");
}

function callout(node) {
  const kind = calloutKind(node.dataset.callout);
  const labelEl = node.querySelector(".callout-label");
  const label = labelEl ? inline(labelEl.childNodes).replace(/\n+/g, " ").trim() : "";
  const head = "[!" + kind.toUpperCase() + "]" + (label && label !== CALLOUTS[kind].label ? " " + label : "");
  const body = node.querySelector(".callout-body");
  const inner = body ? blocks(body).join("\n\n") : "";
  return quote(inner ? head + "\n" + inner : head);
}

function fencedCode(pre) {
  const code = pre.querySelector("code");
  const text = (code || pre).textContent.replace(/\n$/, "");
  const m = code && /language-([\w-]+)/.exec(code.className);
  let fence = "```";
  while (text.includes(fence)) fence += "`";
  return fence + (m ? m[1] : "") + "\n" + text + "\n" + fence;
}

function list(node, depth) {
  const ordered = node.tagName === "OL";
  let n = Number((node.getAttribute && node.getAttribute("start")) || 1) || 1;
  const items = [];
  Array.from(node.children).forEach((li) => {
    if (li.tagName !== "LI") {
      // Chrome can leave a nested list directly inside another list after indenting.
      if (li.tagName === "UL" || li.tagName === "OL") {
        const nested = list(li, (depth || 0) + 1);
        if (nested && items.length) items[items.length - 1] += "\n" + indent(nested, ordered ? 3 : 2);
        else if (nested) items.push(nested);
      }
      return;
    }
    const marker = ordered ? n + ". " : "- ";
    n += 1;
    let prefix = "";
    const box = li.querySelector(":scope > input[type=checkbox]");
    if (box) prefix = box.checked ? "[x] " : "[ ] ";
    const parts = blocks(li).filter((p) => p.trim());
    const first = parts.shift() || "";
    const rest = parts.map((p) => indent(p, marker.length)).join("\n");
    items.push(marker + prefix + indent(first, marker.length).trimStart() + (rest ? "\n" + rest : ""));
  });
  return items.join("\n");
}

function indent(text, n) {
  const pad = " ".repeat(n);
  return text
    .split("\n")
    .map((line) => (line ? pad + line : line))
    .join("\n");
}

function table(node) {
  const rows = Array.from(node.querySelectorAll("tr")).map((tr) =>
    Array.from(tr.children).map((cell) =>
      inline(cell.childNodes).replace(/\|/g, "\\|").replace(/\n/g, "<br>").trim()
    )
  );
  if (!rows.length) return "";
  const cols = Math.max(...rows.map((r) => r.length));
  const pad = (r) => r.concat(new Array(cols - r.length).fill(""));
  const line = (r) => "| " + pad(r).join(" | ") + " |";
  const out = [line(rows[0]), "| " + new Array(cols).fill("---").join(" | ") + " |"];
  rows.slice(1).forEach((r) => out.push(line(r)));
  return out.join("\n");
}

// ----- Inline serialization -----

function inline(nodes) {
  let out = "";
  Array.from(nodes).forEach((node) => {
    out += inlineNode(node);
  });
  return out;
}

function inlineNode(node) {
  // The editor puts a zero width space after inline formatting so the caret can leave it.
  if (node.nodeType === 3) return escapeText(node.data.replace(/\u200b/g, "").replace(/\u00a0/g, " ").replace(/[ \t\r\n]+/g, " "));
  if (node.nodeType !== 1) return "";
  const tag = node.tagName;
  if (node.getAttribute("contenteditable") === "false" && !node.classList.contains("wikilink")) return "";
  switch (tag) {
    case "BR":
      return "\n";
    case "STRONG":
    case "B":
      return wrap(node, "**");
    case "EM":
    case "I":
      return wrap(node, "*");
    case "S":
    case "DEL":
    case "STRIKE":
      return wrap(node, "~~");
    case "U":
      return "<u>" + inline(node.childNodes) + "</u>";
    case "CODE":
      return inlineCode(node.textContent);
    case "A":
      return link(node);
    case "IMG":
      return "![" + escapeText(node.getAttribute("alt") || "") + "](" + urlPart(node.getAttribute("src") || "") + ")";
    case "INPUT":
      return "";
    case "SPAN":
    case "FONT":
      return styledSpan(node);
    default:
      return inline(node.childNodes);
  }
}

function styledSpan(node) {
  const st = node.style || {};
  const bold = st.fontWeight === "bold" || Number(st.fontWeight) >= 600;
  const italic = st.fontStyle === "italic";
  let text = inline(node.childNodes);
  if (bold) text = wrapText(text, "**");
  if (italic) text = wrapText(text, "*");
  return text;
}

function wrap(node, marker) {
  return wrapText(inline(node.childNodes), marker);
}

// Markers must hug the text, so surrounding spaces move outside them.
function wrapText(text, marker) {
  const m = /^(\s*)([\s\S]*?)(\s*)$/.exec(text);
  if (!m[2]) return text;
  return m[1] + marker + m[2] + marker + m[3];
}

function inlineCode(text) {
  const t = text.replace(/\u00a0/g, " ").replace(/\n/g, " ");
  if (!t) return "";
  let fence = "`";
  while (t.includes(fence)) fence += "`";
  const pad = t.startsWith("`") || t.endsWith("`") ? " " : "";
  return fence + pad + t + pad + fence;
}

function link(a) {
  if (a.classList.contains("wikilink")) {
    const target = a.dataset.target || (a.getAttribute("href") || "").replace(/^#/, "");
    const label = a.textContent.trim();
    const page = resolvePage(target);
    const plain = !label || label === target || (page && label === page.title);
    return "[[" + target + (plain ? "" : "|" + label) + "]]";
  }
  const href = a.getAttribute("href") || "";
  const text = inline(a.childNodes);
  if (!href) return text;
  if (text === href && /^https?:\/\//.test(href)) return "<" + href + ">";
  return "[" + text + "](" + urlPart(href) + ")";
}

function urlPart(url) {
  return /[\s()<>]/.test(url) ? "<" + url.replace(/>/g, "%3E") + ">" : url;
}

function escapeText(s) {
  let out = s.replace(/[\\*`[\]<]/g, "\\$&");
  // Underscores inside words (logic_relay) are literal in GFM, only boundary ones need escaping.
  out = out.replace(/_/g, (m, i, str) => {
    const inWord = /[A-Za-z0-9]/.test(str[i - 1] || "") && /[A-Za-z0-9]/.test(str[i + 1] || "");
    return inWord ? "_" : "\\_";
  });
  if ((out.match(/~/g) || []).length > 1) out = out.replace(/~/g, "\\~");
  return out.replace(/&(?=#?\w+;)/g, "&amp;");
}

// Text that would otherwise start a heading, quote, list, rule, fence or table.
function escapeLineStart(line) {
  return line.replace(/^(#{1,6}(?=\s|$)|>|[-+](?=\s|$)|={3,}|-{3,}|\|)/, "\\$1").replace(/^(\d+)([.)])(?=\s|$)/, "$1\\$2");
}
