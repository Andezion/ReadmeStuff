use crate::models::{CodeforcesData, LeetcodeData, SourceStatus, UserProfile};
use readme_stuff_api::{
    codeforce::CodeforcesApi,
    codewars::CodewarsApi,
    github_client::GitHubClient,
    github_langs::{GitHubLangsApi, LangQueryOptions},
    github_statistic::GitHubStatisticApi,
    github_streak::GitHubStreakApi,
    leetcode::LeetcodeApi,
};

#[cfg(test)]
fn load_github_token() -> Option<String> {
    if let Ok(t) = std::env::var("GITHUB_TOKEN") {
        return Some(t);
    }
    let env_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(".env");
    if let Ok(contents) = std::fs::read_to_string(&env_path) {
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(val) = line.strip_prefix("GITHUB_TOKEN=") {
                return Some(val.trim().to_owned());
            }
            // raw token (no key=value format)
            if line.starts_with("ghp_") || line.starts_with("github_pat_") {
                return Some(line.to_owned());
            }
        }
    }
    None
}

pub async fn build_profile(
    github_login: &str,
    cf_handle: &str,
    cw_username: &str,
    lc_username: &str,
) -> UserProfile {
    let gh_client = GitHubClient::from_env().ok();

    let github_fut = {
        let login = github_login.to_owned();
        let client = gh_client.clone();
        async move {
            let c = client.ok_or_else(|| "GITHUB_TOKEN not set".to_string())?;
            GitHubStatisticApi::new(c)
                .fetch_profile_stats(&login)
                .await
                .map_err(|e| e.to_string())
        }
    };

    let streak_fut = {
        let login = github_login.to_owned();
        let client = gh_client.clone();
        async move {
            let c = client.ok_or_else(|| "GITHUB_TOKEN not set".to_string())?;
            GitHubStreakApi::new(c)
                .fetch_streak_stats(&login)
                .await
                .map_err(|e| e.to_string())
        }
    };

    let langs_fut = {
        let login = github_login.to_owned();
        let client = gh_client.clone();
        async move {
            let c = client.ok_or_else(|| "GITHUB_TOKEN not set".to_string())?;
            GitHubLangsApi::new(c)
                .fetch_lang_stats(&login, &LangQueryOptions::default())
                .await
                .map_err(|e| e.to_string())
        }
    };

    let cf = cf_handle.to_owned();
    let cf_fut = tokio::task::spawn_blocking(move || {
        let api = CodeforcesApi::default();
        let user = api
            .user_info(cf.as_str())
            .map_err(|e| e.to_string())?
            .into_iter()
            .next()
            .ok_or_else(|| "user not found".to_string())?;
        let rating_history = api.user_rating(cf.as_str()).map_err(|e| e.to_string())?;
        Ok::<CodeforcesData, String>(CodeforcesData {
            user,
            rating_history,
        })
    });

    let cw = cw_username.to_owned();
    let cw_fut = tokio::task::spawn_blocking(move || {
        CodewarsApi::default().user(&cw).map_err(|e| e.to_string())
    });

    let lc = lc_username.to_owned();
    let lc_fut = tokio::task::spawn_blocking(move || {
        let api = LeetcodeApi::default();
        let solved = api
            .amount_of_solved_problems(&lc)
            .map_err(|e| e.to_string())?;
        let languages = api.languages(&lc).map_err(|e| e.to_string())?;
        let skills = api.skills(&lc).map_err(|e| e.to_string())?;
        Ok::<LeetcodeData, String>(LeetcodeData {
            solved,
            languages,
            skills_advanced: skills.advanced,
            skills_intermediate: skills.intermediate,
            skills_fundamental: skills.fundamental,
        })
    });

    let (github_res, streak_res, langs_res, cf_join, cw_join, lc_join) =
        tokio::join!(github_fut, streak_fut, langs_fut, cf_fut, cw_fut, lc_fut,);

    let cf_res = cf_join.unwrap_or_else(|e| Err(e.to_string()));
    let cw_res = cw_join.unwrap_or_else(|e| Err(e.to_string()));
    let lc_res = lc_join.unwrap_or_else(|e| Err(e.to_string()));

    UserProfile {
        sources: SourceStatus {
            github: github_res.as_ref().map(|_| ()).map_err(|e| e.clone()),
            codeforces: cf_res.as_ref().map(|_| ()).map_err(|e| e.clone()),
            codewars: cw_res.as_ref().map(|_| ()).map_err(|e| e.clone()),
            leetcode: lc_res.as_ref().map(|_| ()).map_err(|e| e.clone()),
            visitors: Err("not implemented".to_string()),
        },
        github: github_res.ok(),
        streak: streak_res.ok(),
        langs: langs_res.ok(),
        codeforces: cf_res.ok(),
        codewars: cw_res.ok(),
        leetcode: lc_res.ok(),
        visitors: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn diagnose_all_sources() {
        let token = load_github_token().expect("GITHUB_TOKEN not found in env or .env");
        unsafe { std::env::set_var("GITHUB_TOKEN", &token) };

        let profile = build_profile("Andezion", "Andezion", "Andezion", "Andezion").await;

        println!("\n         GITHUB STATS             \n");
        match &profile.sources.github {
            Ok(_) => {
                let g = profile.github.as_ref().unwrap();
                let m = &g.metadata;
                println!("login          : {}", m.login);
                println!("name           : {:?}", m.name);
                println!("bio            : {:?}", m.bio);
                println!("company        : {:?}", m.company);
                println!("location       : {:?}", m.location);
                println!("website        : {:?}", m.website_url);
                println!("created_at     : {}", m.created_at);
                println!("followers      : {}   following: {}", m.followers, m.following);
                println!("public_repos   : {}   gists: {}", m.public_repos, m.public_gists);
                println!("organizations  :");
                for org in &g.organizations {
                    println!("  - {} ({:?})", org.login, org.name);
                }
                let c = &g.contributions;
                println!("commits        : {}", c.total_commits);
                println!("pull_requests  : {}", c.total_pull_requests);
                println!("issues         : {}", c.total_issues);
                println!("pr_reviews     : {}", c.total_pull_request_reviews);
                println!("repos_contrib  : {}", c.repos_contributed_to);
                println!("restricted     : {}", c.restricted_contributions);
                println!("commits by language (this year) :");
                let mut cbl: Vec<_> = c.commits_by_language.iter().collect();
                cbl.sort_by_key(|(_, v)| std::cmp::Reverse(**v));
                for (lang, cnt) in &cbl {
                    println!("  {:20} {}", lang, cnt);
                }
                let r = &g.repos;
                println!("total_repos    : {}", r.total_repos);
                println!("total_stars    : {}", r.total_stars);
                println!("total_forks    : {}", r.total_forks);
                println!("total_watchers : {}", r.total_watchers);
                println!("top repos (up to 10):");
                for (i, repo) in r.top_repos.iter().enumerate() {
                    println!(
                        "  {}. {} ⭐{} fork={} archived={} lang={:?}",
                        i + 1,
                        repo.name,
                        repo.stars,
                        repo.is_fork,
                        repo.is_archived,
                        repo.primary_language,
                    );
                }
                println!(
                    "rank           : {} (score={:.1}, percentile={:.1}%)",
                    g.rank.grade,
                    g.rank.score,
                    g.rank.percentile * 100.0,
                );
            }
            Err(e) => println!("ERROR: {e}"),
        }

        println!("\n         GITHUB STREAK            \n");
        match &profile.streak {
            Some(s) => {
                println!("total_contributions    : {}", s.total_contributions);
                println!("current_streak         : {} days", s.current_streak);
                println!("current_streak_start   : {:?}", s.current_streak_start);
                println!("longest_streak         : {} days", s.longest_streak);
                println!("longest_streak_start   : {:?}", s.longest_streak_start);
                println!("longest_streak_end     : {:?}", s.longest_streak_end);
                println!("average_daily          : {:.2}", s.average_daily_contributions);
                println!("daily_history (days)   : {}", s.daily_history.len());
                println!("contribution_gaps (3+) : {}", s.contribution_gaps.len());
                let mut months: Vec<_> = s.monthly_totals.iter().collect();
                months.sort_by_key(|(k, _)| k.as_str());
                println!("monthly_totals         :");
                for (month, count) in &months {
                    println!("  {} : {}", month, count);
                }
                let days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
                println!("weekday_distribution   :");
                for (i, count) in s.weekday_distribution.iter().enumerate() {
                    println!("  {} : {}", days[i], count);
                }
            }
            None => println!(
                "ERROR: {:?}",
                profile.sources.github.as_ref().err()
            ),
        }

        println!("\n         GITHUB LANGUAGES         \n");
        match &profile.langs {
            Some(l) => {
                println!("total_bytes   : {}", l.total_bytes);
                println!("most_used     : {:?}", l.most_used);
                println!("repos_analysed: {}", l.repos.len());
                println!("languages     :");
                for lang in &l.languages {
                    println!(
                        "  {:20} {:5.1}%  {:>10} bytes  ~{:>8} lines  {} repos",
                        lang.name,
                        lang.percentage,
                        lang.total_bytes,
                        lang.estimated_lines,
                        lang.repo_count,
                    );
                }
            }
            None => println!("ERROR: no langs data"),
        }

        println!("\n         CODEFORCES               \n");
        match &profile.sources.codeforces {
            Ok(_) => {
                let cf = profile.codeforces.as_ref().unwrap();
                let u = &cf.user;
                println!("handle         : {}", u.handle);
                println!("first/last name: {:?} {:?}", u.first_name, u.last_name);
                println!("country/city   : {:?} / {:?}", u.country, u.city);
                println!("organization   : {:?}", u.organization);
                println!("rank           : {}   rating: {}", u.rank, u.rating);
                println!("max_rank       : {}   max_rating: {}", u.max_rank, u.max_rating);
                println!("contribution   : {}", u.contribution);
                println!("friend_of      : {}", u.friend_of_count);
                let reg = chrono::DateTime::from_timestamp(u.registration_time_seconds, 0)
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| u.registration_time_seconds.to_string());
                println!("registered     : {}", reg);
                println!("contests       : {}", cf.rating_history.len());
                for rc in &cf.rating_history {
                    println!(
                        "  {} | {} → {} (rank #{})",
                        rc.contest_name, rc.old_rating, rc.new_rating, rc.rank
                    );
                }
            }
            Err(e) => println!("ERROR: {e}"),
        }

        println!("\n         CODEWARS                 \n");
        match &profile.sources.codewars {
            Ok(_) => {
                let cw = profile.codewars.as_ref().unwrap();
                println!("username           : {}", cw.username);
                println!("name               : {:?}", cw.name);
                println!("honor              : {}", cw.honor);
                println!("clan               : {:?}", cw.clan);
                println!("leaderboard_pos    : {:?}", cw.leaderboard_position);
                println!("skills             : {:?}", cw.skills);
                println!("overall rank       : {} ({}) score={}", cw.ranks.overall.name, cw.ranks.overall.color, cw.ranks.overall.score);
                println!("per-language ranks :");
                let mut langs: Vec<_> = cw.ranks.languages.iter().collect();
                langs.sort_by_key(|(k, _)| k.as_str());
                for (lang, entry) in &langs {
                    println!("  {:12} {} score={}", lang, entry.name, entry.score);
                }
                println!("kata_completed     : {}", cw.code_challenges.total_completed);
                println!("kata_authored      : {}", cw.code_challenges.total_authored);
            }
            Err(e) => println!("ERROR: {e}"),
        }

        println!("\n         LEETCODE                 \n");
        match &profile.sources.leetcode {
            Ok(_) => {
                let lc = profile.leetcode.as_ref().unwrap();
                let s = &lc.solved;
                println!(
                    "solved  : {} (easy: {}, medium: {}, hard: {})",
                    s.solved_problem, s.easy_solved, s.medium_solved, s.hard_solved,
                );
                println!("languages :");
                for lang in &lc.languages {
                    println!("  {:12} {} problems", lang.name, lang.solved_amount);
                }
                println!("skills (advanced) :");
                for sk in &lc.skills_advanced {
                    println!("  {:30} {}", sk.name, sk.amount);
                }
                println!("skills (intermediate) :");
                for sk in &lc.skills_intermediate {
                    println!("  {:30} {}", sk.name, sk.amount);
                }
                println!("skills (fundamental) :");
                for sk in &lc.skills_fundamental {
                    println!("  {:30} {}", sk.name, sk.amount);
                }
            }
            Err(e) => println!("ERROR: {e}"),
        }

        println!("\n");
    }
}
