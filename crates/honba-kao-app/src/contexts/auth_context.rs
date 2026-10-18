//! Authentication context. Mirrors `web/src/contexts/AuthContext.tsx`.

use leptos::prelude::*;

/// The signed-in principal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: &'static str,
}

/// Session state shared across the app. `RwSignal<T>` is always `Copy`, so the
/// whole context is `Copy` and can be captured by any number of closures.
#[derive(Copy, Clone)]
pub struct AuthContext {
    pub user: RwSignal<Option<User>>,
    pub token: RwSignal<Option<String>>,
}

impl AuthContext {
    pub fn new() -> Self {
        Self {
            user: RwSignal::new(None),
            token: RwSignal::new(None),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.user.get().is_some() && self.token.get().is_some()
    }

    /// Demo sign-in: replaces what would be a network call + token storage.
    pub fn login(&self, name: &str) {
        self.user.set(Some(User {
            id: "usr_demo".into(),
            name: name.to_string(),
            email: "trader@honba.in".into(),
            role: "owner",
        }));
        self.token.set(Some("demo-token".into()));
    }

    pub fn logout(&self) {
        self.user.set(None);
        self.token.set(None);
    }

    pub fn display_name(&self) -> String {
        self.user
            .get()
            .map(|u| u.name)
            .unwrap_or_else(|| "Guest".to_string())
    }
}

impl Default for AuthContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn use_auth() -> AuthContext {
    expect_context::<AuthContext>()
}
