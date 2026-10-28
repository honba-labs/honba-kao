pub fn calculate_sma(prices: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut res = Vec::with_capacity(prices.len());
    let mut sum = 0.0;
    for (i, &p) in prices.iter().enumerate() {
        sum += p;
        if i >= period {
            sum -= prices[i - period];
        }
        if i + 1 >= period {
            res.push(Some(sum / period as f64));
        } else {
            res.push(None);
        }
    }
    res
}

pub fn calculate_ema(prices: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut res = Vec::with_capacity(prices.len());
    if period == 0 || prices.is_empty() {
        return res;
    }

    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut prev_ema: Option<f64> = None;
    let mut sum = 0.0;

    for (i, &p) in prices.iter().enumerate() {
        if i + 1 < period {
            sum += p;
            res.push(None);
        } else if i + 1 == period {
            sum += p;
            let sma = sum / period as f64;
            prev_ema = Some(sma);
            res.push(Some(sma));
        } else {
            let ema = (p - prev_ema.unwrap()) * multiplier + prev_ema.unwrap();
            prev_ema = Some(ema);
            res.push(Some(ema));
        }
    }
    res
}

pub fn calculate_vwap(prices: &[f64], volumes: &[f64]) -> Vec<f64> {
    let mut cum_pv = 0.0;
    let mut cum_v = 0.0;
    let mut res = Vec::with_capacity(prices.len());

    for (&p, &v) in prices.iter().zip(volumes.iter()) {
        cum_pv += p * v;
        cum_v += v;
        if cum_v > 0.0 {
            res.push(cum_pv / cum_v);
        } else {
            res.push(p);
        }
    }
    res
}
