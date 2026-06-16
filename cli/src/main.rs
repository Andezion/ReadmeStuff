use readme_stuff_aggregator::{
    profile::build_profile,
    widgets::{competitive_widget, github_stats_widget, langs_widget, streak_widget},
};
use readme_stuff_draw::{
    render_competitive, render_github_stats, render_langs, render_streak, Theme,
};
use std::path::{Path, PathBuf};

#[tokio::main]
async fn main() {
    load_dotenv();

    let gh_login = env("GH_LOGIN", "Andezion");
    let cf_handle = env("CF_HANDLE", "Andezion");
    let cw_user = env("CW_USER", "Andezion");
    let lc_user = env("LC_USER", "Andezion");
    let out_dir = PathBuf::from(std::env::var("OUTPUT_DIR").unwrap_or_else(|_| "profile".into()));

    std::fs::create_dir_all(&out_dir).expect("cannot create output dir");

    eprintln!("Fetching profile for {gh_login}...");
    let profile = build_profile(&gh_login, &cf_handle, &cw_user, &lc_user).await;

    if let Some(w) = github_stats_widget(&profile) {
        write_svg(&out_dir, "github-dark.svg",  &render_github_stats(&w, Theme::Dark));
        eprintln!("  github stats OK");
    } else {
        eprintln!("  github stats SKIP: {:?}", profile.sources.github.as_ref().err());
    }

    if let Some(w) = streak_widget(&profile) {
        write_svg(&out_dir, "streak-dark.svg",  &render_streak(&w, Theme::Dark));
        eprintln!("  streak OK");
    } else {
        eprintln!("  streak SKIP");
    }

    if let Some(w) = langs_widget(&profile, 6) {
        write_svg(&out_dir, "langs-dark.svg",  &render_langs(&w, Theme::Dark));
        eprintln!("  languages OK");
    } else {
        eprintln!("  languages SKIP");
    }

    if let Some(w) = competitive_widget(&profile) {
        write_svg(&out_dir, "competitive-dark.svg",  &render_competitive(&w, Theme::Dark));
        eprintln!("  competitive OK");
    }

    eprintln!("Done - {}", out_dir.display());
}

fn env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_owned())
}

fn write_svg(dir: &Path, name: &str, content: &str) {
    let path = dir.join(name);
    std::fs::write(&path, content).unwrap_or_else(|e| panic!("write {name}: {e}"));
    eprintln!("    - {}", path.display());
}

fn find_dotenv() -> Option<std::path::PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let candidate = dir.join(".env");
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn load_dotenv() {
    let Some(path) = find_dotenv() else { return };
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, val)) = line.split_once('=') {
            if std::env::var(key).is_err() {
                unsafe { std::env::set_var(key, val.trim()) };
            }
        } else if line.starts_with("ghp_") || line.starts_with("github_pat_") {
            if std::env::var("GITHUB_TOKEN").is_err() {
                unsafe { std::env::set_var("GITHUB_TOKEN", line) };
            }
        }
    }
}
