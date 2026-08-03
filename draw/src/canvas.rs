pub struct PlacedTile<'a> {
    pub svg: &'a str,
    pub x: u32,
    pub y: u32,
}

pub fn svg_size(svg: &str) -> Option<(u32, u32)> {
    match (attr_number(svg, "width=\""), attr_number(svg, "height=\"")) {
        (Some(w), Some(h)) => Some((w, h)),
        _ => view_box_size(svg),
    }
}

fn attr_number(svg: &str, marker: &str) -> Option<u32> {
    let start = svg.find(marker)? + marker.len();
    let end = svg[start..].find('"')? + start;
    let raw = svg[start..end].trim();
    let numeric: String = raw
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    numeric.parse::<f64>().ok().map(|v| v.round() as u32)
}

fn view_box_size(svg: &str) -> Option<(u32, u32)> {
    let marker = "viewBox=\"";
    let start = svg.find(marker)? + marker.len();
    let end = svg[start..].find('"')? + start;
    let mut parts = svg[start..end].split_whitespace();
    parts.next()?;
    parts.next()?;
    let w = parts.next()?.parse::<f64>().ok()?;
    let h = parts.next()?.parse::<f64>().ok()?;
    Some((w.round() as u32, h.round() as u32))
}

fn strip_root(svg: &str) -> Option<&str> {
    let tag_start = svg.find("<svg")?;
    let inner_start = svg[tag_start..].find('>')? + tag_start + 1;
    let svg_close = svg.rfind("</svg>")?;
    if inner_start > svg_close {
        return None;
    }
    Some(&svg[inner_start..svg_close])
}

fn collect_ids(fragment: &str) -> Vec<String> {
    let marker = "id=\"";
    let mut ids = Vec::new();
    let mut from = 0;
    while let Some(pos) = fragment[from..].find(marker) {
        let start = from + pos + marker.len();
        let Some(end_rel) = fragment[start..].find('"') else {
            break;
        };
        ids.push(fragment[start..start + end_rel].to_string());
        from = start + end_rel;
    }
    ids
}

fn namespace_ids(fragment: &str, slot: usize) -> String {
    let mut out = fragment.to_string();
    for id in collect_ids(fragment) {
        let renamed = format!("{id}s{slot}");
        out = out.replace(&format!("id=\"{id}\""), &format!("id=\"{renamed}\""));
        out = out.replace(&format!("#{id})"), &format!("#{renamed})"));
        out = out.replace(&format!("#{id}\""), &format!("#{renamed}\""));
    }
    out
}

pub fn compose_freeform(
    canvas_w: u32,
    canvas_h: u32,
    bg: &str,
    tiles: &[PlacedTile],
) -> Result<String, String> {
    let mut content = String::new();
    for (slot, tile) in tiles.iter().enumerate() {
        let inner =
            strip_root(tile.svg).ok_or_else(|| format!("tile {slot}: could not parse svg"))?;
        let namespaced = namespace_ids(inner, slot);
        content.push_str(&format!(
            r#"<g transform="translate({x},{y})">{namespaced}</g>"#,
            x = tile.x,
            y = tile.y
        ));
    }

    Ok(format!(
        r#"<svg width="{canvas_w}" height="{canvas_h}" viewBox="0 0 {canvas_w} {canvas_h}" xmlns="http://www.w3.org/2000/svg">
<rect width="{canvas_w}" height="{canvas_h}" fill="{bg}"/>
{content}</svg>"#,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tile(w: u32, h: u32, clip_id: &str, text: &str) -> String {
        format!(
            r##"<svg width="{w}" height="{h}" viewBox="0 0 {w} {h}" xmlns="http://www.w3.org/2000/svg">
<defs><clipPath id="{clip_id}"><rect width="{w}" height="{h}"/></clipPath></defs>
<g clip-path="url(#{clip_id})"><text>{text}</text></g>
</svg>"##
        )
    }

    #[test]
    fn svg_size_reads_width_and_height_attributes() {
        let svg = tile(200, 100, "a", "hi");
        assert_eq!(svg_size(&svg), Some((200, 100)));
    }

    #[test]
    fn svg_size_falls_back_to_view_box() {
        let svg = r#"<svg viewBox="0 0 150 75" xmlns="http://www.w3.org/2000/svg"></svg>"#;
        assert_eq!(svg_size(svg), Some((150, 75)));
    }

    #[test]
    fn svg_size_ignores_non_numeric_units() {
        let svg = r#"<svg width="150px" height="75.5" xmlns="http://www.w3.org/2000/svg"></svg>"#;
        assert_eq!(svg_size(svg), Some((150, 76)));
    }

    #[test]
    fn compose_freeform_places_tiles_at_their_offsets() {
        let a = tile(100, 50, "a", "first");
        let b = tile(100, 50, "b", "second");
        let out = compose_freeform(
            200,
            50,
            "#ffffff",
            &[
                PlacedTile { svg: &a, x: 0, y: 0 },
                PlacedTile {
                    svg: &b,
                    x: 100,
                    y: 0,
                },
            ],
        )
        .expect("compose should succeed");

        assert!(out.contains(r#"translate(0,0)"#));
        assert!(out.contains(r#"translate(100,0)"#));
        assert!(out.contains("first"));
        assert!(out.contains("second"));
    }

    #[test]
    fn compose_freeform_namespaces_colliding_ids() {
        let a = tile(100, 50, "clip0", "first");
        let b = tile(100, 50, "clip0", "second");
        let out = compose_freeform(
            200,
            50,
            "#ffffff",
            &[
                PlacedTile { svg: &a, x: 0, y: 0 },
                PlacedTile {
                    svg: &b,
                    x: 100,
                    y: 0,
                },
            ],
        )
        .expect("compose should succeed");

        assert!(out.contains(r#"id="clip0s0""#));
        assert!(out.contains(r#"id="clip0s1""#));
        assert!(out.contains(r#"url(#clip0s0)"#));
        assert!(out.contains(r#"url(#clip0s1)"#));
    }

    #[test]
    fn compose_freeform_crops_to_the_given_canvas_height() {
        let a = tile(100, 50, "a", "only");
        let out = compose_freeform(100, 50, "#ffffff", &[PlacedTile { svg: &a, x: 0, y: 0 }])
            .expect("compose should succeed");
        assert!(out.contains(r#"height="50""#));
        assert!(!out.contains(r#"height="990""#));
    }
}
