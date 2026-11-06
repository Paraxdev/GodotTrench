use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use super::catalogue::{self, Catalogue, Entry};
use super::*;

const DE: &str = r#"{ "Open": "Öffnen", "Open#verb": "Aufmachen", "Deleted {n} objects in {layer}": "{layer}: {n} Objekte gelöscht",
    "{n} widget": { "one": "ein Brush", "other": "{n} Brushes" } }"#;
const FR: &str = r#"{ "Save": "Enregistrer" }"#;
const RU: &str = r#"{ "{n} widget": { "one": "{n} браш", "few": "{n} браша", "many": "{n} брашей" } }"#;

#[test]
fn lookup_falls_back_to_english_per_key() {
    let _de = test_catalogue(Lang::De, DE);
    assert_eq!((tr("Open"), tr("Save")), ("Open", "Save"), "English by default");
    {
        let _s = scope(Lang::De);
        assert_eq!((tr("Open"), tr("Save")), ("Öffnen", "Save"));
        {
            let _s = scope(Lang::En);
            assert_eq!((tr("Open"), tr("Save")), ("Open", "Save"));
        }

        assert_eq!(current(), Lang::De, "an inner scope restores the outer language");
    }

    assert_eq!(current(), Lang::En);
}

#[test]
fn a_key_passed_as_a_string_shows_translated_and_other_text_stays_whole() {
    let _de = test_catalogue(Lang::De, DE);
    assert_eq!(Msg::from(String::from("Open")), Msg { en: "Open".into(), shown: "Open".into() }, "English");
    let _s = scope(Lang::De);
    assert_eq!(Msg::from(String::from("Open")), Msg { en: "Open".into(), shown: "Öffnen".into() });
    assert_eq!(Msg::from("Open"), Msg { en: "Open".into(), shown: "Öffnen".into() });
    assert_eq!(Msg::from("Open now"), Msg { en: "Open now".into(), shown: "Open now".into() });
    assert_eq!(tr_known("Open#verb"), "Aufmachen");
    assert_eq!((tr("walls/brick#dirty"), tr_known("walls/brick#dirty")), ("walls/brick", "walls/brick#dirty"), "a run time label keeps its #");
    let _en = scope(Lang::En);
    assert_eq!((tr_known("Open"), tr_known("walls/brick#dirty")), ("Open", "walls/brick#dirty"));
}

#[test]
fn a_second_language_leaks_nothing_from_the_first() {
    let _de = test_catalogue(Lang::De, DE);
    let _fr = test_catalogue(Lang::Fr, FR);
    let _s = scope(Lang::De);
    assert_eq!((tr("Open"), tr("Save")), ("Öffnen", "Save"));
    set(Lang::Fr);
    assert_eq!((tr("Open"), tr("Save")), ("Open", "Enregistrer"));
    assert_eq!(tr_n!(3, "{n} widget", "{n} widgets"), "3 widgets", "fr has no plural of its own");
}

#[test]
fn the_language_is_per_thread() {
    let _s = scope(Lang::De);
    let other = std::thread::spawn(|| {
        let started = current();
        let _s = scope(Lang::Ja);
        (started, current())
    });
    assert_eq!(other.join().unwrap(), (Lang::En, Lang::Ja));
    assert_eq!(current(), Lang::De, "another thread's scope does not reach this one");
}

#[test]
fn codes_and_names() {
    let codes: Vec<&str> = Lang::ALL.iter().map(|l| l.code()).collect();
    assert_eq!(codes, ["en", "de", "fr", "es", "pt", "ru", "zh-Hans", "ja"]);
    let names: Vec<&str> = Lang::ALL.iter().map(|l| l.name()).collect();
    assert_eq!(names, ["English", "Deutsch", "Français", "Español", "Português", "Русский", "简体中文", "日本語"]);
    for lang in Lang::ALL {
        assert_eq!(Lang::from_code(lang.code()), Some(lang));
    }

    assert_eq!(Lang::from_code("ZH-hans"), Some(Lang::ZhHans));
    assert_eq!(Lang::from_code("JA"), Some(Lang::Ja));
    for unknown in ["", "zh", "pt-BR", "english", "de "] {
        assert_eq!(Lang::from_code(unknown), None, "{unknown:?}");
    }
}

#[test]
fn plural_forms_by_cldr_rules() {
    use Form::*;
    let ns = [0, 1, 2, 4, 5, 11, 12, 14, 21, 22, 25, 101, 111, 1000];
    let one_only = [Other, One, Other, Other, Other, Other, Other, Other, Other, Other, Other, Other, Other, Other];
    let zero_one = [One, One, Other, Other, Other, Other, Other, Other, Other, Other, Other, Other, Other, Other];
    let ru = [Many, One, Few, Few, Many, Many, Many, Many, One, Few, Many, One, Many, Many];
    let none = [Other; 14];
    for (lang, want) in [
        (Lang::En, one_only),
        (Lang::De, one_only),
        (Lang::Es, one_only),
        (Lang::Fr, zero_one),
        (Lang::Pt, zero_one),
        (Lang::Ru, ru),
        (Lang::ZhHans, none),
        (Lang::Ja, none),
    ] {
        let got: Vec<Form> = ns.iter().map(|n| lang.plural(*n)).collect();
        assert_eq!(got, want, "{lang:?}");
        for form in &got {
            assert!(lang.forms().contains(form), "{lang:?} has no {form:?} form");
        }
    }

    let _ru = test_catalogue(Lang::Ru, RU);
    let _de = test_catalogue(Lang::De, DE);
    let shown = |lang| {
        let _s = scope(lang);
        [0u32, 1, 3, 5, 21, 111].map(|n| tr_n!(n, "{n} widget", "{n} widgets"))
    };
    assert_eq!(shown(Lang::En), ["0 widgets", "1 widget", "3 widgets", "5 widgets", "21 widgets", "111 widgets"]);
    assert_eq!(shown(Lang::Ru), ["0 брашей", "1 браш", "3 браша", "5 брашей", "21 браш", "111 брашей"]);
    assert_eq!(shown(Lang::De), ["0 Brushes", "ein Brush", "3 Brushes", "5 Brushes", "21 Brushes", "111 Brushes"]);
    assert_eq!(shown(Lang::Ja), ["0 widgets", "1 widget", "3 widgets", "5 widgets", "21 widgets", "111 widgets"]);
    assert_eq!(tr_n!(-1i64, "{n} widget", "{n} widgets"), "-1 widgets", "a negative count takes the other form");
}

#[test]
fn fill_by_name() {
    let (a, b) = (3, "Walls");
    assert_eq!(fill("{a} in {b}", &[("a", &a), ("b", &b)]), "3 in Walls");
    assert_eq!(fill("{b}: {a} {a}", &[("a", &a), ("b", &b)]), "Walls: 3 3", "reordered and repeated");
    assert_eq!(fill("{{a}} is {a}", &[("a", &a)]), "{a} is 3");
    assert_eq!(fill("}} {{ { }", &[]), "} { { }", "lone braces stay");
    let tricky = "{b}";
    assert_eq!(fill("{a} and {b}", &[("a", &tricky), ("b", &b)]), "{b} and Walls", "a value is not filled again");
    assert_eq!(fill("{a} of {missing}", &[("a", &a)]), "3 of {missing}", "a missing argument shows");
    assert_eq!(fill("", &[("a", &a)]), "");
    assert_eq!(fill("Größe {a}", &[("a", &"日本")]), "Größe 日本");
}

