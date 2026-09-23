require(["gitbook", "jQuery"], function (gitbook, $) {
  var KEY = "td-show-mcp";

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
      $(this).addClass("td-mcp");
      $(this).nextUntil("li.header, li.divider").addClass("td-mcp");
    });
  }

  function addSwitch() {
    if ($(".td-mcp-switch").length) return;
    var sw = $(
      '<label class="td-mcp-switch" title="Show the parts of these docs about driving the editor over MCP, for AI agents and scripts">' +
        '<input type="checkbox"><span class="td-mcp-track"><span class="td-mcp-knob"></span></span>MCP</label>'
    );
    sw.find("input").on("change", function () {
      setMcpShown(this.checked);
    });
    $(".book-header").append(sw);
  }

  function apply() {
    var on = mcpShown();
    $("body").toggleClass("td-hide-mcp", !on);
    $(".td-mcp-switch input").prop("checked", on);
    $(".td-mcp-row").closest("tr").addClass("td-mcp");
  }

  // Glossary terms get a card on hover or focus instead of the browser's slow plain tooltip.
  function glossaryCards() {
    $(".glossary-term").each(function () {
      var term = $(this);
      if (term.data("td-card")) return;
      term.data("td-card", term.attr("title") || "").removeAttr("title").attr("tabindex", "0");
    });
  }

  var card = null;
  function showCard(el) {
    var text = $(el).data("td-card");
    if (!text) return;
    hideCard();
    card = $('<div class="td-card" role="tooltip"></div>').text(text).appendTo("body");
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
