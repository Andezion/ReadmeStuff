use crate::github_client::{GitHubClient, Result};
use crate::github_streak::GitHubStreakApi;
use chrono::NaiveDate;

#[derive(Debug, Clone, Default)]
pub struct CommitStreakStats {
    pub total_commits: u64,
    pub days_with_commits: u32,
    pub current_streak: u32,
    pub current_streak_start: Option<NaiveDate>,
    pub longest_streak: u32,
    pub longest_streak_start: Option<NaiveDate>,
    pub longest_streak_end: Option<NaiveDate>,
}

pub struct GitHubCommitStreakApi {
    inner: GitHubStreakApi,
}

impl GitHubCommitStreakApi {
    pub fn new(client: GitHubClient) -> Self {
        Self {
            inner: GitHubStreakApi::new(client),
        }
    }

    pub async fn fetch_stats(&self, login: &str) -> Result<CommitStreakStats> {
        let s = self.inner.fetch_streak_stats(login).await?;
        let days_with_commits = s
            .daily_history
            .iter()
            .filter(|d| d.contribution_count > 0)
            .count() as u32;

        Ok(CommitStreakStats {
            total_commits: s.total_contributions as u64,
            days_with_commits,
            current_streak: s.current_streak,
            current_streak_start: s.current_streak_start,
            longest_streak: s.longest_streak,
            longest_streak_start: s.longest_streak_start,
            longest_streak_end: s.longest_streak_end,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn live_commit_streak_andezion() {
        let Ok(client) = GitHubClient::from_env() else {
            eprintln!("GITHUB_TOKEN not set - skipping live test");
            return;
        };
        let api = GitHubCommitStreakApi::new(client);
        let stats = api.fetch_stats("Andezion").await.unwrap();

        println!("total_commits       : {}", stats.total_commits);
        println!("days_with_commits   : {}", stats.days_with_commits);
        println!("current_streak      : {} days", stats.current_streak);
        println!("current_streak_start: {:?}", stats.current_streak_start);
        println!("longest_streak      : {} days", stats.longest_streak);
        println!("longest_streak_start: {:?}", stats.longest_streak_start);
        println!("longest_streak_end  : {:?}", stats.longest_streak_end);

        assert!(stats.total_commits > 0 || stats.days_with_commits == 0);
    }
}
