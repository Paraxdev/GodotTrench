extends RefCounted
## Regression tests for the Godot-vs-editor parity audit: unique instance ids, mesh blend option passthrough,
## quad and long face triangulation matching td_geom::polygon::triangulate, terrain collision holes and the
## alternating diagonal, rotated prefab terrain placement, concave mesh containment, tool/transparent
## containers not burying faces, terrain layers keeping their material's maps, terrain height_at following the
## triangles, and out of range brush indices being skipped.
##
## res://tests/run_tests.gd calls [method run] from its own [method SceneTree._initialize]. [param t] is that
## script's own instance, used for its [code]check[/code]/[code]near[/code]/[code]find_named[/code] helpers.

const SETTINGS := "res://demo/demo_map_settings.tres"

static func run(t) -> void:
	_test_instance_id_namespacing(t)
	_test_mesh_blend_options(t)
	_test_quad_triangulation_matches_editor(t)
	_test_long_face_triangulation_matches_editor(t)
	_test_terrain_collision_holes_and_diagonal(t)
	_test_rotated_instance_terrain_position(t)
	_test_concave_mesh_containment(t)
	_test_tool_and_transparent_containers_do_not_bury(t)
	_test_face_over_a_mesh_tunnel_stays(t)
	_test_buried_faces_keep_their_triangles(t)
	_test_buried_brush_faces_clip_from_the_cull_budget(t)
	_test_smoothing_a_vertex_of_thousands_of_faces_shades_flat(t)
	_test_a_face_of_collinear_corners_does_not_count_toward_smoothing(t)
	_test_terrain_splat_shader_defaults_to_first_layer(t)
	_test_terrain_missing_layer_slots_use_first_layer(t)
	_test_terrain_emissive_layers_glow(t)
	_test_terrain_layers_use_normal_and_roughness_maps(t)
	_test_terrain_layer_ambient_occlusion(t)
	_test_terrain_height_at_follows_triangles(t)
	_test_brush_bad_index_is_skipped(t)
	_test_property_values_read_like_the_editor(t)
	_test_environment_sun_keys(t)
	_test_decal_uses_the_map_scale(t)
	_test_origin_texture_and_zero_uv_scale(t)
	await t.process_frame

static func _box(id: int, mn: Vector3, mx: Vector3, material: String) -> Dictionary:
	var vertices := []
	for y in [mn.y, mx.y]:
		for c in [[mn.x, mn.z], [mx.x, mn.z], [mx.x, mx.z], [mn.x, mx.z]]:
			vertices.append([c[0], y, c[1]])
	var faces := []
	for indices in [[4, 7, 6, 5], [0, 1, 2, 3], [1, 5, 6, 2], [0, 3, 7, 4], [3, 2, 6, 7], [0, 4, 5, 1]]:
		faces.append({ "indices": indices, "material": material })
	return { "type": "brush", "id": id, "vertices": vertices, "faces": faces }

static func _write(name: String, json: Dictionary) -> String:
	var path := OS.get_temp_dir().path_join(name)
	FileAccess.open(path, FileAccess.WRITE).store_string(JSON.stringify(json))
	return path

## Finding 1: two occurrences of one prefab must not share group, entity, terrain or brush ids.
static func _test_instance_id_namespacing(t) -> void:
	print("- instance id namespacing")
	var settings: FuncGodotMapSettings = load(SETTINGS)
	var prefab := {
		"format": "trenchdot-map", "version": 1, "properties": {},
		"layers": [{ "id": 1, "type": "layer", "name": "Default", "children": [
			{ "id": 2, "type": "group", "name": "Inner", "children": [
				{ "id": 3, "type": "entity", "classname": "light", "origin": [0.0, 0.0, 0.0], "angles": [0.0, 0.0, 0.0], "properties": { "targetname": "plight" } },
				_box(4, Vector3(-8, -8, -8), Vector3(8, 8, 8), "showcase/cobble"),
			] },
		] }],
	}
	_write("td_ns_prefab.tdm", prefab)
	var main_map := {
		"format": "trenchdot-map", "version": 1, "properties": {},
		"layers": [{ "id": 1, "type": "layer", "name": "Default", "children": [
			{ "id": 10, "type": "instance", "path": "td_ns_prefab.tdm", "origin": [0.0, 0.0, 0.0], "angles": [0.0, 0.0, 0.0], "fixup": "a" },
			{ "id": 11, "type": "instance", "path": "td_ns_prefab.tdm", "origin": [256.0, 0.0, 0.0], "angles": [0.0, 0.0, 0.0], "fixup": "b" },
		] }],
	}
	var main_path := _write("td_ns_main.tdm", main_map)
	var data := FuncGodotParser.new().parse_map_data(main_path, settings)

	var group_ids: Array = data.groups.map(func(g): return g.id)
	t.check(group_ids.size() == group_ids.duplicate().reduce(func(acc, x): return acc if x in acc else acc + [x], []).size(), "two instances of one prefab keep every group id unique, got %s" % [group_ids])

	var lights := data.entities.filter(func(e): return str(e.properties.get("classname", "")) == "light")
	t.check(lights.size() == 2, "both instances built their entity")
	if lights.size() == 2:
		t.check(lights[0].node_id != lights[1].node_id and lights[0].node_id != 0 and lights[1].node_id != 0, "the two instances' light entities keep distinct ids, got %s" % [lights.map(func(e): return e.node_id)])
		var names := lights.map(func(e): return e.properties.get("targetname", ""))
		t.check(names.has("a-plight") and names.has("b-plight"), "instance fixups still apply, got %s" % [names])

	var world_brushes := data.entities[0].brushes
	t.check(world_brushes.size() == 2, "both instances' boxes were kept, got %d" % world_brushes.size())
	if world_brushes.size() == 2:
		t.check(world_brushes[0].node_id != world_brushes[1].node_id, "the two instances' brushes keep distinct ids, got %s" % [world_brushes.map(func(b): return b.node_id)])

	# Building the scene must not hit FuncGodot's "already has a parent" collision: each instance's group
	# node is real and the entities end up under their own instance's group, not the other one's.
	var map := FuncGodotMap.new()
	map.map_settings = settings
	map.local_map_file = main_path
	t.root.add_child(map)
	map.build()
	await t.process_frame
	var light_a: Node3D = t.find_named(map, "entity_a-plight")
	var light_b: Node3D = t.find_named(map, "entity_b-plight")
	t.check(light_a != null and light_b != null, "both instance entities are in the built scene")
	if light_a and light_b:
		t.check(light_a.get_parent() != light_b.get_parent(), "the two instances keep their own group node, not a shared, collided one")
	map.queue_free()
	await t.process_frame

