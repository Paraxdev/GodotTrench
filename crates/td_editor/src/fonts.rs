//! The editor's font set for a language. Plex stays first so English looks the same in every language, the bundled
//! Noto Sans faces trail it for Chinese and Japanese text.

use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily, FontTweak};

use crate::i18n::Lang;

const SC: &str = "Noto Sans SC";
const JP: &str = "Noto Sans JP";

/// True when Japanese Han forms come before the Chinese ones. The two fonts share the code points and draw some of
/// them differently, so the first one in the list decides.
pub fn cjk_order(lang: Lang) -> bool {
    lang == Lang::Ja
}

pub fn definitions(lang: Lang) -> FontDefinitions {
    let mut fonts = crate::theme::fonts();
    // The Noto ascent is taller than Plex, this puts ideographs on the same baseline.
    let tweak = FontTweak { y_offset_factor: -0.03, ..Default::default() };
    for (name, data) in
        [(SC, &include_bytes!("../assets/fonts/NotoSansSC-Regular.otf")[..]), (JP, &include_bytes!("../assets/fonts/NotoSansJP-Regular.otf")[..])]
    {
        fonts.font_data.insert(name.to_owned(), Arc::new(FontData::from_static(data).tweak(tweak.clone())));
    }

    let order = if cjk_order(lang) { [JP, SC] } else { [SC, JP] };
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts.families.entry(family).or_default().extend(order.map(String::from));
    }

    fonts
}

#[cfg(test)]
mod tests {
    use egui::epaint::text::{Fonts, TextOptions};
    use egui::{FontFamily, FontId};

    use super::*;

    fn fonts_for(lang: Lang) -> Fonts {
        Fonts::new(TextOptions::default(), definitions(lang))
    }

    fn missing(fonts: &mut Fonts, family: FontFamily, text: &str) -> Vec<char> {
        let id = FontId::new(14.0, family);
        let mut out: Vec<char> = text.chars().filter(|c| *c != '\n' && !fonts.has_glyph(&id, *c)).collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    #[test]
    fn every_language_name_draws_in_every_language() {
        let names: String = Lang::ALL.iter().map(|l| l.name()).collect::<Vec<_>>().join(" ");
        assert!(names.contains("简体中文") && names.contains("日本語"));
        for lang in Lang::ALL {
            let mut fonts = fonts_for(lang);
            for family in [FontFamily::Proportional, FontFamily::Monospace] {
                assert_eq!(missing(&mut fonts, family.clone(), &names), Vec::<char>::new(), "{lang:?} {family:?}");
            }
        }
    }

    #[test]
    fn cjk_fonts_trail_and_follow_the_language() {
        let base = crate::theme::fonts();
        for lang in Lang::ALL {
            let defs = definitions(lang);
            let first = if lang == Lang::Ja { ["Noto Sans JP", "Noto Sans SC"] } else { ["Noto Sans SC", "Noto Sans JP"] };
            for family in [FontFamily::Proportional, FontFamily::Monospace] {
                let before = &base.families[&family];
                let after = &defs.families[&family];
                assert_eq!(&after[..before.len()], &before[..], "{lang:?} {family:?} keeps the existing fonts first");
                assert_eq!(&after[before.len()..], &first.map(String::from)[..], "{lang:?} {family:?}");
            }

            for name in first {
                assert!(defs.font_data.contains_key(name), "{name}");
            }
        }
    }

    #[test]
    fn a_private_use_char_has_no_glyph() {
        let mut fonts = fonts_for(Lang::Ja);
        assert_eq!(missing(&mut fonts, FontFamily::Proportional, "a\u{E000}b\u{E000}"), vec!['\u{E000}']);
    }

    #[test]
    fn every_translation_has_glyphs() {
        for lang in Lang::ALL {
            let mut fonts = fonts_for(lang);
            for text in crate::i18n::shipped_texts(lang) {
                assert_eq!(missing(&mut fonts, FontFamily::Proportional, &text), Vec::<char>::new(), "{lang:?}: {text}");
            }
        }
    }
}
