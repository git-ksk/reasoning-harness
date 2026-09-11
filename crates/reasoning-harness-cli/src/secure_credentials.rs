use std::{env, ffi::OsString};

use keyring::{Entry, Error as KeyringError};

use super::Provider;

pub(crate) const CREDENTIAL_SERVICE: &str = "io.github.git-ksk.reason-cli.credentials.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CredentialSource {
    Environment,
    OsStore,
}

pub(crate) struct ResolvedCredential {
    secret: String,
    pub source: CredentialSource,
}

impl ResolvedCredential {
    pub(crate) fn expose_secret(&self) -> &str {
        &self.secret
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CredentialErrorKind {
    Missing,
    InvalidEnvironment,
    StoreUnavailable,
    StoreFailure,
    InvalidSecret,
}

#[derive(Debug)]
pub(crate) struct CredentialError {
    pub kind: CredentialErrorKind,
    pub message: String,
}

impl CredentialError {
    fn new(kind: CredentialErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub(crate) const fn failure_class(&self) -> &'static str {
        match self.kind {
            CredentialErrorKind::Missing | CredentialErrorKind::InvalidEnvironment => "credentials",
            CredentialErrorKind::StoreUnavailable => "credential_store_unavailable",
            CredentialErrorKind::StoreFailure => "credential_store_error",
            CredentialErrorKind::InvalidSecret => "credentials",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CredentialIdentity {
    pub service: &'static str,
    pub account: &'static str,
}

pub(crate) fn credential_identity(provider: Provider) -> CredentialIdentity {
    CredentialIdentity {
        service: CREDENTIAL_SERVICE,
        account: match canonical_provider(provider) {
            Provider::Mistral => "provider:mistral:account:default",
            Provider::Google => "provider:google:account:default",
            Provider::Groq => "provider:groq:account:default",
            Provider::Nvidia => "provider:nvidia:account:default",
            Provider::Gemma => unreachable!("Gemma canonicalizes to Google"),
        },
    }
}

pub(crate) const fn credential_env(provider: Provider) -> &'static str {
    match canonical_provider(provider) {
        Provider::Mistral => "MISTRAL_API_KEY",
        Provider::Google => "GEMINI_API_KEY",
        Provider::Groq => "GROQ_API_KEY",
        Provider::Nvidia => "NVIDIA_API_KEY",
        Provider::Gemma => unreachable!(),
    }
}

pub(crate) const fn canonical_provider(provider: Provider) -> Provider {
    match provider {
        Provider::Gemma => Provider::Google,
        other => other,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnvironmentCredentialState {
    Missing,
    Present,
    Invalid,
}

impl EnvironmentCredentialState {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Present => "present",
            Self::Invalid => "invalid",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StoredCredentialState {
    Missing,
    Present,
    Invalid,
    Unavailable,
}

impl StoredCredentialState {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Present => "present",
            Self::Invalid => "invalid",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EffectiveCredentialSource {
    Environment,
    OsStore,
    Missing,
    InvalidEnvironment,
    InvalidOsStore,
    StoreUnavailable,
}

impl EffectiveCredentialSource {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Environment => "environment",
            Self::OsStore => "os_store",
            Self::Missing => "missing",
            Self::InvalidEnvironment => "invalid_environment",
            Self::InvalidOsStore => "invalid_os_store",
            Self::StoreUnavailable => "credential_store_unavailable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProviderCredentialStatus {
    pub environment: EnvironmentCredentialState,
    pub os_store: StoredCredentialState,
    pub effective: EffectiveCredentialSource,
}

trait CredentialStore {
    fn get(&self, identity: CredentialIdentity) -> Result<Option<String>, CredentialError>;
    fn set(&self, identity: CredentialIdentity, secret: &str) -> Result<(), CredentialError>;
    fn delete(&self, identity: CredentialIdentity) -> Result<bool, CredentialError>;
}

struct OsCredentialStore;

impl OsCredentialStore {
    fn entry(identity: CredentialIdentity) -> Result<Entry, CredentialError> {
        Entry::new(identity.service, identity.account).map_err(map_store_error)
    }
}

impl CredentialStore for OsCredentialStore {
    fn get(&self, identity: CredentialIdentity) -> Result<Option<String>, CredentialError> {
        let entry = Self::entry(identity)?;
        match entry.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(KeyringError::NoEntry) => Ok(None),
            Err(error) => Err(map_store_error(error)),
        }
    }

    fn set(&self, identity: CredentialIdentity, secret: &str) -> Result<(), CredentialError> {
        if secret.trim().is_empty() {
            return Err(CredentialError::new(
                CredentialErrorKind::InvalidSecret,
                "refusing to store an empty provider credential",
            ));
        }
        Self::entry(identity)?
            .set_password(secret)
            .map_err(map_store_error)
    }

    fn delete(&self, identity: CredentialIdentity) -> Result<bool, CredentialError> {
        let entry = Self::entry(identity)?;
        match entry.delete_credential() {
            Ok(()) => Ok(true),
            Err(KeyringError::NoEntry) => Ok(false),
            Err(error) => Err(map_store_error(error)),
        }
    }
}

fn map_store_error(error: KeyringError) -> CredentialError {
    let kind = match error {
        KeyringError::NoDefaultStore
        | KeyringError::NoStorageAccess(_)
        | KeyringError::PlatformFailure(_)
        | KeyringError::NotSupportedByStore(_) => CredentialErrorKind::StoreUnavailable,
        KeyringError::NoEntry => CredentialErrorKind::Missing,
        KeyringError::BadEncoding(_)
        | KeyringError::BadDataFormat(_, _)
        | KeyringError::BadStoreFormat(_)
        | KeyringError::TooLong(_, _)
        | KeyringError::Invalid(_, _)
        | KeyringError::Ambiguous(_) => CredentialErrorKind::StoreFailure,
        _ => CredentialErrorKind::StoreFailure,
    };
    let message = match kind {
        CredentialErrorKind::StoreUnavailable => {
            "OS credential store is unavailable; use the provider environment variable for CI/headless use or configure the platform credential service"
        }
        CredentialErrorKind::Missing => "no stored provider credential was found",
        CredentialErrorKind::StoreFailure => {
            "OS credential store returned unusable credential data; remove/replace the stored credential and retry"
        }
        CredentialErrorKind::InvalidEnvironment | CredentialErrorKind::InvalidSecret => {
            "invalid provider credential"
        }
    };
    CredentialError::new(kind, message)
}

pub(crate) fn provider_environment_credential(
    provider: Provider,
) -> Result<Option<String>, CredentialError> {
    let Some(value) = env::var_os(credential_env(provider)) else {
        return Ok(None);
    };
    let secret = value.into_string().map_err(|_| {
        CredentialError::new(
            CredentialErrorKind::InvalidEnvironment,
            format!("{} is set but is not valid UTF-8", credential_env(provider)),
        )
    })?;
    if secret.trim().is_empty() {
        return Err(CredentialError::new(
            CredentialErrorKind::InvalidEnvironment,
            format!("{} is set but empty", credential_env(provider)),
        ));
    }
    Ok(Some(secret))
}

pub(crate) fn inspect_provider_credential(
    provider: Provider,
) -> Result<ProviderCredentialStatus, CredentialError> {
    let environment = match env::var_os(credential_env(provider)) {
        None => EnvironmentCredentialState::Missing,
        Some(value) => match value.into_string() {
            Ok(secret) if !secret.trim().is_empty() => EnvironmentCredentialState::Present,
            _ => EnvironmentCredentialState::Invalid,
        },
    };
    let os_store = match OsCredentialStore.get(credential_identity(provider)) {
        Ok(Some(secret)) if !secret.trim().is_empty() => StoredCredentialState::Present,
        Ok(Some(_)) => StoredCredentialState::Invalid,
        Ok(None) => StoredCredentialState::Missing,
        Err(error) if error.kind == CredentialErrorKind::StoreUnavailable => {
            StoredCredentialState::Unavailable
        }
        Err(error) => return Err(error),
    };
    let effective = match (environment, os_store) {
        (EnvironmentCredentialState::Present, _) => EffectiveCredentialSource::Environment,
        (EnvironmentCredentialState::Invalid, _) => EffectiveCredentialSource::InvalidEnvironment,
        (EnvironmentCredentialState::Missing, StoredCredentialState::Present) => {
            EffectiveCredentialSource::OsStore
        }
        (EnvironmentCredentialState::Missing, StoredCredentialState::Invalid) => {
            EffectiveCredentialSource::InvalidOsStore
        }
        (EnvironmentCredentialState::Missing, StoredCredentialState::Unavailable) => {
            EffectiveCredentialSource::StoreUnavailable
        }
        (EnvironmentCredentialState::Missing, StoredCredentialState::Missing) => {
            EffectiveCredentialSource::Missing
        }
    };
    Ok(ProviderCredentialStatus {
        environment,
        os_store,
        effective,
    })
}

pub(crate) fn resolve_provider_credential(
    provider: Provider,
) -> Result<ResolvedCredential, CredentialError> {
    let env_name = credential_env(provider);
    resolve_with_store(provider, env::var_os(env_name), &OsCredentialStore)
}

fn resolve_with_store(
    provider: Provider,
    environment_value: Option<OsString>,
    store: &dyn CredentialStore,
) -> Result<ResolvedCredential, CredentialError> {
    if let Some(value) = environment_value {
        let secret = value.into_string().map_err(|_| {
            CredentialError::new(
                CredentialErrorKind::InvalidEnvironment,
                format!(
                    "{0} is set but is not valid UTF-8; refusing OS-store fallback",
                    credential_env(provider)
                ),
            )
        })?;
        if secret.trim().is_empty() {
            return Err(CredentialError::new(
                CredentialErrorKind::InvalidEnvironment,
                format!(
                    "{0} is set but empty; refusing OS-store fallback",
                    credential_env(provider)
                ),
            ));
        }
        return Ok(ResolvedCredential {
            secret,
            source: CredentialSource::Environment,
        });
    }

    let identity = credential_identity(provider);
    let stored = store.get(identity).map_err(|error| {
        if error.kind == CredentialErrorKind::StoreUnavailable {
            CredentialError::new(
                CredentialErrorKind::StoreUnavailable,
                format!(
                    "OS credential store is unavailable; set {} for CI/headless use or configure the platform credential service",
                    credential_env(provider)
                ),
            )
        } else {
            error
        }
    })?;
    match stored {
        Some(secret) if !secret.trim().is_empty() => Ok(ResolvedCredential {
            secret,
            source: CredentialSource::OsStore,
        }),
        Some(_) => Err(CredentialError::new(
            CredentialErrorKind::InvalidSecret,
            "stored provider credential is empty; remove/replace it and retry",
        )),
        None => Err(CredentialError::new(
            CredentialErrorKind::Missing,
            format!(
                "no credential is available for {}; set {} for CI/headless use or save a credential with Reason",
                provider_label(provider),
                credential_env(provider)
            ),
        )),
    }
}

pub(crate) fn save_provider_credential(
    provider: Provider,
    secret: &str,
) -> Result<(), CredentialError> {
    OsCredentialStore.set(credential_identity(provider), secret)
}

pub(crate) fn delete_provider_credential(provider: Provider) -> Result<bool, CredentialError> {
    OsCredentialStore.delete(credential_identity(provider))
}

pub(crate) fn stored_provider_credential_exists(
    provider: Provider,
) -> Result<bool, CredentialError> {
    Ok(OsCredentialStore
        .get(credential_identity(provider))?
        .is_some())
}

const fn provider_label(provider: Provider) -> &'static str {
    match canonical_provider(provider) {
        Provider::Mistral => "mistral",
        Provider::Google => "google",
        Provider::Groq => "groq",
        Provider::Nvidia => "nvidia",
        Provider::Gemma => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::BTreeMap};

    use super::*;

    #[derive(Default)]
    struct MemoryStore {
        values: RefCell<BTreeMap<String, String>>,
        gets: RefCell<usize>,
        unavailable: bool,
    }

    impl MemoryStore {
        fn key(identity: CredentialIdentity) -> String {
            format!("{}::{}", identity.service, identity.account)
        }
    }

    impl CredentialStore for MemoryStore {
        fn get(&self, identity: CredentialIdentity) -> Result<Option<String>, CredentialError> {
            *self.gets.borrow_mut() += 1;
            if self.unavailable {
                return Err(CredentialError::new(
                    CredentialErrorKind::StoreUnavailable,
                    "store unavailable",
                ));
            }
            Ok(self.values.borrow().get(&Self::key(identity)).cloned())
        }

        fn set(&self, identity: CredentialIdentity, secret: &str) -> Result<(), CredentialError> {
            self.values
                .borrow_mut()
                .insert(Self::key(identity), secret.to_string());
            Ok(())
        }

        fn delete(&self, identity: CredentialIdentity) -> Result<bool, CredentialError> {
            Ok(self
                .values
                .borrow_mut()
                .remove(&Self::key(identity))
                .is_some())
        }
    }

    #[test]
    fn environment_credential_wins_without_reading_store() {
        let store = MemoryStore::default();
        store
            .set(credential_identity(Provider::Mistral), "stored-secret")
            .unwrap();
        let resolved = resolve_with_store(
            Provider::Mistral,
            Some(OsString::from("env-secret")),
            &store,
        )
        .unwrap();
        assert_eq!(resolved.expose_secret(), "env-secret");
        assert_eq!(resolved.source, CredentialSource::Environment);
        assert_eq!(*store.gets.borrow(), 0);
    }

    #[test]
    fn empty_environment_fails_without_store_fallback() {
        let store = MemoryStore::default();
        store
            .set(credential_identity(Provider::Mistral), "stored-secret")
            .unwrap();
        let error = resolve_with_store(Provider::Mistral, Some(OsString::from("")), &store)
            .err()
            .unwrap();
        assert_eq!(error.kind, CredentialErrorKind::InvalidEnvironment);
        assert_eq!(error.failure_class(), "credentials");
        assert_eq!(*store.gets.borrow(), 0);
        assert!(!error.message.contains("stored-secret"));
    }

    #[test]
    fn absent_environment_uses_os_store() {
        let store = MemoryStore::default();
        store
            .set(credential_identity(Provider::Groq), "stored-secret")
            .unwrap();
        let resolved = resolve_with_store(Provider::Groq, None, &store).unwrap();
        assert_eq!(resolved.expose_secret(), "stored-secret");
        assert_eq!(resolved.source, CredentialSource::OsStore);
        assert_eq!(*store.gets.borrow(), 1);
    }

    #[test]
    fn missing_and_unavailable_store_are_distinct_and_secret_free() {
        let missing = resolve_with_store(Provider::Nvidia, None, &MemoryStore::default())
            .err()
            .unwrap();
        assert_eq!(missing.kind, CredentialErrorKind::Missing);
        assert_eq!(missing.failure_class(), "credentials");

        let unavailable = resolve_with_store(
            Provider::Nvidia,
            None,
            &MemoryStore {
                unavailable: true,
                ..Default::default()
            },
        )
        .err()
        .unwrap();
        assert_eq!(unavailable.kind, CredentialErrorKind::StoreUnavailable);
        assert_eq!(unavailable.failure_class(), "credential_store_unavailable");
    }

    #[test]
    fn provider_identity_is_versioned_and_google_gemma_share_one_secret() {
        assert_eq!(
            credential_identity(Provider::Mistral).service,
            CREDENTIAL_SERVICE
        );
        assert_eq!(
            credential_identity(Provider::Google),
            credential_identity(Provider::Gemma)
        );
        assert_eq!(credential_env(Provider::Google), "GEMINI_API_KEY");
        assert_eq!(credential_env(Provider::Gemma), "GEMINI_API_KEY");
        assert_ne!(
            credential_identity(Provider::Google),
            credential_identity(Provider::Groq)
        );
    }

    #[test]
    fn memory_store_replacement_and_delete_are_scoped() {
        let store = MemoryStore::default();
        let mistral = credential_identity(Provider::Mistral);
        let groq = credential_identity(Provider::Groq);
        store.set(mistral, "one").unwrap();
        store.set(groq, "groq").unwrap();
        store.set(mistral, "two").unwrap();
        assert_eq!(store.get(mistral).unwrap().as_deref(), Some("two"));
        assert_eq!(store.get(groq).unwrap().as_deref(), Some("groq"));
        assert!(store.delete(mistral).unwrap());
        assert_eq!(store.get(mistral).unwrap(), None);
        assert_eq!(store.get(groq).unwrap().as_deref(), Some("groq"));
    }

    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    #[test]
    #[ignore = "mutates the native OS credential store; run only in isolated CI"]
    fn native_os_store_round_trip() {
        let unique = format!(
            "reason-cli-credential-v1-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let service: &'static str = Box::leak(unique.into_boxed_str());
        let identity = CredentialIdentity {
            service,
            account: "default",
        };
        let store = OsCredentialStore;
        let _ = store.delete(identity);
        store.set(identity, "native-roundtrip-secret").unwrap();
        assert_eq!(
            store.get(identity).unwrap().as_deref(),
            Some("native-roundtrip-secret")
        );
        assert!(store.delete(identity).unwrap());
        assert_eq!(store.get(identity).unwrap(), None);
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "requires a deliberately headless Linux environment"]
    fn headless_linux_store_failure_is_typed_and_never_plaintext() {
        let identity = CredentialIdentity {
            service: "reason-cli-credential-v1-headless-test",
            account: "default",
        };
        let error = OsCredentialStore.get(identity).err().unwrap();
        assert_eq!(error.kind, CredentialErrorKind::StoreUnavailable);
        assert_eq!(error.failure_class(), "credential_store_unavailable");
    }
}
