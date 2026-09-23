extends RefCounted
## Regression tests for the Godot-vs-editor parity audit: unique instance ids, mesh blend option passthrough,
## quad triangulation matching gt_geom::polygon::triangulate, terrain collision holes and the alternating
## diagonal, rotated prefab terrain placement, concave mesh containment, tool/transparent containers not
## burying faces, terrain layers keeping their material's maps, terrain height_at following the triangles, and out
## of range brush indices being skipped.
##
## res://tests/run_tests.gd calls [method run] from its own [method SceneTree._initialize]. [param t] is that
## script's own instance, used for its [code]check[/code]/[code]near[/code]/[code]find_named[/code] helpers.

const SETTINGS := "res://demo/demo_map_settings.tres"

static func run(t) -> void:
	_test_instance_id_namespacing(t)
	_test_mesh_blend_options(t)
	_test_quad_triangulation_matches_editor(t)
	_test_terrain_collision_holes_and_diagonal(t)
	_test_rotated_instance_terrain_position(t)
	_test_concave_mesh_containment(t)
	_test_tool_and_transparent_containers_do_not_bury(t)
	_test_terrain_splat_shader_defaults_to_first_layer(t)
	_test_terrain_missing_layer_slots_use_first_layer(t)
	_test_terrain_emissive_layers_glow(t)
	_test_terrain_layers_use_normal_and_roughness_maps(t)
	_test_terrain_layer_ambient_occlusion(t)
	_test_terrain_height_at_follows_triangles(t)
	_test_brush_bad_index_is_skipped(t)
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
		"format": "godottrench-map", "version": 1, "properties": {},
		"layers": [{ "id": 1, "type": "layer", "name": "Default", "children": [
			{ "id": 2, "type": "group", "name": "Inner", "children": [
				{ "id": 3, "type": "entity", "classname": "light", "origin": [0.0, 0.0, 0.0], "angles": [0.0, 0.0, 0.0], "properties": { "targetname": "plight" } },
				_box(4, Vector3(-8, -8, -8), Vector3(8, 8, 8), "showcase/cobble"),
			] },
		] }],
	}
	_write("gt_ns_prefab.gtm", prefab)
	var main_map := {
		"format": "godottrench-map", "version": 1, "properties": {},
		"layers": [{ "id": 1, "type": "layer", "name": "Default", "children": [
			{ "id": 10, "type": "instance", "path": "gt_ns_prefab.gtm", "origin": [0.0, 0.0, 0.0], "angles": [0.0, 0.0, 0.0], "fixup": "a" },
			{ "id": 11, "type": "instance", "path": "gt_ns_prefab.gtm", "origin": [256.0, 0.0, 0.0], "angles": [0.0, 0.0, 0.0], "fixup": "b" },
		] }],
	}
	var main_path := _write("gt_ns_main.gtm", main_map)
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
	var brush := GodotTrenchMesh.parse(node, Transform3D.IDENTITY, 1.0, "")
	t.check(brush != null and brush.faces.size() == 1, "mesh face parsed")
	if brush and brush.faces.size() == 1:
		var expected := GodotTrenchBlend.key("base/floor", "base/grass", 0.4, 2.5, 0.75)
		t.check(brush.faces[0].texture == expected, "mesh blend options ride along with the texture name, got %s want %s" % [brush.faces[0].texture, expected])
		var opts := GodotTrenchBlend.options(brush.faces[0].texture)
		t.check(t.near(opts[0], 0.4) and t.near(opts[1], 2.5) and t.near(opts[2], 0.75), "the options round trip, got %s" % [opts])

