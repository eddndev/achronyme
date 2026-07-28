use std::fs;

use tempfile::TempDir;

use super::{CacheIdentity, CacheMetadata, CachedObject, ObjectCache};
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
