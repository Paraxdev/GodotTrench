//! MCP scripts: JSON lists of tool calls replayed against the editor, with results saved under names and referenced by
//! later steps. `"$door.id"` is replaced by that JSON value, `"${door.id}"` inside a longer string by its text, and `$$` is a
//! literal `$`. `"$a.ids + $b.ids"` joins values into one array. When a step replaces brushes (CSG, clip) the ids saved
//! by earlier steps are rewritten to their replacements, see [`follow_replacements`].

use std::collections::BTreeMap;

use serde_json::Value;

pub const FORMAT: &str = "godottrench-mcp-script";

#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub tool: String,
    pub args: Value,
    /// Name the result is stored under.
    pub save: Option<String>,
}

/// Steps of a script document: `{"format": ..., "steps": [...]}` or a bare array of steps.
pub fn parse(doc: &Value) -> Result<Vec<Step>, String> {
    let steps = match doc {
        Value::Array(a) => a,
        Value::Object(o) => {
            if let Some(f) = o.get("format").and_then(|f| f.as_str())
                && f != FORMAT
            {
                return Err(format!("not a GodotTrench MCP script (format {f})"));
            }

            o.get("steps").and_then(|s| s.as_array()).ok_or("script needs a steps array")?
        }
        _ => return Err("script must be an object or an array".into()),
    };
    steps
        .iter()
        .enumerate()
        .filter(|(_, s)| s.get("tool").is_some())
        .map(|(i, s)| {
            let tool = s["tool"].as_str().ok_or_else(|| format!("step {i}: tool must be a string"))?.to_string();
            Ok(Step {
                tool,
                args: s.get("args").cloned().unwrap_or(Value::Object(Default::default())),
                save: s.get("save").and_then(|v| v.as_str()).map(str::to_string),
            })
        })
        .collect()
}

fn lookup<'a>(vars: &'a BTreeMap<String, Value>, path: &str) -> Option<&'a Value> {
    let mut parts = path.split('.');
    let mut cur = vars.get(parts.next()?)?;
    for p in parts {
        cur = match p.parse::<usize>() {
            Ok(i) if cur.is_array() => cur.get(i)?,
            _ => cur.get(p)?,
        };
    }

    Some(cur)
}

