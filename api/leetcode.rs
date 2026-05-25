use serde::Deserialize;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://alfa-leetcode-api.onrender.com";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Deserialize)]
pub struct Badge {
    pub id: u32,
    pub name: String,
    pub icon_url: String,
    pub date: String,
}

#[derive(Deserialize)]
pub struct BadgeStorage {
    pub counter: u32,
    pub badges: Vec<Badge>,
}

#[derive(Deserialize)]
pub struct Language {
    pub name: String,
    pub solved_amount: u32,
}

#[derive(Deserialize)]
pub struct Solved {
    #[serde(rename = "solvedProblem")]
    pub solved_problem: u32,
    #[serde(rename = "easySolved")]
    pub easy_solved: u32,
    #[serde(rename = "mediumSolved")]
    pub medium_solved: u32,
    #[serde(rename = "hardSolved")]
    pub hard_solved: u32,
}

#[derive(Deserialize)]
pub struct Skill {
    pub name: String,
    pub amount: u32,
}

pub struct LeetcodeApi {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl LeetcodeApi {
    pub fn new(base_url: impl Into<String>) -> Self {
        LeetcodeApi {
            base_url: base_url.into(),
            client: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("failed to build HTTP client"),
        }
    }

    pub fn amount_of_solved_problems(&self, username: &str) -> Result<Solved> {
        let url = format!("{}/{}/solved", self.base_url, username);
        let solved = self.client.get(&url).send()?.json::<Solved>()?;
        Ok(solved)
    }

    pub fn badges(&self, username: &str) -> Result<Vec<BadgeStorage>> {
        let url = format!("{}/{}/badges", self.base_url, username);
        let badges = self.client.get(&url).send()?.json::<Vec<BadgeStorage>>()?;
        Ok(badges)
    }

    pub fn languages(&self, username: &str) -> Result<Vec<Language>> {
        let url = format!("{}/{}/language", self.base_url, username);
        let languages = self.client.get(&url).send()?.json::<Vec<Language>>()?;
        Ok(languages)
    }

