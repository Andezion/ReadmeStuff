// Half-width katakana + digits for the Matrix effect
const CHARS: &[char] = &[
    'ｦ', 'ｧ', 'ｨ', 'ｩ', 'ｪ', 'ｫ', 'ｬ', 'ｭ', 'ｮ', 'ｯ',
    'ｱ', 'ｲ', 'ｳ', 'ｴ', 'ｵ', 'ｶ', 'ｷ', 'ｸ', 'ｹ', 'ｺ',
    'ｻ', 'ｼ', 'ｽ', 'ｾ', 'ｿ', 'ﾀ', 'ﾁ', 'ﾂ', 'ﾃ', 'ﾄ',
    'ﾅ', 'ﾆ', 'ﾇ', 'ﾈ', 'ﾉ', 'ﾊ', 'ﾋ', 'ﾌ', 'ﾍ', 'ﾎ',
    'ﾏ', 'ﾐ', 'ﾑ', 'ﾒ', 'ﾓ', 'ﾔ', 'ﾕ', 'ﾖ', 'ﾗ', 'ﾘ',
    'ﾙ', 'ﾚ', 'ﾛ', 'ﾜ', 'ﾝ',
    '0', '1', '0', '1', '4', '8', '6', '5', '2', '7', '3', '9',
];

struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_add(1))
    }

    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }

    fn range(&mut self, lo: u64, hi: u64) -> u64 {
        lo + self.next() % (hi - lo)
    }

    fn rangef(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (self.next() as f64 / u64::MAX as f64) * (hi - lo)
    }

    fn pick_char(&mut self) -> char {
        CHARS[self.range(0, CHARS.len() as u64) as usize]
    }
}

pub fn generate(width: u32, height: u32, color: &str, opacity: f64, seed: u64, prefix: &str) -> String {
    let mut rng = Lcg::new(seed);
    let char_h: i32 = 14;
    let num_drops = ((width / 24).max(8)) as usize;

    let mut css = String::new();
    let mut drops = String::new();

    for i in 0..num_drops {
        let x = rng.range(8, (width - 8) as u64) as i32;
        let trail = rng.range(3, 7) as i32;
        let duration = rng.rangef(1.5, 3.2);
        let delay = -rng.rangef(0.0, duration);
        let start_y = -(trail * char_h + 4);
        let end_y = height as i32 + 14;

        css.push_str(&format!(
            "@keyframes {prefix}{i}{{from{{transform:translateY({start_y}px)}}to{{transform:translateY({end_y}px)}}}}",
        ));

        drops.push_str(&format!(
            "<g style=\"animation:{prefix}{i} {duration:.2}s linear infinite;animation-delay:{delay:.2}s\">",
        ));

        // Leading character at full brightness
        let ch = rng.pick_char();
        drops.push_str(&format!(
            "<text x=\"{x}\" y=\"0\" font-family=\"monospace\" font-size=\"12\" fill=\"{color}\">{ch}</text>",
        ));

        // Trailing characters fading out
        for j in 1..=trail {
            let ch = rng.pick_char();
            let alpha = 1.0 - j as f64 / (trail + 1) as f64;
            let y = -(j * char_h);
            drops.push_str(&format!(
                "<text x=\"{x}\" y=\"{y}\" font-family=\"monospace\" font-size=\"12\" fill=\"{color}\" opacity=\"{alpha:.2}\">{ch}</text>",
            ));
        }

        drops.push_str("</g>");
    }

    format!(
        "<g opacity=\"{opacity:.2}\"><style>{css}</style>{drops}</g>"
    )
}
