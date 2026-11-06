//! Text of the editor UI in the user's language. The English text is the key and the fallback, the translations are
//! in `crates/td_editor/locales`, see the README there.
//!
//! MCP replies, logs and map content stay English. Code under `src/mcp` never calls [`tr`], [`trf!`] or [`tr_n!`].

mod catalogue;
#[cfg(test)]
mod tests;

use std::cell::Cell;
use std::fmt::{Display, Write};
use std::sync::OnceLock;

pub use crate::{en, msg, msg_n, tr_n, trf};
use catalogue::{Catalogue, Entry};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Lang {
    #[default]
    En,
    De,
    Fr,
    Es,
    Pt,
    Ru,
    ZhHans,
    Ja,
}

impl Lang {
    pub const ALL: [Lang; 8] = [Lang::En, Lang::De, Lang::Fr, Lang::Es, Lang::Pt, Lang::Ru, Lang::ZhHans, Lang::Ja];

    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::De => "de",
            Lang::Fr => "fr",
            Lang::Es => "es",
            Lang::Pt => "pt",
            Lang::Ru => "ru",
            Lang::ZhHans => "zh-Hans",
            Lang::Ja => "ja",
        }
    }

    pub fn from_code(code: &str) -> Option<Lang> {
        Lang::ALL.into_iter().find(|l| l.code().eq_ignore_ascii_case(code))
    }

    /// The language's name in itself, as the language picker lists it.
    pub fn name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::De => "Deutsch",
            Lang::Fr => "Français",
            Lang::Es => "Español",
            Lang::Pt => "Português",
            Lang::Ru => "Русский",
            Lang::ZhHans => "简体中文",
            Lang::Ja => "日本語",
        }
    }

    fn source(self) -> &'static str {
        match self {
            Lang::En => "{}",
            Lang::De => include_str!("../../locales/de.json"),
            Lang::Fr => include_str!("../../locales/fr.json"),
            Lang::Es => include_str!("../../locales/es.json"),
            Lang::Pt => include_str!("../../locales/pt.json"),
            Lang::Ru => include_str!("../../locales/ru.json"),
            Lang::ZhHans => include_str!("../../locales/zh-Hans.json"),
            Lang::Ja => include_str!("../../locales/ja.json"),
        }
    }

    /// The plural forms a translation of a counted text holds, by the CLDR integer rules.
    pub(crate) fn forms(self) -> &'static [Form] {
        match self {
            Lang::En | Lang::De | Lang::Fr | Lang::Es | Lang::Pt => &[Form::One, Form::Other],
            Lang::Ru => &[Form::One, Form::Few, Form::Many],
            Lang::ZhHans | Lang::Ja => &[Form::Other],
        }
    }

    pub(crate) fn plural(self, n: u64) -> Form {
        match self {
            Lang::En | Lang::De | Lang::Es if n == 1 => Form::One,
            Lang::Fr | Lang::Pt if n <= 1 => Form::One,
            Lang::Ru => match (n % 10, n % 100) {
                (1, r) if r != 11 => Form::One,
                (2..=4, r) if !(12..=14).contains(&r) => Form::Few,
                _ => Form::Many,
            },
            _ => Form::Other,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Form {
    One,
    Few,
    Many,
    Other,
}

impl Form {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Form::One => "one",
            Form::Few => "few",
            Form::Many => "many",
            Form::Other => "other",
        }
    }
}

thread_local! {
    // Per thread, so tests on parallel threads can each pick a language.
    static CURRENT: Cell<Lang> = const { Cell::new(Lang::En) };
}

pub fn current() -> Lang {
    CURRENT.with(Cell::get)
}

pub fn set(lang: Lang) {
    CURRENT.with(|c| c.set(lang));
}

/// Switches this thread to `lang` until the returned guard drops. A worker thread that words text for the user takes
/// [`current`] along and opens a scope inside.
pub fn scope(lang: Lang) -> Scope {
    let previous = current();
    set(lang);
    Scope { previous }
}

