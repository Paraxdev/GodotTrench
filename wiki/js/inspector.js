// The inspector panel. Shown when a card is selected in edit mode, it edits the selected
// card's content live. Each control mutates the card object and calls ctx.update(),
// which re-renders the card and autosaves a draft.

import { TYPE_LABELS } from "./cards.js";
import { icon } from "./icons.js";

// ctx: { card, update(), bringToFront(), sendToBack(), remove(), uploadAsset(file) }
export function renderInspector(container, ctx) {
  container.innerHTML = "";
  const card = ctx.card;
  if (!card) {
    const empty = document.createElement("div");
    empty.className = "inspector-empty";
    empty.textContent = "Select a card to edit it.";
    container.appendChild(empty);
    return;
  }

  const head = document.createElement("div");
  head.className = "inspector-head";
  const title = document.createElement("span");
  title.className = "inspector-title";
  title.textContent = TYPE_LABELS[card.type] || card.type;
  head.appendChild(title);

  const actions = document.createElement("div");
  actions.className = "inspector-actions";
  actions.appendChild(iconButton("To front", () => ctx.bringToFront(), "bring-to-front"));
  actions.appendChild(iconButton("To back", () => ctx.sendToBack(), "send-to-back"));
  const del = iconButton("Delete", () => ctx.remove(), "x");
  del.classList.add("danger");
  actions.appendChild(del);
  head.appendChild(actions);
  container.appendChild(head);

  const body = document.createElement("div");
  body.className = "inspector-body";
  container.appendChild(body);

  switch (card.type) {
    case "text":
      textInspector(body, ctx);
      break;
    case "code":
      codeInspector(body, ctx);
      break;
    case "image":
      imageInspector(body, ctx);
      break;
    case "video":
      videoInspector(body, ctx);
      break;
    case "table":
      tableInspector(body, ctx);
      break;
    case "shape":
      shapeInspector(body, ctx);
      break;
    case "draw":
      drawInspector(body, ctx);
      break;
    default:
      break;
  }

  geometryInspector(body, ctx);
}

// ----- Shared control builders -----

function field(labelText, control) {
  const wrap = document.createElement("label");
  wrap.className = "field";
  const span = document.createElement("span");
  span.className = "field-label";
  span.textContent = labelText;
  wrap.appendChild(span);
  wrap.appendChild(control);
  return wrap;
}

function textInput(value, onInput, attrs) {
  const input = document.createElement("input");
  input.type = "text";
  input.value = value || "";
  if (attrs) Object.keys(attrs).forEach((k) => input.setAttribute(k, attrs[k]));
  input.addEventListener("input", () => onInput(input.value));
  return input;
}

function iconButton(label, onClick, iconName) {
  const btn = document.createElement("button");
  btn.type = "button";
  btn.className = "btn btn-small";
  if (iconName) {
    btn.appendChild(icon(iconName));
    const span = document.createElement("span");
    span.className = "btn-label";
    span.textContent = label;
    btn.appendChild(span);
    btn.setAttribute("aria-label", label);
  } else {
    btn.textContent = label;
  }
  btn.addEventListener("click", onClick);
  return btn;
}

// An upload button whose label can change (for example to "Uploading...") without losing
// the icon.
function uploadButton(label, onClick) {
  const btn = document.createElement("button");
  btn.type = "button";
  btn.className = "btn btn-small";
  btn.appendChild(icon("upload"));
  const span = document.createElement("span");
  span.className = "btn-label";
  span.textContent = label;
  btn.appendChild(span);
  btn.setAttribute("aria-label", label);
  btn.addEventListener("click", onClick);
  btn.setLabel = (text) => {
    span.textContent = text;
  };
  return btn;
}

// ----- Per type inspectors -----

const TEXT_FORMATS = [
  ["markdown", "Markdown"],
  ["bbcode", "BBCode"],
  ["html", "HTML"],
];

function textInspector(body, ctx) {
  const ta = document.createElement("textarea");
  ta.className = "inspector-textarea";
  ta.rows = 10;
  ta.value = ctx.card.md || "";
  ta.spellcheck = false;

  const label = field("Markdown", ta);
  const labelSpan = label.querySelector(".field-label");

  const select = document.createElement("select");
  TEXT_FORMATS.forEach(([value, text]) => {
    const o = document.createElement("option");
    o.value = value;
    o.textContent = text;
    if ((ctx.card.format || "markdown") === value) o.selected = true;
    select.appendChild(o);
  });
  const applyLabel = () => {
    const cur = TEXT_FORMATS.find(([v]) => v === (ctx.card.format || "markdown"));
    if (labelSpan) labelSpan.textContent = cur ? cur[1] : "Text";
    ta.classList.toggle("mono", (ctx.card.format || "markdown") !== "markdown");
  };
  select.addEventListener("change", () => {
    ctx.card.format = select.value;
    applyLabel();
    ctx.update();
    ctx.rerender();
  });
  body.appendChild(field("Format", select));

  ta.addEventListener("input", () => {
    ctx.card.md = ta.value;
    ctx.update();
  });
  applyLabel();
  body.appendChild(label);
}

