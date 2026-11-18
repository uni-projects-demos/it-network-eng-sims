const LOSS_AT_ONE_METER_DB: f64 = 40.0;
const PATH_LOSS_EXPONENT: f64 = 2.2;

pub(super) fn snr_db(tx_power_dbm: i16, noise_floor_dbm: i16, distance_m: f64) -> f64 {
    let loss = LOSS_AT_ONE_METER_DB + 10.0 * PATH_LOSS_EXPONENT * distance_m.max(1.0).log10();
    f64::from(tx_power_dbm) - loss - f64::from(noise_floor_dbm)
}

pub(super) fn packet_failure_prob(snr: f64) -> f64 {
    1.0 / (1.0 + ((snr - 6.0) * 0.9).exp())
}
