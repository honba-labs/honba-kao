//! Procedural outline SVG glyphs (zero external font / image asset dependencies).

pub mod outline;

pub use outline::{
    SvgBtc, SvgChevronDown, SvgChevronLeft, SvgChevronRight, SvgChevronUp, SvgClose,
    SvgEth, SvgExternalLink, SvgInfo, SvgMenu, SvgMinus, SvgOrderBook, SvgPlus, SvgPrompt,
    SvgRupee, SvgSearch, SvgSentimentBearish, SvgSentimentBullish, SvgSentimentNeutral,
    SvgSpinner, SvgSortAsc, SvgSortChevron, SvgSortDesc, SvgThinkingOrb, SvgUsdc, SvgWallet,
};
