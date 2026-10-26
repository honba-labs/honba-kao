//! Command grammar for the CommandBar (`Ctrl+G`) and CommandPalette (`Ctrl+K`).
//!
//! Faithful port of OpenTerminalUI `frontend/src/components/layout/commanding.ts`:
//! the function table, the `TICKER FUNCTION [modifiers]` grammar, the fuzzy
//! scorer and the ticker hint builder. Pure and browser-free, so it is unit
//! tested from `tests/command_parse.rs`.
//!
//! Deliberate deviation: the route table targets honba-kao's own routes (see
//! [`route_for`]). OpenTerminalUI's `/equity/stocks`, `/equity/cockpit` and
//! friends have no counterpart yet, so those commands report
//! [`CommandOutcome::Unsupported`] instead of navigating to a missing page.

/// One entry of the command function table (`COMMAND_FUNCTIONS`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommandFunctionSpec {
    pub code: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub aliases: &'static [&'static str],
    /// True when the function needs a symbol (`TCS DES`, `INFY FA`, ...).
    pub security_scoped: bool,
}

/// OpenTerminalUI's full function table, in its order.
pub const COMMAND_FUNCTIONS: &[CommandFunctionSpec] = &[
    CommandFunctionSpec { code: "DESK", label: "Analyst Desk", description: "Open the cockpit analyst workspace", aliases: &["COCKPIT", "MONITOR"], security_scoped: false },
    CommandFunctionSpec { code: "DES", label: "Description / Security Hub", description: "Open security hub overview", aliases: &["SECURITY", "HUB"], security_scoped: true },
    CommandFunctionSpec { code: "GP", label: "Graph Price", description: "Open chart tab", aliases: &["CHART"], security_scoped: true },
    CommandFunctionSpec { code: "CH", label: "Chart Workstation", description: "Open chart workstation with active symbol", aliases: &["WORKSTATION"], security_scoped: true },
    CommandFunctionSpec { code: "FA", label: "Financial Analysis", description: "Open financials tab", aliases: &["FIN", "FUNDAMENTALS"], security_scoped: true },
    CommandFunctionSpec { code: "NEWS", label: "News", description: "Open news (global or ticker-specific)", aliases: &["N"], security_scoped: false },
    CommandFunctionSpec { code: "OPT", label: "Options", description: "Open options / F&O view", aliases: &["OPTIONS"], security_scoped: true },
    CommandFunctionSpec { code: "ALERT", label: "Alerts", description: "Open symbol alert workflows", aliases: &["ALERTS", "AL"], security_scoped: true },
    CommandFunctionSpec { code: "EST", label: "Estimates", description: "Open analyst estimates tab", aliases: &["ESTIMATES"], security_scoped: true },
    CommandFunctionSpec { code: "PEER", label: "Peers", description: "Open peers comparison tab", aliases: &["PEERS"], security_scoped: true },
    CommandFunctionSpec { code: "OWN", label: "Ownership", description: "Open ownership tab", aliases: &["OWNERSHIP"], security_scoped: true },
    CommandFunctionSpec { code: "EQS", label: "Equity Screener", description: "Open equity screener", aliases: &["SCREENER"], security_scoped: false },
    CommandFunctionSpec { code: "PORT", label: "Portfolio", description: "Open portfolio", aliases: &["PF", "PORTFOLIO"], security_scoped: false },
    CommandFunctionSpec { code: "WL", label: "Watchlist", description: "Open watchlist", aliases: &["WATCHLIST"], security_scoped: false },
    CommandFunctionSpec { code: "TOP", label: "Top Stories", description: "Open top market stories", aliases: &["HEADLINES"], security_scoped: false },
    CommandFunctionSpec { code: "BT", label: "Backtesting", description: "Open backtesting workspace", aliases: &["BACKTEST"], security_scoped: false },
    CommandFunctionSpec { code: "SET", label: "Settings", description: "Open settings", aliases: &["SETTINGS"], security_scoped: false },
    CommandFunctionSpec { code: "OPS", label: "Ops Dashboard", description: "Open operations dashboard", aliases: &["OPERATIONS"], security_scoped: false },
    CommandFunctionSpec { code: "LAUNCH", label: "Launchpad", description: "Open multi-panel launchpad", aliases: &["LP", "LAUNCHPAD"], security_scoped: false },
    CommandFunctionSpec { code: "COMP", label: "Split Compare", description: "Open split-screen comparison", aliases: &["COMPARE"], security_scoped: false },
    CommandFunctionSpec { code: "YCURVE", label: "Yield Curve", description: "Open US Treasury yield curve dashboard", aliases: &["GC", "YIELD", "CURVE"], security_scoped: false },
    CommandFunctionSpec { code: "ECAL", label: "Economic Calendar", description: "Open global economic calendar", aliases: &["CALENDAR"], security_scoped: false },
    CommandFunctionSpec { code: "ECOF", label: "Macro Dashboard", description: "Open macro indicators dashboard", aliases: &["MACRO", "INDICATORS"], security_scoped: false },
    CommandFunctionSpec { code: "FRED", label: "FRED Series", description: "Chart a FRED economic series", aliases: &["SERIES"], security_scoped: false },
    CommandFunctionSpec { code: "RRG", label: "Sector Rotation Map", description: "Relative Rotation Graph (RRG)", aliases: &["SROT", "SECTOR"], security_scoped: false },
    CommandFunctionSpec { code: "CRYP", label: "Crypto Workspace", description: "Open dedicated crypto workspace", aliases: &["CRYPTO"], security_scoped: false },
    CommandFunctionSpec { code: "CMDTY", label: "Commodities", description: "Open commodity market view", aliases: &["COMMODITY", "GOLD", "OIL"], security_scoped: false },
    CommandFunctionSpec { code: "FX", label: "Forex", description: "Open FX market view", aliases: &["FOREX", "CURRENCY"], security_scoped: false },
    CommandFunctionSpec { code: "ETFA", label: "ETF Analytics", description: "Open ETF market view", aliases: &["ETF"], security_scoped: false },
    CommandFunctionSpec { code: "BOND", label: "Bonds", description: "Open bond and yield views", aliases: &["CREDIT", "FIXED"], security_scoped: false },
    CommandFunctionSpec { code: "HOT", label: "Hotlists", description: "Open movers and hotlists", aliases: &["MOVERS", "GAINERS", "LOSERS", "HOTLISTS"], security_scoped: false },
    CommandFunctionSpec { code: "TCA", label: "Transaction Costs", description: "Open TCA workflow", aliases: &["COST", "SLIPPAGE"], security_scoped: false },
    CommandFunctionSpec { code: "COMM", label: "Community", description: "Open community and idea flow", aliases: &["IDEAS", "SOCIAL"], security_scoped: false },
    CommandFunctionSpec { code: "DEPTH", label: "Market Depth", description: "Open market depth workflow", aliases: &["DOM", "L2", "BOOK"], security_scoped: false },
];

