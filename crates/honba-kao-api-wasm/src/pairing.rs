//! Fill → round-trip pairing (HBD-2026-DES-005 P3).
//!
//! The Rust port of honba-fe `core/simulator/trade-pairing.ts`. The journal
//! (`GET /backtests/{id}/journal`, decoded `TradesResponse`) carries
//! **fills**, not round trips: there is no entry/exit field, no PnL, no
//! size-in-size-out and no signal reason. Everything a round trip reports is
//! therefore *derived* here, by one documented rule set, so the chart markers,
//! the tooltip and the P5 trade journal all agree:
//!
//! - Fills are ordered by `ts_event`, stable, per instrument.
//! - A **round trip = one position episode**: it opens when the net position
//!   leaves zero and closes when it returns to zero (or flips through it).
//! - Scale-in fills join the open episode as more lots; scale-out fills consume
//!   those lots **FIFO**. One episode with three exits is one round trip with
//!   three exit legs — not three trips.
//! - Realised PnL is **derived**: `(exit − entry) × closed qty × direction`,
//!   net of the run's fill costs, with entry costs allocated pro-rata to the
//!   closed quantity so an open trip is not charged for inventory it still
//!   holds.
//! - A position still open when the run ends is reported with
//!   [`RoundTripStatus::Open`] and no exit reason, and is labelled as derived
//!   wherever it is drawn.
//! - `exit_reason` is one of the three *derived* states below. The journal
//!   carries no strategy-level reason (stop, target, signal) — that is a
//!   contract gap, never a guess.
//!
//! Money deviation from fe (documented per the port's DoD): fe's journal carried
//! `costs.amount` as a float in **major units**; the wire [`Trade`]
//! carries ledger `Money` in integer minor units. The conversion
//! happens **once, at this boundary** — `Money::to_major_f64` — so
//! pairing and everything downstream (analytics, equity, heatmap, CSV) keep raw
//! major-unit floats exactly like fe, and match the server's own
//! `honba-analytics::RoundTrip::from_fills`, which does the same one-time
//! conversion. Presentation is a P5 render-time concern only.
//!
//! Every other guard fe dropped fills for (bad side, non-finite size/price,
//! absent timestamp) is structurally impossible: a wire [`Trade`] is
//! validated at its deserialization door (`side` ∈ {buy, sell}, `quantity` and
//! `price` finite and > 0, costs exact `Money`), so [`PairingResult::dropped`]
//! counts the only remaining failure mode — the epoch `0` placeholder
//! timestamp — and is 0 for any well-formed journal.

use std::collections::BTreeMap;

use honba_core::rust_decimal::prelude::ToPrimitive;
use honba_core::{FillReport as Trade, OrderSide};

use crate::calendar::unix_nanos_to_seconds;

/// Which side of the book a fill sat on (`FillSide`); `isSide`, typed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillSide {
    /// A buy fill.
    Buy,
    /// A sell fill.
    Sell,
}

impl FillSide {
    /// The wire spelling, echoed by chart tooltips and markers.
    pub fn as_str(self) -> &'static str {
        match self {
            FillSide::Buy => "buy",
            FillSide::Sell => "sell",
        }
    }

    /// fe's `isSide` guard.
    ///
    /// `honba_core::OrderSide` is a closed `{Buy, Sell}` enum (the legacy wire
    /// type's `NoOrderSide` sentinel doesn't exist here), so this is total —
    /// kept as `Option` for call-site compatibility.
    pub fn from_order_side(side: OrderSide) -> Option<FillSide> {
        match side {
            OrderSide::Buy => Some(FillSide::Buy),
            OrderSide::Sell => Some(FillSide::Sell),
        }
    }
}

/// One journal fill, normalised and sortable (`PairedFill`).
#[derive(Clone, Debug, PartialEq)]
pub struct PairedFill {
    /// Original index in the response, so ties keep wire order.
    pub index: usize,
    /// UTC seconds.
    pub ts: i64,
    /// Which side of the book.
    pub side: FillSide,
    /// Fill price.
    pub price: f64,
    /// Fill quantity.
    pub quantity: f64,
    /// The fill's transaction costs, in **major units** (see the module doc).
    pub costs: f64,
    /// Costs' ISO 4217 currency code.
    pub currency: String,
    /// The originating order id.
    pub order_id: String,
    /// `SYMBOL.EXCHANGE` instrument id.
    pub instrument: String,
}

