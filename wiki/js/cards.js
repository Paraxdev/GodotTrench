// Card model and per-type renderers.
// A Card is { id, type, x, y, w, h, z, ...typeData }.
// The libraries marked, DOMPurify, and highlight.js are read from the global scope,
// where the classic script tags in index.html place them. Every renderer degrades
// gracefully if a library did not load.

import { el } from "./dom.js";
import { icon } from "./icons.js";
import { renderMarkdownInto, renderHtmlInto } from "./markdown.js";

export const CARD_TYPES = ["text", "code", "image", "video", "table", "shape", "draw"];

export const TYPE_LABELS = {
  text: "Text",
  code: "Code",
  image: "Image",
  video: "Video",
  table: "Table",
  shape: "Shape",
  draw: "Draw",
};

const DEFAULT_SIZE = {
  text: { w: 280, h: 170 },
  code: { w: 360, h: 190 },
  image: { w: 280, h: 210 },
  video: { w: 380, h: 230 },
  table: { w: 320, h: 170 },
  shape: { w: 200, h: 170 },
  draw: { w: 280, h: 210 },
};

function makeId() {
  const rand = Math.random().toString(36).slice(2, 8);
  return "c_" + Date.now().toString(36) + "_" + rand;
}

// Highest z plus one, so a new card lands on top.
export function nextZ(cards) {
  if (!cards.length) return 0;
  return Math.max.apply(null, cards.map((c) => c.z || 0)) + 1;
}

// Reassign z as a clean 0..n-1 integer sequence preserving current stacking order.
export function normalizeZ(cards) {
  cards
    .slice()
    .sort((a, b) => (a.z || 0) - (b.z || 0))
    .forEach((card, index) => {
      card.z = index;
    });
  return cards;
}

export function bringToFront(cards, id) {
  const others = cards.filter((c) => c.id !== id).sort((a, b) => (a.z || 0) - (b.z || 0));
  others.forEach((c, i) => {
    c.z = i;
  });
  const target = cards.find((c) => c.id === id);
  if (target) target.z = others.length;
}

export function sendToBack(cards, id) {
  const others = cards.filter((c) => c.id !== id).sort((a, b) => (a.z || 0) - (b.z || 0));
  others.forEach((c, i) => {
    c.z = i + 1;
  });
  const target = cards.find((c) => c.id === id);
  if (target) target.z = 0;
}

// Build a new card of the given type positioned at { x, y }.
export function newCard(type, at, cards) {
  const size = DEFAULT_SIZE[type] || { w: 260, h: 170 };
  const base = {
    id: makeId(),
    type,
    x: Math.max(0, Math.round(at.x)),
    y: Math.max(0, Math.round(at.y)),
    w: size.w,
    h: size.h,
    z: nextZ(cards || []),
  };
  return Object.assign(base, typeDefaults(type));
}

function typeDefaults(type) {
  switch (type) {
    case "text":
      return { format: "markdown", md: "" };
    case "code":
      return { lang: "gdscript", code: "" };
    case "image":
      return { src: "", alt: "" };
    case "video":
      return { src: "", embed: false };
    case "table":
      return { rows: [["Column", "Column"], ["", ""]] };
    case "shape":
      return { shape: "rect", stroke: "#58b3e6", fill: "rgba(88,179,230,0.14)", strokeWidth: 2 };
    case "draw":
      return { path: "" };
    default:
      return {};
  }
}

// ----- Renderers. Each returns an HTMLElement for the card body. -----

export function renderContent(card) {
  switch (card.type) {
    case "text":
      return renderText(card);
    case "code":
      return renderCode(card);
    case "image":
      return renderImage(card);
    case "video":
      return renderVideo(card);
    case "table":
      return renderTable(card);
    case "shape":
      return renderShape(card);
    case "draw":
      return renderDraw(card);
    default:
      return placeholder("Unknown card type");
  }
}

function placeholder(text) {
  return el("div", { class: "card-placeholder", text });
}

