//! The block harness (`docs/API.md` §1 `Inputs::interface`, 1.0
//! `library::Interface`): a fixed die the floorplan sits centred in, and the
//! boundary pins a parent connects to. Each pin is a macro at the die edge,
//! on its metal, bound to its port net, with a via stack down to the lowest
//! routed metal where `dr` lands it; signoff labels those pins, so LVS ports
//! are the harness's.

use analog::base::netlist::Netlist as Flat;
use analog::base::{LayerId, Macro, NetId, Pin, Rect, Shape};
use verify::{LabeledPin, Pdk};

use crate::{refuse, Refusal};

/// A die edge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    North,
    South,
    East,
    West,
}

/// One boundary pin: `frac` ∈ [0, 1] along `side` from its low end (west or
/// south).
#[derive(Clone, Debug, PartialEq)]
pub struct IoPin {
    pub net: String,
    pub side: Side,
    pub frac: f32,
    /// Along the edge, nm; `None`: the metal's via pad.
    pub width_nm: Option<i32>,
    /// Deck layer name (`met3`).
    pub layer: String,
}

/// A block's fixed outline and boundary pins.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Interface {
    /// `(w, h)`, nm; `None`: the block's bbox plus a pin margin.
    pub die_nm: Option<(i32, i32)>,
    pub pins: Vec<IoPin>,
}

impl Interface {
    /// `{"die": {"w": nm, "h": nm}, "pins": [{"net", "side":
    /// "north"|"south"|"east"|"west", "frac", "width": nm, "layer"}]}`, `die`
    /// and `width` optional (`benchmarks/fixtures/*.interface.json`).
    ///
    /// # Errors
    /// `interface`: malformed JSON or a field out of its domain.
    pub fn from_json(text: &str) -> Result<Interface, Refusal> {
        let bad = |m: String| refuse("interface", m);
        let v: serde_json::Value = serde_json::from_str(text).map_err(|e| bad(e.to_string()))?;
        let int = |o: &serde_json::Value, k: &str| -> Result<Option<i32>, Refusal> {
            match o.get(k) {
                None | Some(serde_json::Value::Null) => Ok(None),
                Some(x) => x
                    .as_i64()
                    .and_then(|x| i32::try_from(x).ok())
                    .filter(|&x| x > 0)
                    .map(Some)
                    .ok_or_else(|| bad(format!("`{k}` must be a positive integer, nm"))),
            }
        };
        let die_nm = match v.get("die") {
            None | Some(serde_json::Value::Null) => None,
            Some(d) => Some((int(d, "w")?.ok_or_else(|| bad("die.w missing".into()))?, int(d, "h")?.ok_or_else(|| bad("die.h missing".into()))?)),
        };
        let pins = v.get("pins").and_then(serde_json::Value::as_array).map_or(&[][..], Vec::as_slice);
        let pins = pins
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let text = |k: &str| p.get(k).and_then(serde_json::Value::as_str).ok_or_else(|| bad(format!("pins[{i}].{k} must be a string")));
                let side = match text("side")? {
                    "north" => Side::North,
                    "south" => Side::South,
                    "east" => Side::East,
                    "west" => Side::West,
                    s => return Err(bad(format!("pins[{i}].side {s:?} is not north|south|east|west"))),
                };
                let frac = p.get("frac").and_then(serde_json::Value::as_f64).filter(|f| (0.0..=1.0).contains(f));
                let frac = frac.ok_or_else(|| bad(format!("pins[{i}].frac must be in [0, 1]")))? as f32;
                Ok(IoPin { net: text("net")?.to_string(), side, frac, width_nm: int(p, "width")?, layer: text("layer")?.to_string() })
            })
            .collect::<Result<_, Refusal>>()?;
        Ok(Interface { die_nm, pins })
    }
}

/// A pin resolved against the netlist and the deck: its net, its metal's
/// index in `pdk.routing_metals`, and its size, nm: `along` its edge (the
/// given width, at least its metal's via pad and `min_width`) and `inward`
/// (the least of the pad, `min_width` and `min_area / along`: pin access
/// lands a cut on it).
pub(crate) struct Bound {
    net: NetId,
    metal: usize,
    along: i32,
    inward: i32,
}

