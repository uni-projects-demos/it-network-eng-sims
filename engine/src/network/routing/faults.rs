use super::{LONG_OUTAGE, Rand, SHORT_OUTAGE};

pub(crate) fn should_fail(rng: &mut Rand, per_minute_percent: u8, tick_secs: f64) -> bool {
    if per_minute_percent == 0 {
        return false;
    }
    let per_min = f64::from(per_minute_percent) / 100.0;
    let tick_prob = 1.0 - (1.0 - per_min).powf(tick_secs / 60.0);
    rng.float() < tick_prob
}

pub(crate) fn random_outage(rng: &mut Rand) -> f64 {
    let (min, max) = if rng.float() < 0.5 {
        SHORT_OUTAGE
    } else {
        LONG_OUTAGE
    };
    min + rng.float() * (max - min)
}
