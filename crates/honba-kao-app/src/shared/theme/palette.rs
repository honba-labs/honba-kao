//! 256-Color Named Token Palette definitions and metadata.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorToken {
    pub index: u16,
    pub name: &'static str,
    pub hex: &'static str,
    pub group: &'static str,
}

pub fn get_palette_256() -> Vec<ColorToken> {
    let mut tokens = Vec::with_capacity(256);

    // 000-015: ANSI Terminal Core
    let ansi = [
        (0, "term-black", "#0a1017"),
        (1, "term-red", "#d9383e"),
        (2, "term-green", "#00b88d"),
        (3, "term-yellow", "#d9b300"),
        (4, "term-blue", "#0073d9"),
        (5, "term-magenta", "#b340b3"),
        (6, "term-cyan", "#00a3a3"),
        (7, "term-white", "#d0d7de"),
        (8, "term-bright-black", "#4f5a62"),
        (9, "term-bright-red", "#f05359"),
        (10, "term-bright-green", "#00e0ad"),
        (11, "term-bright-yellow", "#fcd206"),
        (12, "term-bright-blue", "#0089ff"),
        (13, "term-bright-magenta", "#e05ce0"),
        (14, "term-bright-cyan", "#00d9d9"),
        (15, "term-bright-white", "#ffffff"),
    ];

    for (idx, name, hex) in ansi {
        tokens.push(ColorToken {
            index: idx,
            name,
            hex,
            group: "terminal",
        });
    }

    // 016-231: Chromatic ramps (18 hues x 12 steps)
    let hues = [
        "slate",
        "marine",
        "azure",
        "cyan",
        "teal",
        "emerald",
        "lime",
        "chartreuse",
        "amber",
        "gold",
        "orange",
        "coral",
        "crimson",
        "rose",
        "magenta",
        "purple",
        "violet",
        "indigo",
    ];

    let mut curr_idx = 16u16;
    for hue in hues {
        for _step in 1..=12 {
            tokens.push(ColorToken {
                index: curr_idx,
                name: hue,
                hex: "#3e5d76", // dynamic reference from CSS var(--c-...)
                group: "chromatic",
            });
            curr_idx += 1;
        }
    }

    // 232-255: Monochrome 24-step ramp
    for step in 0..24 {
        tokens.push(ColorToken {
            index: 232 + step,
            name: "monochrome",
            hex: "#7a8894",
            group: "monochrome",
        });
    }

    tokens
}
