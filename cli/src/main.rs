use readme_stuff_catalog::{BuildOutput, WidgetOutcome, build as build_pipeline};
use readme_stuff_config::{Config, defaults, io as config_io};
use readme_stuff_draw::{
    Align, DEFAULT_HEIGHT, DEFAULT_WIDTH, Theme, parse_lines, render_text_card,
};
use std::path::{Path, PathBuf};

#[tokio::main]
async fn main() {
    config_io::load_dotenv();

    let out_dir = PathBuf::from(std::env::var("OUTPUT_DIR").unwrap_or_else(|_| "profile".into()));
    std::fs::create_dir_all(&out_dir).expect("cannot create output dir");

    let text_only = cli_flag("--text-only").is_some()
        || matches!(std::env::var("TEXT_ONLY").as_deref(), Ok("1") | Ok("true"));
    if text_only {
        render_custom_text_card(&out_dir);
        eprintln!("Done - {}", out_dir.display());
        return;
    }

    let cfg = load_config();
    let login = cfg.profile.github_login.as_deref().unwrap_or("<none>");
    eprintln!("Fetching profile for {login}...");

    match build_pipeline(&cfg, &out_dir).await {
        Ok(output) => report_build(&output),
        Err(e) => {
            eprintln!("build FAILED: {e}");
            std::process::exit(1);
        }
    }

    render_custom_text_card(&out_dir);

    eprintln!("Done - {}", out_dir.display());
}

fn load_config() -> Config {
    if let Some(path) = config_io::find_config() {
        match config_io::load(&path) {
            Ok(cfg) => {
                eprintln!("  using config: {}", path.display());
                return cfg;
            }
            Err(e) => {
                eprintln!(
                    "  config at {} failed to load ({e}), falling back to defaults",
                    path.display()
                );
            }
        }
    }
    env_fallback_config()
}

fn env_fallback_config() -> Config {
    let mut cfg = defaults::default_config();
    if let Ok(v) = std::env::var("GH_LOGIN") {
        cfg.profile.github_login = Some(v);
    }
    if let Ok(v) = std::env::var("CF_HANDLE") {
        cfg.profile.codeforces_handle = Some(v);
    }
    if let Ok(v) = std::env::var("CW_USER") {
        cfg.profile.codewars_username = Some(v);
    }
    if let Ok(v) = std::env::var("LC_USER") {
        cfg.profile.leetcode_username = Some(v);
    }
    cfg
}

fn report_build(output: &BuildOutput) {
    for outcome in &output.widgets {
        match outcome {
            WidgetOutcome::Written { id, path } => eprintln!("  {id} OK -> {}", path.display()),
            WidgetOutcome::Skipped { id, reason } => eprintln!("  {id} SKIP ({reason})"),
            WidgetOutcome::Error { id, reason } => eprintln!("  {id} ERROR ({reason})"),
        }
    }
    match &output.mosaic_path {
        Some(p) => eprintln!("  compose OK -> {}", p.display()),
        None => eprintln!("  compose SKIP (no rows produced output)"),
    }
}

fn cli_flag(name: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn cli_bool_flag(name: &str) -> bool {
    std::env::args().any(|a| a == name)
}

fn positional_args() -> Vec<String> {
    const VALUE_FLAGS: &[&str] = &["--text-file", "--text-align"];

    const BOOL_FLAGS: &[&str] = &["--text-only", "-c", "--compose"];

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut positional = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if VALUE_FLAGS.contains(&a.as_str()) {
            i += 2;
        } else if BOOL_FLAGS.contains(&a.as_str()) {
            i += 1;
        } else {
            positional.push(a.clone());
            i += 1;
        }
    }
    positional
}

fn render_custom_text_card(out_dir: &Path) {
    let Some(path) = cli_flag("--text-file").or_else(|| std::env::var("TEXT_FILE").ok()) else {
        return;
    };

    let align_str = cli_flag("--text-align")
        .or_else(|| std::env::var("TEXT_ALIGN").ok())
        .unwrap_or_else(|| "left".to_owned());
    let centered = cli_bool_flag("-c")
        || matches!(
            std::env::var("TEXT_ALIGN_CENTER").as_deref(),
            Ok("1") | Ok("true")
        );
    let align = Align::parse(&align_str, centered).unwrap_or_else(|| {
        eprintln!("  custom-text: unknown align {align_str:?}, defaulting to left");
        Align::DEFAULT
    });

    let positional = positional_args();
    let svg_name = positional
        .first()
        .cloned()
        .unwrap_or_else(|| "custom-text-dark.svg".to_owned());
    let width = positional
        .get(1)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(DEFAULT_WIDTH);
    let height = positional
        .get(2)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(DEFAULT_HEIGHT);

    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("  custom-text SKIP (cannot read {path}: {e})");
            return;
        }
    };

    let lines = parse_lines(&content);
    if lines.is_empty() {
        eprintln!("  custom-text SKIP (no text lines in {path})");
        return;
    }

    let svg = render_text_card(&lines, align, Theme::Dark, width, height);
    write_svg(out_dir, &svg_name, &svg);
    eprintln!(
        "  custom-text OK ({path}, align={align_str}{c}, size={width}x{height}, file={svg_name})",
        c = if centered { " -c" } else { "" }
    );
}

fn write_svg(dir: &Path, name: &str, content: &str) {
    let path = dir.join(name);
    std::fs::write(&path, content).unwrap_or_else(|e| panic!("write {name}: {e}"));
    eprintln!("    - {}", path.display());
}
