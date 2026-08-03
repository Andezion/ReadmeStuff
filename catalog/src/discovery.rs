use std::path::{Path, PathBuf};

pub const WIDGETS_DIR: &str = "readme_test";
pub const TEXT_WIDGETS_DIR: &str = "readme_test/text";

const FALLBACK_SIZE: (u32, u32) = (200, 100);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredWidget {
    pub id: String,
    pub path: PathBuf,
    pub size: (u32, u32),
}

pub fn scan_widgets(dirs: &[&Path]) -> Vec<DiscoveredWidget> {
    let mut out = Vec::new();
    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        let mut paths: Vec<PathBuf> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("svg"))
            .collect();
        paths.sort();

        for path in paths {
            let size = std::fs::read_to_string(&path)
                .ok()
                .and_then(|content| readme_stuff_draw::svg_size(&content))
                .unwrap_or(FALLBACK_SIZE);
            out.push(DiscoveredWidget {
                id: path.display().to_string(),
                path,
                size,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "readme-stuff-catalog-discovery-{name}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn svg(w: u32, h: u32) -> String {
        format!(r#"<svg width="{w}" height="{h}" xmlns="http://www.w3.org/2000/svg"></svg>"#)
    }

    #[test]
    fn finds_only_svg_files_and_parses_their_size() {
        let dir = temp_dir("only-svg");
        std::fs::write(dir.join("a.svg"), svg(120, 60)).unwrap();
        std::fs::write(dir.join("notes.txt"), "not an svg").unwrap();

        let found = scan_widgets(&[&dir]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].size, (120, 60));
        assert!(found[0].id.ends_with("a.svg"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn results_are_sorted_and_scan_multiple_dirs() {
        let dir_a = temp_dir("multi-a");
        let dir_b = temp_dir("multi-b");
        std::fs::write(dir_a.join("z.svg"), svg(10, 10)).unwrap();
        std::fs::write(dir_a.join("a.svg"), svg(10, 10)).unwrap();
        std::fs::write(dir_b.join("m.svg"), svg(10, 10)).unwrap();

        let found = scan_widgets(&[&dir_a, &dir_b]);
        let ids: Vec<&str> = found.iter().map(|w| w.id.as_str()).collect();
        assert_eq!(ids.len(), 3);
        assert!(ids[0].ends_with("a.svg"));
        assert!(ids[1].ends_with("z.svg"));
        assert!(ids[2].ends_with("m.svg"));
        std::fs::remove_dir_all(&dir_a).ok();
        std::fs::remove_dir_all(&dir_b).ok();
    }

    #[test]
    fn missing_directory_yields_no_widgets_instead_of_erroring() {
        let missing = PathBuf::from("/does/not/exist/hopefully");
        assert!(scan_widgets(&[&missing]).is_empty());
    }

    #[test]
    fn unparseable_size_falls_back_instead_of_dropping_the_widget() {
        let dir = temp_dir("bad-size");
        std::fs::write(dir.join("weird.svg"), "<svg></svg>").unwrap();

        let found = scan_widgets(&[&dir]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].size, FALLBACK_SIZE);
        std::fs::remove_dir_all(&dir).ok();
    }
}