/// Financial statement focus words accepted after `FA` (`INCOME`, `BS`, ...).
/// Returns the canonical lowercase sub-tab id.
pub fn financial_subtab(token: &str) -> Option<&'static str> {
    match token.trim().to_ascii_uppercase().as_str() {
        "INCOME" | "IS" => Some("income"),
        "BALANCE" | "BS" => Some("balance"),
        "CASHFLOW" | "CF" => Some("cashflow"),
        "MARGINS" => Some("margins"),
        "RATIOS" => Some("ratios"),
        _ => None,
    }
}

/// A parsed command-bar input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParsedCommand {
    /// `RELIANCE`: a bare ticker, opens its overview.
    Ticker { raw: String, ticker: String },
    /// `RELIANCE FA income`: a ticker followed by a function.
    TickerFunction { raw: String, ticker: String, func: &'static str, modifiers: Vec<String> },
    /// `PORT`, `DESK RELIANCE`: a function with no ticker in front.
    Function { raw: String, func: &'static str, modifiers: Vec<String> },
    /// Anything else. Routed to the AI agent (not wired yet).
    NaturalLanguage { raw: String, query: String },
}

/// What executing a parsed command does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandOutcome {
    /// Navigate to this route (may carry a query string).
    Navigate(String),
    /// The command is valid OpenTerminalUI grammar but has no page here yet.
    Unsupported(String),
}

