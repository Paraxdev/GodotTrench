//! MCP scripts: JSON lists of tool calls replayed against the editor, with results saved under names and referenced by
//! later steps. `"$door.id"` is replaced by that JSON value, `"${door.id}"` inside a longer string by its text, and `$$` is a
//! literal `$`.

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

/// Replaces variable references in `args`. Unknown names are an error so typos do not silently pass, `$$` is a literal `$`,
/// and a `$` that starts no reference (such as `$5`) stays as it is.
pub fn resolve(args: &Value, vars: &BTreeMap<String, Value>) -> Result<Value, String> {
    Ok(match args {
        Value::String(s) => {
            if let Some(path) = whole_reference(s) {
                return lookup(vars, path).cloned().ok_or_else(|| missing(vars, path));
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
}
