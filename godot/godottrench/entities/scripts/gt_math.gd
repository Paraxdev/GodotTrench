@tool
class_name GTMath extends RefCounted
## Number handling shared by the math and logic entities. Map parameters arrive as numbers, text or nothing.

## [param v] as a number, an int when it is whole. Text such as "2.5" counts, anything else gives [param fallback].
static func to_number(v: Variant, fallback: Variant = null) -> Variant:
	if v is bool:
		return int(v)
	if v is int:
		return v
	if v is float:
		var n: Variant = tidy(v)
		return fallback if n == null else n
	if v is String or v is StringName:
		var text := str(v).strip_edges()
		if text.is_valid_float():
			var n: Variant = tidy(text.to_float())
			return fallback if n == null else n
	return fallback

## [param x] as an int when it is whole and small enough to keep exactly, else as it is. Null for nan and infinity, which
## a division by zero or a root of a negative number produces.
static func tidy(x: float) -> Variant:
	if is_nan(x) or is_inf(x):
		return null
	if x == floorf(x) and absf(x) < 1.0e15:
		return int(x)
	return x

## The value a math entity stores: a number, else the text, or null for nothing and for a node.
static func normalize(v: Variant) -> Variant:
	if v == null or v is Object:
		return null
	return to_number(v, str(v))

## -1, 0 or 1 for [param a] before, equal to or after [param b]. Two numbers compare by value, anything else by its text.
static func compare(a: Variant, b: Variant) -> int:
	var x: Variant = to_number(a)
	var y: Variant = to_number(b)
	if x != null and y != null:
		if x is int and y is int:
			return signi(x - y)
		var fx := float(x)
		var fy := float(y)
		if absf(fx - fy) <= 1.0e-9 * maxf(1.0, maxf(absf(fx), absf(fy))):
			return 0
		return -1 if fx < fy else 1
	var s := "" if a == null else str(a)
	var t := "" if b == null else str(b)
	if s == t:
		return 0
	return -1 if s < t else 1
