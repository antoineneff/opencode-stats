use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::cache::models_cache::PricingCatalog;
use crate::db::models::{TokenUsage, UsageEvent};

#[derive(Clone, Debug)]
pub struct DailyUsage {
    pub date: NaiveDate,
    pub tokens: TokenUsage,
    pub interactions: usize,
    pub cost: Decimal,
}

pub fn aggregate_daily(
    events: &[UsageEvent],
    pricing: &PricingCatalog,
    today: NaiveDate,
) -> Vec<DailyUsage> {
    let mut grouped = std::collections::BTreeMap::<NaiveDate, DailyUsage>::new();
    for event in events {
        let Some(date) = event.activity_date() else {
            continue;
        };
        if date > today {
            continue;
        }
        let entry = grouped.entry(date).or_insert_with(|| DailyUsage {
            date,
            tokens: TokenUsage::default(),
            interactions: 0,
            cost: Decimal::ZERO,
        });
        entry.tokens.add_assign(&event.tokens);
        entry.interactions += 1;
        entry.cost += pricing.cost_for_event(event);
    }

    grouped.into_values().collect()
}
