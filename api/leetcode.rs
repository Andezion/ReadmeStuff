struct Badge {
    id: u32,
    name: String,
    icon_url: String,
    date: String
}

struct BadgeStorage {
    counter: u32, 
    badges: Vec<Badge>
}

struct Language {
    name: String, 
    solved_amount: u32
}

impl LeetcodeApi {
    pub fn amount_of_solved_problems(&self, username: &str) -> Option<u32> {
        
    }

    pub fn badges(&self, username: &str) -> Option<Vec<BadgeStorage>> {

    }

    pub fn languages(&self, username: &str) -> Option<Vec<Language>> {

    }
}