    pub fn skills(&self, username: &str) -> Result<Vec<Skill>> {
        let url = format!("{}/{}/skill", self.base_url, username);
        let skills = self.client.get(&url).send()?.json::<Vec<Skill>>()?;
        Ok(skills)
    }
}

impl Default for LeetcodeApi {
    fn default() -> Self {
        Self::new(DEFAULT_BASE_URL)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;

    #[test]
    fn test_leetcode() -> Result<()> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        let solved: serde_json::Value = client
            .get(format!("{DEFAULT_BASE_URL}/Andezion/solved"))
            .send()?
            .json()?;
        println!("Solved:\n{}", serde_json::to_string_pretty(&solved)?);
        assert_eq!(solved["solvedProblem"], 611);

        let language: serde_json::Value = client
            .get(format!("{DEFAULT_BASE_URL}/Andezion/language"))
            .send()?
            .json()?;
        println!("Language:\n{}", serde_json::to_string_pretty(&language)?);

        let skill: serde_json::Value = client
            .get(format!("{DEFAULT_BASE_URL}/Andezion/skill"))
            .send()?
            .json()?;
        println!("Skill:\n{}", serde_json::to_string_pretty(&skill)?);

        let badges: serde_json::Value = client
            .get(format!("{DEFAULT_BASE_URL}/Andezion/badges"))
            .send()?
            .json()?;
        println!("Badges:\n{}", serde_json::to_string_pretty(&badges)?);

        Ok(())
    }
}


/*

Solved:
{
  "acSubmissionNum": [
    {
      "count": 611,
      "difficulty": "All",
      "submissions": 1600
    },
    {
      "count": 377,
      "difficulty": "Easy",
      "submissions": 1010
    },
    {
      "count": 207,
      "difficulty": "Medium",
      "submissions": 550
    },
    {
      "count": 27,
      "difficulty": "Hard",
      "submissions": 40
    }
  ],
  "easySolved": 377,
  "hardSolved": 27,
  "mediumSolved": 207,
  "solvedProblem": 611,
  "totalSubmissionNum": [
    {
      "count": 694,
      "difficulty": "All",
      "submissions": 2631
    },
    {
      "count": 402,
      "difficulty": "Easy",
      "submissions": 1620
    },
    {
      "count": 245,
      "difficulty": "Medium",
      "submissions": 928
    },
    {
      "count": 47,
      "difficulty": "Hard",
      "submissions": 83
    }
  ]
}
Language:
{
  "languageProblemCount": [
    {
      "languageName": "C++",
      "problemsSolved": 535
    },
    {
      "languageName": "C",
      "problemsSolved": 213
    },
    {
      "languageName": "C#",
      "problemsSolved": 93
    },
    {
      "languageName": "Rust",
      "problemsSolved": 76
    }
  ]
}
Skill:
{
  "advanced": [
    {
      "problemsSolved": 6,
      "tagName": "Game Theory",
      "tagSlug": "game-theory"
    },
    {
      "problemsSolved": 1,
      "tagName": "Sweep Line",
      "tagSlug": "line-sweep"
    },
    {
      "problemsSolved": 11,
      "tagName": "Backtracking",
      "tagSlug": "backtracking"
    },
    {
      "problemsSolved": 2,
      "tagName": "Bitmask",
      "tagSlug": "bitmask"
    },
    {
      "problemsSolved": 3,
      "tagName": "Quickselect",
      "tagSlug": "quickselect"
    },
    {
      "problemsSolved": 42,
      "tagName": "Dynamic Programming",
      "tagSlug": "dynamic-programming"
    },
    {
      "problemsSolved": 10,
      "tagName": "Divide and Conquer",
      "tagSlug": "divide-and-conquer"
    },
    {
      "problemsSolved": 2,
      "tagName": "Trie",
      "tagSlug": "trie"
    },
    {
      "problemsSolved": 2,
      "tagName": "Union-Find",
      "tagSlug": "union-find"
    },
    {
      "problemsSolved": 1,
      "tagName": "Binary Indexed Tree",
      "tagSlug": "binary-indexed-tree"
    },
    {
      "problemsSolved": 1,
      "tagName": "Segment Tree",
      "tagSlug": "segment-tree"
    },
    {
      "problemsSolved": 5,
      "tagName": "Monotonic Stack",
      "tagSlug": "monotonic-stack"
    },
    {
      "problemsSolved": 1,
      "tagName": "Monotonic Queue",
      "tagSlug": "monotonic-queue"
    },
    {
      "problemsSolved": 1,
      "tagName": "Topological Sort",
      "tagSlug": "topological-sort"
    },
    {
      "problemsSolved": 2,
      "tagName": "Shortest Path",
      "tagSlug": "shortest-path"
    }
  ],
  "fundamental": [
    {
      "problemsSolved": 369,
      "tagName": "Array",
      "tagSlug": "array"
    },
    {
      "problemsSolved": 37,
      "tagName": "Matrix",
      "tagSlug": "matrix"
    },
    {
      "problemsSolved": 150,
      "tagName": "String",
      "tagSlug": "string"
    },
    {
      "problemsSolved": 63,
      "tagName": "Simulation",
      "tagSlug": "simulation"
    },
    {
      "problemsSolved": 25,
      "tagName": "Enumeration",
      "tagSlug": "enumeration"
    },
    {
      "problemsSolved": 87,
      "tagName": "Sorting",
      "tagSlug": "sorting"
    },
    {
      "problemsSolved": 19,
      "tagName": "Stack",
      "tagSlug": "stack"
    },
    {
      "problemsSolved": 4,
      "tagName": "Queue",
      "tagSlug": "queue"
    },
    {
      "problemsSolved": 6,
      "tagName": "Linked List",
      "tagSlug": "linked-list"
    },
    {
      "problemsSolved": 60,
      "tagName": "Two Pointers",
      "tagSlug": "two-pointers"
    }
  ],
  "intermediate": [
    {
      "problemsSolved": 19,
      "tagName": "Tree",
      "tagSlug": "tree"
    },
    {
      "problemsSolved": 16,
      "tagName": "Binary Tree",
      "tagSlug": "binary-tree"
    },
    {
      "problemsSolved": 128,
      "tagName": "Hash Table",
      "tagSlug": "hash-table"
    },
    {
      "problemsSolved": 2,
      "tagName": "Ordered Set",
      "tagSlug": "ordered-set"
    },
    {
      "problemsSolved": 7,
      "tagName": "Graph Theory",
      "tagSlug": "graph"
    },
    {
      "problemsSolved": 56,
      "tagName": "Greedy",
      "tagSlug": "greedy"
    },
    {
      "problemsSolved": 51,
      "tagName": "Binary Search",
      "tagSlug": "binary-search"
    },
    {
      "problemsSolved": 23,
      "tagName": "Depth-First Search",
      "tagSlug": "depth-first-search"
    },
    {
      "problemsSolved": 15,
      "tagName": "Breadth-First Search",
      "tagSlug": "breadth-first-search"
    },
    {
      "problemsSolved": 13,
      "tagName": "Recursion",
      "tagSlug": "recursion"
    },
    {
      "problemsSolved": 19,
      "tagName": "Sliding Window",
      "tagSlug": "sliding-window"
    },
    {
      "problemsSolved": 50,
      "tagName": "Bit Manipulation",
      "tagSlug": "bit-manipulation"
    },
    {
      "problemsSolved": 198,
      "tagName": "Math",
      "tagSlug": "math"
    },
    {
      "problemsSolved": 5,
      "tagName": "Design",
      "tagSlug": "design"
    },
    {
      "problemsSolved": 6,
      "tagName": "Brainteaser",
      "tagSlug": "brainteaser"
    }
  ]
}
Badges:
{
  "activeBadge": {
    "creationDate": "2025-10-02",
    "displayName": "365 Days Badge",
    "icon": "https://assets.leetcode.com/static_assets/marketing/lg365.png",
    "id": "8300820"
  },
  "badges": [
    {
      "creationDate": "2025-10-02",
      "displayName": "365 Days Badge",
      "icon": "https://assets.leetcode.com/static_assets/marketing/lg365.png",
      "id": "8300820"
    },
    {
      "creationDate": "2025-10-11",
      "displayName": "100 Days Badge 2025",
      "icon": "https://assets.leetcode.com/static_assets/others/lg25100.png",
      "id": "8394295"
    },
    {
      "creationDate": "2025-07-12",
      "displayName": "50 Days Badge 2025",
      "icon": "https://assets.leetcode.com/static_assets/others/lg2550.png",
      "id": "7539505"
    },
    {
      "creationDate": "2024-09-13",
      "displayName": "50 Days Badge 2024",
      "icon": "https://assets.leetcode.com/static_assets/marketing/2024-50-lg.png",
      "id": "4929959"
    },
    {
      "creationDate": "2023-12-15",
      "displayName": "50 Days Badge 2023",
      "icon": "https://assets.leetcode.com/static_assets/marketing/lg50.png",
      "id": "2812241"
    },
    {
      "creationDate": "2023-12-15",
      "displayName": "100 Days Badge 2023",
      "icon": "https://assets.leetcode.com/static_assets/marketing/lg100.png",
      "id": "2812240"
    },
    {
      "creationDate": "2025-10-31",
      "displayName": "Oct LeetCoding Challenge",
      "icon": "/static/images/badges/dcc-2025-10.png",
      "id": "8563179"
    },
    {
      "creationDate": "2025-05-31",
      "displayName": "May LeetCoding Challenge",
      "icon": "/static/images/badges/dcc-2025-5.png",
      "id": "7175761"
    }
  ],
  "badgesCount": 8,
  "upcomingBadges": [
    {
      "icon": "/static/images/badges/dcc-2026-5.png",
      "name": "May LeetCoding Challenge"
    },
    {
      "icon": "/static/images/badges/dcc-2026-6.png",
      "name": "Jun LeetCoding Challenge"
    },
    {
      "icon": "/static/images/badges/dcc-2026-7.png",
      "name": "Jul LeetCoding Challenge"
    }
  ]
}

*/