fn normalize_token(value: &str) -> String {
    value.trim().to_ascii_uppercase()
}

/// `^[A-Z0-9.\-]{1,20}$` on an already-uppercased token.
pub fn looks_like_ticker(token: &str) -> bool {
    let len = token.chars().count();
    (1..=20).contains(&len)
        && token
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '.' || c == '-')
}

/// Code or alias to canonical function code. Like the TS `Map`, a later
/// duplicate wins, so the last matching entry is returned.
pub fn function_code(token: &str) -> Option<&'static str> {
    let token = normalize_token(token);
    COMMAND_FUNCTIONS
        .iter()
        .filter(|spec| spec.code == token || spec.aliases.iter().any(|a| *a == token))
        .last()
        .map(|spec| spec.code)
}

fn is_known_function_token(token: &str) -> bool {
    function_code(token).is_some()
}

/// Parse a raw command-bar string. Port of `parseCommand`.
pub fn parse_command(input: &str) -> ParsedCommand {
    let raw = input.trim().to_string();
    let tokens: Vec<String> = raw.split_whitespace().map(normalize_token).collect();

    if tokens.is_empty() {
        return ParsedCommand::NaturalLanguage { raw: raw.clone(), query: String::new() };
    }

    if tokens.len() == 1 {
        let only = &tokens[0];
        if let Some(spec) = COMMAND_FUNCTIONS.iter().find(|spec| spec.code == only.as_str()) {
            return ParsedCommand::Function { raw, func: spec.code, modifiers: vec![] };
        }
        if looks_like_ticker(only) {
            return ParsedCommand::Ticker { raw, ticker: only.clone() };
        }
        if let Some(func) = function_code(only) {
            return ParsedCommand::Function { raw, func, modifiers: vec![] };
        }
        return ParsedCommand::NaturalLanguage { query: raw.clone(), raw };
    }

    if let Some(func) = function_code(&tokens[0]) {
        return ParsedCommand::Function { raw, func, modifiers: tokens[1..].to_vec() };
    }

    if looks_like_ticker(&tokens[0]) {
        if let Some(func) = function_code(&tokens[1]) {
            return ParsedCommand::TickerFunction {
                raw,
                ticker: tokens[0].clone(),
                func,
                modifiers: tokens[2..].to_vec(),
            };
        }

        let last = &tokens[tokens.len() - 1];
        if let Some(func) = function_code(last) {
            let previous = &tokens[..tokens.len() - 1];
            if previous.iter().all(|t| looks_like_ticker(t)) {
                if previous.len() > 1 {
                    return ParsedCommand::Function { raw, func, modifiers: previous.to_vec() };
                }
                return ParsedCommand::TickerFunction {
                    raw,
                    ticker: previous[0].clone(),
                    func,
                    modifiers: vec![],
                };
            }
        }

        return ParsedCommand::Ticker { raw, ticker: tokens[0].clone() };
    }

    ParsedCommand::NaturalLanguage { query: raw.clone(), raw }
}

/// Security-hub tab id for a security-scoped function. Port of `securityFuncToTab`.
pub fn security_tab_for(func: &str) -> &'static str {
    match func {
        "GP" | "OPT" => "chart",
        "FA" => "financials",
        "NEWS" => "news",
        "EST" => "estimates",
        "PEER" => "peers",
        "OWN" => "ownership",
        _ => "overview",
    }
}

