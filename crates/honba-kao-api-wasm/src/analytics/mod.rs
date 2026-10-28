pub mod cointegration;
pub mod drawdown;
pub mod ratios;
pub mod returns;
pub mod sip;
pub mod value_at_risk;

pub use cointegration::{CointegrationResult, calculate_cointegration};
pub use drawdown::{DrawdownSeries, calculate_drawdown};
pub use ratios::{PerformanceRatios, calculate_performance_ratios};
pub use returns::{ReturnMetrics, calculate_daily_returns, calculate_return_metrics};
pub use sip::{SipProjection, calculate_sip_projection};
pub use value_at_risk::{ValueAtRisk, calculate_var_and_cvar};
