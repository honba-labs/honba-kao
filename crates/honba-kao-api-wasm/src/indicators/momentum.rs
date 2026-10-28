pub fn calculate_rsi(prices: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut res = vec![None; prices.len()];
    if prices.len() <= period || period == 0 {
        return res;
    }

    let mut gains = Vec::with_capacity(prices.len() - 1);
    let mut losses = Vec::with_capacity(prices.len() - 1);

    for w in prices.windows(2) {
        let diff = w[1] - w[0];
        if diff >= 0.0 {
            gains.push(diff);
            losses.push(0.0);
        } else {
            gains.push(0.0);
            losses.push(-diff);
        }
    }

    let mut avg_gain = gains[..period].iter().sum::<f64>() / period as f64;
    let mut avg_loss = losses[..period].iter().sum::<f64>() / period as f64;

    // #62: RSI edge cases — flat prices (no gain, no loss) is neutral 50.0;
    // pure gains (no loss) is fully overbought 100.0.
    let rsi_first = if avg_loss == 0.0 {
        if avg_gain == 0.0 { 50.0 } else { 100.0 }
    } else {
        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    };
    res[period] = Some(rsi_first);

    for i in period..gains.len() {
        avg_gain = (avg_gain * (period as f64 - 1.0) + gains[i]) / period as f64;
        avg_loss = (avg_loss * (period as f64 - 1.0) + losses[i]) / period as f64;

        let rsi = if avg_loss == 0.0 {
            if avg_gain == 0.0 { 50.0 } else { 100.0 }
        } else {
            let rs = avg_gain / avg_loss;
            100.0 - (100.0 / (1.0 + rs))
        };
        res[i + 1] = Some(rsi);
    }

    res
}

pub fn calculate_roc(prices: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut res = Vec::with_capacity(prices.len());
    for i in 0..prices.len() {
        if i >= period && prices[i - period] > 0.0 {
            let roc = ((prices[i] - prices[i - period]) / prices[i - period]) * 100.0;
            res.push(Some(roc));
        } else {
            res.push(None);
        }
    }
    res
}
