use std::collections::BTreeSet;
use std::fmt::Write;

use crate::rng::Lcg;
use crate::theme::Tint;

const CHARS: &[char] = &[
    'ｦ', 'ｧ', 'ｨ', 'ｩ', 'ｪ', 'ｫ', 'ｬ', 'ｭ', 'ｮ', 'ｯ', 'ｱ', 'ｲ', 'ｳ', 'ｴ', 'ｵ', 'ｶ', 'ｷ', 'ｸ', 'ｹ',
    'ｺ', 'ｻ', 'ｼ', 'ｽ', 'ｾ', 'ｿ', 'ﾀ', 'ﾁ', 'ﾂ', 'ﾃ', 'ﾄ', 'ﾅ', 'ﾆ', 'ﾇ', 'ﾈ', 'ﾉ', 'ﾊ', 'ﾋ', 'ﾌ',
    'ﾍ', 'ﾎ', 'ﾏ', 'ﾐ', 'ﾑ', 'ﾒ', 'ﾓ', 'ﾔ', 'ﾕ', 'ﾖ', 'ﾗ', 'ﾘ', 'ﾙ', 'ﾚ', 'ﾛ', 'ﾜ', 'ﾝ', '0', '1',
    '0', '1', '4', '8', '6', '5', '2', '7', '3', '9',
];

const CHAR_H: i32 = 14;

trait PickChar {
    fn pick_char(&mut self) -> char;
}

impl PickChar for Lcg {
    fn pick_char(&mut self) -> char {
        CHARS[self.range(0, CHARS.len() as u64) as usize]
    }
}

fn short_alpha(alpha: f64) -> String {
    let s = format!("{alpha:.2}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.strip_prefix('0').unwrap_or(s).to_string()
}

pub fn generate(
    width: u32,
    height: u32,
    tint: Tint,
    opacity: f64,
    seed: u64,
    prefix: &str,
) -> String {
    let mut rng = Lcg::new(seed);
    let num_drops = ((width / 13).max(12)) as usize;
    let end_y = height as i32 + 14;

    let mut trails = BTreeSet::new();
    let mut drops = String::with_capacity(num_drops * 400);

    for _ in 0..num_drops {
        let x = rng.range(8, (width - 8) as u64) as i32;
        let color = tint.color_at(x as u32, width);
        let trail = rng.range(3, 7) as i32;
        let duration = rng.rangef(1.5, 3.2);
        let delay = -rng.rangef(0.0, duration);
        trails.insert(trail);

        let ch = rng.pick_char();
        let _ = write!(
            drops,
            "<g fill=\"{color}\" style=\"animation:{prefix}{trail} {duration:.2}s linear {delay:.2}s infinite\"><text x=\"{x}\">{ch}</text>",
        );

        for j in 1..=trail {
            let ch = rng.pick_char();
            let alpha = short_alpha(1.0 - j as f64 / (trail + 1) as f64);
            let y = -(j * CHAR_H);
            let _ = write!(
                drops,
                "<text x=\"{x}\" y=\"{y}\" opacity=\"{alpha}\">{ch}</text>"
            );
        }

        drops.push_str("</g>");
    }

    let mut css = String::new();
    for trail in trails {
        let start_y = -(trail * CHAR_H + 4);
        let _ = write!(
            css,
            "@keyframes {prefix}{trail}{{from{{transform:translateY({start_y}px)}}to{{transform:translateY({end_y}px)}}}}",
        );
    }

    format!(
        "<g opacity=\"{opacity:.2}\" font-family=\"monospace\" font-size=\"12\"><style>{css}</style>{drops}</g>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_alpha_trims() {
        assert_eq!(short_alpha(0.80), ".8");
        assert_eq!(short_alpha(0.75), ".75");
        assert_eq!(short_alpha(0.142857), ".14");
    }

    #[test]
    fn keyframes_are_shared_per_trail_length() {
        let svg = generate(495, 120, Tint::Green, 0.22, 42, "t");
        let keyframes = svg.matches("@keyframes ").count();
        assert!(keyframes <= 4, "expected at most 4 keyframes, got {keyframes}");
        for name in svg.split("animation:").skip(1) {
            let name = name.split(' ').next().unwrap();
            assert!(svg.contains(&format!("@keyframes {name}{{")), "missing {name}");
        }
    }

    #[test]
    fn output_is_deterministic_and_balanced() {
        let a = generate(990, 260, Tint::Rainbow, 0.22, 7, "x");
        assert_eq!(a, generate(990, 260, Tint::Rainbow, 0.22, 7, "x"));
        assert_eq!(a.matches("<g").count(), a.matches("</g>").count());
        assert_eq!(a.matches("<text").count(), a.matches("</text>").count());
    }
}
