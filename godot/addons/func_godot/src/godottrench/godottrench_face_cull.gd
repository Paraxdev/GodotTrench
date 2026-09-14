class_name GodotTrenchFaceCull extends RefCounted
## Leaves faces out of the visual mesh when a coplanar face of another solid fully covers them, like the GodotTrench
## editor does. Overlapping faces that face the same way z-fight, back to back faces between closed solids are inside
## the shape. Collision keeps every face.
##
## Priority for faces facing the same way: open meshes (sheets such as blend layers laid on a floor), then brushes,
## then closed meshes, older map nodes first. Only whole faces are hidden, a partly covered face still draws.

const COPLANAR_DIST := 0.02
const SAME_NORMAL := 0.9995
const MIN_AREA := 0.01
const MOVING_CLASSES := ["Area3D", "AnimatableBody3D", "RigidBody3D", "CharacterBody3D", "VehicleBody3D"]

class Entry:
	var face: FuncGodotData.FaceData
	var normal: Vector3
	var dist: float
	var bounds: AABB
	## (rank, node id, face index), lower wins.
	var priority: Vector3i
	var closed: bool
	var brush: FuncGodotData.BrushData
	## Corners in map units.
	var points: PackedVector3Array

## Marks hidden faces, returns how many.
static func apply(entities: Array[FuncGodotData.EntityData], settings: FuncGodotMapSettings, materials: Dictionary) -> int:
	var inv_scale := 1.0 / maxf(settings.scale_factor, 1e-9)
	var planes: Dictionary = {}
	for entity in entities:
		if not _static_entity(entity):
			continue
		for brush in entity.brushes:
			if brush.origin or (brush.has_disp and not brush.is_mesh):
				continue
			for fi in brush.faces.size():
				var face := brush.faces[fi]
				if FuncGodotUtil.filter_face(face.texture, settings) or not _opaque(materials.get(face.texture)):
					continue
				var entry := _entry(face, brush, fi, inv_scale)
				if entry:
					var key := _plane_key(entry.normal, entry.dist)
					if not planes.has(key):
						planes[key] = []
					planes[key].append(entry)

	var hidden := 0
	for members: Array in planes.values():
		if members.size() < 2:
			continue
		for entry: Entry in members:
			var covers: Array[PackedVector2Array] = []
			var basis := _plane_basis(entry.normal)
			for other: Entry in members:
				if other.brush == entry.brush or not other.bounds.grow(COPLANAR_DIST).intersects(entry.bounds.grow(COPLANAR_DIST)):
					continue
				var dot := other.normal.dot(entry.normal)
				var overlap := dot > SAME_NORMAL and absf(other.dist - entry.dist) < COPLANAR_DIST and _before(other.priority, entry.priority)
				var backing := dot < -SAME_NORMAL and absf(other.dist + entry.dist) < COPLANAR_DIST and other.closed and entry.closed
				if overlap or backing:
					covers.append(_flatten(other.points, basis))
			if not covers.is_empty() and fully_covered(_flatten(entry.points, basis), covers):
				entry.face.render_hidden = true
				hidden += 1
	return hidden

## True when [param covers] leave nothing of [param polygon] larger than a sliver.
static func fully_covered(polygon: PackedVector2Array, covers: Array[PackedVector2Array]) -> bool:
	var remaining: Array[PackedVector2Array] = [_counter_clockwise(polygon)]
	var pending: Array[PackedVector2Array] = []
	for cover in covers:
		pending.append(_counter_clockwise(cover))
	# Covers that would punch a hole wait until the covers around them have reached the outline, Geometry2D returns
	# holes as separate clockwise polygons that cannot be clipped further.
	var progress := true
	while progress and not pending.is_empty() and not remaining.is_empty():
		progress = false
		for cover in pending.duplicate():
			var next: Array[PackedVector2Array] = []
			var makes_hole := false
			for piece in remaining:
				for p in Geometry2D.clip_polygons(piece, cover):
					if Geometry2D.is_polygon_clockwise(p):
						makes_hole = true
					elif absf(_area(p)) >= MIN_AREA:
						next.append(p)
			if makes_hole:
				continue
			remaining = next
			pending.erase(cover)
			progress = true
			if remaining.is_empty():
				break
	return remaining.is_empty()