/// One entry or exit fill leg of a round trip (`RoundTripLeg`).
#[derive(Clone, Debug, PartialEq)]
pub struct RoundTripLeg {
    /// UTC seconds.
    pub ts: i64,
    /// Leg price.
    pub price: f64,
    /// Leg quantity.
    pub quantity: f64,
    /// Leg costs, major units.
    pub costs: f64,
    /// Originating order id.
    pub order_id: String,
}

/// Why an episode ended, as far as the journal lets anyone know
/// (`RoundTripExitReason`) — always derived, never a strategy-level guess.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundTripExitReason {
    /// The exit fill brought the position flat.
    PositionClosed,
    /// The exit fill overran flat and opened the opposite position; the
    /// residue may be an open episode.
    Flipped,
    /// The run ended while the position was still open.
    OpenAtEnd,
}

impl RoundTripExitReason {
    /// fe's wire spelling, shown in the journal table and markers' tooltips.
    pub fn as_str(self) -> &'static str {
        match self {
            RoundTripExitReason::PositionClosed => "position_closed",
            RoundTripExitReason::Flipped => "flipped",
            RoundTripExitReason::OpenAtEnd => "open_at_end",
        }
    }
}

/// Direction a round trip opened with (`direction`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Position opened with a buy.
    Long,
    /// Position opened with a sell.
    Short,
}

impl Direction {
    /// fe's wire spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Direction::Long => "long",
            Direction::Short => "short",
        }
    }
}

/// Whether a trip is fully realised (`status`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundTripStatus {
    /// The episode's inventory was fully closed.
    Closed,
    /// The run ended with inventory still held.
    Open,
}

impl RoundTripStatus {
    /// fe's wire spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            RoundTripStatus::Closed => "closed",
            RoundTripStatus::Open => "open",
        }
    }
}

/// A derived round trip (`RoundTrip`). All PnL-bearing values are derived from
/// the journal fills; nothing is reported the fills cannot support.
#[derive(Clone, Debug, PartialEq)]
pub struct RoundTrip {
    /// Stable id, `trip-<n>` in entry order.
    pub id: String,
    /// `SYMBOL.EXCHANGE` instrument id.
    pub instrument: String,
    /// Direction the position opened with.
    pub direction: Direction,
    /// Whether the run ended with the position closed.
    pub status: RoundTripStatus,
    /// Quantity paired into this trip (its entry size).
    pub quantity: f64,
    /// Quantity still held at the run's end; 0 for a closed trip.
    pub open_quantity: f64,
    /// First fill's timestamp, UTC seconds.
    pub entry_ts: i64,
    /// Last (or only) exit fill's timestamp; `None` while nothing closed.
    pub exit_ts: Option<i64>,
    /// Weighted-average entry price.
    pub avg_entry: f64,
    /// Weighted-average exit price; `None` until something has closed.
    pub avg_exit: Option<f64>,
    /// Entry fill legs.
    pub entry_fills: Vec<RoundTripLeg>,
    /// Exit fill legs.
    pub exit_fills: Vec<RoundTripLeg>,
    /// Recorded fill costs, entry + exit (major units, as the venue reported).
    pub fees: f64,
    /// Derived, net of allocated costs. `None` until part of the trip closed.
    pub realized_pnl: Option<f64>,
    /// Derived `realized_pnl / (avg_entry × closed qty)`.
    pub return_pct: Option<f64>,
    /// Derived — the journal carries no strategy-level exit reason.
    pub exit_reason: RoundTripExitReason,
}

/// A position still held when the run ended (`OpenPosition`).
#[derive(Clone, Debug, PartialEq)]
pub struct OpenPosition {
    /// `SYMBOL.EXCHANGE` instrument id.
    pub instrument: String,
    /// Direction the position opened with.
    pub direction: Direction,
    /// Quantity still held.
    pub quantity: f64,
    /// Average price of the remaining lots.
    pub avg_price: f64,
    /// When the position was first opened, UTC seconds.
    pub since_ts: i64,
}

