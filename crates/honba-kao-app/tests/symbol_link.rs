//! Symbol link book: group symbols, panel membership, and cycling.

use honba_kao_app::shared::contexts::symbol_link::LinkBook;
use honba_kao_ui_core::LinkGroup;

#[test]
fn panels_in_one_group_share_its_symbol_and_other_groups_stay_intact() {
    let mut book = LinkBook::new();
    book.set_panel_group("chart", LinkGroup::Red);
    book.set_panel_group("dom", LinkGroup::Red);
    book.set_panel_group("news", LinkGroup::Blue);

    assert!(book.set_group_symbol(LinkGroup::Red, Some("RELIANCE")));
    assert_eq!(book.group_symbol(LinkGroup::Red), Some("RELIANCE"));
    assert_eq!(book.group_symbol(LinkGroup::Blue), None);

    assert_eq!(book.panels_in(LinkGroup::Red), vec!["chart".to_string(), "dom".to_string()]);
    assert_eq!(book.panel_group("news"), LinkGroup::Blue);
}

#[test]
fn setting_the_same_symbol_reports_no_change() {
    let mut book = LinkBook::new();
    assert!(book.set_group_symbol(LinkGroup::Green, Some("TCS")));
    assert!(!book.set_group_symbol(LinkGroup::Green, Some("TCS")));
    assert!(!book.set_group_symbol(LinkGroup::Green, Some("  TCS  ")));
}

#[test]
fn blank_symbol_clears_the_group() {
    let mut book = LinkBook::new();
    book.set_group_symbol(LinkGroup::Yellow, Some("INFY"));
    assert!(book.set_group_symbol(LinkGroup::Yellow, Some("   ")));
    assert_eq!(book.group_symbol(LinkGroup::Yellow), None);
}

#[test]
fn unlinked_group_cannot_hold_a_symbol() {
    let mut book = LinkBook::new();
    assert!(!book.set_group_symbol(LinkGroup::None, Some("NIFTY")));
    assert_eq!(book.group_symbol(LinkGroup::None), None);
}

#[test]
fn assigning_none_removes_panel_membership() {
    let mut book = LinkBook::new();
    book.set_panel_group("chart", LinkGroup::Red);
    book.set_panel_group("chart", LinkGroup::None);
    assert_eq!(book.panel_group("chart"), LinkGroup::None);
    assert!(book.panels_in(LinkGroup::Red).is_empty());
}

#[test]
fn group_cycle_visits_all_four_then_returns_to_none() {
    let mut g = LinkGroup::None;
    let mut seen = Vec::new();
    for _ in 0..5 {
        g = g.cycle();
        seen.push(g);
    }
    assert_eq!(
        seen,
        vec![LinkGroup::Red, LinkGroup::Blue, LinkGroup::Green, LinkGroup::Yellow, LinkGroup::None]
    );
}

#[test]
fn group_letters_follow_plan_naming() {
    let letters: Vec<&str> = LinkGroup::LINKED.iter().map(|g| g.letter()).collect();
    assert_eq!(letters, vec!["A", "B", "C", "D"]);
    assert_eq!(LinkGroup::from_name("Yellow"), LinkGroup::Yellow);
    assert_eq!(LinkGroup::from_name("bogus"), LinkGroup::None);
}
