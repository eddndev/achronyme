use std::env;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::UNIX_EPOCH;

use sha2::{Digest, Sha256};

use super::LlvmVersion;

const CACHE_MAGIC: [u8; 8] = *b"AKJITC\0\0";
const CACHE_FORMAT_VERSION: u32 = 2;
const METADATA_FIELDS: usize = 5;
const HEADER_SIZE: usize = 8 + 4 + 32 + 32 + (METADATA_FIELDS * 8) + 8;
const DEFAULT_MAX_BYTES: u64 = 128 * 1024 * 1024;
const ABSOLUTE_MAX_ENTRY_BYTES: u64 = 64 * 1024 * 1024;
const ABSOLUTE_MAX_CACHE_BYTES: u64 = 1024 * 1024 * 1024;
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JitCacheConfig {
    directory: Option<PathBuf>,
    max_bytes: u64,
}

impl JitCacheConfig {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: Some(directory.into()),
            max_bytes: DEFAULT_MAX_BYTES,
        }
    }

    pub fn disabled() -> Self {
        Self {
            directory: None,
            max_bytes: 0,
        }
    }

    pub fn with_max_bytes(mut self, max_bytes: u64) -> Self {
        self.max_bytes = max_bytes;
        self
    }

    pub(super) fn from_environment() -> Self {
        if env::var_os("AKRON_JIT_CACHE")
            .and_then(|value| value.into_string().ok())
            .is_some_and(|value| matches!(value.as_str(), "0" | "false" | "off"))
        {
            return Self::disabled();
        }
        let directory = env::var_os("AKRON_JIT_CACHE_DIR")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .or_else(default_directory);
        let Some(directory) = directory else {
            return Self::disabled();
        };
        let max_bytes = env::var("AKRON_JIT_CACHE_MAX_BYTES")
            .ok()
            .and_then(|value| value.parse().ok())
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_MAX_BYTES);
        Self::new(directory).with_max_bytes(max_bytes)
    }
}

fn default_directory() -> Option<PathBuf> {
    env::var_os("XDG_CACHE_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("HOME")
                .filter(|value| !value.is_empty())
                .map(PathBuf::from)
                .map(|path| path.join(".cache"))
        })
        .map(|path| path.join("akron").join("jit"))
}

#[derive(Clone)]
pub(super) struct CacheIdentity {
    pub(super) program: Vec<u8>,
    pub(super) runtime_abi_version: u32,
    pub(super) runtime_abi_size: u32,
    pub(super) runtime_capabilities: u64,
    pub(super) llvm_version: LlvmVersion,
    pub(super) target_triple: String,
    pub(super) cpu: String,
    pub(super) features: String,
    pub(super) optimization_pipeline: String,
    pub(super) lowering_revision: u32,
}

impl CacheIdentity {
    pub(super) fn key(&self) -> CacheKey {
        let mut hash = Sha256::new();
        hash.update(b"Akron JIT object cache key v1\0");
        hash_field(&mut hash, &self.program);
        hash_field(&mut hash, &self.runtime_abi_version.to_le_bytes());
        hash_field(&mut hash, &self.runtime_abi_size.to_le_bytes());
        hash_field(&mut hash, &self.runtime_capabilities.to_le_bytes());
        hash_field(&mut hash, &self.llvm_version.major.to_le_bytes());
        hash_field(&mut hash, &self.llvm_version.minor.to_le_bytes());
        hash_field(&mut hash, &self.llvm_version.patch.to_le_bytes());
        hash_field(&mut hash, self.target_triple.as_bytes());
        hash_field(&mut hash, self.cpu.as_bytes());
        hash_field(&mut hash, canonical_features(&self.features).as_bytes());
        hash_field(&mut hash, self.optimization_pipeline.as_bytes());
        hash_field(&mut hash, &self.lowering_revision.to_le_bytes());
        CacheKey(hash.finalize().into())
    }
}

fn canonical_features(features: &str) -> String {
    let mut values = features
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    values.sort_unstable();
    values.dedup();
    values.join(",")
}