/// The pairing of a journal (`PairingResult`).
#[derive(Clone, Debug, PartialEq)]
pub struct PairingResult {
    /// Normalised, ordered fills.
    pub fills: Vec<PairedFill>,
    /// Round trips in entry order (`trip-1`, `trip-2`, …).
    pub trips: Vec<RoundTrip>,
    /// Positions still open at the run's end, per instrument.
    pub open_positions: Vec<OpenPosition>,
    /// Fills that could not be paired. For wire trades this is structurally
    /// 0 except for the epoch-placeholder timestamp (see the module doc).
    pub dropped: usize,
}

/// Chart marker kind (`TradeMarkerKind`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TradeMarkerKind {
    /// An entry fill leg.
    Entry,
    /// An exit fill leg.
    Exit,
    /// The labelled tail of a still-open position (derived, not a fill).
    Open,
}

impl TradeMarkerKind {
    /// fe's wire spelling, echoed by the chart's click handler.
    pub fn as_str(self) -> &'static str {
        match self {
            TradeMarkerKind::Entry => "entry",
            TradeMarkerKind::Exit => "exit",
            TradeMarkerKind::Open => "open",
        }
    }
}

/// A trade marker over the chart (`TradeMarker`).
#[derive(Clone, Debug, PartialEq)]
pub struct TradeMarker {
    /// Stable id, echoed by the chart's click handler.
    pub key: String,
    /// UTC seconds; snapped to a bar timestamp before it is drawn (P6).
    pub ts: i64,
    /// Marker side as drawn.
    pub side: FillSide,
    /// Kind.
    pub kind: TradeMarkerKind,
    /// Marker price.
    pub price: f64,
    /// Marker quantity.
    pub quantity: f64,
    /// Owning trip id.
    pub trip_id: String,
    /// `SYMBOL.EXCHANGE` instrument id.
    pub instrument: String,
    /// True when the marker is derived rather than a fill (the open tail).
    pub derived: bool,
}

/// Same float slack fe paired with (`EPSILON`).
const EPSILON: f64 = 1e-9;

/// Builds the normalised fill from a validated wire trade, or `None` when its
/// timestamp is the epoch placeholder (the one non-finite-free drop).
///
/// fe gated this behind an `isSide` type check on untyped JSON; on the Rust
/// side the wire [`Trade`] is validated at its deserialization door, so there
/// is nothing left to check. A plain function — a `From` impl for
/// `Option` would trip the orphan rule.
fn paired_fill_from(trade: &Trade) -> Option<PairedFill> {
    Some(PairedFill {
        index: usize::MAX, // fixed by the caller, which knows the index
        ts: unix_nanos_to_seconds(trade.timestamp)?,
        side: FillSide::from_order_side(trade.side)?,
        price: trade.price.0.to_f64().unwrap_or(0.0),
        quantity: trade.quantity.0.to_f64().unwrap_or(0.0),
        costs: trade.fee.amount.to_f64().unwrap_or(0.0),
        currency: trade.fee.currency.code().to_string(),
        order_id: trade.order_id.clone(),
        instrument: format!(
            "{}.{}",
            trade.instrument.symbol,
            trade.instrument.exchange.as_str()
        ),
    })
}

/// Wire fills → sorted, timestamped, side-checked fills (`fillsFromTrades`).
///
/// fe dropped fills it could not read a side/timestamp/size/price from and
/// counted them; on the Rust side the wire [`Trade`] is validated at the door,
/// so only the epoch-placeholder timestamp can drop. See the module doc.
pub fn fills_from_trades(trades: &[Trade]) -> (Vec<PairedFill>, usize) {
    let mut fills: Vec<PairedFill> = Vec::with_capacity(trades.len());
    let mut dropped = 0;
    for (index, trade) in trades.iter().enumerate() {
        let Some(mut fill) = paired_fill_from(trade) else {
            dropped += 1;
            continue;
        };
        fill.index = index;
        fills.push(fill);
    }
    fills.sort_by_key(|fill| (fill.ts, fill.index));
    (fills, dropped)
}