/// Each pin's net, metal and size.
///
/// # Errors
/// `interface`: a pin on a net that is not a top-cell port, or on a layer
/// that is not a routing metal of the deck.
pub(crate) fn bind(i: &Interface, flat: &Flat, pdk: &Pdk) -> Result<Vec<Bound>, Refusal> {
    let port = |n: &str| flat.ports.iter().copied().find(|p| flat.nets[p.0 as usize].name == n);
    let off: Vec<&str> = i.pins.iter().map(|p| p.net.as_str()).filter(|n| port(n).is_none()).collect();
    if !off.is_empty() {
        return Err(refuse("interface", format!("pins on non-port nets {off:?}")));
    }
    let g = i64::from(pdk.grid.max(1));
    let up = |v: i64| ((v + g - 1).div_euclid(g) * g) as i32;
    let vias = pdk.routing_vias();
    // The via pad on metal `k`: the larger of the cut below's and above's.
    let pad = |k: usize| vias.get(k).map_or(0, |v| v.2).max(k.checked_sub(1).and_then(|j| vias.get(j)).map_or(0, |v| v.3));
    i.pins
        .iter()
        .map(|p| {
            let metal = pdk.routing_metals.iter().position(|&m| pdk.layers.iter().any(|(n, l)| *l == m && *n == p.layer));
            let metal = metal.ok_or_else(|| refuse("interface", format!("pin `{}`: `{}` is not a routing metal of the deck", p.net, p.layer)))?;
            let id = pdk.routing_metals[metal].0;
            let least = pad(metal).max(pdk.min_width(id).unwrap_or(0));
            let along = up(i64::from(p.width_nm.unwrap_or(0).max(least)));
            let area = pdk.min_area(id).map_or(0, |a| (a + i64::from(along) - 1) / i64::from(along));
            Ok(Bound { net: port(&p.net).expect("checked above"), metal, along, inward: up(i64::from(least).max(area)) })
        })
        .collect()
}

/// How far in from each die edge the block must keep, nm: the deepest pin
/// and two routing pitches for the wires that reach it.
pub(crate) fn reach(bound: &[Bound], pdk: &Pdk) -> i32 {
    bound.iter().map(|b| b.inward).max().unwrap_or(0) + 2 * crate::route::routing_stack(pdk, None).map_or(0, |s| s.p0)
}

