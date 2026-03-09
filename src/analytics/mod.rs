pub mod daily;
pub mod heatmap_data;
pub mod model_stats;
pub mod monthly;
pub mod weekly;

use std::collections::BTreeSet;

use chrono::{DateTime, Local, NaiveDate};
use rust_decimal::Decimal;

use crate::analytics::daily::aggregate_daily;
use crate::analytics::heatmap_data::{build_heatmap_data, HeatmapData};
use crate::analytics::model_stats::{build_model_chart, ModelChartData, ModelUsageRow};
use crate::analytics::monthly::aggregate_monthly;
use crate::analytics::weekly::aggregate_weekly;
use crate::cache::models_cache::PricingCatalog;
use crate::db::models::{AppData, UsageEvent};
use crate::utils::time::{current_local_date, TimeRange};

#[derive(Clone, Debug)]
pub struct OverviewStats {
    pub total_tokens: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_tokens: u64,
    pub total_cost: Decimal,
    pub sessions: usize,
    pub interactions: usize,
    pub models_used: usize,
    pub active_days: usize,
    pub average_tokens_per_day: u64,
    pub latest_activity: Option<DateTime<Local>>,
    pub fun_comparison: String,
}

#[derive(Clone, Debug)]
pub struct AnalyticsSnapshot {
    pub overview: OverviewStats,
    pub models: Vec<ModelUsageRow>,
    pub chart: ModelChartData,
    pub heatmap: HeatmapData,
}

pub fn build_snapshot(
    data: &AppData,
    pricing: &PricingCatalog,
    range: TimeRange,
) -> AnalyticsSnapshot {
    let today = current_local_date();
    let filtered_events = filter_events(&data.events, range, today);
    let daily = aggregate_daily(&filtered_events, pricing, today);
    let weekly = aggregate_weekly(&daily, 0);
    let _monthly = aggregate_monthly(&weekly);
    let (models, chart) = build_model_chart(&filtered_events, pricing, range, today);
    let heatmap = build_heatmap_data(&data.events, today);

    let total_tokens = filtered_events
        .iter()
        .map(|event| event.tokens.total())
        .sum::<u64>();
    let input_tokens = filtered_events
        .iter()
        .map(|event| event.tokens.input)
        .sum::<u64>();
    let output_tokens = filtered_events
        .iter()
        .map(|event| event.tokens.output)
        .sum::<u64>();
    let cache_tokens = filtered_events
        .iter()
        .map(|event| event.tokens.cache_read + event.tokens.cache_write)
        .sum::<u64>();
    let total_cost = filtered_events
        .iter()
        .map(|event| pricing.cost_for_event(event))
        .sum::<Decimal>();
    let sessions = filtered_events
        .iter()
        .map(|event| event.session_id.clone())
        .collect::<BTreeSet<_>>()
        .len();
    let interactions = filtered_events.len();
    let models_used = filtered_events
        .iter()
        .map(|event| event.model_id.clone())
        .collect::<BTreeSet<_>>()
        .len();
    let active_days = daily.len();
    let average_tokens_per_day = if active_days == 0 {
        0
    } else {
        total_tokens / active_days as u64
    };
    let latest_activity = filtered_events
        .iter()
        .filter_map(|event| event.completed_at.or(event.created_at))
        .max();

    AnalyticsSnapshot {
        overview: OverviewStats {
            total_tokens,
            input_tokens,
            output_tokens,
            cache_tokens,
            total_cost,
            sessions,
            interactions,
            models_used,
            active_days,
            average_tokens_per_day,
            latest_activity,
            fun_comparison: crate::utils::formatting::tokens_comparison_text(total_tokens),
        },
        models,
        chart,
        heatmap,
    }
}

fn filter_events(events: &[UsageEvent], range: TimeRange, today: NaiveDate) -> Vec<UsageEvent> {
    events
        .iter()
        .filter(|event| {
            event
                .activity_date()
                .is_some_and(|date| crate::utils::time::in_range(date, range, today))
        })
        .cloned()
        .collect()
}
