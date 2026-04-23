use rust_decimal::Decimal;

use crate::utils::pricing::PriceSummary;

pub fn format_tokens(value: u64) -> String {
    match value {
        0..=999 => value.to_string(),
        1_000..=999_999 => format!("{:.2}K", value as f64 / 1_000.0),
        1_000_000..=999_999_999 => format!("{:.2}M", value as f64 / 1_000_000.0),
        _ => format!("{:.2}B", value as f64 / 1_000_000_000.0),
    }
}

pub fn format_usd_precise(value: Decimal) -> String {
    if value >= Decimal::ONE {
        format!("${:.2}", value.round_dp(2))
    } else if value == Decimal::ZERO {
        "$0.00".to_string()
    } else {
        format!("${:.4}", value.round_dp(4))
    }
}

pub fn format_price_summary(value: &PriceSummary) -> String {
    if !value.has_known {
        return if value.missing {
            "--".to_string()
        } else {
            format_usd_precise(Decimal::ZERO)
        };
    }

    let amount = format_usd_precise(value.known);
    if value.missing {
        format!("{amount} + ?")
    } else {
        amount
    }
}

pub fn tokens_comparison_text(total_tokens: u64) -> String {
    const PAGE_TOKENS: f64 = 750.0; // 750 tokens per printed page
    const NOVAL_TOKENS: f64 = PAGE_TOKENS * 300.0; // 300 pages per novel
    const READING_SPEED: f64 = PAGE_TOKENS * 40.0; // pages per hour
    match total_tokens {
        0 => "No activity yet. Try using OpenCode!".to_string(),
        ..100_000 => format!(
            "About {:.1} printed pages of text, or {:.1} hours of reading.",
            total_tokens as f64 / PAGE_TOKENS,
            total_tokens as f64 / READING_SPEED
        ),
        100_000.. => format!(
            "Roughly {:.1} novels of text, or {:.1} hours of nonstop reading.",
            total_tokens as f64 / NOVAL_TOKENS,
            total_tokens as f64 / READING_SPEED
        ),
    }
}

pub fn percentage(part: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (part as f64 / total as f64) * 100.0
    }
}