fn text_of(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// The path of a whole-string reference: `$name` or `$name.path`, the name starting with a letter or underscore.
fn whole_reference(s: &str) -> Option<&str> {
    let path = s.strip_prefix('$')?;
    let first = path.chars().next()?;
    let ident = (first.is_ascii_alphabetic() || first == '_') && path.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.');
    ident.then_some(path)
}

fn missing(vars: &BTreeMap<String, Value>, path: &str) -> String {
    let name = path.split('.').next().unwrap_or(path);
    if vars.contains_key(name) {
        return format!("${path}: {name} has no value at that path");
    }

    if name == "project" {
        return "$project is not set because no Godot project is open, call open_project first".into();
    }

    format!("unknown variable ${path}, write $$ for a literal $")
}

/// `"$a.ids + $b.ids"`: the reference paths of a string made only of whole references joined by `+`.
fn joined_references(s: &str) -> Option<Vec<&str>> {
    if !s.contains('+') {
        return None;
    }

    s.split('+').map(|part| whole_reference(part.trim())).collect()
}

/// Replaces variable references in `args`. Unknown names are an error so typos do not silently pass, `$$` is a literal `$`,
/// and a `$` that starts no reference (such as `$5`) stays as it is.
pub fn resolve(args: &Value, vars: &BTreeMap<String, Value>) -> Result<Value, String> {
    Ok(match args {
        Value::String(s) => {
            if let Some(path) = whole_reference(s) {
                return lookup(vars, path).cloned().ok_or_else(|| missing(vars, path));
            }

            if let Some(paths) = joined_references(s) {
                let mut out = Vec::new();
                for path in paths {
                    match lookup(vars, path).ok_or_else(|| missing(vars, path))? {
                        Value::Array(a) => out.extend(a.iter().cloned()),
                        Value::Null => {}
                        other => out.push(other.clone()),
                    }
                }

                return Ok(Value::Array(out));
            }

            if !s.contains('$') {
                return Ok(args.clone());
            }

            let mut out = String::new();
            let mut rest = s.as_str();
            while let Some(i) = rest.find('$') {
                out.push_str(&rest[..i]);
                let after = &rest[i + 1..];
                if let Some(r) = after.strip_prefix('$') {
                    out.push('$');
                    rest = r;
                } else if let Some(r) = after.strip_prefix('{') {
                    let end = r.find('}').ok_or_else(|| format!("unclosed ${{ in {s}"))?;
                    let path = &r[..end];
                    out.push_str(&text_of(lookup(vars, path).ok_or_else(|| missing(vars, path))?));
                    rest = &r[end + 1..];
                } else {
                    out.push('$');
                    rest = after;
                }
            }

            out.push_str(rest);
            Value::String(out)
        }
        Value::Array(a) => Value::Array(a.iter().map(|v| resolve(v, vars)).collect::<Result<_, _>>()?),
        Value::Object(o) => Value::Object(o.iter().map(|(k, v)| Ok((k.clone(), resolve(v, vars)?))).collect::<Result<_, String>>()?),
        other => other.clone(),
    })
}

/// Keys of tool results whose values are node ids, alone or in (nested) arrays.
fn is_id_key(key: &str) -> bool {
    matches!(key, "id" | "ids" | "selection" | "letters" | "copies" | "entity") || key.ends_with("_id") || key.ends_with("_ids")
}

/// The `replaced` map of a tool result, `{"old id": [new ids]}`.
pub fn replacements(result: &Value) -> BTreeMap<u64, Vec<u64>> {
    let Some(map) = result.get("replaced").and_then(Value::as_object) else { return BTreeMap::new() };
    map.iter().filter_map(|(old, new)| Some((old.parse().ok()?, new.as_array()?.iter().filter_map(Value::as_u64).collect()))).collect()
}

fn rewrite_ids(v: &mut Value, replaced: &BTreeMap<u64, Vec<u64>>) {
    match v {
        Value::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for mut item in std::mem::take(items) {
                match item.as_u64().and_then(|id| replaced.get(&id)) {
                    Some(new) => out.extend(new.iter().map(|n| Value::from(*n))),
                    None => {
                        rewrite_ids(&mut item, replaced);
                        out.push(item);
                    }
                }
            }

            *items = out;
        }
        Value::Number(n) => {
            if let Some(new) = n.as_u64().and_then(|id| replaced.get(&id)) {
                *v = match new.as_slice() {
                    [] => Value::Null,
                    [one] => Value::from(*one),
                    many => Value::from(many.to_vec()),
                };
            }
        }
        _ => {}
    }
}