#[must_use = "the language switches back when the scope drops"]
pub struct Scope {
    previous: Lang,
}

impl Drop for Scope {
    fn drop(&mut self) {
        set(self.previous);
    }
}

fn shipped(lang: Lang) -> &'static Catalogue {
    static LOADED: [OnceLock<Catalogue>; Lang::ALL.len()] = [const { OnceLock::new() }; Lang::ALL.len()];
    LOADED[lang as usize].get_or_init(|| {
        catalogue::parse(lang, lang.source()).unwrap_or_else(|e| {
            eprintln!("The {} translation does not load, showing English: {e}", lang.code());
            Catalogue::default()
        })
    })
}

fn lookup(lang: Lang, key: &str) -> Option<&'static Entry> {
    #[cfg(test)]
    if let Some(c) = OVERRIDES.with(|o| o.borrow().iter().rev().find(|(l, _)| *l == lang).map(|(_, c)| *c)) {
        return c.get(key);
    }

    shipped(lang).get(key)
}

#[cfg(test)]
thread_local! {
    static OVERRIDES: std::cell::RefCell<Vec<(Lang, &'static Catalogue)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Uses `json` in place of the shipped translations of `lang` on this thread until the guard drops, so a test can
/// check translated text before its key is in the shipped files.
#[cfg(test)]
pub(crate) fn test_catalogue(lang: Lang, json: &str) -> TestCatalogue {
    let parsed = catalogue::parse(lang, json).unwrap_or_else(|e| panic!("test catalogue: {e}"));
    OVERRIDES.with(|o| o.borrow_mut().push((lang, Box::leak(Box::new(parsed)))));
    TestCatalogue
}

#[cfg(test)]
#[must_use = "the shipped translations come back when the guard drops"]
pub(crate) struct TestCatalogue;

#[cfg(test)]
impl Drop for TestCatalogue {
    fn drop(&mut self) {
        OVERRIDES.with(|o| o.borrow_mut().pop());
    }
}

/// The English text of a key, without its `#context` suffix.
#[doc(hidden)]
pub fn english(key: &str) -> &str {
    match key.rfind('#') {
        Some(at) if at > 0 && !key[..at].ends_with(char::is_whitespace) && is_context(&key[at + 1..]) => &key[..at],
        _ => key,
    }
}

fn is_context(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
}

/// The text of `key` in the current language, or its English text when there is no translation.
pub fn tr(key: &str) -> &str {
    let lang = current();
    if lang != Lang::En
        && let Some(Entry::Text(t)) = lookup(lang, key)
    {
        return t;
    }

    english(key)
}

/// The translation of `text` when it is a key, else `text` as it is, for text built at run time where a `#` is no
/// context.
pub fn tr_known(text: &str) -> &str {
    let lang = current();
    if lang != Lang::En
        && let Some(Entry::Text(t)) = lookup(lang, text)
    {
        return t;
    }

    text
}

/// The form of a counted text for `n`, keyed by the English `one` text. [`tr_n!`] fills in the count.
pub fn tr_plural(one: &'static str, other: &'static str, n: u64) -> &'static str {
    let lang = current();
    if lang != Lang::En
        && let Some(Entry::Plural(forms)) = lookup(lang, one)
    {
        let form = lang.plural(n);
        if let Some((_, t)) = forms.iter().find(|(f, _)| *f == form) {
            return t;
        }
    }

    english(if n == 1 { one } else { other })
}

#[doc(hidden)]
pub fn english_plural(one: &'static str, other: &'static str, n: u64) -> &'static str {
    english(if n == 1 { one } else { other })
}

#[doc(hidden)]
pub fn count<T: TryInto<u64>>(n: T) -> u64 {
    n.try_into().unwrap_or(u64::MAX)
}

