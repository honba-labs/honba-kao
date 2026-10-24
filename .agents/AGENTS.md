# `honba-kao` Agent Guidelines & Architecture Invariants

**Subsystem**: Face (WebAssembly UI & Strategy Studio)
**Language**: Rust (Leptos)
**Primary Responsibility**: Leptos WebAssembly UI, Strategy Studio visual node canvas, backtest simulator workbench, and openalgo-charts bridge.

---

## 1. Role & Architectural Hints
* `honba-kao` provides the Rust/WASM visualization and studio experience:
  1. **Leptos Reactive Framework**: Zero-JS WebAssembly application compiled directly to browser runtimes.
  2. **Strategy Studio Node Canvas**: Visual drag-and-drop canvas for composing trading indicators, risk rules, and signals.
  3. **Simulator Workbench**: Interactive equity curve visualizer, drawdowns, trade heatmap, and trade execution log.
  4. **OpenAlgo Charts Bridge**: High-performance charting canvas rendering financial candles and technical overlays.
* **Idea Inheritance**:
  - **StockSharp**: Odysseus visual strategy designer and Studio workbench.

---

## 2. Coding Guidelines (DDD & TDD)
* **WASM Targetable**: Compiles cleanly to `wasm32-unknown-unknown`. Never use native OS threads or synchronous blocking filesystem calls.
* **TDD & Parallel Test Directory Rule (MANDATORY)**:
  - All tests must live in `tests/` parallel to `src/`.
  - **STRICTLY FORBIDDEN**: In-file `#[test]` inside `src/`.
* **Check .agents/HANDOFF.md**: Always verify handoff status before starting work.

---

## 3. Development & CI Workflow
* **Linting & Formatting**:
  ```bash
  cargo fmt --check
  cargo clippy --all-targets -- -D warnings
  ```
* **Testing**:
  ```bash
  cargo test --all
  ```