function codeInspector(body, ctx) {
  body.appendChild(
    field(
      "Language",
      textInput(ctx.card.lang, (v) => {
        ctx.card.lang = v.trim();
        ctx.update();
      }, { placeholder: "gdscript, js, python, ..." })
    )
  );
  const ta = document.createElement("textarea");
  ta.className = "inspector-textarea mono";
  ta.rows = 10;
  ta.value = ctx.card.code || "";
  ta.spellcheck = false;
  ta.addEventListener("input", () => {
    ctx.card.code = ta.value;
    ctx.update();
  });
  body.appendChild(field("Code", ta));
}

function imageInspector(body, ctx) {
  body.appendChild(
    field(
      "Source",
      textInput(ctx.card.src, (v) => {
        ctx.card.src = v.trim();
        ctx.update();
      }, { placeholder: "assets/name.png or https://..." })
    )
  );
  body.appendChild(
    field(
      "Alt text",
      textInput(ctx.card.alt, (v) => {
        ctx.card.alt = v;
        ctx.update();
      }, { placeholder: "describe the image" })
    )
  );

  const fileInput = document.createElement("input");
  fileInput.type = "file";
  fileInput.accept = "image/*";
  fileInput.style.display = "none";
  const uploadBtn = uploadButton("Upload image", () => fileInput.click());
  fileInput.addEventListener("change", async () => {
    const file = fileInput.files && fileInput.files[0];
    if (!file) return;
    uploadBtn.disabled = true;
    uploadBtn.setLabel("Uploading...");
    try {
      const relPath = await ctx.uploadAsset(file);
      if (relPath) {
        ctx.card.src = relPath;
        ctx.update();
        ctx.rerender();
      }
    } finally {
      uploadBtn.disabled = false;
      uploadBtn.setLabel("Upload image");
      fileInput.value = "";
    }
  });
  const row = document.createElement("div");
  row.className = "field";
  row.appendChild(uploadBtn);
  row.appendChild(fileInput);
  body.appendChild(row);
}

function videoInspector(body, ctx) {
  const select = document.createElement("select");
  ["file", "embed"].forEach((opt) => {
    const o = document.createElement("option");
    o.value = opt;
    o.textContent = opt === "file" ? "Video file (mp4)" : "Embed (YouTube, Vimeo, iframe)";
    if ((opt === "embed") === !!ctx.card.embed) o.selected = true;
    select.appendChild(o);
  });
  select.addEventListener("change", () => {
    ctx.card.embed = select.value === "embed";
    ctx.update();
    ctx.rerender();
  });
  body.appendChild(field("Kind", select));

  body.appendChild(
    field(
      "Source",
      textInput(ctx.card.src, (v) => {
        ctx.card.src = v.trim();
        ctx.update();
        ctx.rerender();
      }, { placeholder: "assets/clip.mp4 or a YouTube/Vimeo link" })
    )
  );

  const fileInput = document.createElement("input");
  fileInput.type = "file";
  fileInput.accept = "video/*";
  fileInput.style.display = "none";
  const uploadBtn = uploadButton("Upload video", () => fileInput.click());
  fileInput.addEventListener("change", async () => {
    const file = fileInput.files && fileInput.files[0];
    if (!file) return;
    uploadBtn.disabled = true;
    uploadBtn.setLabel("Uploading...");
    try {
      const relPath = await ctx.uploadAsset(file);
      if (relPath) {
        ctx.card.src = relPath;
        ctx.card.embed = false;
        ctx.update();
        ctx.rerender();
      }
    } finally {
      uploadBtn.disabled = false;
      uploadBtn.setLabel("Upload video");
      fileInput.value = "";
    }
  });
  const row = document.createElement("div");
  row.className = "field";
  row.appendChild(uploadBtn);
  row.appendChild(fileInput);
  body.appendChild(row);
}

