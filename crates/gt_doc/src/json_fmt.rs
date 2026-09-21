use serde_json::Value;

const INLINE_LIMIT: usize = 100;

/// Pretty JSON that keeps small values and every brush face on one line, so map diffs stay readable.
pub fn to_string(value: &Value) -> String {
    let mut out = String::new();
    write_value(&mut out, value, 0, false);
    out.push('\n');
    out
}

fn write_value(out: &mut String, value: &Value, indent: usize, force_inline: bool) {
    let compact = serde_json::to_string(value).unwrap_or_default();
    let is_container = matches!(value, Value::Array(_) | Value::Object(_));
    if !is_container || force_inline || compact.len() <= INLINE_LIMIT && !has_children(value) {
        out.push_str(&compact);
        return;
    }

    let pad = "  ".repeat(indent + 1);
    match value {
        Value::Array(items) => {
            out.push_str("[\n");
            for (i, item) in items.iter().enumerate() {
                out.push_str(&pad);
                write_value(out, item, indent + 1, false);
                if i + 1 < items.len() {
                    out.push(',');
                }

                out.push('\n');
            }

            out.push_str(&"  ".repeat(indent));
            out.push(']');
        }
        Value::Object(map) => {
            out.push_str("{\n");
            let len = map.len();
            for (i, (k, v)) in map.iter().enumerate() {
                out.push_str(&pad);
                out.push_str(&serde_json::to_string(k).unwrap_or_default());
                out.push_str(": ");
                if k == "faces" {
                    write_faces(out, v, indent + 1);
                } else {
                    write_value(out, v, indent + 1, k == "vertices" && compact_vertices(v));
                }

                if i + 1 < len {
                    out.push(',');
                }

                out.push('\n');
            }

            out.push_str(&"  ".repeat(indent));
            out.push('}');
        }
        _ => unreachable!(),
    }
}

fn write_faces(out: &mut String, value: &Value, indent: usize) {
    let Value::Array(items) = value else {
        write_value(out, value, indent, false);
        return;
    };
    let pad = "  ".repeat(indent + 1);
    out.push_str("[\n");
    for (i, item) in items.iter().enumerate() {
        out.push_str(&pad);
        write_value(out, item, indent + 1, true);
        if i + 1 < items.len() {
            out.push(',');
        }

        out.push('\n');
    }

    out.push_str(&"  ".repeat(indent));
    out.push(']');
}

fn compact_vertices(v: &Value) -> bool {
    matches!(v, Value::Array(a) if a.len() <= 64)
}

fn has_children(v: &Value) -> bool {
    matches!(v, Value::Object(m) if m.get("children").is_some_and(|c| matches!(c, Value::Array(a) if !a.is_empty())))
}
