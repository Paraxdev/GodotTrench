# Trim

*Brush > Add Trim…* lays strips of brush along the edges of faces, standing out from them. Along the bottom edges of
walls they are baseboards, along the top edges crown molding, around the edges of a floor face a border, around a pool
its coping.

1. Select the walls (every upright face of the selected brushes gets trim), or pick single faces with the face tool.
2. Choose which **Edges**: bottom, top, sides or all.
3. Set **Height** (how far the strip runs over the face), **Depth** (how far it stands out) and **Inset** (a gap
   between the edge and the strip). **Use current** takes the material selected in the Materials panel.
4. Click **Add Trim**. The strips land in one group, or in a `func_detail_illusionary` when **Collision** is off, so
   players do not snag on them.

Where two trimmed faces meet, one strip runs into the corner and the other stops at it, so corners close without two
strips overlapping. Around the end of a free-standing wall or a doorway the strip wraps around. A strip that would end
up inside another brush, like on the ends of a lintel between two wall pieces, is left out.

**Only where a floor lies under the edge** keeps bottom trim to edges that stand on something, so the underside of a
lintel over a doorway gets no baseboard.

> **Tip:** Trim is plain brushes. Retexture, delete or reshape single strips afterwards like any other brush.

{% mcp %}
## MCP

The `trim` tool takes `faces` as `[[brush, face], ...]`, or brush `ids`, or the selection, plus `edges`, `height`,
`depth`, `inset`, `material`, `on_floor` and `collision`. It returns the new group or entity and how many strips it
made.
{% endmcp %}