/// One partial lot of an open episode (`Lot`).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Lot {
    remaining: f64,
    price: f64,
}

/// Pairing workspace for one instrument's episode (`Episode`).
#[derive(Clone, Debug)]
struct Episode {
    instrument: String,
    direction: Direction,
    entry_fills: Vec<RoundTripLeg>,
    exit_fills: Vec<RoundTripLeg>,
    entry_qty: f64,
    entry_notional: f64,
    entry_costs: f64,
    exit_qty: f64,
    exit_notional: f64,
    exit_costs: f64,
    entry_ts: i64,
    exit_ts: Option<i64>,
    lots: Vec<Lot>,
    exit_reason: RoundTripExitReason,
}

fn open_episode(fill: &PairedFill) -> Episode {
    Episode {
        instrument: fill.instrument.clone(),
        direction: if fill.side == FillSide::Buy {
            Direction::Long
        } else {
            Direction::Short
        },
        entry_fills: vec![RoundTripLeg {
            ts: fill.ts,
            price: fill.price,
            quantity: fill.quantity,
            costs: fill.costs,
            order_id: fill.order_id.clone(),
        }],
        exit_fills: Vec::new(),
        entry_qty: fill.quantity,
        entry_notional: fill.quantity * fill.price,
        entry_costs: fill.costs,
        exit_qty: 0.0,
        exit_notional: 0.0,
        exit_costs: 0.0,
        entry_ts: fill.ts,
        exit_ts: None,
        lots: vec![Lot {
            remaining: fill.quantity,
            price: fill.price,
        }],
        exit_reason: RoundTripExitReason::OpenAtEnd,
    }
}

fn add_entry(episode: &mut Episode, fill: &PairedFill) {
    episode.entry_fills.push(RoundTripLeg {
        ts: fill.ts,
        price: fill.price,
        quantity: fill.quantity,
        costs: fill.costs,
        order_id: fill.order_id.clone(),
    });
    episode.entry_qty += fill.quantity;
    episode.entry_notional += fill.quantity * fill.price;
    episode.entry_costs += fill.costs;
    episode.lots.push(Lot {
        remaining: fill.quantity,
        price: fill.price,
    });
}

fn episode_exit_ts(episode: &Episode) -> i64 {
    episode.exit_ts.unwrap_or(i64::MIN)
}

/// Pair one instrument's ordered fills into position episodes (`pairInstrument`).
///
/// Closing fills consume the open lots FIFO; remaining quantity past flat
/// flips through zero and opens the opposite episode with the residue.
fn pair_instrument(fills: &[&PairedFill]) -> (Vec<Episode>, Option<Episode>) {
    let mut episodes: Vec<Episode> = Vec::new();
    let mut open: Option<Episode> = None;

    for fill in fills {
        let opening_side = if fill.side == FillSide::Buy {
            Direction::Long
        } else {
            Direction::Short
        };
        let Some(episode) = open.as_mut() else {
            open = Some(open_episode(fill));
            continue;
        };

        if opening_side == episode.direction {
            add_entry(episode, fill);
            continue;
        }

        // Closing (or reversing) fill: consume the open lots FIFO.
        let per_unit = fill.costs / fill.quantity;
        let mut remaining = fill.quantity;
        episode.exit_fills.push(RoundTripLeg {
            ts: fill.ts,
            price: fill.price,
            quantity: fill.quantity,
            costs: fill.costs,
            order_id: fill.order_id.clone(),
        });

        while remaining > EPSILON && !episode.lots.is_empty() {
            let mut lot = episode.lots[0];
            let taken = lot.remaining.min(remaining);
            lot.remaining -= taken;
            remaining -= taken;
            episode.exit_qty += taken;
            episode.exit_notional += taken * fill.price;
            episode.exit_costs += taken * per_unit;
            if lot.remaining <= EPSILON {
                episode.lots.remove(0);
            } else {
                // The fill ran out mid-lot; the rest stays open.
                episode.lots[0] = lot;
            }
        }
        episode.exit_ts = Some(fill.ts);

        if episode.lots.is_empty() {
            episode.exit_reason = if remaining > EPSILON {
                RoundTripExitReason::Flipped
            } else {
                RoundTripExitReason::PositionClosed
            };
            episodes.push(open.take().expect("episode is present"));
            if remaining > EPSILON {
                // The fill ran past flat: the residue opens the opposite episode.
                let mut residue = (*fill).clone();
                residue.quantity = remaining;
                residue.costs = per_unit * remaining;
                open = Some(open_episode(&residue));
            }
        }
    }

    (episodes, open)
}

