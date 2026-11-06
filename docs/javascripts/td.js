// The MCP switch, the glossary cards, old heading ids and the repository facts. navigation.instant swaps pages without
// reloading this script, so the page specific work runs again on every document$ emission.
(function () {
  var KEY = "td-show-mcp";
  var SKIP = "code, pre, a, abbr, kbd, script, style, svg, button, h1, h2, h3, h4, h5, h6, .admonition-title, " +
    ".no-glossary";

  function mcpShown() {
    try {
      return localStorage.getItem(KEY) === "1";
    } catch (e) {
      return false;
    }
  }

  function setMcpShown(on) {
    try {
      localStorage.setItem(KEY, on ? "1" : "0");
    } catch (e) {}
    apply();
  }

  function apply() {
    var on = mcpShown();
    document.body.classList.toggle("td-hide-mcp", !on);
    var input = document.querySelector(".td-mcp-switch input");
    if (input) input.checked = on;
  }

  // The header survives instant navigation, so the switch is added once.
  function addSwitch() {
    var header = document.querySelector(".md-header__inner");
    if (!header || header.querySelector(".td-mcp-switch")) return;
    var label = document.createElement("label");
    label.className = "td-mcp-switch";
    var tip = document.querySelector('meta[name="td-mcp-switch"]');
    if (tip) label.title = tip.content;
    label.innerHTML = '<input type="checkbox"><span class="td-mcp-track"><span class="td-mcp-knob"></span></span>MCP';
    label.querySelector("input").addEventListener("change", function () {
      setMcpShown(this.checked);
    });
    var before = header.querySelector("[data-md-component=palette]") || header.querySelector(".md-search");
    header.insertBefore(label, before);
  }

  // The MCP pages are found by their URL, not by the title of the nav section that holds them, which is Advanced.
  var MCP_PAGES = ["mcp", "mcp-scripts"];

  function isMcpPage(href) {
    var page = new URL(href, location.href).pathname.replace(/(\/|\.html|\/index\.html)$/, "").split("/").pop();
    return MCP_PAGES.indexOf(page) >= 0;
  }

  // Tags what the switch hides besides the blocks themselves: the nav entries of the MCP pages and any section left
  // empty without them, the table of contents entries of headings inside a block, footer links into the MCP pages and
  // table rows holding a .td-mcp-row marker.
  function markMcp() {
    var pages = [];
    document.querySelectorAll(".md-nav--primary a.md-nav__link[href]").forEach(function (a) {
      if (!isMcpPage(a.href)) return;
      a.closest("li").classList.add("td-mcp");
      pages.push(a.href.split("#")[0]);
    });
    document.querySelectorAll(".md-nav--primary .md-nav__item--nested").forEach(function (item) {
      var pageItems = item.querySelectorAll("li.md-nav__item:not(.md-nav__item--nested)");
      var allMcp = Array.prototype.every.call(pageItems, function (li) {
        return li.classList.contains("td-mcp") || li.closest(".md-nav--secondary");
      });
      if (pageItems.length && allMcp) item.classList.add("td-mcp");
    });

    var ids = {};
    document.querySelectorAll(".td-mcp [id]:is(h2, h3, h4, h5, h6)").forEach(function (h) {
      ids[h.id] = true;
    });
    // Instant navigation rewrites links to absolute URLs, so the hash is compared rather than the href attribute.
    document.querySelectorAll(".md-nav--secondary a.md-nav__link").forEach(function (a) {
      if (ids[decodeURIComponent(a.hash.slice(1))]) a.closest("li").classList.add("td-mcp");
    });

    document.querySelectorAll("a.md-footer__link").forEach(function (a) {
      a.classList.toggle("td-mcp", pages.indexOf(a.href.split("#")[0]) >= 0);
    });

    document.querySelectorAll(".td-mcp-row").forEach(function (el) {
      var row = el.closest("tr");
      if (row) row.classList.add("td-mcp");
    });
  }

  // A term edge next to a letter or digit is part of a longer word, except next to scripts that write no spaces, where
  // a term sits right against the text around it, Latin ones too (MCPサーバー).
  var WORD = "(?![\\p{Script=Han}\\p{Script=Hiragana}\\p{Script=Katakana}\\p{Script=Hangul}])[\\p{L}\\p{N}_]";
  function termPattern(name) {
    var escaped = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    var guarded = function (ch) {
      return new RegExp("^" + WORD, "u").test(ch);
    };
    return (guarded(name[0]) ? "(?<!" + WORD + ")" : "") + escaped +
      (guarded(name[name.length - 1]) ? "(?!" + WORD + ")" : "");
  }

  // The glossary page is the single source: its h2 headings are the terms and the paragraph under each the card text.
  var glossary = null;
  function loadGlossary() {
    if (glossary) return glossary;
    var url = new URL("GLOSSARY.html", __md_scope);
    glossary = fetch(url)
      .then(function (r) {
        if (!r.ok) throw new Error(r.status);
        return r.text();
      })
      .then(function (html) {
        var doc = new DOMParser().parseFromString(html, "text/html");
        var entries = {};
        var names = [];
        doc.querySelectorAll(".md-content h2[id]").forEach(function (h) {
          var def = h.nextElementSibling;
          if (!def || def.tagName !== "P") return;
          var name = "";
          h.childNodes.forEach(function (n) {
            if (!(n.classList && n.classList.contains("headerlink"))) name += n.textContent;
          });
          name = name.trim();
          var text = def.textContent.replace(/\s+/g, " ").trim();
          var entry = { href: url.href + "#" + h.id, text: text };
          // Inflected forms of the term, for languages where the word changes with the sentence.
          var forms = (h.getAttribute("data-forms") || "").split("|");
          [name].concat(forms).forEach(function (form) {
            form = form.trim();
            if (!form) return;
            entries[form.toLowerCase()] = entry;
            names.push(form);
          });
        });
        // Longest first, so "UV lock" wins over "UV" and "brush entity" over "brush".
        names.sort(function (a, b) {
          return b.length - a.length;
        });
        return { entries: entries, re: new RegExp(names.map(termPattern).join("|"), "giu") };
      })
      .catch(function () {
        glossary = null;
        return null;
      });
    return glossary;
  }

  // Links the first use of each glossary term on the page, never in code, links or headings. Text inside MCP blocks is
  // looked at last, so a term first used there still gets its card in the text everyone sees.
  function annotate(g) {
    var root = document.querySelector(".md-content__inner");
    if (!g || !root || root.dataset.gtTerms) return;
    root.dataset.gtTerms = "1";
    var nodes = [];
    var walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
      acceptNode: function (n) {
        return n.parentElement.closest(SKIP) ? NodeFilter.FILTER_REJECT : NodeFilter.FILTER_ACCEPT;
      },
    });
    while (walker.nextNode()) nodes.push(walker.currentNode);
    var visible = nodes.filter(function (n) {
      return !n.parentElement.closest(".td-mcp");
    });
    var mcp = nodes.filter(function (n) {
      return visible.indexOf(n) < 0;
    });

    var seen = {};
    visible.concat(mcp).forEach(function (node) {
      var found = [];
      var m;
      g.re.lastIndex = 0;
      while ((m = g.re.exec(node.nodeValue))) {
        var entry = g.entries[m[0].toLowerCase()];
        if (!entry || seen[entry.href]) continue;
        seen[entry.href] = true;
        found.push({ at: m.index, text: m[0], entry: entry });
      }
      // From the back, so the earlier offsets stay valid while the node is split.
      for (var i = found.length - 1; i >= 0; i--) {
        var term = node.splitText(found[i].at);
        term.splitText(found[i].text.length);
        var a = document.createElement("a");
        a.className = "td-term";
        a.href = found[i].entry.href;
        a.dataset.gtCard = found[i].entry.text;
        a.textContent = found[i].text;
        term.replaceWith(a);
      }
    });
  }

  var card = null;
  function showCard(el) {
    hideCard();
    card = document.createElement("div");
    card.className = "td-card";
    card.setAttribute("role", "tooltip");
    card.textContent = el.dataset.gtCard;
    document.body.appendChild(card);
    var r = el.getBoundingClientRect();
    var left = Math.max(8, Math.min(r.left, document.documentElement.clientWidth - card.offsetWidth - 8));
    var top = r.bottom + 6;
    if (top + card.offsetHeight + 6 > window.innerHeight) top = r.top - card.offsetHeight - 6;
    card.style.left = left + window.scrollX + "px";
    card.style.top = top + window.scrollY + "px";
  }

  function hideCard() {
    if (card) card.remove();
    card = null;
  }

  ["mouseover", "focusin"].forEach(function (type) {
    document.addEventListener(type, function (e) {
      var term = e.target.closest && e.target.closest(".td-term");
      if (term && (!card || card.textContent !== term.dataset.gtCard)) showCard(term);
    });
  });
  ["mouseout", "focusout"].forEach(function (type) {
    document.addEventListener(type, function (e) {
      var term = e.target.closest && e.target.closest(".td-term");
      if (term && !term.contains(e.relatedTarget)) hideCard();
    });
  });

  // HonKit, which built these docs before, dropped underscores from heading ids. Links from that time, like the F1 help
  // of older editor builds, ask for #pathcorner where the heading is now #path_corner.
  function followOldAnchor() {
    var hash = location.hash.slice(1);
    if (!hash || document.getElementById(hash)) return;
    var headings = document.querySelectorAll(".md-content :is(h1, h2, h3, h4, h5, h6)[id]");
    for (var i = 0; i < headings.length; i++) {
      if (headings[i].id.replace(/_/g, "") === hash) {
        location.replace("#" + headings[i].id);
        return;
      }
    }
  }

  // Stars and forks for the repository link, which overrides/partials/source.html takes away from the theme. Fetched
  // once per session, like the theme does.
  var FACTS = "td-repo-facts";
  var facts = null;
  function loadFacts(href) {
    if (facts) return facts;
    try {
      var cached = JSON.parse(sessionStorage.getItem(FACTS));
      if (cached) return (facts = Promise.resolve(cached));
    } catch (e) {}
    facts = fetch("https://api.github.com/repos" + new URL(href).pathname.replace(/\/$/, ""))
      .then(function (r) {
        if (!r.ok) throw new Error(r.status);
        return r.json();
      })
      .then(function (repo) {
        var found = { stars: repo.stargazers_count, forks: repo.forks_count };
        try {
          sessionStorage.setItem(FACTS, JSON.stringify(found));
        } catch (e) {}
        return found;
      })
      .catch(function () {
        return null;
      });
    return facts;
  }

  function count(n) {
    return n > 999 ? (n / 1000).toFixed(1).replace(/\.0$/, "") + "k" : String(n);
  }

  function addFacts() {
    var link = document.querySelector("a.td-source");
    if (!link) return;
    loadFacts(link.href).then(function (found) {
      if (!found) return;
      document.querySelectorAll("a.td-source .md-source__repository").forEach(function (repo) {
        if (repo.querySelector(".md-source__facts")) return;
        var list = document.createElement("ul");
        list.className = "md-source__facts";
        ["stars", "forks"].forEach(function (key) {
          var li = document.createElement("li");
          li.className = "md-source__fact md-source__fact--" + key;
          li.textContent = count(found[key]);
          list.appendChild(li);
        });
        repo.appendChild(list);
        repo.classList.add("md-source__repository--active");
      });
    });
  }

  // The theme renders the language links once, but instant navigation changes the page under them. So a click builds the
  // link from where the reader is now, the same page and heading in the other language. It runs in the capture phase,
  // before the instant navigation reads the link. The 404 page has no canonical link and keeps the language home.
  document.addEventListener(
    "click",
    function (e) {
      var link = e.target.closest && e.target.closest(".md-select__link");
      if (!link) return;
      var target = link.dataset.root;
      if (document.querySelector("link[rel=canonical]")) {
        target += location.pathname.slice(__md_scope.pathname.length);
      }
      link.href = target + location.hash;
    },
    true
  );

  window.addEventListener("hashchange", followOldAnchor);

  document$.subscribe(function () {
    hideCard();
    addSwitch();
    markMcp();
    apply();
    followOldAnchor();
    addFacts();
    loadGlossary().then(annotate);
  });
})();