function tableInspector(body, ctx) {
  const rebuild = () => {
    grid.innerHTML = "";
    const rows = ctx.card.rows || [];
    rows.forEach((row, ri) => {
      const tr = document.createElement("div");
      tr.className = "cell-row";
      (row || []).forEach((cell, ci) => {
        const input = document.createElement("input");
        input.type = "text";
        input.className = "cell-input";
        input.value = cell == null ? "" : String(cell);
        input.addEventListener("input", () => {
          ctx.card.rows[ri][ci] = input.value;
          ctx.update();
        });
        tr.appendChild(input);
      });
      const rmRow = document.createElement("button");
      rmRow.type = "button";
      rmRow.className = "btn btn-tiny";
      rmRow.textContent = "x";
      rmRow.title = "Remove row";
      rmRow.addEventListener("click", () => {
        ctx.card.rows.splice(ri, 1);
        if (!ctx.card.rows.length) ctx.card.rows.push([""]);
        ctx.update();
        ctx.rerender();
        rebuild();
      });
      tr.appendChild(rmRow);
      grid.appendChild(tr);
    });
  };

  const grid = document.createElement("div");
  grid.className = "cell-grid";
  body.appendChild(field("Cells (first row is the header)", grid));

  const controls = document.createElement("div");
  controls.className = "field row";
  controls.appendChild(
    iconButton("Add row", () => {
      const cols = (ctx.card.rows[0] || [""]).length;
      ctx.card.rows.push(new Array(cols).fill(""));
      ctx.update();
      ctx.rerender();
      rebuild();
    })
  );
  controls.appendChild(
    iconButton("Add column", () => {
      ctx.card.rows.forEach((row) => row.push(""));
      ctx.update();
      ctx.rerender();
      rebuild();
    })
  );
  controls.appendChild(
    iconButton("Remove column", () => {
      if ((ctx.card.rows[0] || []).length <= 1) return;
      ctx.card.rows.forEach((row) => row.pop());
      ctx.update();
      ctx.rerender();
      rebuild();
    })
  );
  body.appendChild(controls);
  rebuild();
}

function shapeInspector(body, ctx) {
  const select = document.createElement("select");
  ["rect", "ellipse", "line", "arrow"].forEach((opt) => {
    const o = document.createElement("option");
    o.value = opt;
    o.textContent = opt;
    if (opt === (ctx.card.shape || "rect")) o.selected = true;
    select.appendChild(o);
  });
  select.addEventListener("change", () => {
    ctx.card.shape = select.value;
    ctx.update();
    ctx.rerender();
  });
  body.appendChild(field("Shape", select));

  const stroke = document.createElement("input");
  stroke.type = "color";
  stroke.value = toHex(ctx.card.stroke, "#7aa2f7");
  stroke.addEventListener("input", () => {
    ctx.card.stroke = stroke.value;
    ctx.update();
  });
  body.appendChild(field("Stroke", stroke));

  const fill = document.createElement("input");
  fill.type = "text";
  fill.className = "cell-input";
  fill.value = ctx.card.fill || "none";
  fill.setAttribute("placeholder", "none or rgba(...)/#hex");
  fill.addEventListener("input", () => {
    ctx.card.fill = fill.value.trim() || "none";
    ctx.update();
  });
  body.appendChild(field("Fill", fill));

  const sw = document.createElement("input");
  sw.type = "number";
  sw.min = "0";
  sw.max = "20";
  sw.step = "0.5";
  sw.value = ctx.card.strokeWidth == null ? 2 : ctx.card.strokeWidth;
  sw.addEventListener("input", () => {
    ctx.card.strokeWidth = Number(sw.value) || 0;
    ctx.update();
  });
  body.appendChild(field("Stroke width", sw));
}

function drawInspector(body, ctx) {
  const note = document.createElement("p");
  note.className = "inspector-note";
  note.textContent = "Draw freehand over the card body in edit mode.";
  body.appendChild(note);
  body.appendChild(
    iconButton("Clear drawing", () => {
      ctx.card.path = "";
      ctx.update();
      ctx.rerender();
    })
  );
}

function geometryInspector(body, ctx) {
  const wrap = document.createElement("div");
  wrap.className = "field geo-grid";
  wrap.appendChild(numberField("W", ctx.card.w, (v) => {
    ctx.card.w = Math.max(40, v);
    ctx.update();
  }));
  wrap.appendChild(numberField("H", ctx.card.h, (v) => {
    ctx.card.h = Math.max(40, v);
    ctx.update();
  }));
  wrap.appendChild(numberField("X", ctx.card.x, (v) => {
    ctx.card.x = Math.max(0, v);
    ctx.update();
  }));
  wrap.appendChild(numberField("Y", ctx.card.y, (v) => {
    ctx.card.y = Math.max(0, v);
    ctx.update();
  }));
  body.appendChild(wrap);
}

function numberField(label, value, onInput) {
  const l = document.createElement("label");
  l.className = "geo-field";
  const s = document.createElement("span");
  s.textContent = label;
  const input = document.createElement("input");
  input.type = "number";
  input.value = Math.round(value);
  input.addEventListener("input", () => onInput(Math.round(Number(input.value) || 0)));
  l.appendChild(s);
  l.appendChild(input);
  return l;
}

function toHex(value, fallback) {
  if (typeof value === "string" && /^#[0-9a-fA-F]{6}$/.test(value)) return value;
  return fallback;
}
