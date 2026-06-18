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

pub fn rank_color(rank: &str) -> &'static str {
    match rank.trim() {
        "S+" | "S" => "#ff6e96",
        "A+" => "#ff8c00",
        "A" => "#fe428e",
        "A-" => "#fc7ef7",
        "B+" => "#a9fef7",
        "B" => "#79e7ff",
        "B-" => "#9be1fe",
        _ => "#6cb6ff",
    }
}

pub fn cw_color(codewars_color: &str) -> &'static str {
    match codewars_color.to_lowercase().as_str() {
        "white" => "#b0b0b0",
        "yellow" => "#e8b400",
        "blue" => "#4285f4",
        "purple" => "#9b59b6",
        "black" => "#f0f0f0",
        "red" => "#ff6b6b",
        _ => "#8b949e",
    }
}

pub fn cf_rank_color(rank: &str) -> &'static str {
    let r = rank.to_lowercase();
    let r = r.trim();
    if r.contains("legendary") || r.contains("international grand") || r.contains("grandmaster") {
        "#f44336"
    } else if r.contains("international master") || r.contains("master") {
        "#ff8c00"
    } else if r.contains("candidate") {
        "#ab47bc"
    } else if r.contains("expert") {
        "#42a5f5"
    } else if r.contains("specialist") {
        "#26c6da"
    } else if r.contains("pupil") {
        "#66bb6a"
    } else {
        "#9e9e9e"
    }
}