## Finding 3: a convex quad splits along the shorter diagonal, exactly like gt_geom::polygon::triangulate.
static func _test_quad_triangulation_matches_editor(t) -> void:
	print("- quad triangulation matches the editor")
	# Diagonal 0-2 is shorter.
	var a := PackedVector3Array([Vector3(0, 0, 0), Vector3(4, 0, 0), Vector3(3, 0, 2), Vector3(0, 0, 2)])
	var tris_a := GodotTrenchMesh.triangulate(a, GodotTrenchMesh._newell(a))
	t.check(tris_a == PackedInt32Array([0, 1, 2, 0, 2, 3]), "splits along the shorter 0-2 diagonal, got %s" % [tris_a])
	# Diagonal 1-3 is shorter.
	var b := PackedVector3Array([Vector3(0, 0, 0), Vector3(4, 0, 0), Vector3(4, 0, 2), Vector3(1, 0, 2)])
	var tris_b := GodotTrenchMesh.triangulate(b, GodotTrenchMesh._newell(b))
	t.check(tris_b == PackedInt32Array([0, 1, 3, 1, 2, 3]), "splits along the shorter 1-3 diagonal, got %s" % [tris_b])

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
	var terrain := GodotTrenchTerrain.create(data, Vector3.ZERO, settings)
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
	var terrain := GodotTrenchTerrain.create(data, xform, settings)
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
	var brush := GodotTrenchMesh.parse(node, Transform3D.IDENTITY, 1.0, "")
	t.check(brush != null and brush.closed, "the dart prism is a closed mesh")
	if brush and brush.closed:
		var solid := GodotTrenchFaceCull._solid(brush, 1.0)
		t.check(solid != null, "a closed mesh builds a containment solid")
		if solid:
			# GodotTrenchMesh stores vertices rotated into FuncGodot's id space (id = (z, x, y)), so probe points
			# go through the same GodotTrenchParser.to_id conversion as the mesh's own vertices did.
			t.check(solid.contains(GodotTrenchParser.to_id(Vector3(2.0, 1.0, 1.0))), "a point genuinely inside the dart is inside")
			t.check(not solid.contains(GodotTrenchParser.to_id(Vector3(1.8333, 1.0, 1.5))), "a point in the notch, which only a fan triangulation would wrongly cover, is outside")

## Finding 7: only opaque, non-tool containers may bury the faces of what they enclose.
static func _test_tool_and_transparent_containers_do_not_bury(t) -> void:
	print("- tool and transparent containers do not bury detail")
	var settings: FuncGodotMapSettings = load(SETTINGS)
	for case in [["showcase/cobble", true], ["special/clip", false], ["showcase/water", false]]:
		var outer_material: String = case[0]
		var should_bury: bool = case[1]
		var children := [_box(2, Vector3(-256, -256, -256), Vector3(256, 256, 256), outer_material), _box(3, Vector3(-32, -32, -32), Vector3(32, 32, 32), "showcase/cobble")]
		var map_json := { "format": "godottrench-map", "properties": {}, "layers": [{ "type": "layer", "id": 1, "children": children }] }
		var path := _write("gt_container_rule_test.gtm", map_json)
		var data := FuncGodotParser.new().parse_map_data(path, settings)
		FuncGodotGeometryGenerator.new(settings).build(0, data.entities)
		var inner: FuncGodotData.BrushData = data.entities[0].brushes[1]
		if should_bury:
			t.check(inner.faces.all(func(f): return f.render_hidden), "an opaque non-tool container (%s) still buries what is inside it" % outer_material)
		else:
			t.check(inner.faces.all(func(f): return not f.render_hidden), "a %s container must not bury what is inside it" % outer_material)

## Finding 8: an all zero splat weight must render as the first layer, matching Terrain::weights and
## docs/format/terrain-scatter.md, not black. Not practically capturable headlessly, so this pins the fixed GLSL
## expression in the shader source as a regression guard.
static func _test_terrain_splat_shader_defaults_to_first_layer(t) -> void:
	print("- terrain splat shader defaults to the first layer")
	var code: String = GodotTrenchTerrain.SHADER.code
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
	var terrain := GodotTrenchTerrain.create(data, Vector3.ZERO, load(SETTINGS))
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
	var terrain := GodotTrenchTerrain.create(data, Vector3.ZERO, load(SETTINGS))
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
	var code: String = GodotTrenchTerrain.SHADER.code
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
	var terrain := GodotTrenchTerrain.create(data, Vector3.ZERO, load(SETTINGS))
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
	var code: String = GodotTrenchTerrain.SHADER.code
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
	mat.shader = GodotTrenchTerrain.SHADER
	var surface := GodotTrenchTerrain._default_surface()
	for slot in 4:
		GodotTrenchTerrain._set_layer_surface(mat, slot, [orm, packed, ao_only, separate][slot], surface)
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
	var terrain := GodotTrenchTerrain.new()
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
	var map_json := { "format": "godottrench-map", "properties": {}, "layers": [{ "type": "layer", "id": 1, "children": [good, bad] }] }
	var path := _write("gt_bad_index_test.gtm", map_json)
	var data := FuncGodotParser.new().parse_map_data(path, settings)
	t.check(data != null, "the map still parses")
	if data:
		t.check(data.entities[0].brushes.size() == 1, "the malformed brush is dropped, the valid one is kept, got %d brushes" % data.entities[0].brushes.size())
