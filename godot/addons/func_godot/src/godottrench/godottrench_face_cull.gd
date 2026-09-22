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

## A closed solid that can swallow another solid's faces. Convex brushes test against their own planes, closed
## meshes by ray parity. Everything is in map units, like [member Entry.points].
class Solid:
	var brush: FuncGodotData.BrushData
	var bounds: AABB
	var is_mesh: bool
	var normals: PackedVector3Array
	var dists: PackedFloat32Array
	var tris: PackedVector3Array

	func contains(p: Vector3) -> bool:
		if not bounds.grow(COPLANAR_DIST).has_point(p):
			return false
		if not is_mesh:
			for i in normals.size():
				if normals[i].dot(p) - dists[i] > 0.0:
					return false
			return true
		# A ray leaving a point inside a closed surface crosses it an odd number of times. The direction is
		# irregular so it rarely grazes an edge, and hits at the same distance (a shared edge) count once.
		var dir := Vector3(0.5773, 0.5574, 0.5964).normalized()
		var hits := PackedFloat32Array()
		var i := 0
		while i < tris.size():
			var at = Geometry3D.ray_intersects_triangle(p, dir, tris[i], tris[i + 1], tris[i + 2])
			if at != null:
				hits.append(p.distance_to(at))
			i += 3
		hits.sort()
		var crossings := 0
		for k in hits.size():
			if k == 0 or absf(hits[k] - hits[k - 1]) > 1e-5:
				crossings += 1
		return crossings % 2 == 1

## Segment a-b passing through the inside of triangle t0-t1-t2. Touching an edge or ending on the plane does not
## count. Mirrors segment_hits_triangle in crates/gt_editor/src/face_cull.rs.
static func _segment_hits_triangle(a: Vector3, b: Vector3, t0: Vector3, t1: Vector3, t2: Vector3) -> bool:
	const EPS := 1e-6
	var dir := b - a
	var e1 := t1 - t0
	var e2 := t2 - t0
	var p := dir.cross(e2)
	var det := e1.dot(p)
	if absf(det) < 1e-12:
		return false
	var inv := 1.0 / det
	var s := a - t0
	var u := s.dot(p) * inv
	var q := s.cross(e1)
	var v := dir.dot(q) * inv
	var tt := e2.dot(q) * inv
	return u > EPS and v > EPS and u + v < 1.0 - EPS and tt > EPS and tt < 1.0 - EPS

## Whether [param solid]'s own surface triangles cut through [param entry]'s polygon anywhere, meaning part of
## it lies outside a concave mesh even though every sampled corner tested inside. Mirrors crosses_surface.
static func _crosses_surface(entry: Entry, solid: Solid) -> bool:
	var poly := entry.points
	var face_tris := GodotTrenchMesh.triangulate(poly, entry.normal)
	var reach := entry.bounds.grow(COPLANAR_DIST)
	var tris := solid.tris
	var i := 0
	while i < tris.size():
		var a: Vector3 = tris[i]
		var b: Vector3 = tris[i + 1]
		var c: Vector3 = tris[i + 2]
		var tri_bounds := AABB(a, Vector3.ZERO).expand(b).expand(c)
		if tri_bounds.intersects(reach):
			for k in poly.size():
				if _segment_hits_triangle(poly[k], poly[(k + 1) % poly.size()], a, b, c):
					return true
			for k in 3:
				var sa: Vector3 = tris[i + k]
				var sb: Vector3 = tris[i + (k + 1) % 3]
				for t in range(0, face_tris.size(), 3):
					if _segment_hits_triangle(sa, sb, poly[face_tris[t]], poly[face_tris[t + 1]], poly[face_tris[t + 2]]):
						return true
		i += 3
	return false

