#[derive(Clone, Debug)]
pub struct StarVisualTokens {
    pub core_gradient: String,
    pub glow_color: String,
    pub halo_color: String,
    pub badge_tint: String,
}

pub fn star_visual_tokens(teff: Option<f64>) -> StarVisualTokens {
    let (r, g, b) = teff
        .map(interpolate_temperature_color)
        .unwrap_or((245.0, 198.0, 92.0));

    let highlight = mix((r, g, b), (255.0, 251.0, 235.0), 0.72);
    let rim = darken((r, g, b), 0.26);
    let glow = with_alpha((r, g, b), 0.30);
    let halo = with_alpha((r, g, b), 0.18);
    let badge = with_alpha(darken((r, g, b), 0.45), 0.24);

    StarVisualTokens {
        core_gradient: format!(
            "radial-gradient(circle at 30% 30%, {}, {} 34%, {} 76%, {})",
            rgb_css(highlight),
            rgb_css((r, g, b)),
            rgb_css(rim),
            rgb_css(darken(rim, 0.18))
        ),
        glow_color: glow,
        halo_color: halo,
        badge_tint: badge,
    }
}

fn interpolate_temperature_color(teff: f64) -> (f64, f64, f64) {
    const ANCHORS: &[(f64, (f64, f64, f64))] = &[
        (3000.0, (220.0, 104.0, 60.0)),
        (3500.0, (230.0, 132.0, 72.0)),
        (4500.0, (242.0, 179.0, 92.0)),
        (5300.0, (247.0, 213.0, 146.0)),
        (5800.0, (245.0, 234.0, 210.0)),
        (6800.0, (239.0, 243.0, 247.0)),
        (8500.0, (205.0, 224.0, 255.0)),
        (12000.0, (180.0, 205.0, 255.0)),
    ];

    let teff = teff.clamp(ANCHORS[0].0, ANCHORS[ANCHORS.len() - 1].0);

    for window in ANCHORS.windows(2) {
        let (start_temp, start_color) = window[0];
        let (end_temp, end_color) = window[1];
        if teff >= start_temp && teff <= end_temp {
            let t = (teff - start_temp) / (end_temp - start_temp);
            return lerp_color(start_color, end_color, t);
        }
    }

    ANCHORS[ANCHORS.len() - 1].1
}

fn lerp_color(
    start: (f64, f64, f64),
    end: (f64, f64, f64),
    t: f64,
) -> (f64, f64, f64) {
    (
        start.0 + (end.0 - start.0) * t,
        start.1 + (end.1 - start.1) * t,
        start.2 + (end.2 - start.2) * t,
    )
}

fn mix(
    base: (f64, f64, f64),
    other: (f64, f64, f64),
    amount: f64,
) -> (f64, f64, f64) {
    lerp_color(base, other, amount.clamp(0.0, 1.0))
}

fn darken(color: (f64, f64, f64), amount: f64) -> (f64, f64, f64) {
    let scale = (1.0 - amount).clamp(0.0, 1.0);
    (color.0 * scale, color.1 * scale, color.2 * scale)
}

fn rgb_css(color: (f64, f64, f64)) -> String {
    format!(
        "rgb({:.0} {:.0} {:.0})",
        color.0.clamp(0.0, 255.0),
        color.1.clamp(0.0, 255.0),
        color.2.clamp(0.0, 255.0)
    )
}

fn with_alpha(color: (f64, f64, f64), alpha: f64) -> String {
    format!(
        "rgba({:.0}, {:.0}, {:.0}, {:.2})",
        color.0.clamp(0.0, 255.0),
        color.1.clamp(0.0, 255.0),
        color.2.clamp(0.0, 255.0),
        alpha.clamp(0.0, 1.0)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cool_stars_are_warmer_than_hot_stars() {
        let cool = star_visual_tokens(Some(3500.0));
        let hot = star_visual_tokens(Some(9000.0));

        assert_ne!(cool.core_gradient, hot.core_gradient);
        assert_ne!(cool.glow_color, hot.glow_color);
    }

    #[test]
    fn missing_temperature_uses_fallback_palette() {
        let fallback = star_visual_tokens(None);

        assert!(fallback.core_gradient.contains("radial-gradient"));
        assert!(fallback.glow_color.contains("rgba"));
    }
}