/// Map a parsed command to a honba-kao route.
///
/// Security-scoped commands go to `/equity/security/:ticker?tab=...`, with the
/// financial focus carried as `subtab=`. Global functions map onto the routes
/// that exist in this build; anything else is [`CommandOutcome::Unsupported`].
pub fn route_for(parsed: &ParsedCommand) -> CommandOutcome {
    match parsed {
        ParsedCommand::NaturalLanguage { query, .. } => {
            if query.trim().is_empty() {
                CommandOutcome::Unsupported("Empty command".into())
            } else {
                CommandOutcome::Unsupported("AI query routing is not wired yet".into())
            }
        }
        ParsedCommand::Ticker { ticker, .. } => {
            CommandOutcome::Navigate(security_route(ticker, "overview", &[]))
        }
        ParsedCommand::TickerFunction { ticker, func, modifiers, .. } => {
            let tab = security_tab_for(func);
            CommandOutcome::Navigate(security_route(ticker, tab, modifiers))
        }
        ParsedCommand::Function { func, modifiers, .. } => match *func {
            // Ticker-scoped functions typed without a ticker: open the hub's default.
            "DES" | "GP" | "FA" | "NEWS" | "EST" | "PEER" | "OWN" | "OPT" | "ALERT" => {
                match modifiers.first().filter(|m| looks_like_ticker(m)) {
                    Some(ticker) => CommandOutcome::Navigate(security_route(
                        ticker,
                        security_tab_for(func),
                        &modifiers[1..],
                    )),
                    None => CommandOutcome::Navigate("/equity/security".into()),
                }
            }
            "EQS" => CommandOutcome::Navigate("/screener".into()),
            "PORT" => CommandOutcome::Navigate("/portfolio".into()),
            "BT" => CommandOutcome::Navigate("/simulator".into()),
            "SET" => CommandOutcome::Navigate("/settings".into()),
            "DESK" => CommandOutcome::Navigate("/portfolio".into()),
            "CH" | "LAUNCH" => CommandOutcome::Navigate("/workbench".into()),
            other => CommandOutcome::Unsupported(format!("{other} has no page in this build yet")),
        },
    }
}

fn security_route(ticker: &str, tab: &str, modifiers: &[String]) -> String {
    let mut route = format!("/equity/security/{ticker}?tab={tab}");
    if tab == "chart" && !modifiers.is_empty() {
        route.push_str(&format!("&compare={}", modifiers.join(",")));
    }
    if tab == "financials" {
        if let Some(sub) = modifiers.first().and_then(|m| financial_subtab(m)) {
            route.push_str(&format!("&subtab={sub}"));
        }
    }
    route
}

/// Fuzzy score of `needle` against `haystack`, or `-1` when it does not match.
/// Port of `fuzzyScore`: exact > prefix > substring > in-order subsequence.
pub fn fuzzy_score(haystack: &str, needle: &str) -> i32 {
    let h = haystack.to_lowercase();
    let n = needle.to_lowercase();
    let n = n.trim();
    if n.is_empty() {
        return 0;
    }
    if h == n {
        return 1000;
    }
    let h_len = h.chars().count() as i32;
    let n_len = n.chars().count() as i32;
    if h.starts_with(n) {
        return 800 - (h_len - n_len);
    }
    if let Some(byte_idx) = h.find(n) {
        let char_idx = h[..byte_idx].chars().count() as i32;
        return 500 - char_idx;
    }
    let h_chars: Vec<char> = h.chars().collect();
    let mut score = 0;
    let mut cursor = 0usize;
    for ch in n.chars() {
        match h_chars[cursor.min(h_chars.len())..].iter().position(|c| *c == ch) {
            Some(offset) => {
                let idx = cursor + offset;
                score += if idx == cursor { 12 } else { 4 };
                cursor = idx + 1;
            }
            None => return -1,
        }
    }
    score
}

/// One row of the ticker hint list shown under the bar (`CommandTickerHint`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TickerHint {
    pub key: String,
    pub title: String,
    pub subtitle: String,
    pub command: String,
}

