//! Offline market-simulation fixtures: a compressed L2 order book
//! snapshot and a binary tick replay stream, compiled into the binary so
//! offline development needs no data downloads. Pure byte-slice access —
//! zero I/O in this module.
//!
//! # Replay stream wire format (`replay_sample_ticks.bin`)
//!
//! ```text
//! offset  len  field
//! ------  ---  ---------------------------------------------------
//!      0    9  magic  = b"HBREPLAY1"
//!      9    1  u8     format version (currently 1)
//!     10    4  u32 BE tick record count (128 in the sample)
//!     14   25  record stream, `count` records of 25 bytes each:
//!              +0   u64 BE ts_unix_ns
//!              +8   u8  side (0 = bid, 1 = ask, 2 = trade)
//!              +9   i64 BE price_scaled_e4   (price × 10⁴)
//!              +17  u64 BE qty_scaled_e4     (qty × 10⁴)
//! ```
//!
//! All integers are big-endian; scaled fields are exact integers (no
//! floats cross the wire).

/// Magic header of the honba replay tick format (`bytes[0..9]`).
pub const REPLAY_MAGIC: &[u8; 9] = b"HBREPLAY1";

/// Byte length of a single replay tick record in the stream body.
pub const REPLAY_RECORD_LEN: usize = 25;

/// The compiled-in sample replay stream (128 synthetic tick records).
pub const REPLAY_SAMPLE_TICKS: &[u8] = include_bytes!("../assets/fixtures/replay_sample_ticks.bin");

/// The compiled-in L2 order book snapshot (NSE:RELIANCE-EQ, 10 bids ×
/// 10 asks; each level is a `[price, qty]` pair of decimal strings).
pub const ORDERBOOK_L2_SNAPSHOT_JSON: &str =
    include_str!("../assets/fixtures/orderbook_l2_snapshot.json");

/// Parses the tick count from the replay stream header
/// (`REPLAY_SAMPLE_TICKS`).
///
/// Returns `None` when the magic bytes or the format version deviate from
/// [`REPLAY_MAGIC`] / version `1`; never panics on a truncated header.
pub fn replay_tick_count() -> Option<u32> {
    let bytes = REPLAY_SAMPLE_TICKS;
    if bytes.len() < 14 || !bytes.starts_with(REPLAY_MAGIC) || bytes[9] != 1 {
        return None;
    }
    Some(u32::from_be_bytes([
        bytes[10], bytes[11], bytes[12], bytes[13],
    ]))
}
