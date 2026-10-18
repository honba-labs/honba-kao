//! FAQ content. Mirrors `web/src/data/faqData.ts` (subset).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FaqEntry {
    pub category: &'static str,
    pub question: &'static str,
    pub answer: &'static str,
}

pub const FAQ_ENTRIES: &[FaqEntry] = &[
    FaqEntry {
        category: "Getting started",
        question: "What is honba-kao?",
        answer: "honba-kao is a console for configuring and supervising AI trading agents on the Indian markets. You describe intent, the agents plan and execute under risk gates.",
    },
    FaqEntry {
        category: "Getting started",
        question: "Do I need a broker account?",
        answer: "In paper-trading mode, no. Live mode connects through a broker adapter you configure under Settings.",
    },
    FaqEntry {
        category: "Risk",
        question: "How are trades approved?",
        answer: "Every candidate order passes a risk gate. Rejections and approvals are visible in the decision log on the dashboard.",
    },
    FaqEntry {
        category: "Risk",
        question: "Can I stop an agent?",
        answer: "Yes. Pause stops new entries; square-off closes open positions at market.",
    },
    FaqEntry {
        category: "Data",
        question: "Where does market data come from?",
        answer: "The data page lists feed providers, freshness and coverage. The scaffold ships with sample data.",
    },
    FaqEntry {
        category: "Account",
        question: "How do I reset my password?",
        answer: "Use the reset-password screen. A one-time link is issued to the address on file.",
    },
];

/// Distinct categories in declaration order.
pub fn faq_categories() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for entry in FAQ_ENTRIES {
        if !out.contains(&entry.category) {
            out.push(entry.category);
        }
    }
    out
}
