//! Explicit host-authority policy for filesystem and network natives.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use crate::specs::CapabilitySet;
use crate::RuntimeError;

/// Runtime grants are separate from the artifact's requested capability
/// manifest. Loading can inspect the manifest; invocation must also pass this
/// host policy.
#[derive(Debug, Clone)]
pub struct HostPolicy {
    granted: CapabilitySet,
    read_roots: Vec<PathBuf>,
    write_roots: Vec<PathBuf>,
    connect_addrs: Vec<SocketAddr>,
    listen_addrs: Vec<SocketAddr>,
}

impl Default for HostPolicy {
    fn default() -> Self {
        Self::untrusted()
    }
}

impl HostPolicy {
    /// No ambient host authority. Embedders must grant every capability they
    /// intentionally virtualize or expose.
    pub fn untrusted() -> Self {
        Self {
            granted: CapabilitySet::empty(),
            read_roots: Vec::new(),
            write_roots: Vec::new(),
            connect_addrs: Vec::new(),
            listen_addrs: Vec::new(),
        }
    }

    /// Compatibility policy used by the interactive CLI. Filesystem and
    /// networking remain opt-in even here.
    pub fn standard_cli() -> Self {
        let mut policy = Self::untrusted();
        policy.grant(
            CapabilitySet::CONSOLE_READ
                | CapabilitySet::CONSOLE_WRITE
                | CapabilitySet::CLOCK
                | CapabilitySet::RANDOM,
        );
        policy
    }

    pub fn granted(&self) -> CapabilitySet {
        self.granted
    }

    pub fn grant(&mut self, capabilities: CapabilitySet) {
        self.granted |= capabilities;
    }

    pub fn require(&self, required: CapabilitySet, operation: &str) -> Result<(), RuntimeError> {
        if self.granted.contains(required) {
            Ok(())
        } else {
            Err(RuntimeError::capability_denied(format!(
                "`{operation}` requires {required:?}"
            )))
        }
    }

    /// Validate an artifact's complete host request before execution begins.
    pub fn require_program(&self, requested: CapabilitySet) -> Result<(), RuntimeError> {
        if self.granted.contains(requested) {
            return Ok(());
        }
        let denied = requested.difference(self.granted);
        Err(RuntimeError::capability_denied(format!(
            "program requests `{denied}` but the host granted `{}`",
            self.granted
        )))
    }

    pub fn allow_read_root(&mut self, root: impl AsRef<Path>) -> Result<(), RuntimeError> {
        let root = canonical_directory(root.as_ref(), "read root")?;
        if !self.read_roots.contains(&root) {
            self.read_roots.push(root);
        }
        self.grant(CapabilitySet::FILE_READ);
        Ok(())
    }

    pub fn allow_write_root(&mut self, root: impl AsRef<Path>) -> Result<(), RuntimeError> {
        let root = canonical_directory(root.as_ref(), "write root")?;
        if !self.write_roots.contains(&root) {
            self.write_roots.push(root);
        }
        self.grant(CapabilitySet::FILE_WRITE);
        Ok(())
    }

    pub fn authorize_read(&self, path: impl AsRef<Path>) -> Result<PathBuf, RuntimeError> {
        self.require(CapabilitySet::FILE_READ, "file read")?;
        let canonical = path
            .as_ref()
            .canonicalize()
            .map_err(|error| RuntimeError::io_error("authorize file read", error.to_string()))?;
        if self
            .read_roots
            .iter()
            .any(|root| canonical.starts_with(root))
        {
            Ok(canonical)
        } else {
            Err(RuntimeError::capability_denied(format!(
                "read path `{}` is outside granted roots",
                path.as_ref().display()
            )))
        }
    }