## Finding 2: a mesh face's blend_detile/blend_uv_scale/blend_detile_sharpen props must reach the composite
## texture name, like the brush path already does.
static func _test_mesh_blend_options(t) -> void:
	print("- mesh blend option passthrough")
	var uv := { "u_axis": [1.0, 0.0, 0.0], "v_axis": [0.0, 0.0, 1.0], "offset": [0.0, 0.0], "scale": [1.0, 1.0] }
	var node := { "id": 1, "type": "mesh",
		"vertices": [[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [4.0, 0.0, 4.0], [0.0, 0.0, 4.0]],
		"faces": [{ "indices": [0, 1, 2, 3], "material": "base/floor", "uv": uv,
			"props": { "blend_material": "base/grass", "blend_detile": 0.4, "blend_uv_scale": 2.5, "blend_detile_sharpen": 0.75 } }] }
	var brush := TrenchDotMesh.parse(node, Transform3D.IDENTITY, 1.0, "")
	t.check(brush != null and brush.faces.size() == 1, "mesh face parsed")
	if brush and brush.faces.size() == 1:
		var expected := TrenchDotBlend.key("base/floor", "base/grass", 0.4, 2.5, 0.75)
		t.check(brush.faces[0].texture == expected, "mesh blend options ride along with the texture name, got %s want %s" % [brush.faces[0].texture, expected])
		var opts := TrenchDotBlend.options(brush.faces[0].texture)
		t.check(t.near(opts[0], 0.4) and t.near(opts[1], 2.5) and t.near(opts[2], 0.75), "the options round trip, got %s" % [opts])

## Finding 3: a convex quad splits along the shorter diagonal, exactly like td_geom::polygon::triangulate.
static func _test_quad_triangulation_matches_editor(t) -> void:
	print("- quad triangulation matches the editor")
	# Diagonal 0-2 is shorter.
	var a := PackedVector3Array([Vector3(0, 0, 0), Vector3(4, 0, 0), Vector3(3, 0, 2), Vector3(0, 0, 2)])
	var tris_a := TrenchDotMesh.triangulate(a, TrenchDotMesh._newell(a))
	t.check(tris_a == PackedInt32Array([0, 1, 2, 0, 2, 3]), "splits along the shorter 0-2 diagonal, got %s" % [tris_a])
	# Diagonal 1-3 is shorter.
	var b := PackedVector3Array([Vector3(0, 0, 0), Vector3(4, 0, 0), Vector3(4, 0, 2), Vector3(1, 0, 2)])
	var tris_b := TrenchDotMesh.triangulate(b, TrenchDotMesh._newell(b))
	t.check(tris_b == PackedInt32Array([0, 1, 3, 1, 2, 3]), "splits along the shorter 1-3 diagonal, got %s" % [tris_b])

static func _circle(n: int, radius: float) -> PackedVector3Array:
	var out := PackedVector3Array()
	for i in n:
		var a := float(i) / n * TAU
		out.append(Vector3(cos(a) * radius, 0.0, -sin(a) * radius))
	return out

static func _gear(n: int) -> PackedVector3Array:
	var out := _circle(n, 1.0)
	for i in n:
		out[i] *= 512.0 if i % 2 == 0 else 480.0
	return out

## An outer and an inner arc so close that every outer corner's ear holds an inner corner.
static func _thin_band(n: int) -> PackedVector3Array:
	var r := 4096.0
	var step := 1.0 / n
	var w := r * (1.0 - cos(step)) * 0.25
	var out := PackedVector3Array()
	for i in n / 2:
		out.append(Vector3(cos(i * step) * r, 0.0, -sin(i * step) * r))
	for i in range(n / 2 - 1, -1, -1):
		out.append(Vector3(cos(i * step) * (r - w), 0.0, -sin(i * step) * (r - w)))
	return out

## Ear clipping tests only reflex corners and fans the rest of a face once its work budget is spent, like
## td_geom::polygon::triangulate, so big faces clip the editor's triangles and faces made to block their ears cannot
## stall the import.
static func _test_long_face_triangulation_matches_editor(t) -> void:
	print("- long face triangulation matches the editor")
	var gear := _gear(1024)
	var tris := TrenchDotMesh.triangulate(gear, TrenchDotMesh._newell(gear))
	var picks := []
	for k in [0, 1, 600, 1020, 1021]:
		picks.append(Array(tris.slice(k * 3, k * 3 + 3)))
	t.check(tris.size() == 1022 * 3 and picks == [[1023, 0, 1], [1, 2, 3], [1023, 599, 601], [1023, 1019, 1021], [1021, 1022, 1023]], "a 1024 corner gear clips the editor's triangles, got %s" % [picks])
	var repeated := PackedVector3Array()
	var octagon := _circle(8, 64.0)
	for i in 200000:
		repeated.append(octagon[i % 8])
	for points: PackedVector3Array in [_thin_band(2048), repeated]:
		var started := Time.get_ticks_msec()
		tris = TrenchDotMesh.triangulate(points, TrenchDotMesh._newell(points))
		var took := Time.get_ticks_msec() - started
		t.check(tris.size() == (points.size() - 2) * 3 and took < 5000, "a %d corner face made to block its ears triangulates in %d ms" % [points.size(), took])
	var spent := TrenchDotMesh.ClipBudget.new()
	spent.take(TrenchDotMesh.BUILD_CLIP_WORK)
	tris = TrenchDotMesh.triangulate(gear, TrenchDotMesh._newell(gear), spent)
	t.check(tris.slice(0, 6) == PackedInt32Array([0, 1, 2, 0, 2, 3]), "a face is fanned once its build's budget is spent, got %s" % [tris.slice(0, 6)])

## Finding 4: terrain collision must respect per-cell holes exactly and use the same alternating diagonal as
## the visuals, which a HeightMapShape3D (per-vertex NaN, one fixed diagonal) cannot do.
static func _test_terrain_collision_holes_and_diagonal(t) -> void:
	print("- terrain collision holes and diagonal")
	var heights := PackedFloat32Array()
	heights.resize(9)
	var holes := PackedByteArray([0, 1, 0, 0])
	var data := {
		"resolution": [3, 3], "cell_size": 1.0, "chunk_cells": 32,
		"heights": Marshalls.raw_to_base64(heights.to_byte_array()),
		"holes": Marshalls.raw_to_base64(holes),
		"layers": [],
	}
	var settings: FuncGodotMapSettings = load(SETTINGS)
	var terrain := TrenchDotTerrain.create(data, Vector3.ZERO, settings)
	t.check(terrain != null, "terrain built")
	if terrain:
		var shapes := terrain.get_children().filter(func(n): return n is CollisionShape3D)
		t.check(shapes.size() == 1, "one chunk, one collision shape, got %d" % shapes.size())
		if shapes.size() == 1:
			var shape: Shape3D = (shapes[0] as CollisionShape3D).shape
			t.check(shape is ConcavePolygonShape3D, "terrain collides as a trimesh, not a heightmap")
			if shape is ConcavePolygonShape3D:
				var faces: PackedVector3Array = (shape as ConcavePolygonShape3D).get_faces()
				# 4 cells, one is a hole: 3 solid cells * 2 triangles * 3 verts.
				t.check(faces.size() == 18, "the hole cell is missing from collision, got %d verts" % faces.size())
		terrain.free()

## Finding 5: a rotated instance rotates the terrain's center about the instance origin, like
## Terrain::transformed, instead of only translating it (which would leave a rotated prefab's terrain
## sitting where it was authored instead of where the instance places it).
static func _test_rotated_instance_terrain_position(t) -> void:
	print("- rotated instance terrain position")
	var heights := PackedFloat32Array()
	heights.resize(9)
	var data := { "resolution": [3, 3], "cell_size": 2.0, "origin": [0.0, 0.0, 0.0], "layers": [],
		"heights": Marshalls.raw_to_base64(heights.to_byte_array()) }
	var settings: FuncGodotMapSettings = load(SETTINGS)
	var xform := Transform3D(Basis(Vector3.UP, deg_to_rad(90.0)), Vector3(10.0, 0.0, 0.0))
	var terrain := TrenchDotTerrain.create(data, xform, settings)
	t.check(terrain != null, "rotated terrain built")
	if terrain:
		var half := Vector3(2.0, 0.0, 2.0) # (res - 1) * cell_size * 0.5
		var center := xform.basis * half + xform.origin
		var expected := (center - half) * settings.scale_factor
		t.check(t.near(terrain.position, expected, 0.01), "terrain center rotates with the instance, got %s want %s" % [terrain.position, expected])
		var naive := xform.origin * settings.scale_factor
		t.check(not t.near(terrain.position, naive, 0.01), "a rotated instance must not just translate the terrain's authored origin, got %s" % terrain.position)
		terrain.free()

## Finding 6: a closed mesh's containment test must use the face's own triangulation, not a fan, so a concave
## cap does not treat its own notch as solid.
static func _test_concave_mesh_containment(t) -> void:
	print("- concave mesh containment uses real triangulation")
	# A dart shaped prism: profile A,B,C,D is concave at D, so the fan triangle (A,C,D) covers the notch that
	# is actually outside the true polygon. See the test's own point-in-polygon derivation in the audit report.
	var node := { "id": 1, "type": "mesh",
		"vertices": [
			[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [4.0, 0.0, 4.0], [1.5, 0.0, 0.5],
			[0.0, 2.0, 0.0], [4.0, 2.0, 0.0], [4.0, 2.0, 4.0], [1.5, 2.0, 0.5],
		],
		"faces": [
			{ "indices": [0, 1, 2, 3], "material": "showcase/cobble" },
			{ "indices": [4, 7, 6, 5], "material": "showcase/cobble" },
			{ "indices": [0, 1, 5, 4], "material": "showcase/cobble" },
			{ "indices": [1, 2, 6, 5], "material": "showcase/cobble" },
			{ "indices": [2, 3, 7, 6], "material": "showcase/cobble" },
			{ "indices": [3, 0, 4, 7], "material": "showcase/cobble" },
		],
	}
	var brush := TrenchDotMesh.parse(node, Transform3D.IDENTITY, 1.0, "")
	t.check(brush != null and brush.closed, "the dart prism is a closed mesh")
	if brush and brush.closed:
		var solid := TrenchDotFaceCull._solid(brush, 1.0)
		t.check(solid != null, "a closed mesh builds a containment solid")
		if solid:
			# TrenchDotMesh stores vertices rotated into FuncGodot's id space (id = (z, x, y)), so probe points
			# go through the same TrenchDotParser.to_id conversion as the mesh's own vertices did.
			var work := TrenchDotFaceCull.Work.new(TrenchDotFaceCull.INTERIOR_WORK)
			t.check(solid.contains(TrenchDotParser.to_id(Vector3(2.0, 1.0, 1.0)), work), "a point genuinely inside the dart is inside")
			t.check(not solid.contains(TrenchDotParser.to_id(Vector3(1.8333, 1.0, 1.5)), work), "a point in the notch, which only a fan triangulation would wrongly cover, is outside")

## Finding 7: only opaque, non-tool containers may bury the faces of what they enclose.
static func _test_tool_and_transparent_containers_do_not_bury(t) -> void:
	print("- tool and transparent containers do not bury detail")
	var settings: FuncGodotMapSettings = load(SETTINGS)
	for case in [["showcase/cobble", true], ["special/clip", false], ["showcase/water", false]]:
		var outer_material: String = case[0]
		var should_bury: bool = case[1]
		var children := [_box(2, Vector3(-256, -256, -256), Vector3(256, 256, 256), outer_material), _box(3, Vector3(-32, -32, -32), Vector3(32, 32, 32), "showcase/cobble")]
		var map_json := { "format": "trenchdot-map", "properties": {}, "layers": [{ "type": "layer", "id": 1, "children": children }] }
		var path := _write("td_container_rule_test.tdm", map_json)
		var data := FuncGodotParser.new().parse_map_data(path, settings)
		FuncGodotGeometryGenerator.new(settings).build(0, data.entities)
		var inner: FuncGodotData.BrushData = data.entities[0].brushes[1]
		if should_bury:
			t.check(inner.faces.all(func(f): return f.render_hidden), "an opaque non-tool container (%s) still buries what is inside it" % outer_material)
		else:
			t.check(inner.faces.all(func(f): return not f.render_hidden), "a %s container must not bury what is inside it" % outer_material)

## A closed mesh box with a square tunnel through it along z, off centre so a plate around the tunnel has its
## centre in solid material.
static func _tunnel_mesh(id: int) -> Dictionary:
	var vertices := []
	for z in [0.0, 64.0]:
		for c in [[0.0, 0.0], [96.0, 0.0], [96.0, 96.0], [0.0, 96.0], [16.0, 16.0], [32.0, 16.0], [32.0, 32.0], [16.0, 32.0]]:
			vertices.append([c[0], c[1], z])
	var faces := []
	for k in 4:
		var n := (k + 1) % 4
		for indices in [[n, k, 4 + k, 4 + n], [8 + k, 8 + n, 12 + n, 12 + k], [k, n, 8 + n, 8 + k], [4 + n, 4 + k, 12 + k, 12 + n]]:
			faces.append({ "indices": indices, "material": "showcase/cobble" })
	return { "id": id, "type": "mesh", "vertices": vertices, "faces": faces }

## A plate around the tunnel has every corner and its centre in the mesh, only the tunnel's edges passing through
## its middle show it is not buried, and they are found on the plate's own triangles. Brush and mesh plates take
## different paths there.
static func _test_face_over_a_mesh_tunnel_stays(t) -> void:
	print("- a face over a tunnel through a closed mesh stays")
	var settings: FuncGodotMapSettings = load(SETTINGS)
	for type in ["brush", "mesh"]:
		var plate := _box(2, Vector3(8, 8, 24), Vector3(88, 88, 40), "showcase/cobble")
		var buried := _box(3, Vector3(48, 48, 8), Vector3(80, 80, 16), "showcase/cobble")
		plate["type"] = type
		buried["type"] = type
		var map_json := { "format": "trenchdot-map", "properties": {}, "layers": [{ "type": "layer", "id": 1, "children": [_tunnel_mesh(1), plate, buried] }] }
		var data := FuncGodotParser.new().parse_map_data(_write("td_tunnel_cull_test.tdm", map_json), settings)
		FuncGodotGeometryGenerator.new(settings).build(0, data.entities)
		var brushes: Array[FuncGodotData.BrushData] = data.entities[0].brushes
		var shown := brushes[1].faces.filter(func(f): return not f.render_hidden).map(func(f): return f.plane.normal.round())
		# Planes are in id space, map z is id x.
		t.check(shown == [Vector3(1, 0, 0), Vector3(-1, 0, 0)] or shown == [Vector3(-1, 0, 0), Vector3(1, 0, 0)], "a %s plate keeps only its faces over the tunnel, got %s" % [type, shown])
		t.check(brushes[2].faces.all(func(f): return f.render_hidden), "a %s box wholly inside the mesh is dropped" % type)
		t.check(brushes[0].faces.all(func(f): return not f.render_hidden), "the tunnel mesh keeps its faces")

## A closed mesh buried in another keeps the triangles it was built with, so a face made to block its ears is not
## clipped once more by the face cull, and again for every prefab instance of it.
static func _test_buried_faces_keep_their_triangles(t) -> void:
	print("- buried faces keep the triangles they were built with")
	var settings: FuncGodotMapSettings = load(SETTINGS)
	var container := _box(1, Vector3(-8192, -8192, -8192), Vector3(8192, 8192, 8192), "showcase/cobble")
	container["type"] = "mesh"
	var vertices := []
	for p in _thin_band(16384):
		vertices.append([p.x, p.y, p.z])
	var front := range(vertices.size())
	var back := front.duplicate()
	back.reverse()
	var sheet := { "id": 2, "type": "mesh", "vertices": vertices, "faces": [{ "indices": front, "material": "showcase/cobble" }, { "indices": back, "material": "showcase/cobble" }] }
	# With the build's budget spent the sheet is fanned right away, the face cull is all that could clip it.
	var spent := TrenchDotMesh.ClipBudget.new()
	spent.take(TrenchDotMesh.BUILD_CLIP_WORK)
	var entity := FuncGodotData.EntityData.new()
	entity.properties["classname"] = "worldspawn"
	entity.brushes.append(TrenchDotMesh.parse(container, Transform3D.IDENTITY, settings.scale_factor, ""))
	for k in 4:
		entity.brushes.append(TrenchDotMesh.parse(sheet, Transform3D(Basis.IDENTITY, Vector3(0, 64 * k, 0)), settings.scale_factor, "", spent))
	t.check(entity.brushes.all(func(b): return b.closed), "the container and the two sided sheets are closed meshes")
	var entities: Array[FuncGodotData.EntityData] = [entity]
	var started := Time.get_ticks_msec()
	var hidden := TrenchDotFaceCull.apply(entities, settings, { "showcase/cobble": StandardMaterial3D.new() })
	var took := Time.get_ticks_msec() - started
	var sheets := entity.brushes.slice(1)
	t.check(hidden == 8 and sheets.all(func(b): return b.faces.all(func(f): return f.render_hidden)), "every face of the four buried sheets is hidden, %d faces in all" % hidden)
	t.check(took < 2000, "without clipping their 16384 corners again, culling took %d ms" % took)

## A brush face has no triangles of its own, the face cull clips it against a budget that the whole cull shares, so
## brush faces made to block their ears and buried in a closed mesh cannot stall it either.
static func _test_buried_brush_faces_clip_from_the_cull_budget(t) -> void:
	print("- buried brush faces clip from the face cull's budget")
	var settings: FuncGodotMapSettings = load(SETTINGS)
	var container := _box(1, Vector3(-8192, -8192, -8192), Vector3(8192, 8192, 8192), "showcase/cobble")
	container["type"] = "mesh"
	var solid := TrenchDotFaceCull._solid(TrenchDotMesh.parse(container, Transform3D.IDENTITY, settings.scale_factor, ""), 1.0 / settings.scale_factor)
	var entry := TrenchDotFaceCull.Entry.new()
	entry.points = _thin_band(16384)
	entry.normal = TrenchDotMesh._newell(entry.points).normalized()
	entry.bounds = AABB(entry.points[0], Vector3.ZERO)
	for p in entry.points:
		entry.bounds = entry.bounds.expand(p)
	entry.brush = FuncGodotData.BrushData.new()
	entry.face = FuncGodotData.FaceData.new()
	var budget := TrenchDotMesh.ClipBudget.new()
	budget.take(TrenchDotMesh.BUILD_CLIP_WORK - 1000)
	var work := TrenchDotFaceCull.Work.new(TrenchDotFaceCull.INTERIOR_WORK)
	t.check(not TrenchDotFaceCull._crosses_surface(entry, solid, budget, work) and budget.spent(), "clipping a brush face takes from the cull's budget")
	var started := Time.get_ticks_msec()
	var crosses := TrenchDotFaceCull._crosses_surface(entry, solid, budget, work)
	var took := Time.get_ticks_msec() - started
	t.check(not crosses and took < 400, "once the budget is spent a 16384 corner band is fanned, tested in %d ms" % took)

## A cone of [param sides] sides like td_geom::mesh_shapes::cone: one tip vertex shared by every side.
static func _cone(sides: int) -> Dictionary:
	var vertices := []
	for s in sides:
		vertices.append([cos(TAU * s / sides) * 64.0, -64.0, sin(TAU * s / sides) * 64.0])
	vertices.append([0.0, 64.0, 0.0])
	var faces := [{ "indices": range(sides), "material": "showcase/cobble" }]
	for s in sides:
		faces.append({ "indices": [s, sides, (s + 1) % sides], "material": "showcase/cobble" })
	return { "id": 1, "type": "mesh", "vertices": vertices, "faces": faces, "smooth_angle": 89.0 }

## Smoothing compares every corner with each face round its vertex. Past 64 comparisons a corner and 262144 more a
## mesh is shaded flat, so the 650 and 651 sided cones fall either side of it, as in td_geom's
## smoothing_a_vertex_shared_by_thousands_of_faces_shades_flat.
static func _test_smoothing_a_vertex_of_thousands_of_faces_shades_flat(t) -> void:
	print("- smoothing a vertex of thousands of faces shades flat")
	var flat := func(brush: FuncGodotData.BrushData) -> bool:
		return brush.faces.all(func(f): return Array(f.disp_normals).all(func(n): return n.distance_to(f.plane.normal) < 1e-5))
	t.check(not flat.call(TrenchDotMesh.parse(_cone(650), Transform3D.IDENTITY, 1.0, "")), "a 650 sided cone is smoothed")
	t.check(flat.call(TrenchDotMesh.parse(_cone(651), Transform3D.IDENTITY, 1.0, "")), "a 651 sided cone compares one face too many and is shaded flat")
	var started := Time.get_ticks_msec()
	var brush := TrenchDotMesh.parse(_cone(8192), Transform3D.IDENTITY, 1.0, "")
	var took := Time.get_ticks_msec() - started
	t.check(flat.call(brush) and took < 3000, "an 8192 sided cone is not smoothed corner by corner, parsed in %d ms" % took)

## The 650 sided cone again with a face of collinear corners on its tip. The build drops that face, and counting it
## would take the cone past the budget, so it stays smoothed as in td_geom's
## a_face_of_collinear_corners_does_not_count_toward_smoothing.
static func _test_a_face_of_collinear_corners_does_not_count_toward_smoothing(t) -> void:
	print("- a face of collinear corners does not count toward smoothing")
	var node := _cone(650)
	node["vertices"].append_array([[0.0, 32.0, 0.0], [0.0, 0.0, 0.0]])
	node["faces"].append({ "indices": [650, 651, 652], "material": "showcase/cobble" })
	var brush := TrenchDotMesh.parse(node, Transform3D.IDENTITY, 1.0, "")
	var smoothed := brush.faces.any(func(f): return Array(f.disp_normals).any(func(n): return n.distance_to(f.plane.normal) > 1e-5))
	t.check(brush.faces.size() == 651 and smoothed, "the cone keeps its 651 faces and stays smoothed")

## Finding 8: an all zero splat weight must render as the first layer, matching Terrain::weights and
## docs/format/terrain-scatter.md, not black. Not practically capturable headlessly, so this pins the fixed GLSL
## expression in the shader source as a regression guard.
static func _test_terrain_splat_shader_defaults_to_first_layer(t) -> void:
	print("- terrain splat shader defaults to the first layer")
	var code: String = TrenchDotTerrain.SHADER.code
	t.check(code.contains("vec4(1.0, 0.0, 0.0, 0.0)") and code.contains("sum > 0.0001"), "the fragment shader still falls back to the first layer on an all zero splat")

## Auto Paint writes all four weight slots even on a terrain with fewer layers. The editor draws an empty slot
## with the first layer, so Godot must too, not with the last one (rock where the editor shows grass).
static func _test_terrain_missing_layer_slots_use_first_layer(t) -> void:
	print("- terrain slots without a layer use the first layer")
	var heights := PackedFloat32Array()
	heights.resize(4)
	var data := {
		"resolution": [2, 2], "cell_size": 1.0,
		"heights": Marshalls.raw_to_base64(heights.to_byte_array()),
		"layers": [{ "material": "showcase/grass", "tile": 100.0 }, { "material": "showcase/rock", "tile": 50.0 }],
	}
	var terrain := TrenchDotTerrain.create(data, Vector3.ZERO, load(SETTINGS))
	t.check(terrain != null, "terrain built")
	if terrain:
		var mat: ShaderMaterial = (terrain.get_child(0) as MeshInstance3D).mesh.surface_get_material(0)
		var first: Texture2D = mat.get_shader_parameter("layer0")
		t.check(first != null and mat.get_shader_parameter("layer2") == first and mat.get_shader_parameter("layer3") == first, "slots 2 and 3 show the first layer's texture")
		var tiles: Vector4 = mat.get_shader_parameter("tiles")
		t.check(is_equal_approx(tiles.z, tiles.x) and is_equal_approx(tiles.w, tiles.x), "and repeat at its tile size, got %s" % tiles)
		terrain.free()

## An emissive material painted as a terrain layer glows like it does on a brush, weighted by the layer, the
## same (color + texture) * energy the editor preview's terrain.wgsl adds.
static func _test_terrain_emissive_layers_glow(t) -> void:
	print("- terrain layers keep their material's emission")
	var heights := PackedFloat32Array()
	heights.resize(4)
	var data := {
		"resolution": [2, 2], "cell_size": 1.0,
		"heights": Marshalls.raw_to_base64(heights.to_byte_array()),
		"layers": [{ "material": "showcase/grass", "tile": 100.0 }, { "material": "night/neon_red", "tile": 50.0 }],
	}
	var terrain := TrenchDotTerrain.create(data, Vector3.ZERO, load(SETTINGS))
	t.check(terrain != null, "terrain built")
	if not terrain:
		return
	var mat: ShaderMaterial = (terrain.get_child(0) as MeshInstance3D).mesh.surface_get_material(0)
	var neon: StandardMaterial3D = load("res://demo/textures/night/neon_red.tres")
	var energy: Vector4 = mat.get_shader_parameter("glow_energy")
	var textured: Vector4 = mat.get_shader_parameter("glow_textured")
	t.check(is_equal_approx(energy.y, neon.emission_energy_multiplier) and textured.y == 1.0, "the neon layer glows with its energy and texture, got %s %s" % [energy, textured])
	t.check(mat.get_shader_parameter("glow_texture1") == neon.emission_texture, "the layer's emission texture is bound")
	t.check(energy.x == 0.0 and textured.x == 0.0, "the grass layer stays dark")
	t.check(energy.z == 0.0 and energy.w == 0.0, "slots without a layer of their own show the first, dark layer")
	var code: String = TrenchDotTerrain.SHADER.code
	t.check(code.contains("* glow_energy[l] * weight;"), "the shader weights each layer's glow by its splat weight")
	terrain.free()

## A layer shades with its material's normal map and roughness like the material does on a brush. Layers
## without a map keep the flags at 0 so the shader skips their samples.
static func _test_terrain_layers_use_normal_and_roughness_maps(t) -> void:
	print("- terrain layers use their material's normal and roughness maps")
	var heights := PackedFloat32Array()
	heights.resize(4)
	var data := {
		"resolution": [2, 2], "cell_size": 1.0,
		"heights": Marshalls.raw_to_base64(heights.to_byte_array()),
		"layers": [{ "material": "showcase/dirt", "tile": 100.0 }, { "material": "night/rusty_metal", "tile": 50.0 }, { "material": "special/clip", "tile": 50.0 }],
	}
	var terrain := TrenchDotTerrain.create(data, Vector3.ZERO, load(SETTINGS))
	t.check(terrain != null, "terrain built")
	if not terrain:
		return
	var mat: ShaderMaterial = (terrain.get_child(0) as MeshInstance3D).mesh.surface_get_material(0)
	var dirt: StandardMaterial3D = load("res://demo/textures/showcase/dirt.tres")
	var rusty: StandardMaterial3D = load("res://demo/textures/night/rusty_metal.tres")
	var mapped: Vector4 = mat.get_shader_parameter("normal_mapped")
	var scales: Vector4 = mat.get_shader_parameter("normal_scales")
	t.check(mat.get_shader_parameter("normal_texture0") == dirt.normal_texture and mat.get_shader_parameter("normal_texture1") == rusty.normal_texture, "each layer binds its material's normal map")
	t.check(mapped == Vector4(1, 1, 0, 1), "layers with a normal map are flagged, the one without is not, got %s" % mapped)
	t.check(is_equal_approx(scales.x, dirt.normal_scale) and is_equal_approx(scales.y, rusty.normal_scale), "and keep their normal scale, got %s" % scales)
	var roughness: Vector4 = mat.get_shader_parameter("roughnesses")
	var textured: Vector4 = mat.get_shader_parameter("roughness_textured")
	var channels: Projection = mat.get_shader_parameter("roughness_channels")
	t.check(is_equal_approx(roughness.x, dirt.roughness) and is_equal_approx(roughness.y, rusty.roughness), "each layer keeps its material's roughness value, got %s" % roughness)
	t.check(textured == Vector4(0, 1, 0, 0), "only the layer with a roughness texture samples one, got %s" % textured)
	t.check(mat.get_shader_parameter("surface_texture1") == rusty.roughness_texture and channels[1] == Vector4(1, 0, 0, 0), "and reads its red channel like BaseMaterial3D")
	t.check(mat.get_shader_parameter("ao_textured") == Vector4.ZERO, "no demo layer has ambient occlusion")
	var code: String = TrenchDotTerrain.SHADER.code
	t.check(code.count("uniform sampler2D ") == 16, "four textures per layer at most, so the shader stays within sampler limits")
	t.check(code.contains("if (weight <= 0.0)"), "layers the splat leaves out are not sampled")
	terrain.free()

## Occlusion shares the roughness texture's sample: an ORM texture, a StandardMaterial3D packing both in one
## texture, or occlusion alone. A separate occlusion texture next to a roughness texture is left out.
static func _test_terrain_layer_ambient_occlusion(t) -> void:
	print("- terrain layers take ambient occlusion from their roughness texture")
	var tex := ImageTexture.create_from_image(Image.create(2, 2, false, Image.FORMAT_RGBA8))
	var other := ImageTexture.create_from_image(Image.create(2, 2, false, Image.FORMAT_RGBA8))
	var orm := ORMMaterial3D.new()
	orm.orm_texture = tex
	orm.ao_enabled = true
	orm.ao_light_affect = 0.5
	var packed := StandardMaterial3D.new()
	packed.roughness_texture = tex
	packed.roughness_texture_channel = BaseMaterial3D.TEXTURE_CHANNEL_GREEN
	packed.ao_enabled = true
	packed.ao_texture = tex
	packed.ao_texture_channel = BaseMaterial3D.TEXTURE_CHANNEL_BLUE
	var ao_only := StandardMaterial3D.new()
	ao_only.ao_enabled = true
	ao_only.ao_texture = other
	var separate := StandardMaterial3D.new()
	separate.roughness_texture = tex
	separate.ao_enabled = true
	separate.ao_texture = other
	var mat := ShaderMaterial.new()
	mat.shader = TrenchDotTerrain.SHADER
	var surface := TrenchDotTerrain._default_surface()
	for slot in 4:
		TrenchDotTerrain._set_layer_surface(mat, slot, [orm, packed, ao_only, separate][slot], surface)
	var rough_channels: Projection = surface["roughness_channels"]
	var ao_channels: Projection = surface["ao_channels"]
	t.check(mat.get_shader_parameter("surface_texture0") == tex and rough_channels[0] == Vector4(0, 1, 0, 0) and ao_channels[0] == Vector4(1, 0, 0, 0), "an ORM texture gives roughness from green and occlusion from red")
	t.check(is_equal_approx(surface["ao_light_affects"].x, 0.5), "with the material's light affect")
	t.check(rough_channels[1] == Vector4(0, 1, 0, 0) and ao_channels[1] == Vector4(0, 0, 1, 0), "a packed texture keeps the channels the material picks")
	t.check(mat.get_shader_parameter("surface_texture2") == other and surface["roughness_textured"].z == 0.0, "occlusion alone takes the slot without touching roughness")
	t.check(surface["ao_textured"] == Vector4(1, 1, 1, 0) and surface["roughness_textured"] == Vector4(1, 1, 0, 1), "a separate occlusion texture is skipped, got %s" % surface["ao_textured"])

## Finding 9: height_at must follow the same triangles (and alternating diagonal) as the rendered surface,
## not a bilinear blend of the four corners.
static func _test_terrain_height_at_follows_triangles(t) -> void:
	print("- terrain height_at follows the triangles")
	var terrain := TrenchDotTerrain.new()
	terrain.resolution = Vector2i(2, 2)
	terrain.cell_size = 1.0
	# Only the (1,1) corner is raised. Cell (0,0) is even, so its diagonal runs (0,0)-(1,1): the center of the
	# cell sits exactly on that diagonal, at height 5 (its midpoint), not 2.5 (the bilinear blend of all 4 corners).
	terrain.heights = PackedFloat32Array([0.0, 0.0, 0.0, 10.0])
	t.check(t.near(terrain.height_at(0.5, 0.5), 5.0, 0.01), "the diagonal is followed, not a bilinear blend, got %s" % terrain.height_at(0.5, 0.5))
	t.check(is_nan(terrain.height_at(-1.0, 0.0)), "height_at still returns NAN outside the grid")
	terrain.free()

## Finding 10: an out of range vertex index in a hand edited brush must drop just that brush, not error out
## silently while leaving the rest of the map (parsed on a worker thread) intact.
static func _test_brush_bad_index_is_skipped(t) -> void:
	print("- a brush with a bad vertex index is skipped, not the whole map")
	var good := _box(2, Vector3(-16, -16, -16), Vector3(16, 16, 16), "showcase/cobble")
	var bad := { "id": 3, "type": "brush",
		"vertices": [[0.0, 0.0, 0.0], [16.0, 0.0, 0.0], [16.0, 16.0, 0.0], [0.0, 16.0, 0.0]],
		"faces": [{ "indices": [0, 1, 2, 99], "material": "showcase/cobble" }, { "indices": [0, 1, 2, 3], "material": "showcase/cobble" },
			{ "indices": [0, 1, 2, 3], "material": "showcase/cobble" }, { "indices": [0, 1, 2, 3], "material": "showcase/cobble" }] }
	var settings: FuncGodotMapSettings = load(SETTINGS)
	var map_json := { "format": "trenchdot-map", "properties": {}, "layers": [{ "type": "layer", "id": 1, "children": [good, bad] }] }
	var path := _write("td_bad_index_test.tdm", map_json)
	var data := FuncGodotParser.new().parse_map_data(path, settings)
	t.check(data != null, "the map still parses")
	if data:
		t.check(data.entities[0].brushes.size() == 1, "the malformed brush is dropped, the valid one is kept, got %d brushes" % data.entities[0].brushes.size())

## Colors and bools in map values read the way the editor reads them, and a light that starts off stays switchable.
static func _test_property_values_read_like_the_editor(t) -> void:
	print("- colors and bools read like the editor")
	t.check(TrenchDotIO.to_color("1.0 0.8 0.5").is_equal_approx(Color(1.0, 0.8, 0.5)), "unit float colors stay as they are")
	t.check(TrenchDotIO.to_color("255 128 0").is_equal_approx(Color8(255, 128, 0)), "integer colors are 0 to 255")
	t.check(TrenchDotIO.to_color("1 1 1").is_equal_approx(Color8(1, 1, 1)), "1 1 1 without a decimal point is 0 to 255 like in the editor")
	var map := {
		"format": "trenchdot-map", "version": 1, "properties": {},
		"layers": [{ "id": 1, "type": "layer", "name": "Default", "children": [
			{ "id": 2, "type": "entity", "classname": "light", "origin": [0.0, 0.0, 0.0], "angles": [0.0, 0.0, 0.0],
				"properties": { "light_color": "1.0 0.8 0.5", "start_on": "false", "shadows": "True" } },
		] }],
	}
	var data := FuncGodotParser.new().parse_tdm(map, load(SETTINGS), "props.tdm")
	var lights := data.entities.filter(func(e): return str(e.properties.get("classname", "")) == "light")
	t.check(lights.size() == 1, "the light parsed")
	if lights.size() == 1:
		var props: Dictionary = lights[0].properties
		t.check(props["light_color"] is Color and (props["light_color"] as Color).is_equal_approx(Color(1.0, 0.8, 0.5)), "a unit color property is not read as 0 to 255, got %s" % [props["light_color"]])
		t.check(props["start_on"] is bool and props["start_on"] == false, "start_on false is false")
		t.check(props["shadows"] is bool and props["shadows"] == true, "shadows True is true")
		t.check(TrenchDotLightmap.bake_mode(props, "bake_mode") == Light3D.BAKE_DISABLED, "a parsed start_on false keeps the light switchable")
	for value in [false, 0, "0"]:
		t.check(TrenchDotLightmap.bake_mode({ "start_on": value }, "bake_mode") == Light3D.BAKE_DISABLED, "start_on %s keeps the light out of the bake" % [value])
	t.check(TrenchDotLightmap.bake_mode({ "start_on": true }, "bake_mode") == Light3D.BAKE_STATIC, "a light that starts on is baked")

## Sun keys alone build the sun the editor previews, and ssr takes any bool spelling.
static func _test_environment_sun_keys(t) -> void:
	print("- sun keys alone build the sun")
	for key in ["sun_color", "sun_energy", "ambient_color", "sun_bake_mode"]:
		t.check(TrenchDotEnvironment.wants_environment({ key: "1" }), "%s alone builds the environment" % key)
	var holder := Node3D.new()
	var nodes := TrenchDotEnvironment.build(holder, { "sun_color": "255 0 0", "sun_energy": "2", "ssr": "true" })
	var suns := nodes.filter(func(n): return n is DirectionalLight3D)
	t.check(suns.size() == 1 and suns[0].light_color.is_equal_approx(Color.RED) and t.near(suns[0].light_energy, 2.0), "the sun takes sun_color and sun_energy")
	var envs := nodes.filter(func(n): return n is WorldEnvironment)
	t.check(envs.size() == 1 and envs[0].environment.ssr_enabled, "ssr true turns on screen space reflections")
	holder.free()

## A projected decal's size follows its map's units per meter, not the project default.
static func _test_decal_uses_the_map_scale(t) -> void:
	print("- decal size uses the map scale")
	var settings := (load(SETTINGS) as FuncGodotMapSettings).duplicate() as FuncGodotMapSettings
	settings.inverse_scale_factor = 64.0
	var map := FuncGodotMap.new()
	map.map_settings = settings
	var decal := TrenchDotDecal.new()
	decal._func_godot_apply_properties({ "size": Vector3(64, 32, 128) })
	map.add_child(decal)
	t.check(t.near(decal.size, Vector3(1, 0.5, 2)), "decal size in meters at 64 units per meter, got %s" % decal.size)
	map.free()

## Origin brushes match their texture in any case, and a zero UV scale counts as 1 like FaceUv::texel.
static func _test_origin_texture_and_zero_uv_scale(t) -> void:
	print("- origin texture case and zero uv scale")
	var settings: FuncGodotMapSettings = load(SETTINGS)
	var origin := TrenchDotParser._parse_brush(settings, Transform3D.IDENTITY, _box(1, Vector3(-8, -8, -8), Vector3(8, 8, 8), settings.origin_texture.to_upper()))
	t.check(origin != null and origin.origin, "an origin brush in upper case is still an origin brush")
	var box := _box(2, Vector3(-8, -8, -8), Vector3(8, 8, 8), "showcase/cobble")
	for f in box["faces"]:
		f["uv"] = { "u_axis": [1.0, 0.0, 0.0], "v_axis": [0.0, 0.0, 1.0], "offset": [0.0, 0.0], "scale": [0.0, 0.0] }
	var brush := TrenchDotParser._parse_brush(settings, Transform3D.IDENTITY, box)
	t.check(brush != null and t.near(brush.faces[0].uv.x.x, settings.scale_factor) and t.near(brush.faces[0].uv.y.y, settings.scale_factor), "a zero uv scale counts as 1")
	var uv := FuncGodotUtil.get_valve_uv(Vector3(4, 0, 2), Vector3.RIGHT, Vector3.BACK, Transform2D(Vector2.ZERO, Vector2.ZERO, Vector2.ZERO))
	t.check(uv.is_finite() and t.near(uv.x, 4.0) and t.near(uv.y, 2.0), "get_valve_uv treats a zero scale as 1, got %s" % uv)