/// Replaces each `{name}` in `template` with its value from `args`. `{{` and `}}` are literal braces, a value is never
/// expanded again and a placeholder without a value stays as it is, so a missing argument shows.
pub fn fill(template: &str, args: &[(&str, &dyn Display)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(at) = rest.find(['{', '}']) {
        out.push_str(&rest[..at]);
        let tail = &rest[at..];
        if tail.starts_with("{{") || tail.starts_with("}}") {
            out.push_str(&tail[..1]);
            rest = &tail[2..];
            continue;
        }

        if let Some(end) = tail.find('}').filter(|_| tail.starts_with('{'))
            && let Some((_, value)) = args.iter().find(|(name, _)| *name == &tail[1..end])
        {
            let _ = write!(out, "{value}");
            rest = &tail[end + 1..];
            continue;
        }

        out.push_str(&tail[..1]);
        rest = &tail[1..];
    }

    out.push_str(rest);
    out
}

/// Whether every `{name}` in `key` is one of `names`, so the macros reject a placeholder that would show unfilled.
#[doc(hidden)]
pub const fn fills_every_placeholder(key: &str, names: &[&str]) -> bool {
    let k = key.as_bytes();
    let mut i = 0;
    while i < k.len() {
        if k[i] == b'{' && i + 1 < k.len() && k[i + 1] == b'{' {
            i += 2;
            continue;
        }

        if k[i] == b'{' {
            let start = i + 1;
            let mut end = start;
            while end < k.len() && (k[end].is_ascii_alphanumeric() || k[end] == b'_') {
                end += 1;
            }

            if end > start && end < k.len() && k[end] == b'}' && !names_hold(names, k, start, end) {
                return false;
            }
        }

        i += 1;
    }

    true
}

const fn names_hold(names: &[&str], k: &[u8], start: usize, end: usize) -> bool {
    let mut n = 0;
    while n < names.len() {
        let name = names[n].as_bytes();
        if name.len() == end - start {
            let mut j = 0;
            while j < name.len() && name[j] == k[start + j] {
                j += 1;
            }

            if j == name.len() {
                return true;
            }
        }

        n += 1;
    }

    false
}

/// Joins names the way the current language lists them, "A, B and C".
pub fn list(items: &[impl AsRef<str>]) -> String {
    let (sep, and) = match current() {
        Lang::En => (", ", " and "),
        Lang::De => (", ", " und "),
        Lang::Fr => (", ", " et "),
        Lang::Es => (", ", " y "),
        Lang::Pt => (", ", " e "),
        Lang::Ru => (", ", " и "),
        Lang::ZhHans => ("、", "和"),
        Lang::Ja => ("、", "、"),
    };
    match items {
        [] => String::new(),
        [one] => one.as_ref().to_string(),
        [first @ .., last] => {
            let first: Vec<&str> = first.iter().map(AsRef::as_ref).collect();
            format!("{}{and}{}", first.join(sep), last.as_ref())
        }
    }
}

/// A message in English, for MCP, tests and logs, and in the user's language, for the status bar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Msg {
    pub en: String,
    pub shown: String,
}

impl From<String> for Msg {
    /// A key passed on as a `String`, like a fixed error that MCP reads too, shows translated.
    fn from(text: String) -> Self {
        Msg { shown: tr_known(&text).to_string(), en: text }
    }
}

impl From<&String> for Msg {
    fn from(text: &String) -> Self {
        text.clone().into()
    }
}

impl From<&str> for Msg {
    fn from(text: &str) -> Self {
        text.to_string().into()
    }
}

/// Marks an English literal as a key that is translated where it is drawn, works in `const` items too.
#[macro_export]
macro_rules! en {
    ($key:literal) => {
        $key
    };
}

/// `trf!("Deleted {n} objects", n = count, path)` translates the key and fills its placeholders, `path` alone is
/// short for `path = path`.
#[macro_export]
macro_rules! trf {
    ($key:literal $(, $($args:tt)*)?) => {
        $crate::__i18n!(trf ($key) [] $($($args)*)?)
    };
}

