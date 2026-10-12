use super::{Sim, EPS, SHORT_OUTAGE, LONG_OUTAGE};

impl Sim {
    pub(super) fn rand_faults(&mut self, tick_secs: f64) {
        let due: Vec<u16> = self.nodes.values().filter(|router| {
            !router.is_running && router.auto_restart_at.is_some_and(|at| at <= self.now + EPS)
        }).map(|router| router.id).collect();
        for id in &due { self.start_router(*id, true); }
        if self.cfg.rand_fail_perc == 0 { return; }

        let per_min = f64::from(self.cfg.rand_fail_perc) / 100.0;
        let tick_prob = 1.0 - (1.0 - per_min).powf(tick_secs / 60.0);
        let candidates: Vec<u16> = self.nodes.values().filter(|router| {
            router.is_running && !router.is_quiescent && !due.contains(&router.id)
        }).map(|router| router.id).collect();
        for id in candidates {
            if self.rng.float() >= tick_prob { continue; }
            let (min, max) = if self.rng.float() < 0.5 { SHORT_OUTAGE } else { LONG_OUTAGE };
            let downtime = min + self.rng.float() * (max - min);
            self.stop_router(id, Some(downtime));
        }
    }
}
