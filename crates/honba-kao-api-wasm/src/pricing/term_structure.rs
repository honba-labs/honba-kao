use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct YieldPoint {
    pub tenor_months: f64,
    pub rate: f64,
}

pub fn discount_factor(rate: f64, time_years: f64) -> f64 {
    (-rate * time_years).exp()
}

pub fn present_value(cash_flow: f64, rate: f64, time_years: f64) -> f64 {
    cash_flow * discount_factor(rate, time_years)
}

pub fn future_value(cash_flow: f64, rate: f64, time_years: f64) -> f64 {
    cash_flow * (rate * time_years).exp()
}

pub fn interpolate_yield(curve: &[YieldPoint], target_tenor_months: f64) -> f64 {
    if curve.is_empty() {
        return 0.0;
    }
    if curve.len() == 1 || target_tenor_months <= curve[0].tenor_months {
        return curve[0].rate;
    }
    if target_tenor_months >= curve.last().unwrap().tenor_months {
        return curve.last().unwrap().rate;
    }

    for i in 0..curve.len() - 1 {
        let p1 = &curve[i];
        let p2 = &curve[i + 1];
        if target_tenor_months >= p1.tenor_months && target_tenor_months <= p2.tenor_months {
            let t = (target_tenor_months - p1.tenor_months) / (p2.tenor_months - p1.tenor_months);
            return p1.rate + t * (p2.rate - p1.rate);
        }
    }

    curve.last().unwrap().rate
}