/// The boundary pin macros, in world coordinates (past the layout's rows, so
/// `place_macros` leaves them as drawn): the die centred on the placed
/// block's bbox, each pin at its side and fraction, with the deck's via
/// stack at its centre down to the lowest routed metal's pad — the one pin
/// `dr` sees and climbs from.
///
/// # Errors
/// `interface`: the block does not fit in the die [`reach`] from its edges.
pub(crate) fn boundary(i: &Interface, bound: &[Bound], placed: &[Macro], pdk: &Pdk) -> Result<Vec<Macro>, Refusal> {
    let g = pdk.grid.max(1);
    let snap = |v: i32| v.div_euclid(g) * g;
    let metals = &pdk.routing_metals;
    let vias = pdk.routing_vias();
    let stack = crate::route::routing_stack(pdk, None).map_err(|e| refuse("interface", e))?;
    let floor = metals.iter().position(|&m| m == stack.layers[0]).expect("the routing stack is the deck's metals");
    let block = placed.iter().map(|m| m.bbox).reduce(crate::union).unwrap_or_default();
    let margin = reach(bound, pdk);
    let (w, h) = i.die_nm.unwrap_or((block.w + 2 * margin, block.h + 2 * margin));
    if block.w + 2 * margin > w || block.h + 2 * margin > h {
        return Err(refuse(
            "interface",
            format!("the block ({} × {} nm) and its pins' reach ({margin} nm a side) do not fit the die ({w} × {h} nm)", block.w, block.h),
        ));
    }
    let p0 = stack.p0.max(1);
    let die = Rect { x: snap(block.x + block.w / 2 - w / 2), y: snap(block.y + block.h / 2 - h / 2), w, h };
    Ok(i.pins
        .iter()
        .zip(bound)
        .map(|(p, b)| {
            let (along, inward) = (b.along, b.inward);
            // The stack sits on a lattice node (`dr`'s at `k·p0 + p0/2`), so
            // the trunk lands at the pad's centre: one ending part-way into
            // the pad leaves a corner-to-corner neck under `min_width`. The
            // pin runs from its edge in past the node by half its depth.
            let node = |v: i32, lo: i32, hi: i32| {
                let t = v.div_euclid(p0) * p0 + p0 / 2;
                if t < lo {
                    t + p0
                } else if t > hi {
                    t - p0
                } else {
                    t
                }
            };
            let (x1, y1) = (die.x + die.w, die.y + die.h);
            let span = |lo: i32, hi: i32| node(lo + (p.frac * (hi - lo) as f32) as i32, lo + along / 2, hi - along / 2);
            let ((cx, cy), rect) = match p.side {
                Side::North => {
                    let (cx, cy) = (span(die.x, x1), node(y1 - inward / 2, i32::MIN, y1 - inward / 2));
                    ((cx, cy), Rect { x: cx - along / 2, y: cy - inward / 2, w: along, h: y1 - cy + inward / 2 })
                }
                Side::South => {
                    let (cx, cy) = (span(die.x, x1), node(die.y + inward / 2, die.y + inward / 2, i32::MAX));
                    ((cx, cy), Rect { x: cx - along / 2, y: die.y, w: along, h: cy + inward / 2 - die.y })
                }
                Side::East => {
                    let (cx, cy) = (node(x1 - inward / 2, i32::MIN, x1 - inward / 2), span(die.y, y1));
                    ((cx, cy), Rect { x: cx - inward / 2, y: cy - along / 2, w: x1 - cx + inward / 2, h: along })
                }
                Side::West => {
                    let (cx, cy) = (node(die.x + inward / 2, die.x + inward / 2, i32::MAX), span(die.y, y1));
                    ((cx, cy), Rect { x: die.x, y: cy - along / 2, w: cx + inward / 2 - die.x, h: along })
                }
            };
            let square = |l: LayerId, s: i32| Shape { layer: l, rect: Rect { x: cx - s / 2, y: cy - s / 2, w: s, h: s } };
            let mut shapes = vec![Shape { layer: metals[b.metal], rect }];
            // The lowest cut is pin access's: `dr` draws it where its own
            // vias leave room, so a second one here would overlap theirs.
            for k in floor..b.metal {
                let (cut, s, below, above) = vias[k];
                shapes.extend([square(metals[k], below), square(metals[k + 1], above)]);
                if k > floor {
                    shapes.push(square(cut, s));
                }
            }
            // The stack's bottom pad, on the lowest routed metal.
            let low = if b.metal <= floor { rect } else { shapes[1].rect };
            Macro {
                pins: vec![Pin { name: format!("io:{}", p.net), net: b.net, at: low, layer: metals[b.metal.min(floor)] }],
                bbox: shapes.iter().map(|s| s.rect).reduce(crate::union).expect("the pin rect"),
                shapes,
                ..Macro::default()
            }
        })
        .collect())
}

/// One label per boundary pin, at the centre of its top-metal rect (the
/// first shape [`boundary`] draws).
pub(crate) fn labels(pins: &[Macro], nets: &[String]) -> Vec<LabeledPin> {
    pins.iter()
        .map(|m| {
            let s = m.shapes[0];
            LabeledPin { name: nets[m.pins[0].net.0 as usize].clone(), layer: s.layer.0, x: s.rect.x + s.rect.w / 2, y: s.rect.y + s.rect.h / 2 }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fixture_parses_and_a_bad_side_refuses() {
        let i = Interface::from_json(include_str!("../../../../benchmarks/fixtures/ota_constrained.interface.json")).unwrap();
        assert_eq!(i.die_nm, Some((30_000, 70_000)));
        assert_eq!((i.pins.len(), i.pins[0].side, i.pins[0].width_nm), (6, Side::South, Some(800)));
        let tt = Interface::from_json(include_str!("../../../../benchmarks/fixtures/tt_ota.interface.json")).unwrap();
        assert_eq!(tt.pins[2].width_nm, None);
        let r = Interface::from_json(r#"{"pins":[{"net":"a","side":"up","frac":0.5,"layer":"met3"}]}"#).unwrap_err();
        assert_eq!(r.code, "interface");
    }
}
