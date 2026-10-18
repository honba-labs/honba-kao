//! Terminal console components. Mirrors `web/src/components/terminal/`.

pub mod candles;
pub mod execution_log;
pub mod kline_chart;
pub mod order_book;
pub mod orders_table;
pub mod positions_table;
pub mod risk_radar;
pub mod signal_matrix;
pub mod terminal_dashboard;

pub use candles::Candles;
pub use execution_log::ExecutionLog;
pub use kline_chart::KlineChart;
pub use order_book::OrderBook;
pub use orders_table::OrdersTable;
pub use positions_table::PositionsTable;
pub use risk_radar::RiskRadar;
pub use signal_matrix::SignalMatrix;
pub use terminal_dashboard::TerminalDashboard;
