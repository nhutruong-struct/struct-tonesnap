//! API key storage in the OS credential store (Windows Credential Manager / macOS Keychain),
//! one key per provider.

use crate::settings::ProviderKind;

const SERVICE: &str = "ToneSnap";

/// The credential-store account of a provider's key. Claude keeps the name it always had.
fn account(provider: ProviderKind) -> &'static str {
    match provider {
        ProviderKind::Claude => "anthropic_api_key",
        ProviderKind::Gemini => "gemini_api_key",
        ProviderKind::Groq => "groq_api_key",
        ProviderKind::Openrouter => "openrouter_api_key",
    }
}

/// The variable debug builds read when the store has no key.
fn dev_variable(provider: ProviderKind) -> &'static str {
    match provider {
        ProviderKind::Claude => "ANTHROPIC_API_KEY",
        ProviderKind::Gemini => "GEMINI_API_KEY",
        ProviderKind::Groq => "GROQ_API_KEY",
        ProviderKind::Openrouter => "OPENROUTER_API_KEY",
    }
}

fn entry(provider: ProviderKind) -> keyring::Result<keyring::Entry> {
    keyring::Entry::new(SERVICE, account(provider))
}

/// The stored key. Debug builds fall back to the provider's environment variable for local
/// development.
pub fn api_key(provider: ProviderKind) -> Option<String> {
    entry(provider)
        .and_then(|e| e.get_password())
        .ok()
        .filter(|k| !k.is_empty())
        .or_else(|| dev_api_key(provider))
}

/// Release builds ignore the variable: developer tools such as Claude Code read the same one,
/// and ToneSnap must not bill a key the user never gave it, or report it as saved in Settings.
fn dev_api_key(provider: ProviderKind) -> Option<String> {
    if !cfg!(debug_assertions) {
        return None;
    }
    std::env::var(dev_variable(provider))
        .ok()
        .filter(|k| !k.is_empty())
}

pub fn set_api_key(provider: ProviderKind, key: &str) -> anyhow::Result<()> {
    let entry = entry(provider)?;
    if key.is_empty() {
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => {}
            Err(e) => return Err(e.into()),
        }
    } else {
        entry.set_password(key)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [ProviderKind; 4] = [
        ProviderKind::Claude,
        ProviderKind::Gemini,
        ProviderKind::Groq,
        ProviderKind::Openrouter,
    ];

    #[test]
    fn every_provider_has_its_own_account_and_variable() {
        let mut accounts: Vec<_> = ALL.iter().map(|p| account(*p)).collect();
        let mut variables: Vec<_> = ALL.iter().map(|p| dev_variable(*p)).collect();
        accounts.sort_unstable();
        accounts.dedup();
        variables.sort_unstable();
        variables.dedup();
        assert_eq!(accounts.len(), ALL.len());
        assert_eq!(variables.len(), ALL.len());
    }
}
