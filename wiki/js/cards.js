// Card model and per-type renderers.
// A Card is { id, type, x, y, w, h, z, ...typeData }.
// The libraries marked, DOMPurify, and highlight.js are read from the global scope,
// where the classic script tags in index.html place them. Every renderer degrades
// gracefully if a library did not load.

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
      return { md: "## New note\n\nWrite **markdown** here. Links like [Godot](https://godotengine.org) work in view mode." };
    case "code":
      return { lang: "gdscript", code: 'func _ready():\n\tprint("hello from GodotTrench")' };
    case "image":
      return { src: "", alt: "" };
    case "video":
      return { src: "", embed: false };
    case "table":
      return { rows: [["Column A", "Column B"], ["a1", "b1"], ["a2", "b2"]] };
    case "shape":
      return { shape: "rect", stroke: "#7aa2f7", fill: "rgba(122,162,247,0.16)", strokeWidth: 2 };
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
  const div = document.createElement("div");
  div.className = "card-placeholder";
  div.textContent = text;
  return div;
}

function renderText(card) {
  const div = document.createElement("div");
  div.className = "md";
  const md = card.md || "";
  if (window.marked && window.DOMPurify) {
    const raw = window.marked.parse(md);
    const clean = window.DOMPurify.sanitize(raw, { ADD_ATTR: ["target"] });
    div.innerHTML = clean;
    div.querySelectorAll("a[href]").forEach((a) => {
      a.target = "_blank";
      a.rel = "noopener noreferrer";
    });
  } else {
    const pre = document.createElement("pre");
    pre.textContent = md;
    div.appendChild(pre);
  }
  return div;
}

function renderCode(card) {
  const pre = document.createElement("pre");
  pre.className = "code-block";
  const code = document.createElement("code");
  const lang = card.lang || "";
  const source = card.code || "";
  const hljs = window.hljs;
  if (hljs && lang && hljs.getLanguage && hljs.getLanguage(lang)) {
    code.className = "language-" + lang;
    code.textContent = source;
    try {
      hljs.highlightElement(code);
    } catch (err) {
      code.textContent = source;
    }
  } else if (hljs && hljs.highlightAuto) {
    try {
      const result = hljs.highlightAuto(source);
      code.innerHTML = result.value;
      code.classList.add("hljs");
    } catch (err) {
      code.textContent = source;
    }
  } else {
    code.textContent = source;
  }
  pre.appendChild(code);
  return pre;
}

function renderImage(card) {
  const wrap = document.createElement("div");
  wrap.className = "card-media";
  const src = (card.src || "").trim();
  if (!src) {
    wrap.appendChild(placeholder("No image yet. Select the card and set a src or upload one."));
    return wrap;
  }
  const img = document.createElement("img");
  img.className = "card-img";
  img.src = src;
  img.alt = card.alt || "";
  img.loading = "lazy";
  img.addEventListener("error", () => {
    wrap.innerHTML = "";
    wrap.appendChild(placeholder("Image failed to load: " + src));
  });
  wrap.appendChild(img);
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
  const wrap = document.createElement("div");
  wrap.className = "card-media";
  const info = resolveVideo(card);
  if (!info.src) {
    wrap.appendChild(placeholder("No video yet. Set an mp4 src, or a YouTube or Vimeo link."));
    return wrap;
  }
  if (info.kind === "iframe") {
    const iframe = document.createElement("iframe");
    iframe.className = "card-video";
    iframe.src = info.src;
    iframe.setAttribute("frameborder", "0");
    iframe.setAttribute(
      "allow",
      "accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
    );
    iframe.allowFullscreen = true;
    wrap.appendChild(iframe);
  } else {
    const video = document.createElement("video");
    video.className = "card-video";
    video.src = info.src;
    video.controls = true;
    wrap.appendChild(video);
  }
  return wrap;
}

function renderTable(card) {
  const rows = Array.isArray(card.rows) ? card.rows : [];
  const table = document.createElement("table");
  table.className = "card-table";
  const tbody = document.createElement("tbody");
  rows.forEach((row, rowIndex) => {
    const tr = document.createElement("tr");
    (row || []).forEach((cell) => {
      const el = document.createElement(rowIndex === 0 ? "th" : "td");
      el.textContent = cell == null ? "" : String(cell);
      tr.appendChild(el);
    });
    tbody.appendChild(tr);
  });
  table.appendChild(tbody);
  const wrap = document.createElement("div");
  wrap.className = "card-table-wrap";
  wrap.appendChild(table);
  return wrap;
}

function svgEl(name, attrs) {
  const el = document.createElementNS("http://www.w3.org/2000/svg", name);
  for (const key in attrs) {
    el.setAttribute(key, attrs[key]);
  }
  return el;
}

function renderShape(card) {
  const svg = svgEl("svg", {
    class: "card-shape",
    viewBox: "0 0 100 100",
    preserveAspectRatio: "none",
  });
  const stroke = card.stroke || "#7aa2f7";
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
  const wrap = document.createElement("div");
  wrap.className = "card-media";
  wrap.appendChild(svg);
  return wrap;
}

function renderDraw(card) {
  // No viewBox, so SVG user units equal CSS pixels and the freehand path maps 1:1
  // to pointer coordinates captured over the card body.
  const svg = svgEl("svg", { class: "card-draw" });
  const path = svgEl("path", {
    class: "draw-line",
    d: card.path || "",
    fill: "none",
    stroke: "var(--draw-stroke)",
    "stroke-width": 2.5,
    "stroke-linecap": "round",
    "stroke-linejoin": "round",
  });
  svg.appendChild(path);
  const wrap = document.createElement("div");
  wrap.className = "card-media card-draw-wrap";
  wrap.appendChild(svg);
  if (!card.path) {
    const hint = document.createElement("div");
    hint.className = "draw-hint";
    hint.textContent = "Draw here in edit mode";
    wrap.appendChild(hint);
  }
  return wrap;
}