const TICKER_FUNCTION_HINTS: &[(&str, &str, &str)] = &[
    ("DES", "Overview", "Open the security overview"),
    ("FA", "Financials", "Open financial statements and ratios"),
    ("CH", "Chart Workstation", "Load the symbol into chart workstation"),
    ("NEWS", "News", "Open ticker-specific news"),
    ("OPT", "Option Chain", "Open options and derivatives"),
    ("EST", "Estimates", "Open analyst estimates"),
    ("PEER", "Peers", "Open peer comparison"),
    ("ALERT", "Alerts", "Create or review alerts for the symbol"),
    ("COMP", "Compare", "Send the symbol into split compare"),
];

const FINANCIAL_SUB_HINTS: &[(&str, &str)] = &[
    ("INCOME", "Income statement focus"),
    ("BALANCE", "Balance sheet focus"),
    ("CASHFLOW", "Cash flow focus"),
    ("MARGINS", "Margins and profitability focus"),
    ("RATIOS", "Ratio analysis focus"),
];

/// Port of `buildTickerCommandHints`: `TICKER` alone lists every function,
/// `TICKER FA` lists the statement focus words, and a partial second token
/// filters the function hints.
pub fn ticker_hints(input: &str) -> Vec<TickerHint> {
    let tokens: Vec<String> = input.trim().split_whitespace().map(normalize_token).collect();
    if tokens.is_empty() || !looks_like_ticker(&tokens[0]) {
        return vec![];
    }
    if tokens.len() == 1 && is_known_function_token(&tokens[0]) {
        return vec![];
    }

    let ticker = tokens[0].clone();
    let second = tokens.get(1).cloned().unwrap_or_default();
    let third = tokens.get(2).cloned().unwrap_or_default();
    let parsed_function = function_code(&second);

    let hint = |key: String, title: String, subtitle: &str, command: String| TickerHint {
        key,
        title,
        subtitle: subtitle.to_string(),
        command,
    };

    if second.is_empty() {
        return TICKER_FUNCTION_HINTS
            .iter()
            .map(|(func, _, subtitle)| {
                hint(
                    format!("hint:{ticker}:{func}"),
                    format!("{ticker} {func}"),
                    subtitle,
                    format!("{ticker} {func}"),
                )
            })
            .collect();
    }

    if parsed_function == Some("FA") || second == "FA" {
        return FINANCIAL_SUB_HINTS
            .iter()
            .filter(|(sub, subtitle)| third.is_empty() || fuzzy_score(&format!("{sub} {subtitle}"), &third) >= 0)
            .map(|(sub, subtitle)| {
                hint(
                    format!("hint:{ticker}:FA:{sub}"),
                    format!("{ticker} FA {sub}"),
                    subtitle,
                    format!("{ticker} FA {sub}"),
                )
            })
            .collect();
    }

    TICKER_FUNCTION_HINTS
        .iter()
        .filter(|(func, title, subtitle)| {
            let aliases = COMMAND_FUNCTIONS
                .iter()
                .find(|spec| spec.code == *func)
                .map(|spec| spec.aliases)
                .unwrap_or(&[]);
            func.starts_with(second.as_str())
                || fuzzy_score(title, &second) >= 0
                || fuzzy_score(subtitle, &second) >= 0
                || aliases.iter().any(|a| a.starts_with(second.as_str()))
        })
        .map(|(func, _title, subtitle)| {
            hint(
                format!("hint:{ticker}:{func}"),
                format!("{ticker} {func}"),
                subtitle,
                format!("{ticker} {func}"),
            )
        })
        .collect()
}

/// Record a submitted command at the front of the history, de-duplicated and
/// capped (`MAX_HISTORY`). Port of `commitHistory`.
pub const MAX_HISTORY: usize = 20;

pub fn commit_history(history: &mut Vec<String>, command: &str) {
    let normalized = command.trim();
    if normalized.is_empty() {
        return;
    }
    history.retain(|entry| entry != normalized);
    history.insert(0, normalized.to_string());
    history.truncate(MAX_HISTORY);
}
