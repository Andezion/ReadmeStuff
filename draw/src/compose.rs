use crate::theme::Theme;

fn find_g_block(svg: &str, marker: &str) -> Option<(usize, usize)> {
    let tag_start = svg.find(marker)?;
    let inner_start = svg[tag_start..].find('>')? + tag_start + 1;

    let mut depth = 1usize;
    let mut i = inner_start;
    loop {
        let next_open = svg[i..].find("<g").map(|p| p + i);
        let next_close = svg[i..].find("</g>").map(|p| p + i);
        match (next_open, next_close) {
            (Some(o), Some(c)) if o < c => {
                depth += 1;
                i = o + 2;
            }
            (_, Some(c)) => {
                depth -= 1;
                if depth == 0 {
                    return Some((inner_start, c));
                }
                i = c + 4;
            }
            _ => return None,
        }
    }
}

fn extract_widget(svg: &str) -> Option<(String, String)> {
    let (rain_start, rain_end) = find_g_block(svg, "<g clip-path=\"url(#")?;
    let rain = svg[rain_start..rain_end].to_string();

    let after_clip_g = rain_end + "</g>".len();
    let content_start = svg[after_clip_g..].find("/>").map(|p| p + after_clip_g + 2)?;
    let svg_close = svg.rfind("</svg>")?;
    if content_start > svg_close {
        return None;
    }
    let content = svg[content_start..svg_close].to_string();

    Some((rain, content))
}

fn namespace_rain(rain: &str, slot: usize) -> String {
    let marker = "@keyframes ";
    let mut names = Vec::new();
    let mut from = 0;
    while let Some(pos) = rain[from..].find(marker) {
        let start = from + pos + marker.len();
        let Some(brace) = rain[start..].find('{') else {
            break;
        };
        names.push(rain[start..start + brace].to_string());
        from = start + brace;
    }

    let mut out = rain.to_string();
    for name in names {
        let renamed = format!("{name}s{slot}");
        out = out.replace(&format!("@keyframes {name}{{"), &format!("@keyframes {renamed}{{"));
        out = out.replace(&format!("animation:{name} "), &format!("animation:{renamed} "));
    }
    out
}

fn declared_size(svg: &str) -> Option<(u32, u32)> {
    let w = attr_value(svg, "width=\"")?.parse().ok()?;
    let h = attr_value(svg, "height=\"")?.parse().ok()?;
    Some((w, h))
}

fn attr_value<'a>(svg: &'a str, marker: &str) -> Option<&'a str> {
    let start = svg.find(marker)? + marker.len();
    let end = svg[start..].find('"')? + start;
    Some(&svg[start..end])
}

pub struct Tile<'a> {
    pub svg: &'a str,
    pub x: u32,
    pub y: u32,
}