/// Rewrites the node ids inside a saved result after a later step replaced some of them. Ids are the values under
/// `id`, `ids`, `selection`, `letters`, `copies`, `entity` and keys ending in `_id` or `_ids`, at any depth. In a list
/// a replaced id gives way to all of its replacements in place (none when it was removed, like a CSG cutter), a
/// single id becomes its one replacement, the list of them when there are several, and null when it was removed.
pub fn follow_replacements(saved: &mut Value, replaced: &BTreeMap<u64, Vec<u64>>) {
    if replaced.is_empty() {
        return;
    }

    match saved {
        Value::Object(o) => {
            for (k, v) in o.iter_mut() {
                if is_id_key(k) && !v.is_object() {
                    rewrite_ids(v, replaced);
                } else if k != "replaced" {
                    follow_replacements(v, replaced);
                }
            }
        }
        Value::Array(a) => a.iter_mut().for_each(|v| follow_replacements(v, replaced)),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_and_resolves_references() {
        let doc = json!({ "format": FORMAT, "steps": [
            { "note": "comments are steps without a tool" },
            { "tool": "create_brush", "args": { "min": [0, 0, 0], "max": [8, 8, 8] }, "save": "wall" },
            { "tool": "select", "args": { "ids": ["$wall.ids.0"], "label": "wall ${wall.ids.0} done" } }
        ]});
        let steps = parse(&doc).unwrap();
        assert_eq!(steps.len(), 2);
        let mut vars = BTreeMap::new();
        vars.insert("wall".to_string(), json!({ "ids": [42] }));
        let args = resolve(&steps[1].args, &vars).unwrap();
        assert_eq!(args, json!({ "ids": [42], "label": "wall 42 done" }));
        assert!(resolve(&json!("$missing"), &vars).is_err());
        assert_eq!(resolve(&json!("costs $5 each"), &vars).unwrap(), json!("costs $5 each"));
    }

    #[test]
    fn dollar_escapes_and_clear_errors() {
        let mut vars = BTreeMap::new();
        vars.insert("wall".to_string(), json!({ "ids": [42] }));
        assert_eq!(resolve(&json!("$5"), &vars).unwrap(), json!("$5"));
        assert_eq!(resolve(&json!("$$Node"), &vars).unwrap(), json!("$Node"));
        assert_eq!(resolve(&json!("get_node($$Door) ${wall.ids.0} $$${wall.ids.0}"), &vars).unwrap(), json!("get_node($Door) 42 $42"));
        assert!(resolve(&json!("$Node"), &vars).unwrap_err().contains("$$"));
        assert!(resolve(&json!("$wall.nope"), &vars).unwrap_err().contains("no value"));
        assert!(resolve(&json!("${project}/maps"), &vars).unwrap_err().contains("open_project"));
        assert_eq!(resolve(&json!("a $ b"), &vars).unwrap(), json!("a $ b"));
    }

    #[test]
    fn plus_joins_saved_values_into_one_list() {
        let mut vars = BTreeMap::new();
        vars.insert("a".to_string(), json!({ "ids": [1, 2], "id": 9 }));
        vars.insert("b".to_string(), json!({ "ids": [3] }));
        assert_eq!(resolve(&json!({ "ids": "$a.ids + $b.ids" }), &vars).unwrap(), json!({ "ids": [1, 2, 3] }));
        assert_eq!(resolve(&json!("$a.ids+$a.id+$b.ids"), &vars).unwrap(), json!([1, 2, 9, 3]));
        assert!(resolve(&json!("$a.ids + $nope.ids"), &vars).unwrap_err().contains("unknown variable"));
        assert_eq!(resolve(&json!("1 + 2"), &vars).unwrap(), json!("1 + 2"), "plain text with a plus stays text");
        assert_eq!(resolve(&json!("$5 + $6"), &vars).unwrap(), json!("$5 + $6"));
        assert_eq!(resolve(&json!("${a.id} + ${b.ids.0}"), &vars).unwrap(), json!("9 + 3"));
    }

    #[test]
    fn saved_ids_follow_replaced_brushes() {
        let result = json!({ "ok": true, "replaced": { "10": [20, 21], "11": [], "12": [22] } });
        let replaced = replacements(&result);
        assert_eq!(replaced[&10], vec![20, 21]);
        let mut wall = json!({ "ids": [10, 11, 12, 13], "entity": null, "letters": [[10], [13]], "bounds": { "min": [10, 11, 12] } });
        follow_replacements(&mut wall, &replaced);
        assert_eq!(wall["ids"], json!([20, 21, 22, 13]), "pieces take the place of the cut brush, the cutter drops out");
        assert_eq!(wall["letters"], json!([[20, 21], [13]]));
        assert_eq!(wall["bounds"]["min"], json!([10, 11, 12]), "numbers that are not ids stay");
        let mut single = json!({ "id": 12, "other_id": 10, "gone_id": 11, "count": 10, "faces": [[10, 3]] });
        follow_replacements(&mut single, &replaced);
        assert_eq!(single, json!({ "id": 22, "other_id": [20, 21], "gone_id": null, "count": 10, "faces": [[10, 3]] }));
    }
}
