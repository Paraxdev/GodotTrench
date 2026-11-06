//! Lightness flip that redraws line art made for a dark background on light paper. The line and mesh shaders run
//! the same math from [`wgsl`].

/// Lightness scale of the flip. Below 1, so bright accents land on a mid tone instead of near black.
pub const FLIP_K: f32 = 0.8;

const CHROMA_STEPS: usize = 8;

type Mat3 = [[f32; 3]; 3];

const LMS_FROM_RGB: Mat3 = [[0.412_221_46, 0.536_332_55, 0.051_445_995], [0.211_903_5, 0.680_699_5, 0.107_396_96], [0.088_302_46, 0.281_718_85, 0.629_978_7]];
const LAB_FROM_LMS: Mat3 =
    [[0.210_454_26, 0.793_617_8, -0.004_072_047], [1.977_998_5, -2.428_592_2, 0.450_593_7], [0.025_904_037, 0.782_771_77, -0.808_675_77]];
const LMS_FROM_LAB: Mat3 = [[1.0, 0.396_337_78, 0.215_803_76], [1.0, -0.105_561_346, -0.063_854_17], [1.0, -0.089_484_18, -1.291_485_5]];
const RGB_FROM_LMS: Mat3 = [[4.076_741_7, -3.307_711_6, 0.230_969_94], [-1.268_438, 2.609_757_4, -0.341_319_38], [-0.004_196_086_3, -0.703_418_6, 1.707_614_7]];

/// Pivot `A` of the flip `L' = A - FLIP_K * L` in OKLab lightness that lands `dark` exactly on `paper`, both
/// linear RGB.
pub fn flip_pivot(dark: [f32; 3], paper: [f32; 3]) -> f32 {
    oklab(paper)[0] + FLIP_K * oklab(dark)[0]
}

/// Flips the lightness of a linear RGBA color around `pivot`. Hue and alpha stay, chroma shrinks only as far as
/// the result needs to fit in RGB.
pub fn flip_linear(c: [f32; 4], pivot: f32) -> [f32; 4] {
    let [l, a, b] = oklab([c[0], c[1], c[2]]);
    let l = (pivot - FLIP_K * l).clamp(0.0, 1.0);
    let at = |s: f32| linear_from_oklab([l, a * s, b * s]);
    let fits = |rgb: [f32; 3]| rgb.iter().all(|v| (0.0..=1.0).contains(v));
    let mut kept = 1.0;
    if !fits(at(1.0)) {
        let (mut lo, mut hi) = (0.0, 1.0);
        for _ in 0..CHROMA_STEPS {
            let mid = 0.5 * (lo + hi);
            if fits(at(mid)) {
                lo = mid;
            } else {
                hi = mid;
            }
        }

        kept = lo;
    }

    let rgb = at(kept).map(|v| v.clamp(0.0, 1.0));
    [rgb[0], rgb[1], rgb[2], c[3]]
}

/// `ink.wgsl` with the constants it shares with this module in front. The matrices go in as rows, so the shader
/// multiplies vectors from the left.
pub(crate) fn wgsl() -> String {
    let mat = |name: &str, m: Mat3| {
        let rows = m.map(|r| format!("vec3<f32>({:?}, {:?}, {:?})", r[0], r[1], r[2])).join(", ");
        format!("const {name} = mat3x3<f32>({rows});\n")
    };
    format!(
        "const FLIP_K: f32 = {FLIP_K:?};\nconst CHROMA_STEPS: i32 = {CHROMA_STEPS};\n{}{}{}{}\n{}",
        mat("LMS_FROM_RGB", LMS_FROM_RGB),
        mat("LAB_FROM_LMS", LAB_FROM_LMS),
        mat("LMS_FROM_LAB", LMS_FROM_LAB),
        mat("RGB_FROM_LMS", RGB_FROM_LMS),
        include_str!("ink.wgsl"),
    )
}

fn mul(m: Mat3, v: [f32; 3]) -> [f32; 3] {
    m.map(|r| r[0] * v[0] + r[1] * v[1] + r[2] * v[2])
}

fn oklab(c: [f32; 3]) -> [f32; 3] {
    mul(LAB_FROM_LMS, mul(LMS_FROM_RGB, c).map(|v| v.max(0.0).cbrt()))
}

