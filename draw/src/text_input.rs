use crate::text_glyph::TextLine;

const DEFAULT_SIZE: f32 = 16.0;

pub fn parse_lines(content: &str) -> Vec<TextLine> {
    content.lines().filter_map(|raw| parse_line(raw)).collect()
}

fn parse_line(raw: &str) -> Option<TextLine> {
    let line = raw.trim();
    if line.is_empty() {
        return None;
    }

    match (line.rfind('('), line.rfind(')')) {
        (Some(open), Some(close)) if open < close => {
            let size_str = line[open + 1..close].trim();
            match size_str.parse::<f32>() {
                Ok(size) if size > 0.0 => Some(TextLine {
                    text: line[..open].trim_end().to_string(),
                    size,
                }),
                _ => {
                    eprintln!("  custom-text: bad size in line {raw:?}, using default size");
                    Some(TextLine {
                        text: line.to_string(),
                        size: DEFAULT_SIZE,
                    })
                }
            }
        }
        _ => {
            eprintln!("  custom-text: no (size) found in line {raw:?}, using default size");
            Some(TextLine {
                text: line.to_string(),
                size: DEFAULT_SIZE,
            })
        }
    }
}
