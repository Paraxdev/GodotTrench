require(["gitbook", "jQuery"], function (gitbook, $) {
  var KEY = "gt-show-mcp";

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

  // The sidebar's "MCP" part: its header and every chapter up to the next header.
  function markMcpSection() {
    $(".book-summary li.header").each(function () {
      if ($.trim($(this).text()).toUpperCase() !== "MCP") return;
      $(this).addClass("gt-mcp");
      $(this).nextUntil("li.header, li.divider").addClass("gt-mcp");
    });
  }

  function addSwitch() {
    if ($(".gt-mcp-switch").length) return;
    var sw = $(
      '<label class="gt-mcp-switch" title="Show the parts of these docs about driving the editor over MCP, for AI agents and scripts">' +
        '<input type="checkbox"><span class="gt-mcp-track"><span class="gt-mcp-knob"></span></span>MCP</label>'
    );
    sw.find("input").on("change", function () {
      setMcpShown(this.checked);
    });
    $(".book-header").append(sw);
  }

  function apply() {
    var on = mcpShown();
    $("body").toggleClass("gt-hide-mcp", !on);
    $(".gt-mcp-switch input").prop("checked", on);
    $(".gt-mcp-row").closest("tr").addClass("gt-mcp");
  }

  // Glossary terms get a card on hover or focus instead of the browser's slow plain tooltip.
  // Only the first use of each term on a page gets a card, the rest turn back into plain text.
  function glossaryCards() {
    var seen = {};
    $(".page-inner .glossary-term").each(function () {
      var term = $(this);
      var id = (term.attr("href") || "").split("#")[1] || term.text().toLowerCase();
      if (seen[id]) {
        term.replaceWith(document.createTextNode(term.text()));
        return;
      }
      seen[id] = true;
      if (term.data("gt-card")) return;
      term.data("gt-card", term.attr("title") || "").removeAttr("title").attr("tabindex", "0");
    });
  }

  var card = null;
  function showCard(el) {
    var text = $(el).data("gt-card");
    if (!text) return;
    hideCard();
    card = $('<div class="gt-card" role="tooltip"></div>').text(text).appendTo("body");
    var r = el.getBoundingClientRect();
    var w = card.outerWidth();
    var left = Math.max(8, Math.min(r.left + window.scrollX, window.scrollX + document.documentElement.clientWidth - w - 8));
    var top = r.bottom + window.scrollY + 6;
    if (r.bottom + card.outerHeight() + 12 > window.innerHeight) top = r.top + window.scrollY - card.outerHeight() - 6;
    card.css({ left: left, top: top });
  }

  function hideCard() {
    if (card) card.remove();
    card = null;
  }

  $(document).on("mouseenter focus", ".glossary-term", function () {
    showCard(this);
  });
  $(document).on("mouseleave blur", ".glossary-term", hideCard);

  gitbook.events.bind("page.change", function () {
    hideCard();
    markMcpSection();
    addSwitch();
    glossaryCards();
    apply();
  });
});