fn hash_field(hash: &mut Sha256, bytes: &[u8]) {
    hash.update((bytes.len() as u64).to_le_bytes());
    hash.update(bytes);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CacheKey([u8; 32]);

impl CacheKey {
    fn filename(self) -> OsString {
        let mut name = String::with_capacity(64 + ".akobj".len());
        for byte in self.0 {
            use std::fmt::Write as _;
            write!(name, "{byte:02x}").expect("writing to String cannot fail");
        }
        name.push_str(".akobj");
        name.into()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct CacheMetadata {
    pub instruction_count: usize,
    pub native_instruction_count: usize,
    pub direct_instruction_count: usize,
    pub runtime_instruction_count: usize,
    pub compiled_call_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CachedObject {
    pub metadata: CacheMetadata,
    pub object: Vec<u8>,
}

pub(super) struct ObjectCache {
    directory: Option<PathBuf>,
    max_bytes: u64,
}

impl ObjectCache {
    pub(super) fn new(directory: PathBuf, max_bytes: u64) -> Self {
        Self {
            directory: Some(directory),
            max_bytes: max_bytes.min(ABSOLUTE_MAX_CACHE_BYTES),
        }
    }

    pub(super) fn disabled() -> Self {
        Self {
            directory: None,
            max_bytes: 0,
        }
    }

    pub(super) fn from_config(config: &JitCacheConfig) -> Self {
        match (&config.directory, config.max_bytes) {
            (Some(directory), max_bytes) if max_bytes > 0 => {
                Self::new(directory.clone(), max_bytes)
            }
            _ => Self::disabled(),
        }
    }

    pub(super) fn is_enabled(&self) -> bool {
        self.directory.is_some() && self.max_bytes > 0
    }

    pub(super) fn lookup(&self, key: CacheKey) -> Option<CachedObject> {
        if !self.is_enabled() {
            return None;
        }
        let path = self.entry_path(key);
        let file = File::open(&path).ok()?;
        let metadata = file.metadata().ok()?;
        let max_entry_bytes = self.max_bytes.min(ABSOLUTE_MAX_ENTRY_BYTES);
        if !metadata.is_file()
            || metadata.len() < HEADER_SIZE as u64
            || metadata.len() > max_entry_bytes
        {
            let _ = fs::remove_file(path);
            return None;
        }

        let expected_len = usize::try_from(metadata.len()).ok()?;
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(expected_len).ok()?;
        let read_limit = max_entry_bytes.checked_add(1)?;
        if file.take(read_limit).read_to_end(&mut bytes).is_err() {
            return None;
        }
        let actual_len = u64::try_from(bytes.len()).ok()?;
        if bytes.len() != expected_len || actual_len > max_entry_bytes {
            let _ = fs::remove_file(path);
            return None;
        }

        let object = decode_entry(&bytes, key);
        if object.is_none() {
            let _ = fs::remove_file(path);
        }
        object
    }

    pub(super) fn store(&self, key: CacheKey, object: &CachedObject) {
        if !self.is_enabled() {
            return;
        }
        let Some(directory) = &self.directory else {
            return;
        };
        let Some(entry_len) = HEADER_SIZE.checked_add(object.object.len()) else {
            return;
        };
        let Ok(entry_len) = u64::try_from(entry_len) else {
            return;
        };
        if entry_len > self.max_bytes || entry_len > ABSOLUTE_MAX_ENTRY_BYTES {
            return;
        }
        let Some(entry) = encode_entry(key, object) else {
            return;
        };
        if fs::create_dir_all(directory).is_err() {
            return;
        }
        let final_path = self.entry_path(key);
        let temporary = directory.join(format!(
            ".{}.{}.tmp",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        if write_entry(&temporary, &entry).is_err() {
            let _ = fs::remove_file(temporary);
            return;
        }
        if fs::rename(&temporary, &final_path).is_err() {
            let _ = fs::remove_file(temporary);
            return;
        }
        if let Ok(directory_file) = File::open(directory) {
            let _ = directory_file.sync_all();
        }
        self.enforce_bound(&final_path);
    }

    pub(super) fn invalidate(&self, key: CacheKey) {
        if self.directory.is_none() {
            return;
        }
        let _ = fs::remove_file(self.entry_path(key));
    }

    fn entry_path(&self, key: CacheKey) -> PathBuf {
        self.directory
            .as_deref()
            .unwrap_or_else(|| Path::new(""))
            .join(key.filename())
    }

    fn enforce_bound(&self, protected: &Path) {
        let Some(directory) = &self.directory else {
            return;
        };
        let Ok(read_dir) = fs::read_dir(directory) else {
            return;
        };
        let mut entries = read_dir
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let path = entry.path();
                if path.extension().and_then(|value| value.to_str()) != Some("akobj") {
                    return None;
                }
                let metadata = entry.metadata().ok()?;
                let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
                Some((modified, path, metadata.len()))
            })
            .collect::<Vec<_>>();
        let mut total = entries
            .iter()
            .fold(0u64, |total, entry| total.saturating_add(entry.2));
        entries.sort_by(|left, right| (left.0, &left.1).cmp(&(right.0, &right.1)));
        for (_, path, size) in entries {
            if total <= self.max_bytes {
                break;
            }
            if path == protected {
                continue;
            }
            if fs::remove_file(path).is_ok() {
                total = total.saturating_sub(size);
            }
        }
    }
}

fn write_entry(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

fn encode_entry(key: CacheKey, cached: &CachedObject) -> Option<Vec<u8>> {
    let output_len = HEADER_SIZE.checked_add(cached.object.len())?;
    let mut output = Vec::new();
    output.try_reserve_exact(output_len).ok()?;
    output.extend_from_slice(&CACHE_MAGIC);
    output.extend_from_slice(&CACHE_FORMAT_VERSION.to_le_bytes());
    output.extend_from_slice(&key.0);
    output.extend_from_slice(&[0; 32]);
    for value in [
        cached.metadata.instruction_count,
        cached.metadata.native_instruction_count,
        cached.metadata.direct_instruction_count,
        cached.metadata.runtime_instruction_count,
        cached.metadata.compiled_call_count,
    ] {
        output.extend_from_slice(&u64::try_from(value).ok()?.to_le_bytes());
    }
    output.extend_from_slice(&u64::try_from(cached.object.len()).ok()?.to_le_bytes());
    output.extend_from_slice(&cached.object);
    let checksum: [u8; 32] = Sha256::digest(&output[76..]).into();
    output[44..76].copy_from_slice(&checksum);
    Some(output)
}

fn decode_entry(bytes: &[u8], expected: CacheKey) -> Option<CachedObject> {
    if bytes.len() < HEADER_SIZE || bytes[..8] != CACHE_MAGIC {
        return None;
    }
    let version = u32::from_le_bytes(bytes[8..12].try_into().ok()?);
    if version != CACHE_FORMAT_VERSION || bytes[12..44] != expected.0 {
        return None;
    }
    let checksum: [u8; 32] = bytes[44..76].try_into().ok()?;
    let actual: [u8; 32] = Sha256::digest(&bytes[76..]).into();
    if actual != checksum {
        return None;
    }
    let mut offset = 76usize;
    let mut next_usize = || {
        let end = offset.checked_add(8)?;
        let value = u64::from_le_bytes(bytes.get(offset..end)?.try_into().ok()?);
        offset = end;
        usize::try_from(value).ok()
    };
    let metadata = CacheMetadata {
        instruction_count: next_usize()?,
        native_instruction_count: next_usize()?,
        direct_instruction_count: next_usize()?,
        runtime_instruction_count: next_usize()?,
        compiled_call_count: next_usize()?,
    };
    let object_len = next_usize()?;
    let end = offset.checked_add(object_len)?;
    if end != bytes.len() {
        return None;
    }
    let object = bytes.get(offset..end)?.to_vec();
    Some(CachedObject { metadata, object })
}

#[cfg(test)]
mod tests;
