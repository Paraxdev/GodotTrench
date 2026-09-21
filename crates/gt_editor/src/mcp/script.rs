//! MCP scripts: JSON lists of tool calls replayed against the editor, with results saved under names and referenced by
//! later steps. `"$door.id"` is replaced by that JSON value, `"${door.id}"` inside a longer string by its text.

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

/// Replaces variable references in `args`. Unknown names are an error so typos do not silently pass.
pub fn resolve(args: &Value, vars: &BTreeMap<String, Value>) -> Result<Value, String> {
    Ok(match args {
        Value::String(s) => {
            if let Some(path) = s.strip_prefix('$').filter(|p| !p.starts_with('{') && !p.is_empty() && !p.contains(' ')) {
                return lookup(vars, path).cloned().ok_or_else(|| format!("unknown variable ${path}"));
            }

            if !s.contains("${") {
                return Ok(args.clone());
            }

            let mut out = String::new();
            let mut rest = s.as_str();
            while let Some(start) = rest.find("${") {
                out.push_str(&rest[..start]);
                let after = &rest[start + 2..];
                let end = after.find('}').ok_or_else(|| format!("unclosed ${{ in {s}"))?;
                let path = &after[..end];
                out.push_str(&text_of(lookup(vars, path).ok_or_else(|| format!("unknown variable ${{{path}}}"))?));
                rest = &after[end + 1..];
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
}
