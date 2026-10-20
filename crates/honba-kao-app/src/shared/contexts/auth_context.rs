//! Authentication context. Mirrors `web/src/contexts/AuthContext.tsx`.
//!
//! The reference app signs in against its API. honba-kao has no auth backend
//! yet, so accounts live in memory — seeded with the demo account from
//! [`crate::shared::constants::auth`] — and a successful sign-in creates the same
//! session shape the API will return. Swapping to the real client means
//! replacing the bodies of [`AuthContext::sign_in`] and
//! [`AuthContext::register`]; every screen reads the same signals.
//!
//! The *session* is persisted to `localStorage` (email + display name only —
//! never the password) so a reload or a deep link does not sign the user out.
//! Registered accounts themselves stay in memory; the demo account always
//! exists.

use leptos::prelude::*;
use web_sys::window;

use crate::shared::constants::auth as creds;

/// Session key: `"<email>\n<display name>"`.
const SESSION_KEY: &str = "honba-session";

/// The signed-in principal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: &'static str,
}

/// An in-memory account. Passwords never leave the browser; this exists only
/// so the demo sign-in, registration and "already taken" checks behave like
/// the real thing.
#[derive(Clone, Debug)]
struct Account {
    name: String,
    email: String,
    password: String,
}

impl Account {
    fn new(name: &str, email: &str, password: &str) -> Self {
        Self {
            name: name.to_string(),
            email: normalize_email(email),
            password: password.to_string(),
        }
    }
}

/// Session state shared across the app. `RwSignal<T>` is always `Copy`, so the
/// whole context is `Copy` and can be captured by any number of closures.
#[derive(Copy, Clone)]
pub struct AuthContext {
    pub user: RwSignal<Option<User>>,
    pub token: RwSignal<Option<String>>,
    accounts: RwSignal<Vec<Account>>,
}

impl AuthContext {
    pub fn new() -> Self {
        let restored = load_session();

        Self {
            user: RwSignal::new(restored.clone()),
            token: RwSignal::new(restored.map(|user| format!("demo-token:{}", user.email))),
            accounts: RwSignal::new(vec![Account::new(
                creds::DEMO_NAME,
                creds::DEMO_EMAIL,
                creds::DEMO_PASSWORD,
            )]),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.user.get().is_some() && self.token.get().is_some()
    }

    /// Sign in with email + password.
    ///
    /// Returns a human-readable message on failure so the form can render it
    /// inline, which is what the reference app does with API errors.
    pub fn sign_in(&self, email: &str, password: &str) -> Result<(), String> {
        let email = normalize_email(email);
        if email.is_empty() {
            return Err("Enter your email address.".to_string());
        }
        if password.is_empty() {
            return Err("Enter your password.".to_string());
        }

        let account = self.accounts.with(|accounts| {
            accounts
                .iter()
                .find(|account| account.email == email && account.password == password)
                .cloned()
        });

        match account {
            Some(account) => {
                self.start_session(&account.name, &account.email);
                Ok(())
            }
            None => Err("Email or password is incorrect.".to_string()),
        }
    }

    /// Create an account and sign it in.
    pub fn register(&self, name: &str, email: &str, password: &str) -> Result<(), String> {
        let name = name.trim();
        let email = normalize_email(email);

        if name.is_empty() {
            return Err("Enter your name.".to_string());
        }
        if !is_email(&email) {
            return Err("Enter a valid email address.".to_string());
        }
        if password.chars().count() < creds::MIN_PASSWORD_LEN {
            return Err(format!(
                "Use at least {} characters for your password.",
                creds::MIN_PASSWORD_LEN
            ));
        }
        if self
            .accounts
            .with(|accounts| accounts.iter().any(|account| account.email == email))
        {
            return Err("An account with that email already exists.".to_string());
        }

        self.accounts
            .update(|accounts| accounts.push(Account::new(name, &email, password)));
        self.start_session(name, &email);
        Ok(())
    }

    pub fn logout(&self) {
        self.user.set(None);
        self.token.set(None);
        clear_session();
    }

    fn start_session(&self, name: &str, email: &str) {
        let user = User {
            id: format!("usr_{}", stable_slug(email)),
            name: name.to_string(),
            email: email.to_string(),
            role: "owner",
        };

        save_session(&user);
        self.token.set(Some(format!("demo-token:{email}")));
        self.user.set(Some(user));
    }

    pub fn display_name(&self) -> String {
        self.user
            .get()
            .map(|user| user.name)
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

fn normalize_email(raw: &str) -> String {
    raw.trim().to_lowercase()
}

fn is_email(value: &str) -> bool {
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !value.contains(char::is_whitespace)
}

fn stable_slug(email: &str) -> String {
    email
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}

fn storage() -> Option<web_sys::Storage> {
    window()?.local_storage().ok()?
}

fn save_session(user: &User) {
    if let Some(storage) = storage() {
        let value = format!("{}\n{}", user.email, user.name);
        let _ = storage.set_item(SESSION_KEY, &value);
    }
}

fn clear_session() {
    if let Some(storage) = storage() {
        let _ = storage.remove_item(SESSION_KEY);
    }
}

/// `None` when there is no stored session or it is malformed — callers treat
/// that as a signed-out visitor.
fn load_session() -> Option<User> {
    let raw = storage()?.get_item(SESSION_KEY).ok()??;
    let (email, name) = raw.split_once('\n')?;
    if email.is_empty() {
        return None;
    }

    Some(User {
        id: format!("usr_{}", stable_slug(email)),
        name: name.to_string(),
        email: email.to_string(),
        role: "owner",
    })
}
