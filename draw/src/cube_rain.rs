use crate::rng::Lcg;

pub fn generate(
    col_xs: &[i32],
    y_top: i32,
    y_bottom: i32,
    cell: i32,
    color: &str,
    seed: u64,
    prefix: &str,
) -> String {
    let mut rng = Lcg::new(seed);
    let num_drops = (col_xs.len() / 3).max(4);

    let mut css = String::new();
    let mut drops = String::new();

    let start_y = y_top - cell * 6;
    let end_y = y_bottom + cell;

    for i in 0..num_drops {
        let col = col_xs[rng.range(0, col_xs.len() as u64) as usize];
        let trail = rng.range(3, 6) as i32;
        let duration = rng.rangef(2.2, 4.5);
        let delay = -rng.rangef(0.0, duration);

        css.push_str(&format!(
            "@keyframes {prefix}{i}{{from{{transform:translateY({start_y}px)}}to{{transform:translateY({end_y}px)}}}}",
        ));

        drops.push_str(&format!(
            "<g style=\"animation:{prefix}{i} {duration:.2}s linear infinite;animation-delay:{delay:.2}s\">",
        ));

        drops.push_str(&format!(
            "<rect x=\"{x}\" y=\"0\" width=\"{cell}\" height=\"{cell}\" rx=\"2\" fill=\"{color}\"/>",
            x = col - cell / 2,
        ));

        for j in 1..=trail {
            let alpha = 1.0 - j as f64 / (trail + 1) as f64;
            let y = -(j * cell);
            drops.push_str(&format!(
                "<rect x=\"{x}\" y=\"{y}\" width=\"{cell}\" height=\"{cell}\" rx=\"2\" fill=\"{color}\" opacity=\"{alpha:.2}\"/>",
                x = col - cell / 2,
            ));
        }

        drops.push_str("</g>");
    }

    format!("<g style=\"mix-blend-mode:screen\"><style>{css}</style>{drops}</g>")
}
