#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Credential {
    GitHubToken,
    GitHubLogin,
    CodeforcesHandle,
    CodewarsUsername,
    LeetcodeUsername,
}

#[derive(Debug, Clone, Copy)]
pub enum Requirement {
    All(&'static [Credential]),
    
    AnyOf(&'static [Credential]),
}

impl Requirement {
    pub fn is_satisfied(&self, available: &std::collections::HashSet<Credential>) -> bool {
        match self {
            Requirement::All(reqs) => reqs.iter().all(|r| available.contains(r)),
            Requirement::AnyOf(reqs) => reqs.iter().any(|r| available.contains(r)),
        }
    }

    pub fn credentials_to_fetch(&self, available: &std::collections::HashSet<Credential>) -> Vec<Credential> {
        match self {
            Requirement::All(reqs) => {
                if reqs.iter().all(|r| available.contains(r)) {
                    reqs.to_vec()
                } else {
                    Vec::new()
                }
            }
            Requirement::AnyOf(reqs) => reqs.iter().copied().filter(|r| available.contains(r)).collect(),
        }
    }
}
