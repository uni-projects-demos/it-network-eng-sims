use super::{CONTINUOUS_DUR, EPS, HIST_TICK};

pub(crate) const CONT_HIST_LIMIT: usize = 1_201;
const CONT_CHECKPOINT_PERIOD: f64 = 0.25;
const HIST_TRIM_BATCH: usize = 64;

pub(crate) trait HistoryState: Clone {
    type Event: Clone;
    fn now(&self) -> f64;
    fn end_time(&self) -> Option<f64>;
    fn duration(&self) -> u16;
    fn advance(&mut self, delta: f64);
    fn events_mut(&mut self) -> &mut Vec<Self::Event>;
}

pub(crate) struct History<S: HistoryState> {
    pub(crate) state: S,
    checkpoints: Vec<S>,
    cursor: usize,
}

impl<S: HistoryState> History<S> {
    pub(crate) fn new(mut state: S) -> Self {
        let events = std::mem::take(state.events_mut());
        let checkpoint = state.clone();
        *state.events_mut() = events;
        Self {
            state,
            checkpoints: vec![checkpoint],
            cursor: 0,
        }
    }

    fn checkpoint(&mut self) -> S {
        let events = std::mem::take(self.state.events_mut());
        let checkpoint = self.state.clone();
        *self.state.events_mut() = events;
        checkpoint
    }

    fn record(&mut self) {
        self.checkpoints.truncate(self.cursor + 1);
        let checkpoint = self.checkpoint();
        self.checkpoints.push(checkpoint);
        self.cursor += 1;
        if self.state.duration() == CONTINUOUS_DUR && self.checkpoints.len() > CONT_HIST_LIMIT {
            self.checkpoints.drain(..HIST_TRIM_BATCH);
            self.cursor -= HIST_TRIM_BATCH;
        }
    }

    fn restore(&mut self, index: usize) {
        self.cursor = index;
        self.state = self.checkpoints[index].clone();
        self.state.events_mut().clear();
    }

    pub(crate) fn earliest_time(&self) -> f64 {
        self.checkpoints.first().map_or(0.0, |state| state.now())
    }

    pub(crate) fn edit(&mut self, f: impl FnOnce(&mut S) -> bool) {
        self.checkpoints.truncate(self.cursor + 1);
        if f(&mut self.state) {
            if self.state.now() > self.checkpoints[self.cursor].now() + EPS {
                self.record();
            } else {
                self.checkpoints[self.cursor] = self.checkpoint();
            }
        }
    }

    pub(crate) fn advance_to(&mut self, time: f64) {
        let target = self.state.end_time().map_or(time, |limit| time.min(limit));
        while self.state.now() + EPS < target {
            let replaying = self.cursor + 1 < self.checkpoints.len();
            if replaying && self.checkpoints[self.cursor + 1].now() <= target + EPS {
                self.restore(self.cursor + 1);
                continue;
            }
            let delta = (target - self.state.now()).min(HIST_TICK);
            let old = std::mem::take(self.state.events_mut());
            self.state.advance(delta);
            let added = std::mem::take(self.state.events_mut());
            let elapsed = self.state.now() - self.checkpoints[self.cursor].now();
            if !replaying
                && (self.state.duration() != CONTINUOUS_DUR
                    || elapsed + EPS >= CONT_CHECKPOINT_PERIOD)
            {
                self.record();
            }
            *self.state.events_mut() = old;
            self.state.events_mut().extend(added);
        }
    }

    pub(crate) fn step_back(&mut self) {
        let target = (self.state.now() - 1.0).max(0.0);
        let index = self
            .checkpoints
            .iter()
            .rposition(|s| s.now() <= target + EPS)
            .unwrap_or(0);
        self.restore(index);
    }
    pub(crate) fn seek_start(&mut self) {
        self.restore(0);
    }
    pub(crate) fn seek_end(&mut self) {
        if let Some(limit) = self.state.end_time() {
            self.advance_to(limit);
        }
    }
}
