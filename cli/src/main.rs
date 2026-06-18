use readme_stuff_aggregator::{
    profile::build_profile,
    widgets::{
        cf_rating_widget, cf_stats_widget, commit_streak_widget, competitive_widget,
        cw_kata_widget, cw_languages_widget, cw_rank_widget, github_contributions_widget,
        github_heatmap_widget, github_monthly_widget, github_repos_widget, github_social_widget,
        github_stats_widget, github_visitors_widget, langs_widget, lc_badges_widget,
        lc_languages_widget, lc_skills_widget, lc_solved_widget, streak_widget,
    },
};
use readme_stuff_draw::{
    Theme, render_cf_rating, render_cf_stats, render_competitive, render_cw_kata,
    render_cw_languages, render_cw_rank, render_github_commit_streak, render_github_contributions,
    render_github_heatmap, render_github_monthly, render_github_repos, render_github_social,
    render_github_stats, render_github_visitors, render_langs, render_lc_badges,
    render_lc_languages, render_lc_skills, render_lc_solved, render_streak,
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

    render_card(
        "github-stats",
        github_stats_widget(&profile),
        |w| render_github_stats(&w, Theme::Dark),
        &out_dir,
    );
    render_card(
        "github-repos",
        github_repos_widget(&profile),
        |w| render_github_repos(&w, Theme::Dark),
        &out_dir,
    );
    render_card(
        "github-contributions",
        github_contributions_widget(&profile),
        |w| render_github_contributions(&w, Theme::Dark),
        &out_dir,
    );
    render_card(
        "github-social",
        github_social_widget(&profile),
        |w| render_github_social(&w, Theme::Dark),
        &out_dir,
    );
    render_card(
        "github-heatmap",
        github_heatmap_widget(&profile),
        |w| render_github_heatmap(&w, Theme::Dark),
        &out_dir,
    );
    render_card(
        "github-monthly",
        github_monthly_widget(&profile),
        |w| render_github_monthly(&w, Theme::Dark),
        &out_dir,
    );
    render_card(
        "streak",
        streak_widget(&profile),
        |w| render_streak(&w, Theme::Dark),
        &out_dir,
    );
    render_card(
        "langs",
        langs_widget(&profile, 6),
        |w| render_langs(&w, Theme::Dark),
        &out_dir,
    );

    render_card(
        "cf-rating",
        cf_rating_widget(&profile),
        |w| render_cf_rating(&w, Theme::Dark),
        &out_dir,
    );
    render_card(
        "cf-stats",
        cf_stats_widget(&profile),
        |w| render_cf_stats(&w, Theme::Dark),
        &out_dir,
    );

    render_card(
        "cw-rank",
        cw_rank_widget(&profile),
        |w| render_cw_rank(&w, Theme::Dark),
        &out_dir,
    );
    render_card(
        "cw-kata",
        cw_kata_widget(&profile),
        |w| render_cw_kata(&w, Theme::Dark),
        &out_dir,
    );
    render_card(
        "cw-languages",
        cw_languages_widget(&profile),
        |w| render_cw_languages(&w, Theme::Dark),
        &out_dir,
    );

    render_card(
        "lc-solved",
        lc_solved_widget(&profile),
        |w| render_lc_solved(&w, Theme::Dark),
        &out_dir,
    );
    render_card(
        "lc-skills",
        lc_skills_widget(&profile),
        |w| render_lc_skills(&w, Theme::Dark),
        &out_dir,
    );
    render_card(
        "lc-languages",
        lc_languages_widget(&profile),
        |w| render_lc_languages(&w, Theme::Dark),
        &out_dir,
    );
    render_card(
        "lc-badges",
        lc_badges_widget(&profile),
        |w| render_lc_badges(&w, Theme::Dark),
        &out_dir,
    );

    render_card(
        "competitive",
        competitive_widget(&profile),
        |w| render_competitive(&w, Theme::Dark),
        &out_dir,
    );

    render_card(
        "github-visitors",
        github_visitors_widget(&profile),
        |w| render_github_visitors(&w, Theme::Dark),
        &out_dir,
    );
    render_card(
        "github-commit-streak",
        commit_streak_widget(&profile),
        |w| render_github_commit_streak(&w, Theme::Dark),
        &out_dir,
    );

    eprintln!("Done - {}", out_dir.display());
}

fn render_card<W, F>(name: &str, widget: Option<W>, render: F, dir: &Path)
where
    F: FnOnce(W) -> String,
{
    match widget {
        Some(w) => {
            let svg = render(w);
            write_svg(dir, &format!("{name}-dark.svg"), &svg);
            eprintln!("  {name} OK");
        }
        None => eprintln!("  {name} SKIP (no data)"),
    }
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
