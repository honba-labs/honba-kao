//! Factor heatmap — a sector × factor grid of mock 0-100 scores
//! (Value, Quality, Momentum, Growth). Plain divs only; each score cell is
//! shaded on a five-step low→high scale. Scores are static mock data until
//! the factor model is wired to honba-data.

use leptos::prelude::*;

/// Factor column headers, in display order.
const FACTORS: [&str; 4] = ["Value", "Quality", "Momentum", "Growth"];

/// Mock sector rows: (sector, [Value, Quality, Momentum, Growth]).
const SECTOR_SCORES: [(&str, [u8; 4]); 6] = [
    ("Financials", [72, 65, 48, 55]),
    ("Information Technology", [31, 88, 81, 76]),
    ("Energy", [84, 42, 39, 27]),
    ("Consumer Staples", [58, 74, 52, 41]),
    ("Health Care", [45, 79, 63, 70]),
    ("Industrials", [60, 57, 68, 49]),
];

/// Five-step score shading class (low → high).
fn score_class(score: u8) -> &'static str {
    match score {
        0..=19 => " sc-fh-cell--s1",
        20..=39 => " sc-fh-cell--s2",
        40..=59 => " sc-fh-cell--s3",
        60..=79 => " sc-fh-cell--s4",
        _ => " sc-fh-cell--s5",
    }
}

#[component]
pub fn FactorHeatmap() -> impl IntoView {
    view! {
        <div class="sc-fh">
            <div class="sc-fh__grid">
                <div class="sc-fh__corner">"Sector"</div>
                {FACTORS
                    .iter()
                    .map(|factor| view! { <div class="sc-fh__head">{*factor}</div> })
                    .collect_view()}
                {SECTOR_SCORES
                    .iter()
                    .map(|(sector, scores)| {
                        view! {
                            <div class="sc-fh__sector">{*sector}</div>
                            {scores
                                .iter()
                                .zip(FACTORS.iter())
                                .map(|(score, factor)| {
                                    view! {
                                        <div
                                            class=format!("sc-fh__cell{}", score_class(*score))
                                            title=format!("{sector} · {factor}: {score}")
                                        >
                                            {*score}
                                        </div>
                                    }
                                })
                                .collect_view()}
                        }
                    })
                    .collect_view()}
            </div>
            <div class="sc-fh__legend">
                "Mock factor scores (0 = weak, 100 = strong)"
            </div>
        </div>
    }
}
