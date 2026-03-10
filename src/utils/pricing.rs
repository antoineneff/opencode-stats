use rust_decimal::Decimal;

#[derive(Clone, Debug, Default)]
pub struct PriceSummary {
    pub known: Decimal,
    pub has_known: bool,
    pub missing: bool,
}

impl PriceSummary {
    pub fn add_known(&mut self, amount: Decimal) {
        self.known += amount;
        self.has_known = true;
    }

    pub fn add_missing(&mut self) {
        self.missing = true;
    }

    pub fn merge(&mut self, other: &Self) {
        self.known += other.known;
        self.has_known |= other.has_known;
        self.missing |= other.missing;
    }
}