    pub fn authorize_write(&self, path: impl AsRef<Path>) -> Result<PathBuf, RuntimeError> {
        self.require(CapabilitySet::FILE_WRITE, "file write")?;
        let path = path.as_ref();
        let canonical = if path.exists() {
            path.canonicalize().map_err(|error| {
                RuntimeError::io_error("authorize file write", error.to_string())
            })?
        } else {
            let parent = path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));
            let parent = parent.canonicalize().map_err(|error| {
                RuntimeError::io_error("authorize file write parent", error.to_string())
            })?;
            let file_name = path.file_name().ok_or_else(|| {
                RuntimeError::capability_denied("write path must include a file name")
            })?;
            parent.join(file_name)
        };
        if self
            .write_roots
            .iter()
            .any(|root| canonical.starts_with(root))
        {
            Ok(canonical)
        } else {
            Err(RuntimeError::capability_denied(format!(
                "write path `{}` is outside granted roots",
                path.display()
            )))
        }
    }

    /// Grant outbound TCP access to one exact numeric address.
    pub fn allow_connect_addr(&mut self, address: SocketAddr) {
        if !self.connect_addrs.contains(&address) {
            self.connect_addrs.push(address);
        }
        self.grant(CapabilitySet::NETWORK_CONNECT);
    }

    /// Grant TCP listen access to one exact numeric address.
    pub fn allow_listen_addr(&mut self, address: SocketAddr) {
        if !self.listen_addrs.contains(&address) {
            self.listen_addrs.push(address);
        }
        self.grant(CapabilitySet::NETWORK_LISTEN);
    }

    pub fn authorize_connect(&self, address: &str) -> Result<SocketAddr, RuntimeError> {
        self.authorize_address(
            address,
            CapabilitySet::NETWORK_CONNECT,
            "tcp connect",
            &self.connect_addrs,
        )
    }

    pub fn authorize_listen(&self, address: &str) -> Result<SocketAddr, RuntimeError> {
        self.authorize_address(
            address,
            CapabilitySet::NETWORK_LISTEN,
            "tcp listen",
            &self.listen_addrs,
        )
    }

    fn authorize_address(
        &self,
        address: &str,
        capability: CapabilitySet,
        operation: &str,
        allowed: &[SocketAddr],
    ) -> Result<SocketAddr, RuntimeError> {
        self.require(capability, operation)?;
        let address = address.parse::<SocketAddr>().map_err(|_| {
            RuntimeError::capability_denied(format!(
                "{operation} requires an explicit numeric IP:port address"
            ))
        })?;
        if allowed.contains(&address) {
            Ok(address)
        } else {
            Err(RuntimeError::capability_denied(format!(
                "{operation} address `{address}` was not granted"
            )))
        }
    }
}

fn canonical_directory(path: &Path, context: &str) -> Result<PathBuf, RuntimeError> {
    let canonical = path
        .canonicalize()
        .map_err(|error| RuntimeError::io_error(context, error.to_string()))?;
    if !canonical.is_dir() {
        return Err(RuntimeError::capability_denied(format!(
            "{context} `{}` is not a directory",
            path.display()
        )));
    }
    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_starts_without_ambient_authority() {
        let policy = HostPolicy::default();
        assert_eq!(policy.granted(), CapabilitySet::empty());
    }

    #[test]
    fn untrusted_policy_starts_without_ambient_authority() {
        let policy = HostPolicy::untrusted();
        assert_eq!(policy.granted(), CapabilitySet::empty());
    }

    #[test]
    fn standard_cli_authority_is_an_explicit_compatibility_policy() {
        let policy = HostPolicy::standard_cli();
        assert!(policy.granted().contains(CapabilitySet::CONSOLE_READ));
        assert!(policy.granted().contains(CapabilitySet::CONSOLE_WRITE));
        assert!(policy.granted().contains(CapabilitySet::CLOCK));
        assert!(policy.granted().contains(CapabilitySet::RANDOM));
        assert!(!policy.granted().contains(CapabilitySet::FILE_READ));
        assert!(!policy.granted().contains(CapabilitySet::NETWORK_CONNECT));
    }

    #[test]
    fn file_paths_must_remain_under_their_granted_root() {
        let allowed = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let input = allowed.path().join("input.txt");
        std::fs::write(&input, "ok").unwrap();
        let outside_input = outside.path().join("secret.txt");
        std::fs::write(&outside_input, "no").unwrap();
        let mut policy = HostPolicy::default();
        policy.allow_read_root(allowed.path()).unwrap();
        policy.allow_write_root(allowed.path()).unwrap();

        assert_eq!(
            policy.authorize_read(&input).unwrap(),
            input.canonicalize().unwrap()
        );
        assert!(policy.authorize_read(&outside_input).is_err());
        assert!(policy
            .authorize_write(allowed.path().join("new.txt"))
            .is_ok());
        assert!(policy
            .authorize_write(outside.path().join("new.txt"))
            .is_err());
    }

    #[test]
    fn network_grants_are_exact_and_do_not_enable_dns() {
        let allowed: SocketAddr = "127.0.0.1:443".parse().unwrap();
        let mut policy = HostPolicy::untrusted();
        policy.allow_connect_addr(allowed);

        assert_eq!(policy.authorize_connect("127.0.0.1:443").unwrap(), allowed);
        assert!(policy.authorize_connect("127.0.0.1:80").is_err());
        assert!(policy.authorize_connect("example.com:443").is_err());
        assert!(policy.authorize_listen("127.0.0.1:443").is_err());
    }

    #[test]
    fn program_preflight_reports_only_missing_authority() {
        let mut policy = HostPolicy::untrusted();
        policy.grant(CapabilitySet::FILE_READ | CapabilitySet::CLOCK);
        let error = policy
            .require_program(
                CapabilitySet::FILE_READ
                    | CapabilitySet::NETWORK_CONNECT
                    | CapabilitySet::NETWORK_LISTEN,
            )
            .unwrap_err();
        let message = error.to_string();
        assert!(
            message.contains("network.connect,network.listen"),
            "{message}"
        );
        assert!(
            !message.contains("program requests `file.read"),
            "{message}"
        );
    }
}
