use super::{BTreeMap, Config, Edge, Rand};
use serde::Serialize;

#[derive(Debug, Clone)]
pub(super) struct Lsa {
    pub(super) origin: u16,
    pub(super) sequence: u32,
    pub(super) links: BTreeMap<u16, u8>,
}

#[derive(Debug, Clone)]
pub(super) struct Peer {
    pub(super) cost: u8,
    pub(super) is_active: bool,
    pub(super) last_hello: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct Route {
    pub(super) dest: u16,
    pub(super) nxt_hop: u16,
    pub(super) metric: u16,
}

#[derive(Debug, Clone)]
pub(super) struct Router {
    pub(super) id: u16,
    pub(super) is_running: bool,
    pub(super) peers: BTreeMap<u16, Peer>,
    pub(super) database: BTreeMap<u16, Lsa>,
    pub(super) sequence: u32,
    pub(super) routes: BTreeMap<u16, Route>,
    pub(super) next_hello: f64,
    pub(super) next_refresh: f64,
    pub(super) spf_due: Option<f64>,
    pub(super) auto_restart_at: Option<f64>,
}

#[derive(Debug, Clone)]
pub(super) enum Message {
    Hello,
    Lsa(Lsa),
}

#[derive(Debug, Clone)]
pub(super) struct Packet {
    pub(super) from: u16,
    pub(super) to: u16,
    pub(super) at: f64,
    pub(super) message: Message,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum EventType {
    Startup,
    Send,
    Receive,
    Periodic,
    Spf,
    Discover,
    RouteChange,
    Invalidate,
    AdjacencyDown,
    AdjacencyUp,
    RouterDown,
    RouterRestart,
    AutoFailure,
    AutoRecovery,
    LinkCost,
    Converged,
    PacketDropped,
    PacketLost,
    PacketLossSetting,
    PacketInvalidSetting,
    Complete,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct Event {
    pub(super) time: f64,
    pub(super) _type: EventType,
    pub(super) router: u16,
    pub(super) peer: Option<u16>,
    pub(super) dest: Option<u16>,
    pub(super) old_metric: Option<u16>,
    pub(super) new_metric: Option<u16>,
    pub(super) old_hop: Option<u16>,
    pub(super) new_hop: Option<u16>,
    pub(super) msg: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub(super) struct Stats {
    pub(super) updates_sent: u64,
    pub(super) updates_rcvd: u64,
    pub(super) pcks_dropped: u64,
    pub(super) pcks_lost: u64,
    pub(super) periodic_updates: u64,
    pub(super) triggered_updates: u64,
    pub(super) route_changes: u64,
    pub(super) spf_runs: u64,
    pub(super) duplicate_lsas: u64,
    pub(super) hello_sent: u64,
    pub(super) hello_rcvd: u64,
    pub(super) adjacency_changes: u64,
    pub(super) fails: u64,
    pub(super) restarts: u64,
    pub(super) rand_fails: u64,
    pub(super) rand_recovs: u64,
    pub(super) is_converged: bool,
    pub(super) accuracy: f64,
    pub(super) reachable_pairs: usize,
    pub(super) ospf_reachability: f64,
    pub(super) shortest_path_cost_mean: f64,
    pub(super) shortest_path_cost_min: u16,
    pub(super) shortest_path_cost_max: u16,
    pub(super) convergence_secs: Option<f64>,
}

#[derive(Debug, Serialize)]
pub(super) struct NodeView {
    pub(super) id: u16,
    pub(super) is_running: bool,
    pub(super) is_quiescent: bool,
    pub(super) auto_restart_in: Option<f64>,
    pub(super) active_nbrs: usize,
    pub(super) garbage_nbrs: usize,
    pub(super) routes: Vec<Route>,
    pub(super) offline_dests: Vec<u16>,
    pub(super) known_lsas: usize,
}

#[derive(Debug, Serialize)]
pub(super) struct Snapshot {
    pub(super) time: f64,
    pub(super) dur: f64,
    pub(super) hist_start: f64,
    pub(super) is_complete: bool,
    pub(super) nodes: Vec<NodeView>,
    pub(super) edges: Vec<Edge>,
    pub(super) events: Vec<Event>,
    pub(super) stats: Stats,
}

#[derive(Clone)]
pub(super) struct Sim {
    pub(super) cfg: Config,
    pub(super) now: f64,
    pub(super) rng: Rand,
    pub(super) nodes: BTreeMap<u16, Router>,
    pub(super) edges: Vec<Edge>,
    pub(super) pending: Vec<Packet>,
    pub(super) events: Vec<Event>,
    pub(super) stats: Stats,
    pub(super) topology_changed: f64,
}