fn linear_from_oklab(lab: [f32; 3]) -> [f32; 3] {
    mul(RGB_FROM_LMS, mul(LMS_FROM_LAB, lab).map(|v| v * v * v))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::srgb_to_linear;

    fn hex(rgb: u32) -> [f32; 3] {
        [rgb >> 16, rgb >> 8, rgb].map(|v| srgb_to_linear((v & 0xff) as f32 / 255.0))
    }

    fn srgb8(c: [f32; 4]) -> [i32; 3] {
        let encode = |v: f32| if v <= 0.003_130_8 { v * 12.92 } else { 1.055 * v.powf(1.0 / 2.4) - 0.055 };
        [c[0], c[1], c[2]].map(|v| (encode(v) * 255.0).round() as i32)
    }

    fn rgba(c: [f32; 3], a: f32) -> [f32; 4] {
        [c[0], c[1], c[2], a]
    }

    fn rgb(c: [f32; 4]) -> [f32; 3] {
        [c[0], c[1], c[2]]
    }

    fn chroma(lab: [f32; 3]) -> f32 {
        lab[1].hypot(lab[2])
    }

    fn hue(lab: [f32; 3]) -> f32 {
        lab[2].atan2(lab[1]).to_degrees()
    }

    fn in_rgb(c: [f32; 3]) -> bool {
        c.iter().all(|v| (0.0..=1.0).contains(v))
    }

    fn pivot() -> f32 {
        flip_pivot(hex(0x111215), hex(0xeef0f3))
    }

    #[test]
    fn pivot_sums_paper_and_scaled_dark_lightness() {
        assert!((flip_pivot(hex(0x111215), hex(0xeef0f3)) - 1.100_41).abs() < 1e-4);
        assert!((flip_pivot([0.0; 3], [1.0; 3]) - 1.0).abs() < 1e-5, "black on white paper");
        assert!((flip_pivot([1.0; 3], [0.0; 3]) - FLIP_K).abs() < 1e-5, "white on black paper");
    }

    #[test]
    fn alpha_passes_through() {
        let colors = [[0.4, 0.07, 0.02], [0.0, 0.0, 1.0], [0.0; 3], [0.85; 3]];
        for (c, a) in colors.into_iter().zip([0.0, 0.37, 0.75, 1.0]) {
            assert_eq!(flip_linear(rgba(c, a), pivot())[3], a, "{c:?}");
        }
    }

    #[test]
    fn dark_background_lands_on_paper() {
        let out = flip_linear(rgba(hex(0x111215), 1.0), pivot());
        let got = srgb8(out);
        for (g, want) in got.into_iter().zip([0xee, 0xf0, 0xf3]) {
            assert!((g - want).abs() <= 1, "{got:?}");
        }

        assert!((oklab(rgb(out))[0] - oklab(hex(0xeef0f3))[0]).abs() < 1e-4);
        assert_eq!(out[3], 1.0);
    }

    #[test]
    fn gray_lightness_order_reverses_and_tint_stays() {
        let grays = [[0.06, 0.065, 0.07], [0.31, 0.29, 0.33], [0.74, 0.78, 0.71]];
        let flipped = grays.map(|g| flip_linear(rgba(g, 1.0), pivot()));
        let before = grays.map(|g| oklab(g)[0]);
        let after = flipped.map(|f| oklab(rgb(f))[0]);
        assert!(before[0] < before[1] && before[1] < before[2], "{before:?}");
        assert!(after[0] > after[1] && after[1] > after[2], "{after:?}");
        for ((g, f), l) in grays.iter().zip(&flipped).zip(before) {
            let (lab, out) = (oklab(*g), oklab(rgb(*f)));
            assert!((out[0] - (pivot() - FLIP_K * l)).abs() < 1e-4, "{g:?} -> {f:?}");
            assert!((out[1] - lab[1]).abs() < 1e-4 && (out[2] - lab[2]).abs() < 1e-4, "{g:?} lost its tint: {f:?}");
        }
    }

    #[test]
    fn white_turns_dark() {
        let out = flip_linear([1.0; 4], pivot());
        assert_eq!(srgb8(out), [0x2e; 3]);
        assert_eq!(out[3], 1.0);
    }

    #[test]
    fn colors_darker_than_the_cut_clamp_to_white() {
        let cut = (pivot() - 1.0) / FLIP_K;
        let gray = |l: f32| [l * l * l; 3];
        let white = |l: f32, tolerance: f32| {
            let out = flip_linear(rgba(gray(l), 0.6), pivot());
            assert!(out[..3].iter().all(|v| (v - 1.0).abs() < tolerance) && out[3] == 0.6, "lightness {l} gave {out:?}");
        };
        white(0.0, 1e-5);
        white(cut - 0.01, 1e-5);
        white(cut, 1e-3);
        let above = flip_linear(rgba(gray(cut + 0.01), 1.0), pivot());
        let l = oklab(rgb(above))[0];
        assert!((l - (1.0 - FLIP_K * 0.01)).abs() < 1e-4, "{above:?}");
    }

    #[test]
    fn saturated_primaries_keep_hue_and_fit_in_rgb() {
        for c in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]] {
            let lab = oklab(c);
            let target = pivot() - FLIP_K * lab[0];
            assert!(!in_rgb(linear_from_oklab([target, lab[1], lab[2]])), "{c:?} needs its chroma cut");
            let out = flip_linear(rgba(c, 1.0), pivot());
            let flipped = oklab(rgb(out));
            assert!(in_rgb(rgb(out)), "{out:?}");
            assert!((flipped[0] - target).abs() < 1e-3, "{c:?} lightness {}", flipped[0]);
            assert!((hue(flipped) - hue(lab)).abs() < 2.0, "{c:?} hue {} vs {}", hue(flipped), hue(lab));
            let kept = chroma(flipped) / chroma(lab);
            let wider = kept + 2.0 / (1 << CHROMA_STEPS) as f32;
            assert!(kept < 1.0 && !in_rgb(linear_from_oklab([target, lab[1] * wider, lab[2] * wider])), "{c:?} kept {kept} of its chroma");
        }
    }
}
