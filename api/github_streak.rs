use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://api.github.com";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Deserialize)]
pub struct GitHubUser {
	pub login: String,
	pub id: i64,
	pub node_id: String,
	pub avatar_url: String,
	pub gravatar_id: Option<String>,
	pub url: String,
	pub html_url: String,
	pub followers_url: String,
	pub following_url: String,
	pub gists_url: String,
	pub starred_url: String,
	pub subscriptions_url: String,
	pub organizations_url: String,
	pub repos_url: String,
	pub events_url: String,
	pub received_events_url: String,
	#[serde(rename = "type")]
	pub user_type: String,
	pub site_admin: bool,
	pub name: Option<String>,
	pub email: Option<String>,
	pub starred_at: Option<String>,
	pub user_view_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ContributorWeek {
	pub w: i64,
	pub a: i64,
	pub d: i64,
	pub c: i64,
}

#[derive(Debug, Deserialize)]
pub struct ContributorActivity {
	pub author: Option<GitHubUser>,
	pub total: i64,
	pub weeks: Vec<ContributorWeek>,
}

pub struct GithubStreakApi {
	base_url: String,
	client: reqwest::blocking::Client,
}

pub struct GithubFullCommits {
    pub total: i64,
}

impl GithubStreakApi {
	pub fn new(base_url: impl Into<String>) -> Self {
		GithubStreakApi {
			base_url: base_url.into(),
			client: reqwest::blocking::Client::builder()
				.timeout(Duration::from_secs(10))
				.user_agent("readme-stuff-api")
				.build()
				.expect("failed to build HTTP client"),
		}
	}

	fn get<T: DeserializeOwned>(&self, method: &str, params: &[(&str, String)]) -> Result<T> {
		let url = format!("{}/{}", self.base_url.trim_end_matches('/'), method);
		let response = self
			.client
			.get(&url)
			.header("Accept", "application/vnd.github+json")
			.header("X-GitHub-Api-Version", "2026-03-10")
			.query(params)
			.send()?
			.error_for_status()?;

		Ok(response.json::<T>()?)
	}

	pub fn contributor_activity(&self, owner: &str, repo: &str) -> Result<Vec<ContributorActivity>> {
		self.get(
			"repos/stats/contributors",
			&[("owner", owner.to_string()), ("repo", repo.to_string())],
		)
	}

    pub fn full_commits(&self, storage: Vec<ContributorActivity>) -> Result<GithubFullCommits> {
        let total_commits: i64 = storage.iter().map(|activity| activity.total).sum();
        Ok(GithubFullCommits { total: total_commits })
    }
}

impl Default for GithubStreakApi {
	fn default() -> Self {
		Self::new(DEFAULT_BASE_URL)
	}
}