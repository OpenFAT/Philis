//! `routing`: routing recipes ([`requests`], Q23) and the negotiated router moved
//! from Philis 1.0: [`gr`] global routing and [`dr`] detailed routing, which
//! reuses `gr`'s grid and path search.

pub mod dr;
pub mod gr;

pub mod requests;
