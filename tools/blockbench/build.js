(async (spec) => {
  // Close earlier projects without the save prompt, so tabs do not pile up.
  for (const p of [...ModelProject.all]) p.close(true);
  newProject(Formats.free);
  Project.name = spec.name;
  Project.texture_width = spec.tw;
  Project.texture_height = spec.th;
  const tex = new Texture({ name: spec.name + '.png' }).fromDataURL(spec.png).add(false);
  for (let i = 0; i < 100 && !tex.width; i++) await new Promise((r) => setTimeout(r, 20));
  tex.uv_width = spec.tw;
  tex.uv_height = spec.th;

  const groups = {};
  const group = (name) => groups[name] || (groups[name] = new Group({ name, origin: [0, 0, 0] }).init());
  for (const el of spec.elements) {
    const parent = group(el.group);
    if (el.type === 'cube') {
      const c = new Cube({ name: el.name, from: el.from, to: el.to, origin: el.origin, rotation: el.rotation, inflate: el.inflate, box_uv: false, autouv: 0 });
      for (const f of ['north', 'south', 'east', 'west', 'up', 'down']) {
        if (el.faces[f]) {
          c.faces[f].uv = el.faces[f].uv;
          c.faces[f].texture = tex.uuid;
        } else {
          c.faces[f].texture = null;
        }
      }
      c.addTo(parent).init();
    } else {
      const m = new Mesh({ name: el.name, origin: [0, 0, 0], vertices: {} });
      const keys = m.addVertices(...el.vertices);
      const faces = el.faces.map((f) => {
        const uv = {};
        for (const i of f.v) uv[keys[i]] = f.uv[String(i)];
        return new MeshFace(m, { vertices: f.v.map((i) => keys[i]), uv, texture: tex.uuid });
      });
      m.addFaces(...faces);
      m.addTo(parent).init();
    }
  }
  Canvas.updateAll();
  return Codecs.project.compile();
})(__SPEC__)
