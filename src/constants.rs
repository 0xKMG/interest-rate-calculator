use fixed::types::I80F48;
use fixed_macro::types::I80F48;

// Time constants
pub const SECONDS_PER_YEAR: I80F48 = I80F48!(31_557_600); // 60 * 60 * 24 * 365.25

// Curve steepness parameter
pub const CURVE_STEEPNESS: I80F48 = I80F48!(4); // Curve steepness = 4

// Initial rate at target utilization (4% per year)
pub fn initial_rate_at_target() -> I80F48 {
    I80F48!(0.04) / SECONDS_PER_YEAR
}

// Adjustment speed (50% per year)
pub fn adjustment_speed() -> I80F48 {
    I80F48!(50.0) / SECONDS_PER_YEAR // Adjustment speed in per second units
}

// Target utilization ratio (90%)
pub fn target_utilization() -> I80F48 {
    I80F48!(0.9) // Target utilization = 90%
}

// Minimum rate at target utilization (0.1% per year)
pub fn min_rate_at_target() -> I80F48 {
    I80F48!(0.001) / SECONDS_PER_YEAR // Minimum rate in per second units
}

// Maximum rate at target utilization (200% per year)
pub fn max_rate_at_target() -> I80F48 {
    I80F48!(2.0) / SECONDS_PER_YEAR // Maximum rate in per second units
}
