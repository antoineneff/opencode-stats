use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::analytics::weekly::WeeklyUsage;
use crate::db::models::TokenUsage;
use crate::utils::time::month_start;

#[derive(Clone, Debug)]
pub struct MonthlyUsage {
    pub tokens: TokenUsage,
    pub interactions: usize,
    pub cost: Decimal,
}

pub fn aggregate_monthly(weekly: &[WeeklyUsage]) -> Vec<MonthlyUsage> {
    let mut grouped = std::collections::BTreeMap::<NaiveDate, MonthlyUsage>::new();
    for week in weekly {
        let bucket = month_start(week.start_date);
        let entry = grouped.entry(bucket).or_insert_with(|| MonthlyUsage {
            tokens: TokenUsage::default(),
            interactions: 0,
            cost: Decimal::ZERO,
        });
        entry.tokens.add_assign(&week.tokens);
        entry.interactions += week.interactions;
        entry.cost += week.cost;
    }

    grouped.into_values().collect()
}
