//! Command-bar grammar: parsing, routing, fuzzy scoring, hints and history.

use honba_kao_app::shared::command::commanding::{
    CommandOutcome, ParsedCommand, commit_history, fuzzy_score, parse_command, route_for,
    ticker_hints,
};
use honba_kao_app::shared::command::symbols;

#[test]
fn bare_ticker_parses_as_ticker() {
    assert_eq!(
        parse_command("reliance"),
        ParsedCommand::Ticker {
            raw: "reliance".into(),
            ticker: "RELIANCE".into()
        }
    );
}

#[test]
fn ticker_then_function_parses_as_ticker_function() {
    match parse_command("infy fa income") {
        ParsedCommand::TickerFunction {
            ticker,
            func,
            modifiers,
            ..
        } => {
            assert_eq!(ticker, "INFY");
            assert_eq!(func, "FA");
            assert_eq!(modifiers, vec!["INCOME".to_string()]);
        }
        other => panic!("unexpected parse: {other:?}"),
    }
}

#[test]
fn alias_resolves_to_canonical_code() {
    // `HUB` is an alias of DES.
    assert!(matches!(
        parse_command("TCS HUB"),
        ParsedCommand::TickerFunction { func: "DES", .. }
    ));
}

#[test]
fn bare_function_parses_as_function() {
    assert!(matches!(
        parse_command("PORT"),
        ParsedCommand::Function { func: "PORT", .. }
    ));
}

#[test]
fn free_text_is_natural_language() {
    assert!(matches!(
        parse_command("? outlook for banks"),
        ParsedCommand::NaturalLanguage { .. }
    ));
}

#[test]
fn ticker_function_routes_to_security_hub_tab() {
    let parsed = parse_command("RELIANCE FA income");
    assert_eq!(
        route_for(&parsed),
        CommandOutcome::Navigate("/equity/security/RELIANCE?tab=financials&subtab=income".into())
    );
}

#[test]
fn bare_ticker_routes_to_overview() {
    assert_eq!(
        route_for(&parse_command("TCS")),
        CommandOutcome::Navigate("/equity/security/TCS?tab=overview".into())
    );
}

#[test]
fn global_function_routes_to_existing_page() {
    assert_eq!(
        route_for(&parse_command("EQS")),
        CommandOutcome::Navigate("/screener".into())
    );
}

#[test]
fn function_without_page_is_unsupported() {
    assert!(matches!(
        route_for(&parse_command("ECAL")),
        CommandOutcome::Unsupported(_)
    ));
}

#[test]
fn fuzzy_score_ranks_exact_then_prefix_then_substring_then_subsequence() {
    let exact = fuzzy_score("TCS", "tcs");
    let prefix = fuzzy_score("TCSX", "tcs");
    let substring = fuzzy_score("ABTCS", "tcs");
    let subsequence = fuzzy_score("TAXCS", "tcs");
    assert!(exact > prefix, "{exact} > {prefix}");
    assert!(prefix > substring, "{prefix} > {substring}");
    assert!(substring > subsequence, "{substring} > {subsequence}");
    assert!(subsequence >= 0);
}

#[test]
fn fuzzy_score_rejects_non_matches() {
    assert_eq!(fuzzy_score("RELIANCE", "xyz"), -1);
    assert_eq!(fuzzy_score("RELIANCE", "   "), 0);
}

#[test]
fn ticker_alone_lists_all_nine_functions() {
    let hints = ticker_hints("INFY");
    assert_eq!(hints.len(), 9);
    assert_eq!(hints[0].command, "INFY DES");
}

#[test]
fn ticker_fa_lists_statement_focus_words() {
    let hints = ticker_hints("INFY FA");
    assert_eq!(hints.len(), 5);
    assert!(hints.iter().any(|h| h.command == "INFY FA CASHFLOW"));
}

#[test]
fn ticker_hints_empty_for_bare_function_word() {
    assert!(ticker_hints("PORT").is_empty());
}

#[test]
fn history_dedupes_moves_to_front_and_caps() {
    let mut history = vec!["A".to_string(), "B".to_string()];
    commit_history(&mut history, "B");
    assert_eq!(history, vec!["B".to_string(), "A".to_string()]);

    for i in 0..30 {
        commit_history(&mut history, &format!("cmd{i}"));
    }
    assert_eq!(history.len(), 20);
    assert_eq!(history[0], "cmd29");
}

#[test]
fn history_ignores_blank_commands() {
    let mut history = Vec::new();
    commit_history(&mut history, "   ");
    assert!(history.is_empty());
}

#[test]
fn mock_symbol_search_finds_by_name_and_ticker() {
    let by_ticker = symbols::search("rel", 5);
    assert!(by_ticker.iter().any(|s| s.ticker == "RELIANCE"));
    let by_name = symbols::search("infosys", 5);
    assert!(by_name.iter().any(|s| s.ticker == "INFY"));
}
