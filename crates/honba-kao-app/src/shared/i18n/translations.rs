//! Translation table. Mirrors `web/src/i18n/translations.ts` (kept small here —
//! the reference app ships ~145 KB of strings; this is the scaffold subset).

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Language {
    En,
    Ja,
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Ja => "ja",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Language::En => "EN",
            Language::Ja => "日本語",
        }
    }
}

/// Look up a translation key. Returns `""` when the key is unknown so call
/// sites can fall back to their own literals.
pub fn t(key: &str, lang: Language) -> &'static str {
    match (key, lang) {
        ("loading", Language::En) => "Loading…",
        ("loading", Language::Ja) => "読み込み中…",

        ("nav.portfolio", Language::En) => "Portfolio",
        ("nav.portfolio", Language::Ja) => "ポートフォリオ",
        ("nav.screener", Language::En) => "Screener",
        ("nav.screener", Language::Ja) => "スクリーナー",
        ("nav.workbench", Language::En) => "Workbench",
        ("nav.workbench", Language::Ja) => "ワークベンチ",
        ("nav.strategy", Language::En) => "Strategy",
        ("nav.strategy", Language::Ja) => "ストラテジー",
        ("nav.simulator", Language::En) => "Simulator",
        ("nav.simulator", Language::Ja) => "シミュレーター",
        ("nav.tradebook", Language::En) => "Tradebook",
        ("nav.tradebook", Language::Ja) => "トレードブック",
        ("nav.user", Language::En) => "User",
        ("nav.user", Language::Ja) => "ユーザー",

        ("auth.sign_in", Language::En) => "Sign in",
        ("auth.sign_in", Language::Ja) => "サインイン",
        ("auth.sign_out", Language::En) => "Sign out",
        ("auth.sign_out", Language::Ja) => "サインアウト",
        ("auth.register", Language::En) => "Create account",
        ("auth.register", Language::Ja) => "アカウント作成",
        ("auth.reset_password", Language::En) => "Reset password",
        ("auth.reset_password", Language::Ja) => "パスワード再設定",
        ("auth.login_required", Language::En) => "Sign in to continue",
        ("auth.login_required", Language::Ja) => "続行するにはサインインしてください",

        ("action.continue", Language::En) => "Continue",
        ("action.continue", Language::Ja) => "続行",
        ("action.cancel", Language::En) => "Cancel",
        ("action.cancel", Language::Ja) => "キャンセル",
        ("action.confirm", Language::En) => "Confirm",
        ("action.confirm", Language::Ja) => "確定",

        _ => "",
    }
}
