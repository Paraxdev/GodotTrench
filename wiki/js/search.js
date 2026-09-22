// Full text search across every page, the Ctrl+K search dialog, and backlinks. The index is
// built lazily on first use by loading every page (drafts win over the repo copy), and the
// app refreshes the entry of the open page as it changes.

import { el } from "./dom.js";
import { icon } from "./icons.js";
import { markdownToText } from "./markdown.js";

const LINK_RE = /\[\[([^\]|\n]+)(?:\|[^\]\n]+)?\]\]|\]\(#([\w-]+)\)/g;

export class SearchIndex {
  // loadBoard(slug) -> Promise<board>, resolve(target) -> { slug } | null
  constructor(loadBoard, resolve) {
    this.loadBoard = loadBoard;
    this.resolve = resolve;
    this.entries = new Map();
    this.pages = [];
    this.ready = null;
  }

  setPages(pages) {
    this.pages = pages;
    for (const slug of Array.from(this.entries.keys())) {
      if (!pages.some((p) => p.slug === slug)) this.entries.delete(slug);
    }
  }

  ensure() {
    if (!this.ready) {
      this.ready = Promise.all(
        this.pages.map((p) =>
          this.loadBoard(p.slug)
            .then((board) => {
              if (!this.entries.has(p.slug)) this.update(p.slug, board);
            })
            .catch(() => {})
        )
      );
    }
    return this.ready;
  }

  update(slug, board) {
    const blocks = [];
    const links = new Set();
    (board && Array.isArray(board.cards) ? board.cards : []).forEach((c) => {
      let text = "";
      if (c.type === "text") {
        const md = c.md || "";
        text = markdownToText(md);
        let m;
        LINK_RE.lastIndex = 0;
        while ((m = LINK_RE.exec(md))) {
          const page = this.resolve((m[1] || m[2] || "").trim());
          if (page) links.add(page.slug);
        }
      } else if (c.type === "code") text = c.code || "";
      else if (c.type === "table") text = (c.rows || []).map((r) => r.join(" ")).join(" ");
      else if (c.type === "image") text = c.alt || "";
      if (text) blocks.push({ id: c.id, text, hidden: !!c.hidden });
    });
    this.entries.set(slug, { title: (board && board.title) || slug, blocks, links });
  }

  backlinks(slug) {
    return this.ensure().then(() =>
      this.pages.filter((p) => {
        if (p.slug === slug) return false;
        const e = this.entries.get(p.slug);
        return e && e.links.has(slug);
      })
    );
  }

  search(query) {
    const q = query.trim().toLowerCase();
    if (!q) return this.pages.slice(0, 12).map((p) => ({ page: p, snippet: null, cardId: null }));
    const words = q.split(/\s+/);
    const results = [];
    this.pages.forEach((p) => {
      const e = this.entries.get(p.slug);
      const title = (p.title || p.slug).toLowerCase();
      let score = 0;
      if (title === q) score += 100;
      else if (title.startsWith(q)) score += 60;
      else if (title.includes(q)) score += 40;
      else if (words.every((w) => title.includes(w))) score += 25;
      let best = null;
      if (e) {
        for (const b of e.blocks) {
          const t = b.text.toLowerCase();
          const at = t.indexOf(q);
          if (at >= 0) {
            best = best || { id: b.id, text: b.text, at, len: q.length };
            score += 8;
          } else if (words.length > 1 && words.every((w) => t.includes(w))) {
            best = best || { id: b.id, text: b.text, at: t.indexOf(words[0]), len: words[0].length };
            score += 4;
          }
        }
      }
      if (score > 0) results.push({ page: p, score, snippet: best, cardId: best ? best.id : null });
    });
    results.sort((a, b) => b.score - a.score);
    return results.slice(0, 20);
  }
}

// ----- The search dialog -----

export function openSearchDialog(index, onOpen) {
  const existing = document.querySelector(".search-modal");
  if (existing) {
    existing.querySelector("input").focus();
    return;
  }
  const input = el("input", {
    class: "search-input",
    type: "search",
    placeholder: "Search pages",
    spellcheck: false,
    "aria-label": "Search pages",
  });
  const list = el("div", { class: "search-results", role: "listbox" });
  const panel = el("div", { class: "search-panel", role: "dialog", "aria-modal": "true", "aria-label": "Search" }, [
    el("div", { class: "search-field" }, [icon("search", { size: 18 }), input, el("kbd", { text: "Esc" })]),
    list,
  ]);
  const modal = el("div", { class: "search-modal" }, panel);
  let results = [];
  let active = 0;

  const close = () => {
    modal.remove();
    document.removeEventListener("keydown", onKey, true);
  };
  const choose = (i) => {
    const r = results[i];
    if (!r) return;
    close();
    onOpen(r.page.slug, r.cardId);
  };
  const paint = () => {
    list.innerHTML = "";
    if (!results.length) {
      list.appendChild(el("div", { class: "search-empty", text: "Nothing matches that." }));
      return;
    }
    results.forEach((r, i) => {
      list.appendChild(
        el(
          "button",
          {
            type: "button",
            class: "search-item" + (i === active ? " active" : ""),
            on: { click: () => choose(i), mousemove: () => setActive(i) },
          },
          [
            el("span", { class: "search-item-icon" }, icon(r.page.layout === "canvas" ? "layout-dashboard" : "file-text", { size: 16 })),
            el("span", { class: "search-item-main" }, [
              el("span", { class: "search-item-title", text: r.page.title || r.page.slug }),
              r.snippet ? snippet(r.snippet) : el("span", { class: "search-item-cat", text: r.page.category || "" }),
            ]),
          ]
        )
      );
    });
  };
  const setActive = (i) => {
    active = i;
    list.querySelectorAll(".search-item").forEach((b, j) => b.classList.toggle("active", j === i));
    const node = list.querySelectorAll(".search-item")[i];
    if (node) node.scrollIntoView({ block: "nearest" });
  };
  const run = () => {
    results = index.search(input.value);
    active = 0;
    paint();
  };
  const onKey = (e) => {
    if (e.key === "Escape") {
      e.preventDefault();
      close();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      setActive(Math.min(results.length - 1, active + 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setActive(Math.max(0, active - 1));
    } else if (e.key === "Enter") {
      e.preventDefault();
      choose(active);
    }
  };

  input.addEventListener("input", run);
  modal.addEventListener("pointerdown", (e) => {
    if (e.target === modal) close();
  });
  document.addEventListener("keydown", onKey, true);
  document.body.appendChild(modal);
  input.focus();
  run();
  index.ensure().then(() => {
    if (document.body.contains(modal)) run();
  });
}

function snippet(s) {
  const start = Math.max(0, s.at - 40);
  const end = Math.min(s.text.length, s.at + s.len + 80);
  return el("span", { class: "search-item-snippet" }, [
    start > 0 ? "…" : "",
    s.text.slice(start, s.at),
    el("mark", { text: s.text.slice(s.at, s.at + s.len) }),
    s.text.slice(s.at + s.len, end),
    end < s.text.length ? "…" : "",
  ]);
}