/// `tr_n!(count, "{n} brush", "{n} brushes", more = x)` picks the plural form for `count` and fills `{n}` with it.
#[macro_export]
macro_rules! tr_n {
    ($n:expr, $one:literal, $other:literal $(, $($args:tt)*)?) => {
        $crate::__i18n!(tr_n ($n, $one, $other) [] $($($args)*)?)
    };
}

/// Like [`trf!`], returns a [`Msg`](crate::i18n::Msg) that keeps the English text beside the translation.
#[macro_export]
macro_rules! msg {
    ($key:literal $(, $($args:tt)*)?) => {
        $crate::__i18n!(msg ($key) [] $($($args)*)?)
    };
}

/// Like [`tr_n!`], returns a [`Msg`](crate::i18n::Msg).
#[macro_export]
macro_rules! msg_n {
    ($n:expr, $one:literal, $other:literal $(, $($args:tt)*)?) => {
        $crate::__i18n!(msg_n ($n, $one, $other) [] $($($args)*)?)
    };
}

// Gathers `name = value` and `name` arguments, then binds every value once so msg! fills both texts from the same ones.
#[doc(hidden)]
#[macro_export]
macro_rules! __i18n {
    ($mode:ident $keys:tt [$($done:tt)*] $name:ident = $value:expr $(, $($rest:tt)*)?) => {
        $crate::__i18n!($mode $keys [$($done)* ($name, $value)] $($($rest)*)?)
    };
    ($mode:ident $keys:tt [$($done:tt)*] $name:ident $(, $($rest:tt)*)?) => {
        $crate::__i18n!($mode $keys [$($done)* ($name, $name)] $($($rest)*)?)
    };
    (trf ($key:literal) [$(($name:ident, $value:expr))*]) => {{
        $crate::__i18n!(check [$key] [$(stringify!($name)),*]);
        let args: &[(&str, &dyn ::std::fmt::Display)] = &[$((stringify!($name), &$value as &dyn ::std::fmt::Display)),*];
        $crate::i18n::fill($crate::i18n::tr($key), args)
    }};
    (msg ($key:literal) [$(($name:ident, $value:expr))*]) => {{
        $crate::__i18n!(check [$key] [$(stringify!($name)),*]);
        let args: &[(&str, &dyn ::std::fmt::Display)] = &[$((stringify!($name), &$value as &dyn ::std::fmt::Display)),*];
        $crate::i18n::Msg {
            en: $crate::i18n::fill($crate::i18n::english($key), args),
            shown: $crate::i18n::fill($crate::i18n::tr($key), args),
        }
    }};
    (tr_n ($n:expr, $one:literal, $other:literal) [$(($name:ident, $value:expr))*]) => {{
        $crate::__i18n!(check [$one, $other] ["n" $(, stringify!($name))*]);
        let n = $n;
        let args: &[(&str, &dyn ::std::fmt::Display)] =
            &[("n", &n as &dyn ::std::fmt::Display) $(, (stringify!($name), &$value as &dyn ::std::fmt::Display))*];
        $crate::i18n::fill($crate::i18n::tr_plural($one, $other, $crate::i18n::count(n)), args)
    }};
    (msg_n ($n:expr, $one:literal, $other:literal) [$(($name:ident, $value:expr))*]) => {{
        $crate::__i18n!(check [$one, $other] ["n" $(, stringify!($name))*]);
        let n = $n;
        let args: &[(&str, &dyn ::std::fmt::Display)] =
            &[("n", &n as &dyn ::std::fmt::Display) $(, (stringify!($name), &$value as &dyn ::std::fmt::Display))*];
        $crate::i18n::Msg {
            en: $crate::i18n::fill($crate::i18n::english_plural($one, $other, $crate::i18n::count(n)), args),
            shown: $crate::i18n::fill($crate::i18n::tr_plural($one, $other, $crate::i18n::count(n)), args),
        }
    }};
    (check [$($key:literal),*] $names:tt) => {
        $(const _: () = assert!($crate::i18n::fills_every_placeholder($key, &$names), "a placeholder in this text has no argument");)*
    };
}