#[test]
fn trf_and_msg_fill_the_same_values_once() {
    let _de = test_catalogue(Lang::De, DE);
    let layer = String::from("Walls");
    let calls = std::cell::Cell::new(0);
    let count = || {
        calls.set(calls.get() + 1);
        7
    };
    let _s = scope(Lang::De);
    let m = msg!("Deleted {n} objects in {layer}", n = count(), layer);
    assert_eq!(m, Msg { en: "Deleted 7 objects in Walls".into(), shown: "Walls: 7 Objekte gelöscht".into() });
    assert_eq!(calls.get(), 1);
    assert_eq!(trf!("Deleted {n} objects in {layer}", layer, n = 2), "Walls: 2 Objekte gelöscht");
    assert_eq!(trf!("Open"), "Öffnen");
    assert_eq!(msg!("Save"), Msg { en: "Save".into(), shown: "Save".into() });

    let m = msg_n!(count(), "{n} widget", "{n} widgets");
    assert_eq!(m, Msg { en: "7 widgets".into(), shown: "7 Brushes".into() });
    let m = msg_n!(1usize, "{n} widget", "{n} widgets");
    assert_eq!(m, Msg { en: "1 widget".into(), shown: "ein Brush".into() });
    assert_eq!(calls.get(), 2);
    assert_eq!(Msg::from("x"), Msg { en: "x".into(), shown: "x".into() });
    assert_eq!(Msg::from(&layer), Msg { en: "Walls".into(), shown: "Walls".into() });
}

#[test]
fn a_message_from_a_key_and_a_joined_message_keep_both_texts() {
    let _de = test_catalogue(Lang::De, DE);
    let _fr = test_catalogue(Lang::Fr, FR);
    let _s = scope(Lang::De);
    assert_eq!(Msg::key("Open#verb"), Msg { en: "Open".into(), shown: "Aufmachen".into() }, "the context suffix is not English text");
    assert_eq!(Msg::key("Missing"), Msg { en: "Missing".into(), shown: "Missing".into() });
    let joined = msg!("Open").then(". ", msg!("Save")).then(", ", Msg::from("raw 7"));
    assert_eq!(joined, Msg { en: "Open. Save, raw 7".into(), shown: "Öffnen. Save, raw 7".into() });
    set(Lang::Fr);
    assert_eq!(msg!("Open").then("", msg!("Save")), Msg { en: "OpenSave".into(), shown: "OpenEnregistrer".into() }, "fr has no Open");
}

