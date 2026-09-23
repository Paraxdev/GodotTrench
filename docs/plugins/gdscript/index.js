// highlight.js ships no GDScript grammar. HonKit's highlight plugin requires the same highlight.js module, so
// registering here, when HonKit loads plugins, is enough for ```gdscript blocks to highlight.
const hljs = require("highlight.js");

const KEYWORDS = {
  keyword:
    "and as assert await break breakpoint class class_name const continue elif else enum extends for func if in is " +
    "match not or pass preload return self signal static super var when while",
  literal: "true false null PI TAU INF NAN",
  type:
    "void bool int float String StringName NodePath Array Dictionary Callable Signal Object Variant Vector2 Vector2i " +
    "Vector3 Vector3i Vector4 Color Rect2 AABB Plane Quaternion Basis Transform2D Transform3D PackedByteArray " +
    "PackedInt32Array PackedFloat32Array PackedStringArray PackedVector3Array Node Node2D Node3D Resource",
  built_in:
    "print print_debug printerr push_error push_warning len range load str int float abs min max clamp lerp sign " +
    "floor ceil round snapped is_instance_valid typeof get_node get_tree get_parent add_child queue_free emit connect",
};

hljs.registerLanguage("gdscript", (hljs) => ({
  name: "GDScript",
  aliases: ["gd"],
  keywords: KEYWORDS,
  contains: [
    hljs.HASH_COMMENT_MODE,
    { scope: "meta", match: /@[A-Za-z_]\w*/ },
    {
      scope: "string",
      variants: [
        { begin: /[&^]?"""/, end: /"""/ },
        { begin: /[&^]?"/, end: /"/, illegal: /\n/, contains: [hljs.BACKSLASH_ESCAPE] },
        { begin: /[&^]?'/, end: /'/, illegal: /\n/, contains: [hljs.BACKSLASH_ESCAPE] },
      ],
    },
    { scope: "number", match: /\b(0x[0-9a-fA-F_]+|0b[01_]+|\d[\d_]*(\.[\d_]+)?(e[+-]?\d+)?)\b/ },
    { scope: "symbol", match: /[$%][A-Za-z_][\w/]*/ },
    { match: [/\bfunc/, /\s+/, /[A-Za-z_]\w*/], scope: { 1: "keyword", 3: "title.function" } },
    { match: [/\b(?:class_name|extends|class)/, /\s+/, /[A-Za-z_][\w.]*/], scope: { 1: "keyword", 3: "title.class" } },
    { match: [/(?:->|:)/, /\s*/, /[A-Z][A-Za-z0-9_]*/], scope: { 3: "type" } },
    { scope: "title.function.invoke", match: /\b[A-Za-z_]\w*(?=\()/, keywords: KEYWORDS },
  ],
}));

module.exports = {};
