// A small GDScript language definition for highlight.js.
// GDScript is not part of the stock highlight.js bundle, so the wiki registers a compact
// grammar of its own. This keeps code cards labelled GDScript with real syntax highlighting.

export function registerGdscript(hljs) {
  if (!hljs || typeof hljs.registerLanguage !== "function") return;
  if (hljs.getLanguage && hljs.getLanguage("gdscript")) return;

  const KEYWORDS = {
    keyword:
      "and as assert await break breakpoint class class_name const continue elif else enum " +
      "export extends for func if in is match not or pass preload return self signal static " +
      "super tool var while yield void onready remote master puppet",
    literal: "true false null PI TAU INF NAN",
    built_in:
      "print printerr push_error push_warning len range load str int float bool abs min max " +
      "clamp lerp sign floor ceil round Vector2 Vector2i Vector3 Vector3i Color Rect2 Transform2D " +
      "Transform3D Node Node2D Node3D String StringName Array Dictionary Callable Signal Object " +
      "get_node get_tree emit_signal connect",
  };

  hljs.registerLanguage("gdscript", function () {
    return {
      name: "GDScript",
      aliases: ["gd"],
      keywords: KEYWORDS,
      contains: [
        hljs.HASH_COMMENT_MODE,
        // Annotations such as @export, @onready, @tool.
        { className: "meta", begin: /@[A-Za-z_]\w*/ },
        {
          className: "string",
          variants: [
            { begin: /"""/, end: /"""/ },
            { begin: /'''/, end: /'''/ },
            hljs.QUOTE_STRING_MODE,
            hljs.APOS_STRING_MODE,
          ],
        },
        hljs.C_NUMBER_MODE,
        {
          className: "function",
          beginKeywords: "func",
          end: /[:(]/,
          excludeEnd: true,
          contains: [hljs.UNDERSCORE_TITLE_MODE],
        },
        {
          className: "class",
          beginKeywords: "class class_name extends",
          end: /$/,
          keywords: KEYWORDS,
          contains: [hljs.UNDERSCORE_TITLE_MODE],
        },
      ],
    };
  });
}
