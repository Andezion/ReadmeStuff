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

struct Solved {
    full_amount: u32, 
    easy_amount: u32,
    medium_amount: u32,
    hard_amount: u32
}

struct Skill {
    name: String, 
    amount: u32
}

impl LeetcodeApi {
    pub fn amount_of_solved_problems(&self, username: &str) -> Option<Solved> { // ok
        
    }

    pub fn badges(&self, username: &str) -> Option<Vec<BadgeStorage>> { // ok

    }

    pub fn languages(&self, username: &str) -> Option<Vec<Language>> { // ok    
        // https://alfa-leetcode-api.onrender.com/Andezion/language

        let languages: Vec<Language> = Vec::new();
        let url = format!("https://alfa-leetcode-api.onrender.com/{}/language", username);

        let response = reqwest::blocking::get(&url).unwrap().text().unwrap();
        let json: serde_json::Value = serde_json::from_str(&response).unwrap();
        let language_array = json.as_array().unwrap();

        for language in language_array {
            let name = language["name"].as_str().unwrap().to_string();
            let solved_amount = language["solved_amount"].as_u64().unwrap() as u32;
            languages.push(Language { name, solved_amount });
        }
        
        Some(languages)
    }

    pub fn skills(&self, username: &str) -> Option<Vec<Skill>> { // ok
        // https://alfa-leetcode-api.onrender.com/Andezion/skill

        let skills: Vec<Skill> = Vec::new();
        let url = format!("https://alfa-leetcode-api.onrender.com/{}/skill", username);

        let response = reqwest::blocking::get(&url).unwrap().text().unwrap();
        let json: serde_json::Value = serde_json::from_str(&response).unwrap();
        let skill_array = json.as_array().unwrap();

        for skill in skill_array {
            let name = skill["name"].as_str().unwrap().to_string();
            let amount = skill["amount"].as_u64().unwrap() as u32;
            skills.push(Skill { name, amount });
        }

        Some(skills)
    }
}