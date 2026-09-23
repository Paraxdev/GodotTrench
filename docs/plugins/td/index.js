module.exports = {
  website: {
    assets: "./assets",
    js: ["td.js"],
    css: ["td.css"],
  },
  blocks: {
    // {% mcp %} ... {% endmcp %} marks text only people driving the editor over MCP need. The switch in the header
    // shows or hides it.
    mcp: {
      process: function (block) {
        return this.book.renderBlock("markdown", block.body).then(function (html) {
          return '<div class="td-mcp">' + html.trim() + "</div>";
        });
      },
    },
  },
};
