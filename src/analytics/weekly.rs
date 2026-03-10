use crate::analytics::daily::DailyUsage;
use crate::db::models::TokenUsage;
use crate::utils::pricing::PriceSummary;
use crate::utils::time::custom_week_start;
use chrono::NaiveDate;

#[derive(Clone, Debug)]
pub struct WeeklyUsage {
    pub start_date: NaiveDate,
    pub tokens: TokenUsage,
    pub interactions: usize,
    pub cost: PriceSummary,
}

pub fn aggregate_weekly(daily: &[DailyUsage], week_start_day: u32) -> Vec<WeeklyUsage> {
    let mut grouped = std::collections::BTreeMap::<NaiveDate, WeeklyUsage>::new();
    for day in daily {
        let start_date = custom_week_start(day.date, week_start_day);
        let entry = grouped.entry(start_date).or_insert_with(|| WeeklyUsage {
            start_date,
            tokens: TokenUsage::default(),
            interactions: 0,
            cost: PriceSummary::default(),
        });
        entry.tokens.add_assign(&day.tokens);
        entry.interactions += day.interactions;
        entry.cost.merge(&day.cost);
    }

    grouped.into_values().collect()
}
