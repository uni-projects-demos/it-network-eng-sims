use super::model::{FlowConfig, FlowLink, FlowPhase, FlowRoute, FlowSnapshot, FlowStatus, PATHS};

type Triple = [usize; PATHS];

struct Frame {
    order: Vec<usize>,
    cursor: usize,
}

pub struct FlowSolver {
    cfg: FlowConfig,
    demands: Vec<u64>,
    order: Vec<usize>,
    triples: Vec<Triple>,
    rng: u32,
    work: u64,
    search_nodes: u64,
    improvements: u64,
    lower: u64,
    best_routes: Vec<Triple>,
    best_loads: Vec<u64>,
    cur_routes: Vec<Triple>,
    cur_loads: Vec<u64>,
    search_routes: Vec<Triple>,
    search_loads: Vec<u64>,
    stack: Vec<Frame>,
    is_optimal: bool,
}

impl FlowSolver {
    pub fn new(cfg: FlowConfig) -> Result<Self, String> {
        cfg.validate_solver()?;
        let demands: Vec<u64> = (1..=cfg.srcs)
            .flat_map(|i| (1..=cfg.dests).map(move |j| (i + j) as u64))
            .collect();
        let mut order: Vec<usize> = (0..demands.len()).collect();
        order.sort_by_key(|&i| std::cmp::Reverse(demands[i]));
        let mut triples = Vec::new();
        for a in 0..cfg.trans {
            for b in a + 1..cfg.trans {
                for c in b + 1..cfg.trans {
                    triples.push([a, b, c]);
                }
            }
        }
        let lower = cfg.lower_bound_scaled();
        let mut solver = Self {
            rng: cfg.seed.max(1),
            work: 0,
            search_nodes: 0,
            improvements: 0,
            lower,
            best_routes: Vec::new(),
            best_loads: Vec::new(),
            cur_routes: vec![[0; PATHS]; demands.len()],
            cur_loads: vec![0; cfg.trans],
            search_routes: vec![[0; PATHS]; demands.len()],
            search_loads: vec![0; cfg.trans],
            stack: Vec::new(),
            is_optimal: false,
            cfg,
            demands,
            order,
            triples,
        };
        if solver.cfg.trans >= PATHS {
            solver.greedy();
            solver.keep_best();
            solver.push_frame();
        }
        Ok(solver)
    }

    fn random(&mut self, bound: usize) -> usize {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        self.rng as usize % bound
    }

    fn score(loads: &[u64]) -> (u64, u64) {
        (
            *loads.iter().max().unwrap(),
            loads.iter().map(|v| v * v).sum(),
        )
    }

    fn greedy(&mut self) {
        self.cur_loads.fill(0);
        for pos in 0..self.order.len() {
            let demand = self.order[pos];
            let offset = self.random(self.cfg.trans);
            let mut bins: Vec<usize> = (0..self.cfg.trans).collect();
            bins.sort_by_key(|&k| (self.cur_loads[k], (k + offset) % self.cfg.trans));
            let route = [bins[0], bins[1], bins[2]];
            self.cur_routes[demand] = route;
            for k in route {
                self.cur_loads[k] += self.demands[demand];
            }
        }
    }

    fn keep_best(&mut self) {
        if self.best_loads.is_empty()
            || Self::score(&self.cur_loads) < Self::score(&self.best_loads)
        {
            self.best_routes.clone_from(&self.cur_routes);
            self.best_loads.clone_from(&self.cur_loads);
            self.improvements += 1;
        }
        if Self::score(&self.best_loads).0 == self.lower {
            self.is_optimal = true;
        }
    }

    fn improve(&mut self) {
        let a = self.random(self.demands.len());
        let slot_a = self.random(PATHS);
        let from = self.cur_routes[a][slot_a];
        let swap = self.random(2) == 0;
        let b = self.random(self.demands.len());
        let slot_b = self.random(PATHS);
        let to = if swap {
            self.cur_routes[b][slot_b]
        } else {
            self.random(self.cfg.trans)
        };
        if from == to
            || self.cur_routes[a].contains(&to)
            || (swap && self.cur_routes[b].contains(&from))
        {
            return;
        }
        let delta = self.demands[a] as i64 - if swap { self.demands[b] as i64 } else { 0 };
        let old_from = self.cur_loads[from] as i64;
        let old_to = self.cur_loads[to] as i64;
        let energy = 2 * delta * (old_to - old_from + delta);
        if energy > 0 && self.random(250) != 0 {
            return;
        }
        self.cur_loads[from] = (old_from - delta) as u64;
        self.cur_loads[to] = (old_to + delta) as u64;
        self.cur_routes[a][slot_a] = to;
        if swap {
            self.cur_routes[b][slot_b] = from;
        }
        self.keep_best();
    }

    fn push_frame(&mut self) {
        let mut order: Vec<usize> = (0..self.cfg.trans).collect();
        order.sort_by_key(|&k| (self.search_loads[k], k));
        self.stack.push(Frame { order, cursor: 0 });
    }

