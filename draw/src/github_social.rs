use crate::{helpers::fmt_num, matrix, theme::Theme};
use readme_stuff_aggregator::widgets::GithubSocialWidget;

const W: u32 = 300;
const H: u32 = 120;

pub fn render_github_social(w: &GithubSocialWidget, theme: Theme) -> String {
    let c = theme.colors();
    let rain = matrix::generate(W, H, c.matrix_color, c.matrix_opacity, 0x5566_CCDD, "gso");

    let stats: [(&str, String); 2] = [
        ("Followers", fmt_num(w.followers as u64)),
        ("Following", fmt_num(w.following as u64)),
    ];
    let col_x = [25u32, 160];

    let mut stat_svg = String::new();
    for (i, (label, value)) in stats.iter().enumerate() {
        let x = col_x[i];
        stat_svg.push_str(&format!(
            "<text x=\"{x}\" y=\"88\" font-family=\"monospace\" font-size=\"22\" font-weight=\"700\" fill=\"{tv}\">{value}</text>\
             <text x=\"{x}\" y=\"104\" font-family=\"monospace\" font-size=\"11\" fill=\"{tl}\">{label}</text>",
            tv = c.text_primary, tl = c.text_secondary,
        ));
    }

    format!(
        r#"<svg width="{W}" height="{H}" viewBox="0 0 {W} {H}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="gso-clip">
    <rect width="{W}" height="{H}" rx="6"/>
  </clipPath>
</defs>
<rect width="{W}" height="{H}" rx="6" fill="{bg}"/>
<g clip-path="url(#gso-clip)">{rain}</g>
<rect width="{W}" height="{H}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>

<text x="25" y="35" font-family="monospace" font-size="14" font-weight="600" fill="{title}">GitHub Network</text>
<line x1="25" y1="52" x2="285" y2="52" stroke="{sep}" stroke-width="1"/>
{stat_svg}
</svg>"#,
        bg = c.bg,
        border = c.border,
        title = c.title,
        sep = c.separator,
        rain = rain,
        stat_svg = stat_svg,
    )
}
