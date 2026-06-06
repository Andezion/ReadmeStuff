pub async fn build_profile(
    github_login: &str,
    cf_handle: &str,
    cw_username: &str,
    lc_username: &str,
) -> UserProfile {
    // async GitHub calls параллельно
    // blocking CF/CW/LC через spawn_blocking параллельно
    // собираем в UserProfile через .ok()
}