    fn search_step(&mut self) {
        let depth = self.stack.len() - 1;
        let frame = self.stack.last_mut().unwrap();
        if frame.cursor == self.triples.len() {
            self.stack.pop();
            if depth == 0 {
                self.lower += 1;
                if self.lower >= Self::score(&self.best_loads).0 {
                    self.is_optimal = true;
                } else {
                    self.push_frame();
                }
            } else {
                let demand = self.order[depth - 1];
                for k in self.search_routes[demand] {
                    self.search_loads[k] -= self.demands[demand];
                }
            }
            return;
        }
        let pos = self.triples[frame.cursor];
        frame.cursor += 1;
        self.search_nodes += 1;
        for p in 0..PATHS {
            let idx = pos[p];
            if idx > if p == 0 { 0 } else { pos[p - 1] + 1 }
                && self.search_loads[frame.order[idx]] == self.search_loads[frame.order[idx - 1]]
            {
                return;
            }
        }
        let route = pos.map(|p| frame.order[p]);
        let demand = self.order[depth];
        let weight = self.demands[demand];
        if route
            .iter()
            .any(|&k| self.search_loads[k] + weight > self.lower)
        {
            return;
        }
        for k in route {
            self.search_loads[k] += weight;
        }
        self.search_routes[demand] = route;
        if depth + 1 == self.demands.len() {
            self.best_routes.clone_from(&self.search_routes);
            self.best_loads.clone_from(&self.search_loads);
            self.improvements += 1;
            self.is_optimal = true;
        } else {
            self.push_frame();
        }
    }

    pub fn advance(&mut self, work: u32) {
        if self.cfg.trans < PATHS {
            return;
        }
        for _ in 0..work {
            if self.is_optimal {
                break;
            }
            self.work += 1;
            if self.work.is_multiple_of(4) {
                self.search_step();
            } else if self.work.is_multiple_of(4093) {
                self.greedy();
                self.keep_best();
            } else {
                self.improve();
            }
        }
    }

    pub fn is_complete(&self) -> bool {
        self.is_optimal || self.cfg.trans < PATHS
    }

    pub fn snapshot(&self) -> FlowSnapshot {
        let feasible = self.cfg.trans >= PATHS;
        let status = if !feasible {
            FlowStatus::Infeasible
        } else if self.is_optimal {
            FlowStatus::Optimal
        } else {
            FlowStatus::Feasible
        };
        let mut routes = Vec::new();
        let mut src_caps = vec![vec![0_u64; self.cfg.trans]; self.cfg.srcs];
        let mut dest_caps = vec![vec![0_u64; self.cfg.dests]; self.cfg.trans];
        if feasible {
            for (n, path) in self.best_routes.iter().enumerate() {
                let i = n / self.cfg.dests;
                let j = n % self.cfg.dests;
                let mut trans = path.map(|k| k + 1);
                trans.sort_unstable();
                routes.push(FlowRoute {
                    src: i + 1,
                    dest: j + 1,
                    demand: self.demands[n],
                    trans,
                    flow_per_path: self.demands[n] as f64 / 3.0,
                });
                for &k in path {
                    src_caps[i][k] += self.demands[n];
                    dest_caps[k][j] += self.demands[n];
                }
            }
        }
        let mut links = Vec::new();
        if feasible {
            for (i, caps) in src_caps.iter().enumerate() {
                for (k, &cap) in caps.iter().enumerate() {
                    links.push(FlowLink {
                        from: format!("S{}", i + 1),
                        to: format!("T{}", k + 1),
                        cap: cap as f64 / 3.0,
                    });
                }
            }
            for (k, caps) in dest_caps.iter().enumerate() {
                for (j, &cap) in caps.iter().enumerate() {
                    links.push(FlowLink {
                        from: format!("T{}", k + 1),
                        to: format!("D{}", j + 1),
                        cap: cap as f64 / 3.0,
                    });
                }
            }
        }
        let obj = feasible.then(|| Self::score(&self.best_loads).0 as f64 / 3.0);
        let max_cap = links.iter().map(|link| link.cap).reduce(f64::max);
        FlowSnapshot {
            config: self.cfg.clone(),
            status,
            phase: if self.is_optimal || !feasible {
                FlowPhase::Complete
            } else {
                FlowPhase::Search
            },
            work: self.work,
            search_nodes: self.search_nodes,
            improvements: self.improvements,
            lower_bound: if feasible {
                self.lower as f64 / 3.0
            } else {
                0.0
            },
            obj,
            gap: obj.map(|r| (r - self.lower as f64 / 3.0).max(0.0) / r),
            total_demand: self.cfg.total_demand(),
            loads: self.best_loads.iter().map(|&v| v as f64 / 3.0).collect(),
            routes,
            nonzero_links: links.iter().filter(|link| link.cap > 0.0).count(),
            highest_links: links
                .iter()
                .filter(|link| Some(link.cap) == max_cap)
                .cloned()
                .collect(),
            max_cap,
            links,
        }
    }
}
