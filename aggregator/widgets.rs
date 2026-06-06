pub struct StreakWidgetData { pub current: u32, pub longest: u32, pub total: u32 }
pub struct LangsWidgetData  { pub top: Vec<(String, f64)> }  // name + %
pub struct StatsWidgetData  { pub stars: u64, pub commits: u32, pub rank: String }

pub fn streak_widget(p: &UserProfile) -> Option<StreakWidgetData> { ... }
pub fn langs_widget(p: &UserProfile)  -> Option<LangsWidgetData>  { ... }
pub fn stats_widget(p: &UserProfile)  -> Option<StatsWidgetData>  { ... }
