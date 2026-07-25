use crate::registry;
use readme_stuff_config::{Config, ThemeChoice};
use readme_stuff_draw::{
    Align, DEFAULT_HEIGHT, DEFAULT_WIDTH, Theme, Tile, compose, parse_lines, render_text_card,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum WidgetOutcome {
    Written { id: String, path: PathBuf },

    Skipped { id: String, reason: String },

    Error { id: String, reason: String },
}

#[derive(Debug, Clone)]
pub enum TextCardOutcome {
    Written(PathBuf),
    Skipped(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct BuildOutput {
    pub out_dir: PathBuf,
    pub widgets: Vec<WidgetOutcome>,
    pub mosaic_path: Option<PathBuf>,
    pub text_card: Option<TextCardOutcome>,
}

fn draw_theme(theme: ThemeChoice) -> Theme {
    match theme {
        ThemeChoice::Matrix => Theme::Dark,
    }
}

fn needed_credentials(cfg: &Config) -> HashSet<readme_stuff_config::Credential> {
    let available = cfg.profile.available_credentials();
    let mut needed = HashSet::new();
    for row in &cfg.layout.rows {
        for placed in &row.widgets {
            if let Some(spec) = registry::find(&placed.id) {
                needed.extend(spec.requires.credentials_to_fetch(&available));
            }
        }
    }
    needed
}

fn write_svg(dir: &Path, name: &str, content: &str) -> Result<PathBuf, String> {
    let path = dir.join(name);
    std::fs::write(&path, content).map_err(|e| format!("writing {name}: {e}"))?;
    Ok(path)
}

pub async fn build(cfg: &Config, out_dir: &Path) -> Result<BuildOutput, String> {
    std::fs::create_dir_all(out_dir).map_err(|e| format!("cannot create output dir: {e}"))?;

    let needed = needed_credentials(cfg);
    let profile =
        readme_stuff_aggregator::profile::build_profile_selective(&cfg.profile, &needed).await;
    let theme = draw_theme(cfg.theme);

    let mut outcomes = Vec::new();
    let mut row_svgs: Vec<String> = Vec::new();
    let mut row_heights: Vec<u32> = Vec::new();

    for row in &cfg.layout.rows {
        let mut rendered: Vec<(&readme_stuff_config::PlacedWidget, String, (u32, u32))> =
            Vec::new();

        for placed in &row.widgets {
            let Some(spec) = registry::find(&placed.id) else {
                outcomes.push(WidgetOutcome::Error {
                    id: placed.id.clone(),
                    reason: "no widget with this id in the registry".to_string(),
                });
                continue;
            };

            match (spec.render)(&profile, theme) {
                Some(svg) => {
                    let path = write_svg(out_dir, &format!("{}-dark.svg", spec.id), &svg)?;
                    outcomes.push(WidgetOutcome::Written {
                        id: spec.id.to_string(),
                        path,
                    });
                    rendered.push((placed, svg, spec.size));
                }
                None => outcomes.push(WidgetOutcome::Skipped {
                    id: spec.id.to_string(),
                    reason: "no data available for this source".to_string(),
                }),
            }
        }

        if rendered.is_empty() {
            continue;
        }

        let row_h = rendered
            .iter()
            .map(|(p, _, size)| p.y + size.1)
            .max()
            .unwrap_or(0);
        let tiles: Vec<Tile> = rendered
            .iter()
            .map(|(p, svg, _)| Tile {
                svg,
                x: p.x,
                y: p.y,
            })
            .collect();
        let row_svg = compose(cfg.layout.canvas_width, row_h, theme, &tiles)?;
        write_svg(
            out_dir,
            &format!("row-{}.svg", row_svgs.len() + 1),
            &row_svg,
        )?;
        row_svgs.push(row_svg);
        row_heights.push(row_h);
    }

    let mosaic_path = if row_svgs.is_empty() {
        None
    } else {
        let mut y = 0u32;
        let mosaic_tiles: Vec<Tile> = row_svgs
            .iter()
            .zip(row_heights.iter())
            .map(|(svg, h)| {
                let tile = Tile { svg, x: 0, y };
                y += h;
                tile
            })
            .collect();
        let mosaic = compose(cfg.layout.canvas_width, y, theme, &mosaic_tiles)?;
        Some(write_svg(out_dir, "profile-mosaic.svg", &mosaic)?)
    };

    let text_card = build_text_card(cfg, out_dir, theme);

    Ok(BuildOutput {
        out_dir: out_dir.to_path_buf(),
        widgets: outcomes,
        mosaic_path,
        text_card,
    })
}

fn build_text_card(cfg: &Config, out_dir: &Path, theme: Theme) -> Option<TextCardOutcome> {
    let path = cfg.text_card.file.as_ref()?;

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => return Some(TextCardOutcome::Error(format!("cannot read {path}: {e}"))),
    };

    let lines = parse_lines(&content);
    if lines.is_empty() {
        return Some(TextCardOutcome::Skipped(format!("no text lines in {path}")));
    }

    let align_str = cfg.text_card.align.as_deref().unwrap_or("left");
    let align = Align::parse(align_str, cfg.text_card.centered).unwrap_or(Align::DEFAULT);
    let width = cfg.text_card.width.unwrap_or(DEFAULT_WIDTH);
    let height = cfg.text_card.height.unwrap_or(DEFAULT_HEIGHT);
    let name = cfg
        .text_card
        .output
        .as_deref()
        .unwrap_or("custom-text-dark.svg");

    let svg = render_text_card(&lines, align, theme, width, height);
    match write_svg(out_dir, name, &svg) {
        Ok(p) => Some(TextCardOutcome::Written(p)),
        Err(e) => Some(TextCardOutcome::Error(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use readme_stuff_config::{Layout, PlacedWidget, ProfileConfig, Row};

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "readme-stuff-catalog-test-{name}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[tokio::test]
    async fn empty_layout_produces_no_files_and_no_mosaic() {
        let cfg = Config {
            profile: ProfileConfig::default(),
            theme: ThemeChoice::Matrix,
            layout: Layout {
                canvas_width: 990,
                rows: vec![],
            },
            ..Default::default()
        };
        let dir = temp_dir("empty");
        let out = build(&cfg, &dir)
            .await
            .expect("build should not error on an empty layout");
        assert!(out.widgets.is_empty());
        assert!(out.mosaic_path.is_none());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn unconfigured_source_is_skipped_not_errored() {
        let cfg = Config {
            profile: ProfileConfig::default(),
            theme: ThemeChoice::Matrix,
            layout: Layout {
                canvas_width: 990,
                rows: vec![Row {
                    widgets: vec![PlacedWidget {
                        id: "cf-rating".to_string(),
                        x: 0,
                        y: 0,
                    }],
                }],
            },
            ..Default::default()
        };
        let dir = temp_dir("skip");
        let out = build(&cfg, &dir).await.expect("build should not error");
        assert_eq!(out.widgets.len(), 1);
        assert!(matches!(&out.widgets[0], WidgetOutcome::Skipped { id, .. } if id == "cf-rating"));
        assert!(out.mosaic_path.is_none());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn unknown_widget_id_is_reported_as_error() {
        let cfg = Config {
            profile: ProfileConfig::default(),
            theme: ThemeChoice::Matrix,
            layout: Layout {
                canvas_width: 990,
                rows: vec![Row {
                    widgets: vec![PlacedWidget {
                        id: "does-not-exist".to_string(),
                        x: 0,
                        y: 0,
                    }],
                }],
            },
            ..Default::default()
        };
        let dir = temp_dir("unknown");
        let out = build(&cfg, &dir).await.expect("build should not error");
        assert!(
            matches!(&out.widgets[0], WidgetOutcome::Error { id, .. } if id == "does-not-exist")
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn no_text_card_configured_yields_no_outcome() {
        let cfg = Config {
            profile: ProfileConfig::default(),
            theme: ThemeChoice::Matrix,
            layout: Layout {
                canvas_width: 990,
                rows: vec![],
            },
            ..Default::default()
        };
        let dir = temp_dir("text-card-none");
        let out = build(&cfg, &dir).await.expect("build should not error");
        assert!(out.text_card.is_none());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn configured_text_card_is_rendered_alongside_widgets() {
        let dir = temp_dir("text-card-written");
        let text_path = dir.join("greeting.txt");
        std::fs::write(&text_path, "hello there(20)\n").unwrap();

        let cfg = Config {
            profile: ProfileConfig::default(),
            theme: ThemeChoice::Matrix,
            layout: Layout {
                canvas_width: 990,
                rows: vec![],
            },
            text_card: readme_stuff_config::TextCardConfig {
                file: Some(text_path.display().to_string()),
                ..Default::default()
            },
        };
        let out = build(&cfg, &dir).await.expect("build should not error");
        match out.text_card {
            Some(TextCardOutcome::Written(path)) => {
                assert!(path.exists());
                assert_eq!(path.file_name().unwrap(), "custom-text-dark.svg");
            }
            other => panic!("expected a written text card, got {other:?}"),
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn missing_text_card_file_is_reported_as_an_error() {
        let dir = temp_dir("text-card-missing");
        let cfg = Config {
            profile: ProfileConfig::default(),
            theme: ThemeChoice::Matrix,
            layout: Layout {
                canvas_width: 990,
                rows: vec![],
            },
            text_card: readme_stuff_config::TextCardConfig {
                file: Some(dir.join("does-not-exist.txt").display().to_string()),
                ..Default::default()
            },
        };
        let out = build(&cfg, &dir).await.expect("build should not error");
        assert!(matches!(out.text_card, Some(TextCardOutcome::Error(_))));
        std::fs::remove_dir_all(&dir).ok();
    }
}