// Text cards render Markdown, BBCode, or raw HTML. Whatever the format, the final HTML
// is always run through DOMPurify before it touches the DOM, so authored content, even raw
// HTML, cannot inject script or event handlers on this public site.
function renderText(card) {
  const div = el("div", { class: "md" });
  const source = card.md || "";
  const format = card.format || "markdown";

  let ok;
  if (typeof card.html === "string" && card.html.length) {
    // Cards edited by an older build stored rich HTML directly.
    ok = renderHtmlInto(div, card.html);
  } else if (format === "html") {
    ok = renderHtmlInto(div, source);
  } else if (format === "bbcode") {
    ok = renderHtmlInto(div, bbcodeToHtml(source));
  } else {
    ok = renderMarkdownInto(div, source);
  }
  if (!ok) {
    // No sanitizer available (offline without the vendored copy). Show the raw source safely
    // as plain text rather than risk injecting unsanitized markup.
    div.appendChild(el("pre", { text: source }));
  }
  return div;
}

function escapeHtml(str) {
  return String(str)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

// A compact BBCode to HTML converter. The result is sanitized by DOMPurify before use, so it
// only needs to build reasonable markup, not enforce safety itself. Code blocks are pulled out
// first so their contents are never treated as BBCode or line broken.
function bbcodeToHtml(src) {
  let s = escapeHtml(src);

  const codeBlocks = [];
  s = s.replace(/\[code\]([\s\S]*?)\[\/code\]/gi, (_m, body) => {
    codeBlocks.push(body);
    return "\u0000CODE" + (codeBlocks.length - 1) + "\u0000";
  });

  // Lists: turn [list] ... [*] item ... [/list] into <ul><li>...</li></ul>.
  s = s.replace(/\[list\]([\s\S]*?)\[\/list\]/gi, (_m, body) => {
    const items = body
      .split(/\[\*\]/)
      .map((part) => part.trim())
      .filter((part) => part.length);
    return "<ul>" + items.map((it) => "<li>" + it + "</li>").join("") + "</ul>";
  });

  const inline = [
    [/\[b\]([\s\S]*?)\[\/b\]/gi, "<strong>$1</strong>"],
    [/\[i\]([\s\S]*?)\[\/i\]/gi, "<em>$1</em>"],
    [/\[u\]([\s\S]*?)\[\/u\]/gi, "<u>$1</u>"],
    [/\[s\]([\s\S]*?)\[\/s\]/gi, "<s>$1</s>"],
    [/\[quote\]([\s\S]*?)\[\/quote\]/gi, "<blockquote>$1</blockquote>"],
    [/\[center\]([\s\S]*?)\[\/center\]/gi, '<div style="text-align:center">$1</div>'],
    [/\[color=([#\w(),.\s]+)\]([\s\S]*?)\[\/color\]/gi, '<span style="color:$1">$2</span>'],
    [/\[size=(\d{1,3})\]([\s\S]*?)\[\/size\]/gi, '<span style="font-size:$1px">$2</span>'],
    [/\[url=([^\]\s]+)\]([\s\S]*?)\[\/url\]/gi, '<a href="$1">$2</a>'],
    [/\[url\]([^\[]+)\[\/url\]/gi, '<a href="$1">$1</a>'],
    [/\[img\]([^\[]+)\[\/img\]/gi, '<img src="$1" alt="" />'],
  ];
  // A few passes so nested inline tags resolve.
  for (let pass = 0; pass < 3; pass += 1) {
    inline.forEach(([re, out]) => {
      s = s.replace(re, out);
    });
  }

  // Line breaks for the remaining text, then restore protected code blocks.
  s = s.replace(/\r?\n/g, "<br>");
  s = s.replace(/\u0000CODE(\d+)\u0000/g, (_m, i) => "<pre><code>" + codeBlocks[Number(i)] + "</code></pre>");
  return s;
}

function renderCode(card) {
  const pre = el("pre", { class: "code-block" });
  const code = el("code");
  const lang = card.lang || "";
  const source = card.code || "";
  const hljs = window.hljs;
  if (!source) {
    code.appendChild(el("span", { class: "code-empty", text: "Empty code block" }));
  } else if (hljs && lang && hljs.getLanguage && hljs.getLanguage(lang)) {
    code.className = "language-" + lang;
    code.textContent = source;
    try {
      hljs.highlightElement(code);
    } catch (err) {
      code.textContent = source;
    }
  } else {
    code.textContent = source;
  }
  pre.appendChild(code);
  const copy = el(
    "button",
    {
      type: "button",
      class: "code-copy",
      title: "Copy code",
      "aria-label": "Copy code",
      on: {
        pointerdown: (e) => e.stopPropagation(),
        click: (e) => {
          e.stopPropagation();
          copyText(source).then(() => {
            copy.classList.add("done");
            setTimeout(() => copy.classList.remove("done"), 1200);
          });
        },
      },
    },
    [icon("copy", { size: 14 }), el("span", { class: "code-copy-done" }, icon("check", { size: 14 }))]
  );
  return el("div", { class: "code-wrap" }, [
    pre,
    el("div", { class: "code-meta" }, [lang ? el("span", { class: "code-lang-tag", text: lang }) : null, source ? copy : null]),
  ]);
}

function copyText(text) {
  if (navigator.clipboard && navigator.clipboard.writeText) return navigator.clipboard.writeText(text).catch(() => {});
  return Promise.resolve();
}

function renderImage(card) {
  const wrap = el("div", { class: "card-media" });

  // Inline SVG markup takes precedence over a src. It is sanitized before it touches the DOM.
  const svg = (card.svg || "").trim();
  if (svg) {
    if (window.DOMPurify) {
      const clean = window.DOMPurify.sanitize(svg, { USE_PROFILES: { svg: true, svgFilters: true } });
      wrap.appendChild(el("div", { class: "card-svg", html: clean }));
    } else {
      wrap.appendChild(placeholder("SVG needs the sanitizer to render."));
    }
    return wrap;
  }

  const src = (card.src || "").trim();
  if (!src) {
    wrap.appendChild(placeholder("No image yet. Paste an image or SVG, set a src, or upload one."));
    return wrap;
  }
  if (card.alt) wrap.classList.add("has-caption");
  wrap.appendChild(
    el("img", {
      class: "card-img",
      src,
      alt: card.alt || "",
      loading: "lazy",
      on: {
        error: () => {
          wrap.innerHTML = "";
          wrap.appendChild(placeholder("Image failed to load: " + src));
        },
      },
    })
  );
  if (card.alt) wrap.appendChild(el("div", { class: "card-caption", text: card.alt }));
  return wrap;
}

// Detect YouTube and Vimeo URLs and build the correct embed src.
export function youtubeId(url) {
  const patterns = [
    /(?:youtube\.com\/watch\?[^#]*\bv=)([\w-]{6,})/i,
    /(?:youtu\.be\/)([\w-]{6,})/i,
    /(?:youtube\.com\/embed\/)([\w-]{6,})/i,
    /(?:youtube\.com\/shorts\/)([\w-]{6,})/i,
  ];
  for (const re of patterns) {
    const m = url.match(re);
    if (m) return m[1];
  }
  return null;
}

export function vimeoId(url) {
  const m = url.match(/vimeo\.com\/(?:video\/)?(\d{6,})/i);
  return m ? m[1] : null;
}

export function resolveVideo(card) {
  const src = (card.src || "").trim();
  const yt = youtubeId(src);
  const vm = vimeoId(src);
  if (card.embed || yt || vm) {
    if (yt) return { kind: "iframe", src: "https://www.youtube.com/embed/" + yt };
    if (vm) return { kind: "iframe", src: "https://player.vimeo.com/video/" + vm };
    return { kind: "iframe", src };
  }
  return { kind: "file", src };
}

function renderVideo(card) {
  const wrap = el("div", { class: "card-media" });
  const info = resolveVideo(card);
  if (!info.src) {
    wrap.appendChild(placeholder("No video yet. Set an mp4 src, or a YouTube or Vimeo link."));
    return wrap;
  }
  if (info.kind === "iframe") {
    wrap.appendChild(
      el("iframe", {
        class: "card-video",
        src: info.src,
        frameborder: "0",
        allow: "accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture",
        allowFullscreen: true,
      })
    );
  } else {
    wrap.appendChild(el("video", { class: "card-video", src: info.src, controls: true }));
  }
  return wrap;
}

function renderTable(card) {
  const rows = Array.isArray(card.rows) ? card.rows : [];
  const tbody = el("tbody");
  rows.forEach((row, rowIndex) => {
    const tr = el("tr");
    (row || []).forEach((cell) => {
      tr.appendChild(el(rowIndex === 0 ? "th" : "td", { text: cell == null ? "" : String(cell) }));
    });
    tbody.appendChild(tr);
  });
  return el("div", { class: "card-table-wrap" }, el("table", { class: "card-table" }, tbody));
}

function svgEl(name, attrs) {
  const node = document.createElementNS("http://www.w3.org/2000/svg", name);
  for (const key in attrs) {
    node.setAttribute(key, attrs[key]);
  }
  return node;
}

function renderShape(card) {
  const svg = svgEl("svg", {
    class: "card-shape",
    viewBox: "0 0 100 100",
    preserveAspectRatio: "none",
  });
  const stroke = card.stroke || "#58b3e6";
  const fill = card.fill || "none";
  const sw = card.strokeWidth == null ? 2 : card.strokeWidth;
  const kind = card.shape || "rect";

  if (kind === "rect") {
    svg.appendChild(
      svgEl("rect", { x: 5, y: 5, width: 90, height: 90, rx: 4, fill, stroke, "stroke-width": sw })
    );
  } else if (kind === "ellipse") {
    svg.appendChild(
      svgEl("ellipse", { cx: 50, cy: 50, rx: 45, ry: 45, fill, stroke, "stroke-width": sw })
    );
  } else if (kind === "line") {
    svg.appendChild(
      svgEl("line", { x1: 5, y1: 50, x2: 95, y2: 50, stroke, "stroke-width": sw, "stroke-linecap": "round" })
    );
  } else if (kind === "arrow") {
    const defs = svgEl("defs", {});
    const marker = svgEl("marker", {
      id: "arrow-" + card.id,
      viewBox: "0 0 10 10",
      refX: 8,
      refY: 5,
      markerWidth: 6,
      markerHeight: 6,
      orient: "auto-start-reverse",
    });
    marker.appendChild(svgEl("path", { d: "M 0 0 L 10 5 L 0 10 z", fill: stroke }));
    defs.appendChild(marker);
    svg.appendChild(defs);
    svg.appendChild(
      svgEl("line", {
        x1: 8,
        y1: 50,
        x2: 88,
        y2: 50,
        stroke,
        "stroke-width": sw,
        "stroke-linecap": "round",
        "marker-end": "url(#arrow-" + card.id + ")",
      })
    );
  }
  return el("div", { class: "card-media" }, svg);
}

function renderDraw(card) {
  // No viewBox, so SVG user units equal CSS pixels and the freehand path maps 1:1
  // to pointer coordinates captured over the card body.
  const svg = svgEl("svg", { class: "card-draw" });
  const path = svgEl("path", {
    class: "draw-line",
    d: card.path || "",
    fill: "none",
    stroke: card.stroke || "var(--draw-stroke)",
    "stroke-width": 2.5,
    "stroke-linecap": "round",
    "stroke-linejoin": "round",
  });
  svg.appendChild(path);
  const wrap = el("div", { class: "card-media card-draw-wrap" }, svg);
  if (!card.path) {
    wrap.appendChild(el("div", { class: "draw-hint", text: "Draw here in edit mode" }));
  }
  return wrap;
}
