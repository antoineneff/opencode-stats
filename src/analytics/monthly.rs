use crate::analytics::weekly::WeeklyUsage;
use crate::db::models::TokenUsage;
use crate::utils::pricing::PriceSummary;
use crate::utils::time::month_start;
use chrono::NaiveDate;

#[derive(Clone, Debug)]
pub struct MonthlyUsage {
    pub tokens: TokenUsage,
    pub interactions: usize,
    pub cost: PriceSummary,
}

pub fn aggregate_monthly(weekly: &[WeeklyUsage]) -> Vec<MonthlyUsage> {
    let mut grouped = std::collections::BTreeMap::<NaiveDate, MonthlyUsage>::new();
    for week in weekly {
        let bucket = month_start(week.start_date);
        let entry = grouped.entry(bucket).or_insert_with(|| MonthlyUsage {
            tokens: TokenUsage::default(),
            interactions: 0,
            cost: PriceSummary::default(),
        });
        entry.tokens.add_assign(&week.tokens);
        entry.interactions += week.interactions;
        entry.cost.merge(&week.cost);
    }

    grouped.into_values().collect()
}