static func _static_entity(entity: FuncGodotData.EntityData) -> bool:
	var classname := str(entity.properties.get("classname", ""))
	if classname.begins_with("trigger"):
		return false
	var def := entity.definition as FuncGodotFGDSolidClass
	if def and (not def.build_visuals or def.node_class in MOVING_CLASSES):
		return false
	return true

static func _opaque(material: Variant) -> bool:
	if material is BaseMaterial3D:
		var m := material as BaseMaterial3D
		return m.transparency == BaseMaterial3D.TRANSPARENCY_DISABLED and m.cull_mode == BaseMaterial3D.CULL_BACK
	if material is ShaderMaterial:
		var shader := (material as ShaderMaterial).shader
		return shader != null and not shader.code.contains("ALPHA") and not shader.code.contains("cull_disabled")
	return false

static func _entry(face: FuncGodotData.FaceData, brush: FuncGodotData.BrushData, index: int, inv_scale: float) -> Entry:
	var source := face.disp_vertices if brush.is_mesh else face.vertices
	if source.size() < 3:
		return null
	if face.plane.normal.length_squared() < 0.5:
		return null
	var entry := Entry.new()
	var centroid := Vector3.ZERO
	for p in source:
		entry.points.append(p * inv_scale)
		centroid += p * inv_scale
	entry.normal = face.plane.normal
	entry.dist = entry.normal.dot(centroid / entry.points.size())
	entry.bounds = AABB(entry.points[0], Vector3.ZERO)
	for p in entry.points:
		entry.bounds = entry.bounds.expand(p)
	var rank := 1 if not brush.is_mesh else (0 if not brush.closed else 2)
	entry.priority = Vector3i(rank, brush.node_id, index)
	entry.closed = brush.closed
	entry.face = face
	entry.brush = brush
	return entry

## Opposite normals share a key so back to back faces land in one group.
static func _plane_key(normal: Vector3, dist: float) -> Vector4i:
	var flip := false
	for c in [normal.x, normal.y, normal.z]:
		if absf(c) > 1e-6:
			flip = c < 0.0
			break
	var n := -normal if flip else normal
	var d := -dist if flip else dist
	return Vector4i(roundi(n.x * 1000.0), roundi(n.y * 1000.0), roundi(n.z * 1000.0), roundi(d * 8.0))

static func _before(a: Vector3i, b: Vector3i) -> bool:
	if a.x != b.x:
		return a.x < b.x
	if a.y != b.y:
		return a.y < b.y
	return a.z < b.z

static func _plane_basis(normal: Vector3) -> Array[Vector3]:
	var n := normal.abs()
	var helper := Vector3.UP if n.y < 0.9 else Vector3.RIGHT
	var u := helper.cross(normal).normalized()
	return [u, normal.cross(u)]

static func _flatten(points: PackedVector3Array, basis: Array[Vector3]) -> PackedVector2Array:
	var out := PackedVector2Array()
	for p in points:
		out.append(Vector2(p.dot(basis[0]), p.dot(basis[1])))
	return out

static func _counter_clockwise(polygon: PackedVector2Array) -> PackedVector2Array:
	if not Geometry2D.is_polygon_clockwise(polygon):
		return polygon
	var flipped := polygon.duplicate()
	flipped.reverse()
	return flipped

static func _area(polygon: PackedVector2Array) -> float:
	var a := 0.0
	for i in polygon.size():
		a += polygon[i].cross(polygon[(i + 1) % polygon.size()])
	return a * 0.5