## True when [param face] has solid material on both of its sides inside [param solid]. Probing to both sides
## keeps a face resting on the solid's surface out of it, that one belongs to the coplanar pass. Brushes are
## convex, so their corners decide it. A closed mesh can be concave, there the face must also not cross its
## surface, so a face spanning an opening of the mesh is not wrongly counted as buried.
static func _buried_in(solid: Solid, entry: Entry) -> bool:
	var nudge := entry.normal * COPLANAR_DIST
	var centroid := Vector3.ZERO
	for p in entry.points:
		centroid += p
	centroid /= float(entry.points.size())
	var samples := entry.points.duplicate()
	samples.append(centroid)
	for p in samples:
		if not solid.contains(p + nudge) or not solid.contains(p - nudge):
			return false
	if solid.is_mesh and _crosses_surface(entry, solid):
		return false
	return true

static func _solid(brush: FuncGodotData.BrushData, inv_scale: float) -> Solid:
	if not brush.closed:
		return null
	var solid := Solid.new()
	solid.brush = brush
	solid.is_mesh = brush.is_mesh
	var first := true
	for fi in brush.faces.size():
		var face := brush.faces[fi]
		var source := face.disp_vertices if brush.is_mesh else face.vertices
		if source.size() < 3:
			continue
		var points := PackedVector3Array()
		var centroid := Vector3.ZERO
		for p in source:
			var q: Vector3 = p * inv_scale
			points.append(q)
			centroid += q
			solid.bounds = AABB(q, Vector3.ZERO) if first else solid.bounds.expand(q)
			first = false
		centroid /= float(points.size())
		if solid.is_mesh:
			# Faces can be concave, so use the face's own triangulation (disp_indices) instead of a fan.
			var tris: PackedInt32Array = face.disp_indices
			for t in range(0, tris.size() - 2, 3):
				var ia := tris[t]
				var ib := tris[t + 1]
				var ic := tris[t + 2]
				if ia < points.size() and ib < points.size() and ic < points.size():
					solid.tris.append_array([points[ia], points[ib], points[ic]])
		else:
			solid.normals.append(face.plane.normal)
			solid.dists.append(face.plane.normal.dot(centroid))
	if first or (solid.is_mesh and solid.tris.is_empty()) or (not solid.is_mesh and solid.normals.is_empty()):
		return null
	return solid

## Marks hidden faces, returns how many.
static func apply(entities: Array[FuncGodotData.EntityData], settings: FuncGodotMapSettings, materials: Dictionary) -> int:
	var inv_scale := 1.0 / maxf(settings.scale_factor, 1e-9)
	var planes: Dictionary = {}
	var entries: Array[Entry] = []
	var solids: Array[Solid] = []
	for entity in entities:
		if not _static_entity(entity):
			continue
		for brush in entity.brushes:
			if brush.origin or (brush.has_disp and not brush.is_mesh):
				continue
			# A container only hides what is inside it when it is drawn opaque on every one of its faces: a box
			# with any tool textured face (special/clip/trigger/skip/nodraw) or any see-through one (like water)
			# shows what is inside it there, so it must not bury detail placed inside it either.
			var every_face_solid := not brush.faces.is_empty()
			for f in brush.faces:
				if FuncGodotUtil.filter_face(f.texture, settings) or not _opaque(materials.get(f.texture)):
					every_face_solid = false
					break
			if every_face_solid:
				var solid := _solid(brush, inv_scale)
				if solid:
					solids.append(solid)
			for fi in brush.faces.size():
				var face := brush.faces[fi]
				if FuncGodotUtil.filter_face(face.texture, settings) or not _opaque(materials.get(face.texture)):
					continue
				var entry := _entry(face, brush, fi, inv_scale)
				if entry:
					entries.append(entry)
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

	# Interior faces: a face buried inside another closed solid never shows, however the two intersect, so a
	# pile of overlapping solids draws as one outer shell instead of every solid's whole surface.
	for entry: Entry in entries:
		if entry.face.render_hidden or not entry.closed:
			continue
		for solid: Solid in solids:
			if solid.brush == entry.brush or not solid.bounds.encloses(entry.bounds):
				continue
			if _buried_in(solid, entry):
				entry.face.render_hidden = true
				hidden += 1
				break
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
