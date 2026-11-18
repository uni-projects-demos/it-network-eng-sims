use super::{Sim, EPS};
use super::super::routing::{should_fail, random_outage};

impl Sim {
    pub(super) fn rand_faults(&mut self, tick_secs: f64) {
        let due: Vec<u16> = self.nodes.values().filter(|router| {
            !router.is_running && router.auto_restart_at.is_some_and(|at| at <= self.now + EPS)
        }).map(|router| router.id).collect();
        for id in &due { self.start_router(*id, true); }
        let candidates: Vec<u16> = self.nodes.values().filter(|router| {
            router.is_running && !router.is_quiescent && !due.contains(&router.id)
        }).map(|router| router.id).collect();
        for id in candidates {
            if should_fail(&mut self.rng, self.cfg.rand_fail_perc, tick_secs) {
                let downtime = random_outage(&mut self.rng);
                self.stop_router(id, Some(downtime));
            }
        }
    }
}
