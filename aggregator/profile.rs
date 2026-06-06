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

pub async fn build_profile(
    github_login: &str,
    cf_handle: &str,
    cw_username: &str,
    lc_username: &str,
) -> UserProfile {
    let gh_client = GitHubClient::from_env().ok();

    // Три async-вызова к GitHub — идут параллельно
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

    // Три blocking-вызова — нельзя вызывать прямо в async,
    // поэтому каждый уходит в spawn_blocking (отдельный пул потоков)
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
        Ok::<CodeforcesData, String>(CodeforcesData { user, rating_history })
    });

    let cw = cw_username.to_owned();
    let cw_fut = tokio::task::spawn_blocking(move || {
        CodewarsApi::default()
            .user(&cw)
            .map_err(|e| e.to_string())
    });

    let lc = lc_username.to_owned();
    let lc_fut = tokio::task::spawn_blocking(move || {
        let api = LeetcodeApi::default();
        let solved = api
            .amount_of_solved_problems(&lc)
            .map_err(|e| e.to_string())?;
        let languages = api.languages(&lc).map_err(|e| e.to_string())?;
        Ok::<LeetcodeData, String>(LeetcodeData { solved, languages })
    });

    // Все 6 запросов стартуют одновременно, ждём всех
    let (github_res, streak_res, langs_res, cf_join, cw_join, lc_join) = tokio::join!(
        github_fut,
        streak_fut,
        langs_fut,
        cf_fut,
        cw_fut,
        lc_fut,
    );

    // spawn_blocking возвращает JoinHandle<Result<T>> — разворачиваем внешний слой
    let cf_res = cf_join.unwrap_or_else(|e| Err(e.to_string()));
    let cw_res = cw_join.unwrap_or_else(|e| Err(e.to_string()));
    let lc_res = lc_join.unwrap_or_else(|e| Err(e.to_string()));

    UserProfile {
        sources: SourceStatus {
            github:     github_res.as_ref().map(|_| ()).map_err(|e| e.clone()),
            codeforces: cf_res.as_ref().map(|_| ()).map_err(|e| e.clone()),
            codewars:   cw_res.as_ref().map(|_| ()).map_err(|e| e.clone()),
            leetcode:   lc_res.as_ref().map(|_| ()).map_err(|e| e.clone()),
            visitors:   Err("not implemented".to_string()),
        },
        github:     github_res.ok(),
        streak:     streak_res.ok(),
        langs:      langs_res.ok(),
        codeforces: cf_res.ok(),
        codewars:   cw_res.ok(),
        leetcode:   lc_res.ok(),
        visitors:   None,
    }
}
