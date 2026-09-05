use crate::{helpers::xml_escape, matrix, theme::Theme};
use readme_stuff_aggregator::widgets::LinkedinWidget;

const W: u32 = 495;
const H: u32 = 120;
pub const SIZE: (u32, u32) = (W, H);

pub fn render_linkedin(w: &LinkedinWidget, theme: Theme) -> String {
    let c = theme.colors();
    let font = c.font_family;
    let rain = matrix::generate(W, H, c.matrix_color, c.matrix_opacity, 0x00_77B5, "lin");

    let name = xml_escape(&w.name);
    let url = xml_escape(&w.profile_url);

    format!(
        r#"<svg width="{W}" height="{H}" viewBox="0 0 {W} {H}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="lin-clip">
    <rect width="{W}" height="{H}" rx="6"/>
  </clipPath>
</defs>
<rect width="{W}" height="{H}" rx="6" fill="{bg}"/>
<g clip-path="url(#lin-clip)">{rain}</g>
<rect width="{W}" height="{H}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>

<text x="25" y="35" font-family="{font}" font-size="14" font-weight="600" fill="{title}">LinkedIn</text>
<line x1="25" y1="52" x2="470" y2="52" stroke="{sep}" stroke-width="1"/>
<text x="25" y="80" font-family="{font}" font-size="20" font-weight="700" fill="{tv}">{name}</text>
<text x="25" y="102" font-family="{font}" font-size="12" fill="{tl}">{url}</text>
</svg>"#,
        bg = c.bg,
        border = c.border,
        title = c.title,
        sep = c.separator,
        rain = rain,
        tv = c.text_primary,
        tl = c.text_secondary,
    )
}
