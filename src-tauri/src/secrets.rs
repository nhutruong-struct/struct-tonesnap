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

    /// The NSIS uninstaller deletes the keys by their Credential Manager target name, which
    /// must match what the keyring crate files them under.
    #[cfg(windows)]
    #[test]
    fn the_uninstaller_deletes_every_key() {
        let hooks = include_str!("../windows/hooks.nsh");
        for provider in ALL {
            let entry = entry(provider).expect("the account and service are valid");
            let credential = entry
                .get_credential()
                .downcast_ref::<keyring::windows::WinCredential>()
                .expect("windows-native is the credential store on Windows");
            let target = &credential.target_name;
            assert!(
                hooks.contains(&format!("\"{target}\"")),
                "hooks.nsh does not delete {target}"
            );
        }
    }

    /// The Keychain stores, replaces and deletes a key. A throwaway item under its own service
    /// name, so a key the user saved is never touched.
    #[cfg(target_os = "macos")]
    #[test]
    #[ignore = "macOS integration: writes a throwaway item to the login Keychain"]
    fn the_keychain_round_trips_a_key() {
        let entry = keyring::Entry::new("ToneSnap-test", "round_trip").expect("a valid entry");
        entry.set_password("not-a-real-key-1").expect("stored");
        assert_eq!(entry.get_password().expect("read back"), "not-a-real-key-1");
        entry.set_password("not-a-real-key-2").expect("replaced");
        assert_eq!(entry.get_password().expect("read back"), "not-a-real-key-2");
        entry.delete_credential().expect("deleted");
        assert!(matches!(entry.get_password(), Err(keyring::Error::NoEntry)));
    }
}