pub fn compose(canvas_w: u32, canvas_h: u32, theme: Theme, tiles: &[Tile]) -> Result<String, String> {
    let c = theme.colors();
    let sizes: Vec<Option<(u32, u32)>> = tiles.iter().map(|t| declared_size(t.svg)).collect();

    for (slot, (tile, size)) in tiles.iter().zip(sizes.iter()).enumerate() {
        let Some((w, h)) = size else { continue };
        if tile.x + w > canvas_w || tile.y + h > canvas_h {
            eprintln!(
                "compose: warning - tile {slot} is {w}x{h} at ({x},{y}), overflows {canvas_w}x{canvas_h} canvas",
                x = tile.x,
                y = tile.y,
            );
        }
    }

    // A tile whose declared size doesn't match what the layout expected can
    // still fit inside the canvas bounds (e.g. a banner generated at the
    // full row width instead of half of it) while still overlapping its
    // neighbor - that's not caught by the canvas-overflow check above. This
    // is a hard error rather than a warning: a silently overlapping tile
    // would still "succeed" and produce a broken-looking mosaic.
    for i in 0..tiles.len() {
        let Some((wi, hi)) = sizes[i] else { continue };
        for j in (i + 1)..tiles.len() {
            let Some((wj, hj)) = sizes[j] else { continue };
            let (a, b) = (&tiles[i], &tiles[j]);
            let overlap_x = a.x < b.x + wj && b.x < a.x + wi;
            let overlap_y = a.y < b.y + hj && b.y < a.y + hi;
            if overlap_x && overlap_y {
                return Err(format!(
                    "tile {i} ({wi}x{hi} at {ax},{ay}) overlaps tile {j} ({wj}x{hj} at {bx},{by}) - one of them was generated at the wrong size",
                    ax = a.x,
                    ay = a.y,
                    bx = b.x,
                    by = b.y,
                ));
            }
        }
    }

    let mut rain_layer = String::new();
    let mut content_layer = String::new();

    for (slot, tile) in tiles.iter().enumerate() {
        let (rain, content) =
            extract_widget(tile.svg).ok_or_else(|| format!("tile {slot}: could not parse widget svg"))?;
        let rain = namespace_rain(&rain, slot);
        rain_layer.push_str(&format!(
            r#"<g transform="translate({x},{y})">{rain}</g>"#,
            x = tile.x,
            y = tile.y
        ));
        content_layer.push_str(&format!(
            r#"<g transform="translate({x},{y})">{content}</g>"#,
            x = tile.x,
            y = tile.y
        ));
    }

    Ok(format!(
        r#"<svg width="{canvas_w}" height="{canvas_h}" viewBox="0 0 {canvas_w} {canvas_h}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="row-clip">
    <rect width="{canvas_w}" height="{canvas_h}" rx="6"/>
  </clipPath>
</defs>
<rect width="{canvas_w}" height="{canvas_h}" rx="6" fill="{bg}"/>
<g clip-path="url(#row-clip)">{rain_layer}</g>
<rect width="{canvas_w}" height="{canvas_h}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
{content_layer}
</svg>"#,
        bg = c.bg,
        border = c.border,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn widget(w: u32, h: u32, tag: &str) -> String {
        format!(
            r##"<svg width="{w}" height="{h}" viewBox="0 0 {w} {h}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="{tag}-clip">
    <rect width="{w}" height="{h}" rx="6"/>
  </clipPath>
</defs>
<rect width="{w}" height="{h}" rx="6" fill="#0d1117"/>
<g clip-path="url(#{tag}-clip)"><g opacity="0.5"><style>@keyframes {tag}0{{from{{transform:translateY(-10px)}}to{{transform:translateY(10px)}}}}</style><g style="animation:{tag}0 2.00s linear infinite;animation-delay:-1.00s"><text x="1" y="1">A</text></g></g></g>
<rect width="{w}" height="{h}" rx="6" fill="none" stroke="#30363d" stroke-width="1"/>
<text x="25" y="35">hello {tag}</text>
</svg>"##
        )
    }

    #[test]
    fn extracts_rain_and_content() {
        let svg = widget(100, 50, "aa");
        let (rain, content) = extract_widget(&svg).expect("should parse");
        assert!(rain.contains("@keyframes aa0"));
        assert!(rain.contains("<text x=\"1\" y=\"1\">A</text>"));
        assert!(content.contains("hello aa"));
        assert!(!content.contains("<rect"));
    }

    #[test]
    fn namespaces_keyframes_without_prefix_collisions() {
        let rain = "<style>@keyframes aa1{x}@keyframes aa10{y}</style>\
                    <g style=\"animation:aa1 1.00s linear infinite\">\
                    <g style=\"animation:aa10 1.00s linear infinite\">";
        let out = namespace_rain(rain, 3);
        assert!(out.contains("@keyframes aa1s3{"));
        assert!(out.contains("@keyframes aa10s3{"));
        assert!(out.contains("animation:aa1s3 "));
        assert!(out.contains("animation:aa10s3 "));
        assert!(!out.contains("aa1s3s3"));
    }

    #[test]
    fn composes_two_tiles_without_id_collision() {
        let a = widget(100, 50, "aa");
        let b = widget(100, 50, "aa"); 
        let out = compose(
            200,
            50,
            Theme::Dark,
            &[
                Tile { svg: &a, x: 0, y: 0 },
                Tile { svg: &b, x: 100, y: 0 },
            ],
        )
        .expect("compose should succeed");

        assert_eq!(out.matches("@keyframes aa0s0{").count(), 1);
        assert_eq!(out.matches("@keyframes aa0s1{").count(), 1);
        assert_eq!(out.matches("hello aa").count(), 2);
        assert_eq!(out.matches("rx=\"6\"").count(), 3);
    }

    #[test]
    fn rejects_tiles_that_overlap() {
        // "b" was generated at the wrong width (990 instead of its assigned
        // 495-wide slot) and ends up overlapping "a" placed at x=495 - this
        // is exactly the class of bug a wrong CLI size argument produces.
        let a = widget(990, 100, "aa");
        let b = widget(495, 100, "bb");
        let err = compose(
            990,
            100,
            Theme::Dark,
            &[
                Tile { svg: &a, x: 0, y: 0 },
                Tile { svg: &b, x: 495, y: 0 },
            ],
        )
        .expect_err("overlapping tiles must be rejected");
        assert!(err.contains("overlaps"));
    }

    #[test]
    #[ignore = "requires readme_test/ to have been generated by the CLI first"]
    fn extracts_real_generated_widgets() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../readme_test");
        for name in ["cf-rating-dark.svg", "lc-skills-dark.svg", "github-visitors-dark.svg"] {
            let svg = std::fs::read_to_string(format!("{dir}/{name}")).expect("read widget svg");
            extract_widget(&svg).unwrap_or_else(|| panic!("{name}: failed to parse real widget output"));
        }
    }
}
