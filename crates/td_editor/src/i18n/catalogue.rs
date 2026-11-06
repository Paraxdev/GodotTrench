//! Reads one language's JSON file of translations.

use std::collections::HashMap;
use std::fmt;

use serde::de::{self, Deserializer, MapAccess, Visitor};

use super::{Form, Lang};

pub(crate) type Catalogue = HashMap<String, Entry>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Entry {
    Text(String),
    /// One text per plural form of the language, in the order of [`Lang::forms`].
    Plural(Vec<(Form, String)>),
}

enum Raw {
    Text(String),
    Forms(Vec<(String, String)>),
}

/// Reads a JSON object and keeps every key, serde_json's own maps silently keep only the last of a duplicate.
struct Pairs<V>(Vec<(String, V)>);

impl<'de, V: de::Deserialize<'de>> de::Deserialize<'de> for Pairs<V> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct PairsVisitor<V>(std::marker::PhantomData<V>);
        impl<'de, V: de::Deserialize<'de>> Visitor<'de> for PairsVisitor<V> {
            type Value = Pairs<V>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("an object")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut pairs = Vec::new();
                while let Some(pair) = map.next_entry()? {
                    pairs.push(pair);
                }

                Ok(Pairs(pairs))
            }
        }

        d.deserialize_map(PairsVisitor(std::marker::PhantomData))
    }
}

impl<'de> de::Deserialize<'de> for Raw {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct RawVisitor;
        impl<'de> Visitor<'de> for RawVisitor {
            type Value = Raw;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a string, or an object of plural forms")
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Raw, E> {
                Ok(Raw::Text(v.to_string()))
            }
            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Raw, A::Error> {
                let Pairs(forms) = de::Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))?;
                Ok(Raw::Forms(forms))
            }
        }

        d.deserialize_any(RawVisitor)
    }
}

pub(crate) fn parse(lang: Lang, json: &str) -> Result<Catalogue, String> {
    let Pairs(pairs) = serde_json::from_str::<Pairs<Raw>>(json).map_err(|e| e.to_string())?;
    let mut out = Catalogue::with_capacity(pairs.len());
    for (key, raw) in pairs {
        let entry = entry(lang, raw).map_err(|e| format!("{key:?}: {e}"))?;
        if out.insert(key.clone(), entry).is_some() {
            return Err(format!("{key:?} is in the file twice"));
        }
    }

    Ok(out)
}

fn entry(lang: Lang, raw: Raw) -> Result<Entry, String> {
    let text = |t: String| if t.trim().is_empty() { Err("the translation is empty".to_string()) } else { Ok(t) };
    match raw {
        Raw::Text(t) => Ok(Entry::Text(text(t)?)),
        Raw::Forms(forms) => {
            let names: Vec<&str> = forms.iter().map(|(f, _)| f.as_str()).collect();
            let wanted: Vec<&str> = lang.forms().iter().map(|f| f.name()).collect();
            if names.len() != wanted.len() || !wanted.iter().all(|w| names.contains(w)) {
                return Err(format!("{} plural forms are {}, not {}", lang.code(), wanted.join(", "), names.join(", ")));
            }

            let mut out = Vec::with_capacity(forms.len());
            for form in lang.forms() {
                let (_, t) = forms.iter().find(|(f, _)| f == form.name()).expect("checked above");
                out.push((*form, text(t.clone())?));
            }

            Ok(Entry::Plural(out))
        }
    }
}
