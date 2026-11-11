use crate::theme::Colors;

pub fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn fmt_num(n: u64) -> String {
    if n < 1_000 {
        return n.to_string();
    }
    if n < 10_000 {
        return format!("{:.1}k", n as f64 / 1000.0);
    }
    if n < 1_000_000 {
        return format!("{}k", n / 1000);
    }
    format!("{:.1}M", n as f64 / 1_000_000.0)
}

pub fn rank_color(c: &Colors, rank: &str) -> String {
    let (t, orig) = match rank.trim() {
        "S+" | "S" => (1.0, "#ff6e96"),
        "A+" => (0.85, "#ff8c00"),
        "A" => (0.7, "#fe428e"),
        "A-" => (0.55, "#fc7ef7"),
        "B+" => (0.4, "#a9fef7"),
        "B" => (0.3, "#79e7ff"),
        "B-" => (0.2, "#9be1fe"),
        _ => (0.1, "#6cb6ff"),
    };
    c.ranked(t, orig)
}

/// Codewars rank color name (white < yellow < blue < purple < black < red).
pub fn cw_color(c: &Colors, codewars_color: &str) -> String {
    let (t, orig) = match codewars_color.to_lowercase().as_str() {
        "white" => (0.1, "#b0b0b0"),
        "yellow" => (0.3, "#e8b400"),
        "blue" => (0.5, "#4285f4"),
        "purple" => (0.7, "#9b59b6"),
        "black" => (0.85, "#f0f0f0"),
        "red" => (1.0, "#ff6b6b"),
        _ => (0.0, "#8b949e"),
    };
    c.ranked(t, orig)
}

pub fn cf_rank_color(c: &Colors, rank: &str) -> String {
    let r = rank.to_lowercase();
    let r = r.trim();
    let (t, orig) = if r.contains("legendary")
        || r.contains("international grand")
        || r.contains("grandmaster")
    {
        (1.0, "#f44336")
    } else if r.contains("international master") || r.contains("master") {
        (0.85, "#ff8c00")
    } else if r.contains("candidate") {
        (0.7, "#ab47bc")
    } else if r.contains("expert") {
        (0.5, "#42a5f5")
    } else if r.contains("specialist") {
        (0.35, "#26c6da")
    } else if r.contains("pupil") {
        (0.2, "#66bb6a")
    } else {
        (0.05, "#9e9e9e")
    };
    c.ranked(t, orig)
}