/// fe's `round(value, digits = 10)`: trims float noise off derived values.
fn round(value: f64, digits: i32) -> f64 {
    let factor = 10f64.powi(digits);
    (value * factor).round() / factor
}

fn finalize_episode(episode: &Episode, id: String, open: bool) -> RoundTrip {
    let closed_qty = episode.exit_qty;
    let avg_entry = if episode.entry_qty > 0.0 {
        episode.entry_notional / episode.entry_qty
    } else {
        0.0
    };
    let avg_exit = if closed_qty > 0.0 {
        Some(episode.exit_notional / closed_qty)
    } else {
        None
    };
    let open_quantity: f64 = episode.lots.iter().map(|lot| lot.remaining).sum();
    let sign = if episode.direction == Direction::Long {
        1.0
    } else {
        -1.0
    };
    let allocated_entry_costs = if closed_qty > 0.0 && episode.entry_qty > 0.0 {
        episode.entry_costs * (closed_qty / episode.entry_qty)
    } else {
        0.0
    };
    let realized_pnl = avg_exit.map(|avg_exit| {
        sign * (avg_exit - avg_entry) * closed_qty - (allocated_entry_costs + episode.exit_costs)
    });
    let capital = avg_entry * closed_qty;

    RoundTrip {
        id,
        instrument: episode.instrument.clone(),
        direction: episode.direction,
        status: if open {
            RoundTripStatus::Open
        } else {
            RoundTripStatus::Closed
        },
        quantity: round(episode.entry_qty, 8),
        open_quantity: round(open_quantity, 8),
        entry_ts: episode.entry_ts,
        exit_ts: episode.exit_ts,
        avg_entry: round(avg_entry, 10),
        avg_exit: avg_exit.map(|value| round(value, 10)),
        entry_fills: episode.entry_fills.clone(),
        exit_fills: episode.exit_fills.clone(),
        fees: round(episode.entry_costs + episode.exit_costs, 10),
        realized_pnl: realized_pnl.map(|value| round(value, 10)),
        return_pct: realized_pnl
            .filter(|_| capital > 0.0)
            .map(|value| round(value / capital, 10)),
        exit_reason: if open {
            RoundTripExitReason::OpenAtEnd
        } else {
            episode.exit_reason
        },
    }
}

fn open_position_of(episode: &Episode) -> Option<OpenPosition> {
    let quantity: f64 = episode.lots.iter().map(|lot| lot.remaining).sum();
    if quantity <= EPSILON {
        return None;
    }
    let notional: f64 = episode
        .lots
        .iter()
        .map(|lot| lot.remaining * lot.price)
        .sum();
    Some(OpenPosition {
        instrument: episode.instrument.clone(),
        direction: episode.direction,
        quantity: round(quantity, 8),
        avg_price: round(notional / quantity, 10),
        since_ts: episode.entry_ts,
    })
}

