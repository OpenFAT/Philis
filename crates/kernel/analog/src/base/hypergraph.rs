//! The netlist as a bipartite device↔net incidence graph — what each analog
//! rule's `extract` walks.
//!
//! Terminal order follows [`crate::base::Device::terminals`]; for FETs that is
//! **G, D, S, B = 0, 1, 2, 3**, and recognisers key off those positions
//! (diode-connected: `device_nets[d][G] == device_nets[d][D]`).
//!
//! In → out: [`Netlist`] → [`BipartiteHypergraph`] (both incidence directions).
//! Cites: Philis 1.0 `pnr_core` (Q36).
//!
//! Contents:
//! - `pub struct BipartiteHypergraph`
//! - `BipartiteHypergraph::from_netlist(nl: &Netlist) -> Self`
//! - `BipartiteHypergraph::device_count(&self) -> usize`

use crate::base::ids::{DeviceId, NetId};
use crate::base::netlist::{DeviceKind, Netlist};

/// The netlist's incidence in both directions, every row indexed by the same
/// [`DeviceId`] / [`NetId`] as the [`Netlist`] it was built from.
pub struct BipartiteHypergraph {
    /// Nets per device, in terminal order, by [`DeviceId`].
    pub device_nets: Vec<Vec<NetId>>,
    /// Devices per net, by [`NetId`].
    pub net_devices: Vec<Vec<DeviceId>>,
    /// Kind per device, parallel to `device_nets`.
    pub kinds: Vec<DeviceKind>,
    /// Pin names parallel to each `device_nets` row.
    pub terminals: Vec<Vec<String>>,
    /// Net name by [`NetId`] (supply/rail classification).
    pub net_names: Vec<String>,
}

impl BipartiteHypergraph {
    /// Copies `nl`'s incidence; O(devices × terminals), one `String` clone per
    /// pin and per net name.
    #[must_use]
    pub fn from_netlist(nl: &Netlist) -> Self {
        let mut net_devices = vec![Vec::new(); nl.nets.len()];
        for (di, dev) in nl.devices.iter().enumerate() {
            for (_, net) in &dev.terminals {
                net_devices[net.0 as usize].push(DeviceId(di as u16));
            }
        }
        BipartiteHypergraph {
            device_nets: nl.devices.iter().map(|d| d.terminals.iter().map(|t| t.1).collect()).collect(),
            terminals: nl.devices.iter().map(|d| d.terminals.iter().map(|t| t.0.clone()).collect()).collect(),
            kinds: nl.devices.iter().map(|d| d.kind).collect(),
            net_devices,
            net_names: nl.nets.iter().map(|n| n.name.clone()).collect(),
        }
    }

    /// Rows in `device_nets`, which equals the netlist's device count.
    #[must_use]
    pub fn device_count(&self) -> usize {
        self.device_nets.len()
    }
}