#[test]
fn a_message_with_translated_words_keeps_the_english_ones_in_its_english_text() {
    let _de = test_catalogue(Lang::De, r#"{ "{kind} {axis} at {x}": "{kind} auf {axis}, {x}" }"#);
    let words = [("kind", "Grab", "Greifen"), ("axis", "free", "frei"), ("x", "3.50", "3.50")];
    let _s = scope(Lang::De);
    assert_eq!(Msg::filled("{kind} {axis} at {x}", &words), Msg { en: "Grab free at 3.50".into(), shown: "Greifen auf frei, 3.50".into() });
    assert_eq!(Msg::filled("{kind} {axis} at {x}", &words[..1]).en, "Grab {axis} at {x}", "a missing word stays visible");
    assert_eq!(
        Msg::filled("{kind} {axis} here", &words),
        Msg { en: "Grab free here".into(), shown: "Greifen frei here".into() },
        "a key without a translation shows its English template around the shown words"
    );
}

#[test]
fn a_counted_message_with_translated_words_picks_the_form_in_both_texts() {
    let _de = test_catalogue(Lang::De, r#"{ "{n} file{x}": { "one": "{n} Datei{x}", "other": "{n} Dateien{x}" } }"#);
    let _ru = test_catalogue(Lang::Ru, r#"{ "{n} file{x}": { "one": "{n} файл{x}", "few": "{n} файла{x}", "many": "{n} файлов{x}" } }"#);
    let words = [("x", " from the beta", " aus der Beta")];
    let _s = scope(Lang::De);
    assert_eq!(Msg::filled_n("{n} file{x}", "{n} files{x}", 1, &words), Msg { en: "1 file from the beta".into(), shown: "1 Datei aus der Beta".into() });
    assert_eq!(Msg::filled_n("{n} file{x}", "{n} files{x}", 3, &words), Msg { en: "3 files from the beta".into(), shown: "3 Dateien aus der Beta".into() });
    set(Lang::Ru);
    let words = [("x", "", "")];
    let shown: Vec<String> = [1, 2, 5, 21].map(|n| Msg::filled_n("{n} file{x}", "{n} files{x}", n, &words).shown).into();
    assert_eq!(shown, ["1 файл", "2 файла", "5 файлов", "21 файл"]);
    set(Lang::En);
    assert_eq!(Msg::filled_n("{n} file{x}", "{n} files{x}", 2, &words).shown, "2 files");
}

#[test]
fn list_joins_per_language() {
    let three = ["A", "B", "C"];
    let cases = [
        (Lang::En, ["", "A", "A and B", "A, B and C"]),
        (Lang::De, ["", "A", "A und B", "A, B und C"]),
        (Lang::Ja, ["", "A", "A、B", "A、B、C"]),
        (Lang::ZhHans, ["", "A", "A和B", "A、B和C"]),
    ];
    for (lang, want) in cases {
        let _s = scope(lang);
        let got = [0, 1, 2, 3].map(|n| list(&three[..n]));
        assert_eq!(got, want, "{lang:?}");
    }

    assert_eq!(list(&[String::from("x"), String::from("y")]), "x and y");
}

#[test]
fn context_suffix() {
    let _de = test_catalogue(Lang::De, DE);
    assert_eq!((tr("Open#verb"), tr("Open")), ("Open", "Open"));
    assert_eq!(en!("Open#verb"), "Open#verb", "the key keeps it");
    let _s = scope(Lang::De);
    assert_eq!((tr("Open#verb"), tr("Open")), ("Aufmachen", "Öffnen"));
    assert_eq!(tr("Close#verb"), "Close", "untranslated, the suffix still goes");
    for kept in ["C#", "Use C#", "#tag", "Color #ffaa00", "Open#Verb", "Open#v2", "Open#"] {
        assert_eq!(tr(kept), kept);
    }
}

#[test]
fn the_loader_rejects_broken_files() {
    let ok = catalogue::parse(Lang::Ru, RU).unwrap();
    let Some(Entry::Plural(forms)) = ok.get("{n} widget") else { panic!("{ok:?}") };
    assert_eq!(forms.iter().map(|(f, _)| *f).collect::<Vec<_>>(), [Form::One, Form::Few, Form::Many]);
    assert_eq!(catalogue::parse(Lang::De, "{}").unwrap().len(), 0);
    assert_eq!(catalogue::parse(Lang::De, DE).unwrap().get("Open"), Some(&Entry::Text("Öffnen".into())));

    for (lang, json, why) in [
        (Lang::De, r#"{ "Open": "Öffnen", "Open": "Auf" }"#, "twice"),
        (Lang::De, r#"{ "Open": "" }"#, "empty"),
        (Lang::De, r#"{ "Open": "  " }"#, "empty"),
        (Lang::De, r#"{ "{n} brush": { "one": "x", "other": "" } }"#, "empty"),
        (Lang::De, r#"{ "{n} brush": { "one": "x" } }"#, "plural forms"),
        (Lang::De, r#"{ "{n} brush": { "one": "x", "few": "y" } }"#, "plural forms"),
        (Lang::De, r#"{ "{n} brush": { "one": "x", "other": "y", "many": "z" } }"#, "plural forms"),
        (Lang::Ru, r#"{ "{n} brush": { "one": "x", "other": "y" } }"#, "plural forms"),
        (Lang::Ja, r#"{ "{n} brush": { "one": "x", "other": "y" } }"#, "plural forms"),
        (Lang::Ru, r#"{ "{n} brush": { "one": "x", "few": "y", "few": "z" } }"#, "plural forms"),
        (Lang::De, r#"{ "Open": 3 }"#, "invalid type"),
        (Lang::De, r#"{ "Open": ["Öffnen"] }"#, "invalid type"),
        (Lang::De, r#"{ "Open": null }"#, "invalid type"),
        (Lang::De, r#"{ "{n} brush": { "one": 1, "other": "y" } }"#, "invalid type"),
        (Lang::De, r#"["Open"]"#, "invalid type"),
        (Lang::De, r#"{ "Open": "Öffnen" "#, "EOF"),
    ] {
        let err = catalogue::parse(lang, json).expect_err(json);
        assert!(err.contains(why), "{json}: {err}");
    }
}

#[test]
fn shipped_catalogues_load() {
    for lang in Lang::ALL {
        let c = catalogue::parse(lang, lang.source()).unwrap_or_else(|e| panic!("{}: {e}", lang.code()));
        assert_eq!(c.len(), shipped(lang).len());
        let source = lang.source();
        assert!(!source.starts_with('\u{feff}') && !source.contains('\r') && !source.contains("\\u"), "{}", lang.code());
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Tok {
    Ident(String),
    Str(String),
    Punct(char),
    Other,
}

/// Just enough of a Rust lexer to find string literals and the calls around them, comments left out.
fn lex(src: &str) -> Vec<(Tok, usize)> {
    let b: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let (mut i, mut line) = (0, 1);
    while i < b.len() {
        let c = b[i];
        let at = line;
        if c == '\n' {
            line += 1;
            i += 1;
        } else if c.is_whitespace() {
            i += 1;
        } else if c == '/' && b.get(i + 1) == Some(&'/') {
            while i < b.len() && b[i] != '\n' {
                i += 1;
            }
        } else if c == '/' && b.get(i + 1) == Some(&'*') {
            i += 2;
            while i < b.len() && !(b[i] == '*' && b.get(i + 1) == Some(&'/')) {
                line += usize::from(b[i] == '\n');
                i += 1;
            }

            i += 2;
        } else if c == '"' {
            let s;
            (s, i, line) = string(&b, i + 1, line);
            out.push((Tok::Str(s), at));
        } else if c == '\'' {
            if b.get(i + 1) == Some(&'\\') {
                i += 3;
                while b.get(i).is_some_and(|c| *c != '\'') {
                    i += 1;
                }

                i += 1;
                out.push((Tok::Other, at));
            } else if b.get(i + 2) == Some(&'\'') {
                i += 3;
                out.push((Tok::Other, at));
            } else {
                i += 1;
            }
        } else if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < b.len() && (b[i].is_alphanumeric() || b[i] == '_') {
                i += 1;
            }

            let word: String = b[start..i].iter().collect();
            let hashes = b[i..].iter().take_while(|c| **c == '#').count();
            if matches!(word.as_str(), "r" | "br" | "cr") && b.get(i + hashes) == Some(&'"') {
                let body = i + hashes + 1;
                let mut end = body;
                while end < b.len() && !(b[end] == '"' && b[end + 1..].iter().take(hashes).filter(|c| **c == '#').count() == hashes) {
                    end += 1;
                }

                let s: String = b[body..end].iter().collect();
                line += s.matches('\n').count();
                out.push((Tok::Str(s), at));
                i = end + 1 + hashes;
            } else if matches!(word.as_str(), "b" | "c") && b.get(i) == Some(&'"') {
                let s;
                (s, i, line) = string(&b, i + 1, line);
                out.push((Tok::Str(s), at));
            } else {
                out.push((Tok::Ident(word), at));
            }
        } else if c.is_ascii_digit() {
            while i < b.len() && (b[i].is_alphanumeric() || b[i] == '_') {
                i += 1;
            }

            out.push((Tok::Other, at));
        } else {
            out.push((Tok::Punct(c), at));
            i += 1;
        }
    }

    out
}

/// Decodes a string literal from just after its opening quote, returns it, where it ends and the line it ends on.
fn string(b: &[char], mut i: usize, mut line: usize) -> (String, usize, usize) {
    let mut s = String::new();
    while i < b.len() && b[i] != '"' {
        if b[i] != '\\' {
            line += usize::from(b[i] == '\n');
            s.push(b[i]);
            i += 1;
            continue;
        }

        let e = b.get(i + 1).copied().unwrap_or('\\');
        i += 2;
        match e {
            'n' => s.push('\n'),
            't' => s.push('\t'),
            'r' => s.push('\r'),
            '0' => s.push('\0'),
            'x' => {
                let hex: String = b[i..i + 2].iter().collect();
                s.push(char::from(u8::from_str_radix(&hex, 16).unwrap_or(b'?')));
                i += 2;
            }
            'u' => {
                let end = i + b[i..].iter().position(|c| *c == '}').unwrap_or(0);
                let hex: String = b[i + 1..end].iter().collect();
                s.push(u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32).unwrap_or('?'));
                i = end + 1;
            }
            '\n' => {
                line += 1;
                while b.get(i).is_some_and(|c| c.is_whitespace()) {
                    line += usize::from(b[i] == '\n');
                    i += 1;
                }
            }
            other => s.push(other),
        }
    }

    (s, i + 1, line)
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Key {
    Text(String),
    Plural(String, String),
}

impl Key {
    fn id(&self) -> &str {
        match self {
            Key::Text(k) | Key::Plural(k, _) => k,
        }
    }
}

fn is(tokens: &[(Tok, usize)], at: usize, want: &Tok) -> bool {
    tokens.get(at).is_some_and(|(t, _)| t == want)
}

fn str_at(tokens: &[(Tok, usize)], at: usize) -> Option<&str> {
    match tokens.get(at) {
        Some((Tok::Str(s), _)) => Some(s),
        _ => None,
    }
}

/// The index just past the argument that starts at `at`, on its closing `,` or `)`.
fn skip_arg(tokens: &[(Tok, usize)], mut at: usize) -> usize {
    let mut depth = 0;
    while let Some((t, _)) = tokens.get(at) {
        match t {
            Tok::Punct('(' | '[' | '{') => depth += 1,
            Tok::Punct(')' | ']' | '}') if depth == 0 => break,
            Tok::Punct(')' | ']' | '}') => depth -= 1,
            Tok::Punct(',') if depth == 0 => break,
            _ => {}
        }

        at += 1;
    }

    at
}

/// The keys a source file looks up, with their lines. A key built at run time is not seen.
fn keys(src: &str) -> Vec<(Key, usize)> {
    let t = lex(src);
    let (open, bang) = (Tok::Punct('('), Tok::Punct('!'));
    let mut out = Vec::new();
    for (i, (tok, line)) in t.iter().enumerate() {
        let Tok::Ident(name) = tok else { continue };
        match name.as_str() {
            "tr" if is(&t, i + 1, &open) => {
                if let Some(k) = str_at(&t, i + 2) {
                    out.push((Key::Text(k.into()), *line));
                }
            }

            // An undo label, the History panel looks it up
            "edit" | "edit_coalesced" | "begin" if is(&t, i.wrapping_sub(1), &Tok::Punct('.')) && is(&t, i + 1, &open) => {
                if let Some(k) = str_at(&t, i + 2) {
                    out.push((Key::Text(k.into()), *line));
                }
            }

            // A message with translated words in its placeholders
            "filled" if is(&t, i.wrapping_sub(1), &Tok::Punct(':')) && is(&t, i + 1, &open) => {
                if let Some(k) = str_at(&t, i + 2) {
                    out.push((Key::Text(k.into()), *line));
                }
            }

            // A counted message with translated words in its placeholders
            "filled_n" if is(&t, i.wrapping_sub(1), &Tok::Punct(':')) && is(&t, i + 1, &open) => {
                if let (Some(one), Some(other)) = (str_at(&t, i + 2), str_at(&t, i + 4)) {
                    out.push((Key::Plural(one.into(), other.into()), *line));
                }
            }

            // A constant looked up where it is shown
            "key" if is(&t, i.wrapping_sub(1), &Tok::Punct(':')) && is(&t, i.wrapping_sub(3), &Tok::Ident("Msg".into())) && is(&t, i + 1, &open) => {
                if let Some(k) = str_at(&t, i + 2) {
                    out.push((Key::Text(k.into()), *line));
                }
            }
            "trf" | "msg" | "en" if is(&t, i + 1, &bang) && is(&t, i + 2, &open) => {
                if let Some(k) = str_at(&t, i + 3) {
                    out.push((Key::Text(k.into()), *line));
                }
            }
            "tr_n" | "msg_n" if is(&t, i + 1, &bang) && is(&t, i + 2, &open) => {
                let at = skip_arg(&t, i + 3) + 1;
                if let (Some(one), Some(other)) = (str_at(&t, at), str_at(&t, at + 2)) {
                    out.push((Key::Plural(one.into(), other.into()), *line));
                }
            }
            _ => {}
        }
    }

    out
}

fn src_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            out.extend(rust_files(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }

    out.sort();
    out
}

/// The source without its inline test module, whose keys are made up for the tests.
fn without_tests(src: &str) -> &str {
    let mut at = 0;
    let mut lines = src.split_inclusive('\n').peekable();
    while let Some(line) = lines.next() {
        if line.trim() == "#[cfg(test)]" && lines.peek().is_some_and(|next| next.contains("mod ") && next.trim_end().ends_with('{')) {
            return &src[..at];
        }

        at += line.len();
    }

    src
}

fn rel(path: &Path) -> String {
    path.strip_prefix(src_dir()).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

/// Every key the UI looks up, in the order the source has them, with the first place each is used.
fn source_keys() -> Vec<(Key, String)> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for path in rust_files(&src_dir()) {
        let rel = rel(&path);
        if rel.starts_with("i18n/") || rel.ends_with("/tests.rs") {
            continue;
        }

        let src = std::fs::read_to_string(&path).unwrap();
        for (key, line) in keys(without_tests(&src)) {
            if seen.insert(key.clone()) {
                out.push((key, format!("{rel}:{line}")));
            }
        }
    }

    out.extend(foreign_keys().into_iter().map(|k| (k, "foreign_keys".to_string())));
    out
}

/// Keys the scan cannot see because they are spelled in td_geom, td_doc and td_formats and translated where they are drawn.
fn foreign_keys() -> Vec<Key> {
    use td_doc::blend::{BlendMode, Falloff};
    use td_doc::scatter::{ScatterCollision, ScatterKind};
    use td_doc::terrain::SculptMode;
    use td_geom::Justify;

    let mut texts: Vec<&str> = Vec::new();
    texts.extend(Justify::ALL.iter().map(Justify::label));
    texts.extend(SculptMode::ALL.iter().map(SculptMode::label));
    texts.extend(BlendMode::ALL.iter().map(BlendMode::label));
    texts.extend(Falloff::ALL.iter().map(Falloff::label));
    texts.extend(ScatterKind::ALL.iter().map(ScatterKind::label));
    texts.extend(ScatterCollision::ALL.iter().flat_map(|c| [c.label(), c.tooltip()]));
    texts.extend(td_doc::issues::TEMPLATES);
    texts.extend(td_doc::scatter::PRESETS);
    texts.extend(crate::prefab_library::BUILTIN.iter().flat_map(|(category, name, _)| [*category, *name]));
    let mut seen = HashSet::new();
    texts.into_iter().filter(|k| seen.insert(*k)).map(|k| Key::Text(k.into())).collect()
}

#[test]
fn the_scanner_reads_every_literal_form() {
    let src = r##"
        const OPEN: &str = en!("Open \"Recent\"");
        fn f(ui: &mut Ui, n: usize) {
            ui.label(tr("Line\none"));
            ui.label(tr("Arrow \u{2192} next"));
            ui.label(tr(r#"Raw "quoted" text"#));
            ui.label(tr(r"Raw plain"));
            ui.label(tr("Long text \
                         continued"));
            ui.label(trf!(
                "Split {n}",
                n
            ));
            let m = msg!("Status {x}", x = 1);
            let c = tr_n!(items.iter().filter(|i| i.on).count(), "{n} brush", "{n} brushes", more = f(1, 2));
            let d = msg_n!(n, "{n} face", "{n} faces");
            let s = '"'; let q = '\''; let l: &'static str = "plain, not a key";
            // tr("Commented out")
            /* tr("Block comment") */
            attr("Attribute"); str("Str"); x.tr_name("No");
            tr(label);
            doc.edit("Undo label", |m| m); doc.edit_coalesced("Coalesced label", |m| m); doc.edit(&label, |m| m); edit("Free fn", 1);
            doc.begin("Drag label"); progress.begin(false, 3); begin("Free begin");
            Msg::filled("Filled {x}", &[("x", a, b)]); filled("Free filled", &[]);
            Msg::filled_n("{n} file{x}", "{n} files{x}", 2, &[]); filled_n("Free one", "Free other", 1, &[]);
            Msg::key(" from the beta"); key("Free key"); other::key("Other key");
        }
    "##;
    let found = keys(src);
    let text = |k: &str| Key::Text(k.into());
    let plural = |a: &str, b: &str| Key::Plural(a.into(), b.into());
    let want = [
        (text("Open \"Recent\""), 2),
        (text("Line\none"), 4),
        (text("Arrow → next"), 5),
        (text("Raw \"quoted\" text"), 6),
        (text("Raw plain"), 7),
        (text("Long text continued"), 8),
        (text("Split {n}"), 10),
        (text("Status {x}"), 14),
        (plural("{n} brush", "{n} brushes"), 15),
        (plural("{n} face", "{n} faces"), 16),
        (text("Undo label"), 22),
        (text("Coalesced label"), 22),
        (text("Drag label"), 23),
        (text("Filled {x}"), 24),
        (plural("{n} file{x}", "{n} files{x}"), 25),
        (text(" from the beta"), 26),
    ];
    assert_eq!(found, want);
    assert!(keys("").is_empty());
    assert_eq!(keys("tr(\"One\")"), [(text("One"), 1)]);
}

#[test]
fn test_modules_are_left_out_of_the_scan() {
    let src = "fn a() { tr(\"Kept\"); }\n#[cfg(test)]\nmod tests {\n    fn b() { tr(\"Test only\"); }\n}\n";
    assert_eq!(without_tests(src), "fn a() { tr(\"Kept\"); }\n");
    let external = "fn a() {}\n#[cfg(test)]\nmod tests;\nfn c() {}\n";
    assert_eq!(without_tests(external), external);
    assert!(source_keys().iter().all(|(k, _)| !k.id().is_empty()));
}

/// What a catalogue lacks for the source and what it holds that the source no longer looks up.
fn coverage(lang: Lang, source: &[(Key, String)], cat: &Catalogue) -> Vec<String> {
    let mut out = Vec::new();
    for (key, at) in source {
        match (key, cat.get(key.id())) {
            (_, None) => out.push(format!("{}: missing {:?} ({at})", lang.code(), key.id())),
            (Key::Text(_), Some(Entry::Plural(_))) => out.push(format!("{}: {:?} is not a plural ({at})", lang.code(), key.id())),
            (Key::Plural(..), Some(Entry::Text(_))) => out.push(format!("{}: {:?} needs plural forms ({at})", lang.code(), key.id())),
            _ => {}
        }
    }

    let known: HashSet<&str> = source.iter().map(|(k, _)| k.id()).collect();
    let mut orphans: Vec<&String> = cat.keys().filter(|k| !known.contains(k.as_str())).collect();
    orphans.sort();
    out.extend(orphans.into_iter().map(|k| format!("{}: {k:?} is no longer used", lang.code())));
    out
}

/// The missing keys of a language as a JSON file to translate, English values, plural keys with the language's forms.
fn todo_json(lang: Lang, source: &[(Key, String)], cat: &Catalogue) -> String {
    let q = |s: &str| serde_json::to_string(s).unwrap();
    let rows: Vec<String> = source
        .iter()
        .filter(|(k, _)| !cat.contains_key(k.id()))
        .map(|(key, _)| match key {
            Key::Text(k) => format!("  {}: {}", q(k), q(english(k))),
            Key::Plural(one, other) => {
                let forms: Vec<String> = lang.forms().iter().map(|f| format!("{}: {}", q(f.name()), q(if *f == Form::One { one } else { other }))).collect();
                format!("  {}: {{ {} }}", q(one), forms.join(", "))
            }
        })
        .collect();
    if rows.is_empty() { "{}\n".into() } else { format!("{{\n{}\n}}\n", rows.join(",\n")) }
}

#[test]
fn every_language_has_every_key() {
    let source = source_keys();
    let todo = std::env::var_os("TD_L10N_TODO").map(PathBuf::from);
    let mut problems = Vec::new();
    for lang in &Lang::ALL[1..] {
        let cat = shipped(*lang);
        problems.extend(coverage(*lang, &source, cat));
        if let Some(dir) = &todo {
            std::fs::write(dir.join(format!("{}.json", lang.code())), todo_json(*lang, &source, cat)).unwrap();
        }
    }

    assert!(problems.is_empty(), "{} problems, TD_L10N_TODO=<dir> writes the missing keys:\n{}", problems.len(), problems.join("\n"));
}

#[test]
fn coverage_finds_missing_orphan_and_wrong_kind_keys() {
    let source = vec![
        (Key::Text("Open".into()), "a.rs:1".into()),
        (Key::Text("Save".into()), "a.rs:2".into()),
        (Key::Plural("{n} brush".into(), "{n} brushes".into()), "b.rs:3".into()),
    ];
    let full = catalogue::parse(Lang::De, r#"{ "Open": "Öffnen", "Save": "Speichern", "{n} brush": { "one": "1", "other": "{n}" } }"#).unwrap();
    assert_eq!(coverage(Lang::De, &source, &full), Vec::<String>::new());
    let cat = catalogue::parse(Lang::De, r#"{ "Open": { "one": "a", "other": "b" }, "{n} brush": "x", "Gone": "Weg" }"#).unwrap();
    assert_eq!(
        coverage(Lang::De, &source, &cat),
        [
            "de: \"Open\" is not a plural (a.rs:1)",
            "de: missing \"Save\" (a.rs:2)",
            "de: \"{n} brush\" needs plural forms (b.rs:3)",
            "de: \"Gone\" is no longer used",
        ]
    );
    let empty = Catalogue::default();
    assert_eq!(todo_json(Lang::De, &[], &empty), "{}\n");
    let todo = todo_json(Lang::Ru, &source, &empty);
    let parsed: serde_json::Value = serde_json::from_str(&todo).unwrap();
    assert_eq!(parsed, serde_json::json!({ "Open": "Open", "Save": "Save", "{n} brush": { "one": "{n} brush", "few": "{n} brushes", "many": "{n} brushes" } }));
    assert!(todo.find("\"Open\"") < todo.find("\"Save\""), "source order");
    assert_eq!(todo_json(Lang::De, &source, &full), "{}\n");
}

fn placeholders(s: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = s;
    while let Some(at) = rest.find('{') {
        let tail = &rest[at..];
        if let Some(after) = tail.strip_prefix("{{") {
            rest = after;
            continue;
        }

        let Some(end) = tail.find('}') else { break };
        out.insert(tail[1..end].to_string());
        rest = &tail[end + 1..];
    }

    out
}

fn is_name(s: &str) -> bool {
    s.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_') && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// A placeholder a translator cannot move, `{}`, `{0}` or one with a format spec.
fn positional(key: &str) -> Vec<String> {
    placeholders(key).into_iter().filter(|p| !is_name(p)).collect()
}

#[test]
fn no_positional_placeholders() {
    assert_eq!(positional("{n} of {total}, {{literal}} {_x1}"), Vec::<String>::new());
    for (bad, want) in [("Deleted {} objects", ""), ("{0} of {1}", "0"), ("{:.1} m", ":.1"), ("{n:.1}", "n:.1"), ("{n:?}", "n:?")] {
        assert_eq!(positional(bad).first().map(String::as_str), Some(want), "{bad}");
    }

    let bad: Vec<String> = source_keys()
        .iter()
        .flat_map(|(k, at)| {
            let texts = match k {
                Key::Text(t) => vec![t.as_str()],
                Key::Plural(a, b) => vec![a.as_str(), b.as_str()],
            };
            texts.into_iter().filter(|t| !positional(t).is_empty()).map(move |t| format!("{at}: {t:?}"))
        })
        .collect();
    assert!(bad.is_empty(), "name every placeholder:\n{}", bad.join("\n"));
}

#[test]
fn the_macros_want_an_argument_for_every_placeholder() {
    use crate::i18n::fills_every_placeholder as fills;
    assert!(fills("Saved", &[]));
    assert!(fills("{{literal}} braces and a } alone", &[]));
    assert!(fills("{name} saved", &["name"]));
    assert!(fills("{classname} '{name}' created at {x_2}", &["x_2", "name", "classname"]));
    assert!(!fills("{classname} '{name}' created", &["name"]));
    assert!(!fills("{name} saved", &[]));
    assert!(!fills("{name} saved", &["nam"]));
    assert!(!fills("{nam} saved", &["name"]));
    assert!(!fills("{{{name}}}", &[]));
    assert!(fills("{{name}}", &[]));
    assert!(fills("{name", &[]), "an unclosed brace is no placeholder");
}

const KEPT_NAMES: [&str; 26] = [
    "Ctrl",
    "Cmd",
    "Shift",
    "Alt",
    "Esc",
    "F1",
    "F2",
    "F3",
    "F4",
    "F5",
    "F6",
    "F7",
    "F8",
    "F9",
    "F10",
    "F11",
    "F12",
    "TrenchDot",
    "Godot",
    "MCP",
    "FGD",
    "GDScript",
    "C#",
    "UV",
    ".tdm",
    "Blender",
];

/// Enter and Tab are also words ("New Tab"), they count as key names inside a shortcut like Ctrl+Enter.
const SHORTCUT_ONLY: [&str; 2] = ["Enter", "Tab"];

fn has_word(text: &str, word: &str, shortcut: bool) -> bool {
    text.match_indices(word).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + word.len()..].chars().next();
        let edge = |c: Option<char>| !c.is_some_and(|c| c.is_ascii_alphanumeric());
        edge(before) && edge(after) && (!shortcut || before == Some('+') || after == Some('+'))
    })
}

fn is_emoji(c: char) -> bool {
    matches!(c as u32, 0x1F000..=0x1FAFF | 0x2600..=0x27BF | 0xFE0F)
}

fn is_cjk(lang: Lang) -> bool {
    matches!(lang, Lang::ZhHans | Lang::Ja)
}

/// Length in characters with each placeholder counted as one.
fn shown_len(s: &str) -> usize {
    let names = placeholders(s);
    s.chars().count() + names.len() - names.iter().map(|n| n.chars().count() + 2).sum::<usize>()
}

/// What breaks the house style in one translation of `en`.
fn style_problems(lang: Lang, en: &str, value: &str) -> Vec<&'static str> {
    let mut out = Vec::new();
    if value.contains(['\u{2014}', '\u{2013}']) || value.contains(" - ") {
        out.push("a dash as punctuation");
    }

    if value.chars().any(|c| is_emoji(c) && !en.contains(c)) {
        out.push("an emoji");
    }

    let lead = |s: &str| s.len() - s.trim_start().len();
    let trail = |s: &str| s.len() - s.trim_end().len();
    // A clause that joins on in English takes the separator of its own language in Chinese and Japanese.
    let own_separator = is_cjk(lang) && value.starts_with(['，', '、']);
    if (!own_separator && en[..lead(en)] != value[..lead(value)]) || en[en.len() - trail(en)..] != value[value.len() - trail(value)..] {
        out.push("leading or trailing space unlike English");
    }

    if en.ends_with('…') && !value.ends_with('…') {
        out.push("the trailing … is gone");
    }

    if KEPT_NAMES.iter().any(|n| has_word(en, n, false) && !value.contains(n)) || SHORTCUT_ONLY.iter().any(|n| has_word(en, n, true) && !value.contains(n)) {
        out.push("a key or product name is translated");
    }

    let words = en.split_whitespace().filter(|w| !w.starts_with('{') && w.chars().any(char::is_alphabetic)).count();
    if words >= 2 && value == en {
        out.push("untranslated");
    }

    let (en_len, len) = (shown_len(en), shown_len(value));
    let too_long = if is_cjk(lang) { len > en_len.max(4) } else { len * 2 > en_len * 3 + 20 };
    if too_long {
        out.push("too long");
    }

    out
}

/// Placeholders of `value` that differ from its English, for one text or one plural form.
fn placeholder_problems(lang: Lang, form: Option<Form>, en: &BTreeSet<String>, value: &str) -> Option<String> {
    let mut want = en.clone();
    let only_one = form == Some(Form::One) && (0..=1000).filter(|n| lang.plural(*n) == Form::One).eq([1]);
    let got = placeholders(value);
    if only_one && !got.contains("n") {
        want.remove("n");
    }

    (got != want).then(|| format!("has {got:?}, English {want:?}"))
}

fn translation_problems(lang: Lang, cat: &Catalogue) -> Vec<String> {
    let mut out = Vec::new();
    let mut keys: Vec<&String> = cat.keys().collect();
    keys.sort();
    for key in keys {
        let en_text = english(key);
        let mut check = |form: Option<Form>, en: &str, en_names: &BTreeSet<String>, value: &str| {
            let at = form.map_or(String::new(), |f| format!(" {}", f.name()));
            if let Some(p) = placeholder_problems(lang, form, en_names, value) {
                out.push(format!("{} {key:?}{at}: {p}", lang.code()));
            }

            for p in style_problems(lang, en, value) {
                out.push(format!("{} {key:?}{at}: {p}: {value:?}", lang.code()));
            }
        };
        match &cat[key] {
            Entry::Text(value) => check(None, en_text, &placeholders(en_text), value),
            Entry::Plural(forms) => {
                let mut names = placeholders(en_text);
                let counts_in_text = names.contains("n");
                names.insert("n".into());
                for (form, value) in forms {
                    let mut want = names.clone();
                    if !counts_in_text && !placeholders(value).contains("n") {
                        want.remove("n");
                    }

                    check(Some(*form), en_text, &want, value);
                }
            }
        }
    }

    out
}

#[test]
fn translations_keep_placeholders_and_house_style() {
    let problems: Vec<String> = Lang::ALL.iter().flat_map(|l| translation_problems(*l, shipped(*l))).collect();
    assert!(problems.is_empty(), "{}", problems.join("\n"));

    let bad = style_problems;
    assert_eq!(bad(Lang::De, "Open the map", "Karte öffnen"), Vec::<&str>::new());
    assert_eq!(bad(Lang::De, "Snap to grid", "Am Raster einrasten \u{2014} sofort"), ["a dash as punctuation"]);
    assert_eq!(bad(Lang::De, "Snap to grid", "Einrasten \u{2013} Raster"), ["a dash as punctuation"]);
    assert_eq!(bad(Lang::De, "Snap to grid", "Einrasten - Raster"), ["a dash as punctuation"]);
    assert_eq!(bad(Lang::De, "Snap to grid", "Am Raster-Gitter einrasten"), Vec::<&str>::new(), "a hyphen inside a word is fine");
    assert_eq!(bad(Lang::De, "Snap to grid", "Einrasten \u{1F44D}"), ["an emoji"]);
    assert_eq!(bad(Lang::De, "Done \u{2705}", "Fertig \u{2705}"), Vec::<&str>::new(), "an icon English has");
    assert_eq!(bad(Lang::De, "Name: ", "Name:"), ["leading or trailing space unlike English"]);
    assert_eq!(bad(Lang::De, "  Name", "Name"), ["leading or trailing space unlike English"]);
    assert_eq!(bad(Lang::Ja, " with a warning", "、警告あり"), Vec::<&str>::new(), "a joined clause may start with its own separator");
    assert_eq!(bad(Lang::ZhHans, " with a warning", "，有警告"), Vec::<&str>::new());
    assert_eq!(bad(Lang::ZhHans, " with a warning", "有警告"), ["leading or trailing space unlike English"]);
    assert_eq!(bad(Lang::De, " with a warning", "，mit Warnung"), ["leading or trailing space unlike English"]);
    assert_eq!(bad(Lang::Ja, " with a warning", "、警告あり "), ["leading or trailing space unlike English"]);
    assert_eq!(bad(Lang::De, "Save As…", "Speichern unter"), ["the trailing … is gone"]);
    assert_eq!(bad(Lang::De, "Press Ctrl+S to save", "Strg+S drücken zum Speichern"), ["a key or product name is translated"]);
    assert_eq!(bad(Lang::Ja, "Build the map in Godot", "ゴドーでビルド"), ["a key or product name is translated"]);
    assert_eq!(bad(Lang::Ja, "Build the map in Godot", "Godotでビルド"), Vec::<&str>::new());
    assert_eq!(bad(Lang::De, "Ctrl+Enter runs it", "Ctrl+Eingabe führt es aus"), ["a key or product name is translated"]);
    assert_eq!(bad(Lang::De, "Close Tab", "Reiter schließen"), Vec::<&str>::new(), "a tab, not the key");
    assert_eq!(bad(Lang::De, "Open the map", "Open the map"), ["untranslated"]);
    assert_eq!(bad(Lang::De, "Mesh", "Mesh"), Vec::<&str>::new(), "one word may stay");
    assert_eq!(bad(Lang::De, "{n} Godot", "{n} Godot"), Vec::<&str>::new(), "a placeholder is no word");
    let en = "Open the map now"; // 16 chars, so 34 is the most for Latin and Cyrillic
    assert_eq!(bad(Lang::Ru, en, &"я".repeat(34)), Vec::<&str>::new());
    assert_eq!(bad(Lang::Ru, en, &"я".repeat(35)), ["too long"]);
    assert_eq!(bad(Lang::Ja, en, &"開".repeat(16)), Vec::<&str>::new());
    assert_eq!(bad(Lang::Ja, en, &"開".repeat(17)), ["too long"]);
    assert_eq!(bad(Lang::Ja, "Cut", "切り取り"), Vec::<&str>::new(), "a very short text gets four");
    assert_eq!(bad(Lang::Ja, "Cut", "切り取りる"), ["too long"]);

    let names = |s: &str| placeholders(s);
    assert_eq!(placeholder_problems(Lang::De, None, &names("{a} of {b}"), "{b} von {a}"), None);
    assert!(placeholder_problems(Lang::De, None, &names("{a} of {b}"), "{a} von {c}").is_some());
    assert!(placeholder_problems(Lang::De, None, &names("{a}"), "{a} {{x}}").is_none(), "escaped braces are text");
    assert!(placeholder_problems(Lang::De, None, &names("{a}"), "keins").is_some());
    let n = names("{n} brush");
    assert_eq!(placeholder_problems(Lang::De, Some(Form::One), &n, "ein Brush"), None, "de one is only ever 1");
    assert_eq!(placeholder_problems(Lang::Es, Some(Form::One), &n, "un brush"), None);
    assert!(placeholder_problems(Lang::De, Some(Form::Other), &n, "Brushes").is_some());
    assert!(placeholder_problems(Lang::Fr, Some(Form::One), &n, "un brush").is_some(), "fr one also covers 0");
    assert!(placeholder_problems(Lang::Ru, Some(Form::One), &n, "один браш").is_some(), "ru one also covers 21");

    let cat = catalogue::parse(Lang::De, r#"{ "Deleted {n} objects": "Objekte gelöscht", "{n} brush": { "one": "ein Brush", "other": "Brushes" } }"#).unwrap();
    assert_eq!(
        translation_problems(Lang::De, &cat),
        ["de \"Deleted {n} objects\": has {}, English {\"n\"}", "de \"{n} brush\" other: has {}, English {\"n\"}",]
    );

    let cat = catalogue::parse(
        Lang::Fr,
        r#"{ "Add the class to {fgd}": { "one": "Ajoutez la classe à {fgd}", "other": "Ajoutez les {n} classes à {fgd}" }, "Add the classes to {fgd}": { "one": "Ajoutez à {x}", "other": "Ajoutez à {fgd}" } }"#,
    )
    .unwrap();
    assert_eq!(
        translation_problems(Lang::Fr, &cat),
        ["fr \"Add the classes to {fgd}\" one: has {\"x\"}, English {\"fgd\"}"],
        "the count may be left out of a text that does not hold it"
    );
}

/// Lines of `src` that look up translated text.
fn ui_lookups(src: &str) -> Vec<usize> {
    let t = lex(src);
    let mut out = Vec::new();
    for (i, (tok, line)) in t.iter().enumerate() {
        let Tok::Ident(name) = tok else { continue };
        let call = match name.as_str() {
            "tr" => is(&t, i + 1, &Tok::Punct('(')),
            "trf" | "tr_n" => is(&t, i + 1, &Tok::Punct('!')),
            _ => false,
        };
        if call && !is(&t, i.wrapping_sub(1), &Tok::Ident("fn".into())) {
            out.push(*line);
        }
    }

    out
}

#[test]
fn mcp_code_stays_english() {
    assert_eq!(ui_lookups("let a = tr(\"x\");\nlet b = i18n::trf!(\"y {z}\", z);\nlet c = crate::tr_n!(n, \"a\", \"b\");"), [1, 2, 3]);
    assert_eq!(ui_lookups("let a = attr(\"x\"); let m = msg!(\"x\"); fn tr(k: &str) {} // tr(\"x\")\nlet s = \"tr(x)\";"), Vec::<usize>::new());
    let mut found = Vec::new();
    for path in rust_files(&src_dir().join("mcp")) {
        let src = std::fs::read_to_string(&path).unwrap();
        found.extend(ui_lookups(&src).into_iter().map(|line| format!("{}:{line}", rel(&path))));
    }

    assert!(found.is_empty(), "MCP replies stay English, use en!, msg! or the English text:\n{}", found.join("\n"));
}

/// Files whose UI text all goes through the lookup, so a raw literal there is text someone forgot to translate.
const CONVERTED: &[&str] = &[
    "dialogs.rs",
    "help.rs",
    "tools.rs",
    "welcome.rs",
    "icons.rs",
    "toolset.rs",
    "mesh_tool.rs",
    "viewport.rs",
    "gizmos.rs",
    "transform_gizmo.rs",
    "extra_tools.rs",
    "walkable.rs",
    "overlays.rs",
    "texture_ops.rs",
    "texture_tool.rs",
    "blend_tool.rs",
    "volume_tool.rs",
    "scatter_tool.rs",
    "uv_editor.rs",
    "hotspot_editor.rs",
    "trim.rs",
    "texture_convert.rs",
    "prefab_panel.rs",
    "prefab_library.rs",
    "random_fill.rs",
    "scatter_panel.rs",
    "content_wizard.rs",
    "addon_wizard.rs",
    "addon_install.rs",
    "entity_pack.rs",
    "entity_wizards.rs",
    "trust.rs",
    "content.rs",
    "models.rs",
    "materials.rs",
    "model_thumbs.rs",
    "code_scan.rs",
];

/// Functions of a file that is still being converted, as (file, function name).
const CONVERTED_FNS: &[(&str, &str)] = &[
    ("app.rs", "menu_bar"),
    ("app.rs", "help_menu_links"),
    ("app.rs", "toolbar"),
    ("app.rs", "godot_buttons"),
    ("app.rs", "tool_options"),
    ("app.rs", "tab_bar"),
    ("app.rs", "status_bar"),
    ("app.rs", "stats_text"),
    ("app.rs", "theme_pref"),
    ("app.rs", "ui_scale_prefs"),
    ("app.rs", "prefs_window"),
    ("app.rs", "mcp_prefs"),
    ("app.rs", "close_dialog"),
    ("app.rs", "close_tab_dialog"),
    ("app.rs", "tab_title"),
    ("panels.rs", "outliner"),
    ("panels.rs", "outliner_tooltip"),
    ("panels.rs", "inspector"),
    ("panels.rs", "mesh_inspector"),
    ("panels.rs", "section_help"),
    ("panels.rs", "sub_heading"),
    ("panels.rs", "bounds_rows"),
    ("panels.rs", "brush_entity_section"),
    ("panels.rs", "scatter_inspector"),
    ("panels.rs", "terrain_inspector"),
    ("panels.rs", "selection_summary"),
    ("panels.rs", "worldspawn_inspector"),
    ("panels.rs", "add_property_row"),
    ("panels.rs", "entity_inspector"),
    ("panels.rs", "io_editor"),
    ("panels.rs", "face_inspector"),
    ("panels.rs", "texture_settings"),
    ("panels.rs", "material_summary_with"),
    ("panels.rs", "material_cell"),
    ("panels.rs", "material_interactions"),
    ("panels.rs", "material_browser"),
    ("panels.rs", "model_hover_text"),
    ("panels.rs", "model_browser"),
    ("panels.rs", "dnd_preview"),
    ("panels.rs", "entity_card"),
    ("panels.rs", "entity_browser"),
    ("panels.rs", "issues"),
    ("panels.rs", "reference"),
    ("panels.rs", "reference_detail"),
    ("panels.rs", "history"),
];

/// The lines of function `name`, rustfmt closes it with a brace at the indent of its `fn`, and the number of the line
/// before the first one.
fn fn_lines<'a>(src: &'a str, name: &str) -> Option<(usize, Vec<&'a str>)> {
    let lines: Vec<&str> = src.lines().collect();
    let start = lines.iter().position(|l| {
        let l = l.trim_start();
        l.strip_prefix("pub ").unwrap_or(l).starts_with(&format!("fn {name}("))
    })?;
    let indent = &lines[start][..lines[start].len() - lines[start].trim_start().len()];
    let end = lines[start..].iter().position(|l| l.strip_prefix(indent) == Some("}"))? + start;
    Some((start, lines[start..=end].to_vec()))
}

const TEXT_METHODS: [&str; 20] = [
    "button",
    "small_button",
    "label",
    "heading",
    "strong",
    "weak",
    "small",
    "checkbox",
    "radio",
    "radio_value",
    "selectable_label",
    "selectable_value",
    "toggle_value",
    "menu_button",
    "on_hover_text",
    "on_disabled_hover_text",
    "hint_text",
    "collapsing",
    "colored_label",
    "hyperlink_to",
];

const TEXT_TYPES: [(&str, &str); 8] = [
    ("RichText", "new"),
    ("Button", "new"),
    ("Label", "new"),
    ("Checkbox", "new"),
    ("SelectableLabel", "new"),
    ("CollapsingHeader", "new"),
    ("Window", "new"),
    ("ComboBox", "from_label"),
];

/// Prose has two letters or more outside its placeholders, so "X" or "{n}" do not count.
fn is_prose(s: &str) -> bool {
    let mut letters = 0;
    let mut depth = 0;
    for c in s.chars() {
        match c {
            '{' => depth += 1,
            '}' => depth -= 1,
            c if depth == 0 && c.is_alphabetic() => letters += 1,
            _ => {}
        }
    }

    letters >= 2
}

/// Widget calls in `src` that are handed English prose directly, a literal or a `format!`.
fn raw_ui_text(src: &str) -> Vec<(usize, String)> {
    let t = lex(src);
    let mut out = Vec::new();
    for (i, (tok, _)) in t.iter().enumerate() {
        let Tok::Ident(name) = tok else { continue };
        let method = TEXT_METHODS.contains(&name.as_str()) && is(&t, i.wrapping_sub(1), &Tok::Punct('.'));
        let ctor =
            TEXT_TYPES.iter().any(|(ty, f)| name == f && is(&t, i.wrapping_sub(1), &Tok::Punct(':')) && is(&t, i.wrapping_sub(3), &Tok::Ident(ty.to_string())));
        if !(method || ctor) || !is(&t, i + 1, &Tok::Punct('(')) {
            continue;
        }

        let mut at = i + 2;
        while at < t.len() && !is(&t, at, &Tok::Punct(')')) {
            let end = skip_arg(&t, at);
            let text = match &t[at..end] {
                [(Tok::Str(s), _)] => Some(s),
                [(Tok::Ident(f), _), (Tok::Punct('!'), _), (Tok::Punct('('), _), (Tok::Str(s), _), ..] if f == "format" => Some(s),
                _ => None,
            };
            if let Some(s) = text.filter(|s| is_prose(s)) {
                out.push((t[at].1, s.clone()));
            }

            at = end + usize::from(is(&t, end, &Tok::Punct(',')));
        }
    }

    out
}

#[test]
fn converted_files_have_no_raw_ui_text() {
    let lines = |src: &str| raw_ui_text(src).into_iter().map(|(l, s)| format!("{l} {s}")).collect::<Vec<_>>();
    assert_eq!(lines("ui.button(\"Save\");"), ["1 Save"]);
    assert_eq!(lines("ui.button(tr(\"Save\")); ui.label(\"X\"); ui.label(\"{n}\"); ui.label(name);"), Vec::<String>::new());
    assert_eq!(lines("ui.checkbox(&mut s.snap, \"Snap to grid\")\n.on_hover_text(\"Snaps\");"), ["1 Snap to grid", "2 Snaps"]);
    assert_eq!(lines("ui.checkbox(&mut s.snap, tr(\"Snap to grid\"));"), Vec::<String>::new());
    assert_eq!(lines("ui.label(format!(\"{n} brushes\", n = 3)); ui.label(format!(\"{n}\"));"), ["1 {n} brushes"]);
    assert_eq!(
        lines("ui.add(egui::Button::new(\"Bake\")); RichText::new(\"Hi there\"); egui::ComboBox::from_label(\"Mode\");"),
        ["1 Bake", "1 Hi there", "1 Mode"]
    );
    assert_eq!(lines("ui.selectable_value(&mut v, Mode::A, \"Alpha\"); x.label_width(\"Wide\"); Foo::new(\"Text\");"), ["1 Alpha"]);
    assert_eq!(lines("ui.label(trf!(\"{n} brushes\", n)); ui.label(fmt(\"ab\", \"cd\"));"), Vec::<String>::new());

    let mut found = Vec::new();
    for file in CONVERTED {
        let src = std::fs::read_to_string(src_dir().join(file)).unwrap();
        found.extend(raw_ui_text(without_tests(&src)).into_iter().map(|(line, s)| format!("{file}:{line} {s:?}")));
    }

    let src = "fn a() {\n    ui.label(\"Outside\");\n}\n\nimpl X {\n    fn menu_bar(&mut self) {\n        ui.label(\"Inside\");\n        ui.label(tr(\"Done\"));\n    }\n\n    fn b() {\n        ui.label(\"After\");\n    }\n}\n\npub fn shown(ui: &mut Ui) {\n    ui.label(\"Pub\");\n}\n";
    let (start, body) = fn_lines(src, "menu_bar").unwrap();
    assert_eq!((start, body.len()), (5, 4));
    assert!(fn_lines(src, "missing").is_none());
    assert_eq!(raw_ui_text(&body.join("\n")), [(2, "Inside".to_string())]);
    let (_, body) = fn_lines(src, "shown").unwrap();
    assert_eq!(raw_ui_text(&body.join("\n")), [(2, "Pub".to_string())], "a pub fn is found too");

    for (file, name) in CONVERTED_FNS {
        let src = std::fs::read_to_string(src_dir().join(file)).unwrap();
        let (start, body) = fn_lines(without_tests(&src), name).unwrap_or_else(|| panic!("{file} has no fn {name}"));
        found.extend(raw_ui_text(&body.join("\n")).into_iter().map(|(line, s)| format!("{file}:{} {s:?}", start + line)));
    }

    assert!(found.is_empty(), "wrap UI text in tr, trf!, tr_n! or en!:\n{}", found.join("\n"));
}

#[test]
fn status_keeps_english_beside_the_translation() {
    use crate::state::{EditorState, Prefs};
    let _de = test_catalogue(Lang::De, r#"{ "Nothing to undo": "Nichts rückgängig zu machen" }"#);
    let mut state = EditorState::new(Prefs::default());
    state.set_status(msg!("Nothing to undo"));
    assert_eq!((state.status.as_str(), state.status_shown()), ("Nothing to undo", "Nothing to undo"), "English");

    let _s = scope(Lang::De);
    state.fail_status(msg!("Nothing to undo"));
    assert_eq!((state.status.as_str(), state.status_shown()), ("Nothing to undo", "Nichts rückgängig zu machen"));
    assert!(state.status_failed());
    state.status = "x".into();
    assert_eq!(state.status_shown(), "x", "a direct write never shows the old translation");
    state.set_status(format!("Deleted {} objects", 3));
    assert_eq!((state.status.as_str(), state.status_shown()), ("Deleted 3 objects", "Deleted 3 objects"));
    assert!(!state.status_failed());
}
