//! Disabled: conformance of the pure wasm replay surface against `honba-engine`'s own
//! [`ReplayState`].
//!
//! `honba-engine` (package `honba-ya-engine`) was removed from the `honba-ya` workspace in its
//! commit `864b8d3`, with `AuditKind`/`AuditLog`/`JournalHeader`/`JournalV1`/`ReplayState` dropped
//! and no successor reducer. `honba-kao-api-wasm`'s `replay` module (`src/replay.rs`) and its unit
//! tests (`src/tests/replay.rs`) are unwired from the crate for the same reason — see the
//! module-level doc comment in `src/lib.rs`.
//!
//! This file previously built a small deterministic `AuditLog` (dispatch/submit/fill/cancel/
//! reject/state-change events), rendered it to Journal-v1 NDJSON, and asserted the wasm JSON
//! output (`journal_header_json`/`replay_journal_json`) matched the engine's own
//! `ReplayState::from_log` field-by-field (positions, orders, rejections, trading state). Restore
//! it from git history once honba-ya exposes an equivalent reducer (client-side replayable journal
//! format, or a REST endpoint to call instead of reducing in WASM) and `replay` is re-wired in
//! `lib.rs`.
