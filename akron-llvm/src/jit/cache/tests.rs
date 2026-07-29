use std::fs::{self, File};

use sha2::Digest;
use tempfile::TempDir;

use super::{
    decode_entry, encode_entry, CacheIdentity, CacheMetadata, CachedObject, ObjectCache,
    ABSOLUTE_MAX_CACHE_BYTES, ABSOLUTE_MAX_ENTRY_BYTES,
};
use crate::jit::LlvmVersion;

fn identity() -> CacheIdentity {
    CacheIdentity {
        program: b"canonical-program".to_vec(),
        runtime_abi_version: 5,
        runtime_abi_size: 168,
        runtime_capabilities: 0x3fff,
        llvm_version: LlvmVersion {
            major: 21,
            minor: 1,
            patch: 8,
        },
        target_triple: "x86_64-pc-linux-gnu".to_string(),
        cpu: "rocketlake".to_string(),
        features: "+sse2,+avx2".to_string(),
        optimization_pipeline: "default<O2>".to_string(),
        lowering_revision: 1,
    }
}

fn cached(object: &[u8]) -> CachedObject {
    CachedObject {
        metadata: CacheMetadata {
            instruction_count: 11,
            native_instruction_count: 10,
            direct_instruction_count: 9,
            runtime_instruction_count: 1,
            compiled_call_count: 2,
        },
        object: object.to_vec(),
    }
}

#[test]
fn cache_key_is_deterministic_and_covers_every_codegen_input() {
    let base = identity();
    assert_eq!(base.key(), base.clone().key());

    let mut reordered = base.clone();
    reordered.features = "+avx2,+sse2".to_string();
    assert_eq!(base.key(), reordered.key());

    let mut changed = base.clone();
    changed.program.push(0);
    assert_ne!(base.key(), changed.key());

    let mut changed = base.clone();
    changed.runtime_abi_version += 1;
    assert_ne!(base.key(), changed.key());

    let mut changed = base.clone();
    changed.runtime_abi_size += 8;
    assert_ne!(base.key(), changed.key());

    let mut changed = base.clone();
    changed.runtime_capabilities ^= 1;
    assert_ne!(base.key(), changed.key());

    let mut changed = base.clone();
    changed.llvm_version.patch += 1;
    assert_ne!(base.key(), changed.key());

    let mut changed = base.clone();
    changed.target_triple.push_str("-changed");
    assert_ne!(base.key(), changed.key());

    let mut changed = base.clone();
    changed.cpu.push_str("-changed");
    assert_ne!(base.key(), changed.key());

    let mut changed = base.clone();
    changed.features.push_str(",+sha");
    assert_ne!(base.key(), changed.key());

    let mut changed = base.clone();
    changed.optimization_pipeline.push_str(",verify");
    assert_ne!(base.key(), changed.key());

    let mut changed = base;
    changed.lowering_revision += 1;
    assert_ne!(identity().key(), changed.key());
}

#[test]
fn corrupt_and_mismatched_entries_are_safe_misses() {
    let directory = TempDir::new().unwrap();
    let cache = ObjectCache::new(directory.path().to_path_buf(), 1024 * 1024);
    let first = identity().key();
    let artifact = cached(b"relocatable-object");
    cache.store(first, &artifact);
    assert_eq!(cache.lookup(first), Some(artifact.clone()));

    let mut bytes = fs::read(cache.entry_path(first)).unwrap();
    *bytes.last_mut().unwrap() ^= 0xff;
    fs::write(cache.entry_path(first), bytes).unwrap();
    assert_eq!(cache.lookup(first), None);

    cache.store(first, &artifact);
    let mut bytes = fs::read(cache.entry_path(first)).unwrap();
    bytes[76] ^= 0xff;
    fs::write(cache.entry_path(first), bytes).unwrap();
    assert_eq!(cache.lookup(first), None);

    cache.store(first, &artifact);
    let mut stale = identity();
    stale.runtime_abi_version += 1;
    let stale = stale.key();
    fs::copy(cache.entry_path(first), cache.entry_path(stale)).unwrap();
    assert_eq!(cache.lookup(stale), None);

    fs::write(cache.entry_path(stale), b"truncated").unwrap();
    assert_eq!(cache.lookup(stale), None);
}

#[test]
fn writes_are_atomic_and_storage_is_bounded() {
    let directory = TempDir::new().unwrap();
    let cache = ObjectCache::new(directory.path().to_path_buf(), 360);
    for marker in 0..4 {
        let mut value = identity();
        value.program.push(marker);
        cache.store(value.key(), &cached(&[marker; 80]));
    }

    let entries = fs::read_dir(directory.path())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert!(entries
        .iter()
        .all(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("akobj")));
    let total = entries
        .iter()
        .map(|entry| entry.metadata().unwrap().len())
        .sum::<u64>();
    assert!(total <= 360, "cache retained {total} bytes");
}

#[test]
fn disabled_cache_performs_no_io() {
    let cache = ObjectCache::disabled();
    let key = identity().key();
    cache.store(key, &cached(b"object"));
    assert_eq!(cache.lookup(key), None);
}

#[test]
fn configured_bound_is_capped_by_the_absolute_limit() {
    let directory = TempDir::new().unwrap();
    let cache = ObjectCache::new(directory.path().to_path_buf(), u64::MAX);

    assert_eq!(cache.max_bytes, ABSOLUTE_MAX_CACHE_BYTES);
}

#[test]
fn oversized_sparse_entry_is_removed_before_contents_are_read() {
    let directory = TempDir::new().unwrap();
    let cache = ObjectCache::new(directory.path().to_path_buf(), u64::MAX);
    let key = identity().key();
    let path = cache.entry_path(key);
    File::create(&path)
        .unwrap()
        .set_len(ABSOLUTE_MAX_ENTRY_BYTES + 1)
        .unwrap();

    assert_eq!(cache.lookup(key), None);
    assert!(!path.exists());
}

#[test]
fn zero_bound_disables_cache_without_creating_its_directory() {
    let root = TempDir::new().unwrap();
    let directory = root.path().join("disabled");
    let cache = ObjectCache::new(directory.clone(), 0);
    let key = identity().key();

    cache.store(key, &cached(b"object"));

    assert_eq!(cache.lookup(key), None);
    assert!(!directory.exists());
}

#[test]
fn overflowing_object_length_is_a_safe_decode_miss() {
    let key = identity().key();
    let mut bytes = encode_entry(key, &cached(b"object")).unwrap();
    bytes[116..124].copy_from_slice(&u64::MAX.to_le_bytes());
    let checksum: [u8; 32] = sha2::Sha256::digest(&bytes[76..]).into();
    bytes[44..76].copy_from_slice(&checksum);

    assert_eq!(decode_entry(&bytes, key), None);
}

#[cfg(unix)]
#[test]
fn permission_denied_entry_is_a_safe_miss_without_deletion() {
    use std::os::unix::fs::PermissionsExt;

    let directory = TempDir::new().unwrap();
    let cache = ObjectCache::new(directory.path().to_path_buf(), 1024 * 1024);
    let key = identity().key();
    let artifact = cached(b"object");
    cache.store(key, &artifact);
    let path = cache.entry_path(key);
    let original_permissions = fs::metadata(&path).unwrap().permissions();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
    let access_is_denied = File::open(&path).is_err();

    let result = cache.lookup(key);

    fs::set_permissions(&path, original_permissions).unwrap();
    if access_is_denied {
        assert_eq!(result, None);
        assert!(path.exists());
    } else {
        assert_eq!(result, Some(artifact));
    }
}
