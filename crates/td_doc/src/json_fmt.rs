use std::io::{self, Write};

use serde_json::Value;

const INLINE_LIMIT: usize = 100;
/// Levels past this are indented no further, so values nested a hundred deep cannot make the text many times the size
/// of the map. Maps the editor writes stay well within it.
const MAX_INDENT: usize = 32;

fn indentation(level: usize) -> String {
    "  ".repeat(level.min(MAX_INDENT))
}

/// Pretty JSON that keeps small values and every brush face on one line, so map diffs stay readable.
pub fn to_string(value: &Value) -> String {
    let mut out = Vec::new();
    write(&mut out, value).expect("writing to memory does not fail");
    String::from_utf8(out).expect("serde_json writes UTF-8")
}

/// [`to_string`] written to `out` as it is made.
pub fn write(out: &mut impl Write, value: &Value) -> io::Result<()> {
    write_value(out, value, 0, false)?;
    out.write_all(b"\n")
}

fn write_value(out: &mut impl Write, value: &Value, indent: usize, force_inline: bool) -> io::Result<()> {
    let is_container = matches!(value, Value::Array(_) | Value::Object(_));
    if !is_container || force_inline || !has_children(value) && fits(value, INLINE_LIMIT) {
        return compact(out, value);
    }

    let pad = indentation(indent + 1);
    match value {
        Value::Array(items) => {
            out.write_all(b"[\n")?;
            for (i, item) in items.iter().enumerate() {
                out.write_all(pad.as_bytes())?;
                write_value(out, item, indent + 1, false)?;
                if i + 1 < items.len() {
                    out.write_all(b",")?;
                }

                out.write_all(b"\n")?;
            }

            out.write_all(indentation(indent).as_bytes())?;
            out.write_all(b"]")
        }
        Value::Object(map) => {
            out.write_all(b"{\n")?;
            let len = map.len();
            for (i, (k, v)) in map.iter().enumerate() {
                out.write_all(pad.as_bytes())?;
                serde_json::to_writer(&mut *out, k)?;
                out.write_all(b": ")?;
                if k == "faces" {
                    write_faces(out, v, indent + 1)?;
                } else {
                    write_value(out, v, indent + 1, k == "vertices" && compact_vertices(v))?;
                }

                if i + 1 < len {
                    out.write_all(b",")?;
                }

                out.write_all(b"\n")?;
            }

            out.write_all(indentation(indent).as_bytes())?;
            out.write_all(b"}")
        }
        _ => unreachable!(),
    }
}

fn write_faces(out: &mut impl Write, value: &Value, indent: usize) -> io::Result<()> {
    let Value::Array(items) = value else {
        return write_value(out, value, indent, false);
    };
    let pad = indentation(indent + 1);
    out.write_all(b"[\n")?;
    for (i, item) in items.iter().enumerate() {
        out.write_all(pad.as_bytes())?;
        write_value(out, item, indent + 1, true)?;
        if i + 1 < items.len() {
            out.write_all(b",")?;
        }

        out.write_all(b"\n")?;
    }

    out.write_all(indentation(indent).as_bytes())?;
    out.write_all(b"]")
}

fn compact(out: &mut impl Write, value: &Value) -> io::Result<()> {
    Ok(serde_json::to_writer(out, value)?)
}

/// Whether `value` is at most `limit` bytes of compact JSON, found out without writing more than that.
fn fits(value: &Value, limit: usize) -> bool {
    struct Budget(usize);
    impl Write for Budget {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0 = self.0.checked_sub(buf.len()).ok_or(io::ErrorKind::WriteZero)?;
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    serde_json::to_writer(Budget(limit), value).is_ok()
}

fn compact_vertices(v: &Value) -> bool {
    matches!(v, Value::Array(a) if a.len() <= 64)
}

fn has_children(v: &Value) -> bool {
    matches!(v, Value::Object(m) if m.get("children").is_some_and(|c| matches!(c, Value::Array(a) if !a.is_empty())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn small_values_and_faces_stay_on_one_line() {
        let long: Vec<i32> = (0..40).collect();
        let v = json!({
            "id": 1, "name": "a \"quoted\" name", "short": [1, 2, 3], "long": long,
            "faces": [{ "indices": [0, 1, 2], "texture": "base/wall", "u": [1.0, 0.0, 0.0, 0.0], "v": [0.0, 0.0, 1.0, 0.0] }],
            "vertices": [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            "children": [{ "id": 2, "type": "group" }],
        });
        let numbers = long.iter().map(|n| format!("    {n}")).collect::<Vec<_>>().join(",\n");
        let expected = format!(
            "{{\n  \"id\": 1,\n  \"name\": \"a \\\"quoted\\\" name\",\n  \"short\": [1,2,3],\n  \"long\": [\n{numbers}\n  ],\n  \"faces\": [\n    \
             {{\"indices\":[0,1,2],\"texture\":\"base/wall\",\"u\":[1.0,0.0,0.0,0.0],\"v\":[0.0,0.0,1.0,0.0]}}\n  ],\n  \
             \"vertices\": [[0.0,0.0,0.0],[1.0,0.0,0.0]],\n  \"children\": [{{\"id\":2,\"type\":\"group\"}}]\n}}\n"
        );
        assert_eq!(to_string(&v), expected);
    }

    #[test]
    fn a_value_fits_up_to_the_limit() {
        let exactly = json!(["x".repeat(INLINE_LIMIT - 4)]);
        assert_eq!(serde_json::to_string(&exactly).unwrap().len(), INLINE_LIMIT);
        assert!(fits(&exactly, INLINE_LIMIT));
        assert!(!fits(&json!(["x".repeat(INLINE_LIMIT - 3)]), INLINE_LIMIT));
        assert!(!fits(&json!(["x".repeat(1 << 20)]), INLINE_LIMIT));
    }

    #[test]
    fn indentation_stops_growing_past_its_limit() {
        let mut v = json!(["x".repeat(INLINE_LIMIT)]);
        for _ in 0..MAX_INDENT + 8 {
            v = json!([v, "y".repeat(INLINE_LIMIT)]);
        }

        let text = to_string(&v);
        let widest = text.lines().map(|l| l.len() - l.trim_start().len()).max();
        assert_eq!(widest, Some(2 * MAX_INDENT));
        assert_eq!(serde_json::from_str::<Value>(&text).unwrap(), v);
    }

    #[test]
    fn writing_gives_the_same_text() {
        let v = json!({ "layers": [{ "id": 1, "children": [{ "id": 2, "faces": [{ "a": 1 }], "x": "y".repeat(300) }] }] });
        let mut out = Vec::new();
        write(&mut out, &v).unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), to_string(&v));
    }
}
