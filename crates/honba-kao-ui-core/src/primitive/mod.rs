//! Tier 1: ATOMS — primitive tokens & elements.
//!
//! Ported from `/home/pmallapp/tmp/secured-finance-app/src/components/atoms` (54 atoms)
//! with `honba-kao-ui-core` equivalents prefixed `Hf`.
//!
//! NSH port: primitives are either bespoke `View`/`Text` compositions or thin
//! `base/` shims over `nsh_base` components. See `nsh-port.md` §3.

pub mod badge;
pub mod base;
pub mod card;
pub mod counter;
pub mod currency;
pub mod display_box;
pub mod footer;
pub mod identicon;
pub mod kbd;
pub mod panel;
pub mod price_yield;
pub mod prompt;
pub mod segmented;
pub mod separator;
pub mod sort_arrows;
pub mod sparkline;
pub mod tab;
pub mod table;
pub mod timer;

// Base shims (wrap nsh_base components)
pub use base::button::{ButtonSize, ButtonVariant, HfButton, HfChip, HfCloseButton, HfIconButton};
pub use base::dialog::{HfDialog, HfDrawer, HfModal};
pub use base::input::{HfInput, HfNumberInput};
pub use base::link::HfLink;
pub use base::plot::{Curve, HfGrid, HfLine, HfPlot};
pub use base::progress::{HfProgress, HfSpinner};
pub use base::sheet::HfSheet;
pub use base::switch::HfSwitch;
pub use base::tabs::HfTabs;
pub use base::toast::{HfAlertBanner, HfAlertBannerStack, HfToastStack};

// Bespoke primitives
pub use badge::{BadgeVariant, HfBadge, HfMoneynessBadge, HfStatusChip, Moneyness};
pub use card::HfCard;
pub use counter::HfCounter;
pub use currency::{Currency, HfAssetBadge, HfCurrencyIcon, HfCurrencyListItem};
pub use display_box::{HfOrderDisplayBox, OrderDisplayItem};
pub use footer::HfFooterBar;
pub use identicon::HfIdenticon;
pub use kbd::HfKbdBadge;
pub use panel::{HfPanel, HfPanelHeader, HfSectionGrid};
pub use price_yield::HfPriceYieldItem;
pub use prompt::{HfPromptSymbol, HfStatusLed, HfThinkingOrb};
pub use separator::{ColorBarSegment, HfColorBar, HfOiBar, HfSeparator, SeparatorOrientation};
pub use sort_arrows::{HfSortArrows, SortDirection};
pub use sparkline::HfSparklinePrimitive;
pub use tab::{HfSubTab, HfTab};
pub use table::{HfTable, HfTableCell, HfTableHeader, HfTableRow, TableCellAlign};
pub use timer::HfTimer;