/// Pair journal fills into round trips, per instrument (`pairFills`).
///
/// Returns the fills unchanged (they are the journal's raw layer), the trips
/// numbered in entry order and any positions still open at the run's end.
pub fn pair_fills(fills: &[PairedFill]) -> PairingResult {
    let mut by_instrument: BTreeMap<&str, Vec<&PairedFill>> = BTreeMap::new();
    for fill in fills {
        by_instrument
            .entry(fill.instrument.as_str())
            .or_default()
            .push(fill);
    }

    let mut closed: Vec<Episode> = Vec::new();
    let mut opens: Vec<Episode> = Vec::new();
    for bucket in by_instrument.values() {
        let (mut episodes, open) = pair_instrument(bucket);
        closed.append(&mut episodes);
        if let Some(open) = open {
            opens.push(open);
        }
    }

    // fe: closed.sort(entryTs, exitTs); trips numbered over closed then opens;
    // the final list re-sorts by entryTs. All three sorts are stable.
    closed.sort_by_key(|episode| (episode.entry_ts, episode_exit_ts(episode)));
    let mut trips: Vec<RoundTrip> = closed
        .iter()
        .enumerate()
        .map(|(index, episode)| finalize_episode(episode, format!("trip-{}", index + 1), false))
        .collect();

    let closed_len = trips.len();
    let mut open_positions = Vec::new();
    for (index, episode) in opens.iter().enumerate() {
        let id = format!("trip-{}", closed_len + index + 1);
        trips.push(finalize_episode(episode, id, true));
        if let Some(position) = open_position_of(episode) {
            open_positions.push(position);
        }
    }

    trips.sort_by_key(|trip| trip.entry_ts);
    PairingResult {
        fills: fills.to_vec(),
        trips,
        open_positions,
        dropped: 0,
    }
}

/// Wire trades → paired round trips in one call (`pairTrades`).
pub fn pair_trades(trades: &[Trade]) -> PairingResult {
    let (fills, dropped) = fills_from_trades(trades);
    let mut result = pair_fills(&fills);
    result.dropped = dropped;
    result
}

/// Chart markers for a pairing: one marker per fill leg, plus a clearly
/// labelled marker for any position still open at the run's end (derived, not
/// a fill) (`markersFromPairing`). Markers are ordered by timestamp, ties
/// keeping their emission order.
pub fn markers_from_pairing(result: &PairingResult) -> Vec<TradeMarker> {
    let mut markers: Vec<TradeMarker> = Vec::new();
    for trip in &result.trips {
        let marker_side = if trip.direction == Direction::Long {
            FillSide::Buy
        } else {
            FillSide::Sell
        };
        let opposite_side = match marker_side {
            FillSide::Buy => FillSide::Sell,
            FillSide::Sell => FillSide::Buy,
        };
        for (index, leg) in trip.entry_fills.iter().enumerate() {
            markers.push(TradeMarker {
                key: format!("{}:entry:{}", trip.id, index),
                ts: leg.ts,
                side: marker_side,
                kind: TradeMarkerKind::Entry,
                price: leg.price,
                quantity: leg.quantity,
                trip_id: trip.id.clone(),
                instrument: trip.instrument.clone(),
                derived: false,
            });
        }
        for (index, leg) in trip.exit_fills.iter().enumerate() {
            markers.push(TradeMarker {
                key: format!("{}:exit:{}", trip.id, index),
                ts: leg.ts,
                side: opposite_side,
                kind: TradeMarkerKind::Exit,
                price: leg.price,
                quantity: leg.quantity,
                trip_id: trip.id.clone(),
                instrument: trip.instrument.clone(),
                derived: false,
            });
        }
        if trip.status == RoundTripStatus::Open && trip.open_quantity > EPSILON {
            markers.push(TradeMarker {
                key: format!("{}:open", trip.id),
                ts: trip.exit_ts.unwrap_or(trip.entry_ts),
                side: marker_side,
                kind: TradeMarkerKind::Open,
                price: trip.avg_entry,
                quantity: trip.open_quantity,
                trip_id: trip.id.clone(),
                instrument: trip.instrument.clone(),
                derived: true,
            });
        }
    }
    markers.sort_by_key(|marker| marker.ts);
    markers
}

/// Look up the round trip a marker belongs to, for the chart tooltip
/// (`tripForMarker`).
pub fn trip_for_marker<'a>(
    result: &'a PairingResult,
    trip_id: Option<&str>,
) -> Option<&'a RoundTrip> {
    let trip_id = trip_id?;
    result.trips.iter().find(|trip| trip.id == trip_id)
}
