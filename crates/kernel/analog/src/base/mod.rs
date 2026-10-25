//! `analog::base`: the shared tables with no theory in them: geometry, ids,
//! netlist, placement and routing state, macros, reports. Philis 1.0's
//! `pnr_core` crate, folded in as the leaf module of `analog` (Q36). Named
//! `base`, not `core`, so it never shadows `std::core`.

pub mod geom;
pub mod hypergraph;
pub mod ids;
pub mod lanes;
pub mod layout;
pub mod r#macro;
pub mod netlist;
pub mod process;
pub mod report;
pub mod routes;
pub mod thermal;
pub mod unionfind;
pub mod units;

pub use geom::{Dir, LayerId, Orient, Pin, Rect, Shape};
pub use hypergraph::BipartiteHypergraph;
pub use ids::{AxisId, DeviceId, GroupId, NetId, Target};
pub use layout::Layout;
pub use netlist::{Device, DeviceGroup, DeviceKind, MosSize, Net, Netlist, SourceCard, SubcktInst};
pub use process::{MatchClass, Process, SubstrateKind};
pub use r#macro::{pin_shares, place_macro, place_macros, place_rect, Drawn, DrawnKind, Dummy, Figures, KeepWhy, Keepout, Macro, Node};
pub use report::{Report, Violation};
pub use routes::{GatePin, Routes, Terminal};
pub use unionfind::UnionFind;
pub use units::{PlacedUnit, Unit, UnitLib};
