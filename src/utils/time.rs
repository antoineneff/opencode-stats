use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, TimeZone};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TimeRange {
    #[default]
    All,
    Last30Days,
    Last7Days,
}

impl TimeRange {
    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All time",
            Self::Last30Days => "Last 30 days",
            Self::Last7Days => "Last 7 days",
        }
    }

    pub fn cycle(self) -> Self {
        match self {
            Self::All => Self::Last7Days,
            Self::Last7Days => Self::Last30Days,
            Self::Last30Days => Self::All,
        }
    }

    pub fn from_shortcut(value: char) -> Option<Self> {
        match value {
            '1' => Some(Self::All),
            '2' => Some(Self::Last7Days),
            '3' => Some(Self::Last30Days),
            _ => None,
        }
    }

    pub fn start_date(self, today: NaiveDate) -> Option<NaiveDate> {
        match self {
            Self::All => None,
            Self::Last30Days => Some(today - Duration::days(29)),
            Self::Last7Days => Some(today - Duration::days(6)),
        }
    }
}

pub fn timestamp_ms_to_local(timestamp_ms: i64) -> Option<DateTime<Local>> {
    Local.timestamp_millis_opt(timestamp_ms).single()
}

pub fn current_local_date() -> NaiveDate {
    Local::now().date_naive()
}

pub fn custom_week_start(date: NaiveDate, week_start_day: u32) -> NaiveDate {
    let current_weekday = date.weekday().num_days_from_monday();
    let days_back = (7 + current_weekday as i64 - week_start_day as i64) % 7;
    date - Duration::days(days_back)
}

pub fn month_start(date: NaiveDate) -> NaiveDate {
    date.with_day(1).unwrap_or(date)
}

pub fn in_range(date: NaiveDate, range: TimeRange, today: NaiveDate) -> bool {
    match range.start_date(today) {
        Some(start) => date >= start && date <= today,
        None => date <= today,
    }
}

#[cfg(test)]
mod tests {
    use super::{custom_week_start, TimeRange};
    use chrono::NaiveDate;

    #[test]
    fn cycles_ranges() {
        assert_eq!(TimeRange::All.cycle(), TimeRange::Last7Days);
        assert_eq!(TimeRange::Last7Days.cycle(), TimeRange::Last30Days);
        assert_eq!(TimeRange::Last30Days.cycle(), TimeRange::All);
    }

    #[test]
    fn computes_custom_week_start() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 10).unwrap();
        let sunday_start = custom_week_start(date, 6);
        assert_eq!(sunday_start, NaiveDate::from_ymd_opt(2026, 3, 8).unwrap());
    }
}
