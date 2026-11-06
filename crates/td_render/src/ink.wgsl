// ink::wgsl() puts FLIP_K, CHROMA_STEPS and the OKLab matrices in front of this file.

fn oklab(c: vec3<f32>) -> vec3<f32> {
    return pow(max(c * LMS_FROM_RGB, vec3<f32>(0.0)), vec3<f32>(1.0 / 3.0)) * LAB_FROM_LMS;
}

fn linear_from_oklab(lab: vec3<f32>) -> vec3<f32> {
    let lms = lab * LMS_FROM_LAB;
    return (lms * lms * lms) * RGB_FROM_LMS;
}

fn fits(c: vec3<f32>) -> bool {
    return all(c >= vec3<f32>(0.0)) && all(c <= vec3<f32>(1.0));
}

fn flip(c: vec3<f32>, pivot: f32) -> vec3<f32> {
    let lab = oklab(c);
    let l = clamp(pivot - FLIP_K * lab.x, 0.0, 1.0);
    var kept = 1.0;
    if (!fits(linear_from_oklab(vec3<f32>(l, lab.yz)))) {
        var lo = 0.0;
        var hi = 1.0;
        for (var i = 0; i < CHROMA_STEPS; i++) {
            let mid = 0.5 * (lo + hi);
            if (fits(linear_from_oklab(vec3<f32>(l, lab.yz * mid)))) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        kept = lo;
    }
    return clamp(linear_from_oklab(vec3<f32>(l, lab.yz * kept)), vec3<f32>(0.0), vec3<f32>(1.0));
}
