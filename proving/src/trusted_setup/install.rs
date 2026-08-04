use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use super::artifact::{MANIFEST_FILE, TRANSCRIPT_FILE, ZKEY_FILE};
use super::support::open_regular_file;

pub(super) fn install_artifact(
    store: &Path,
    digest: &str,
    zkey: &Path,
    manifest: &[u8],
    transcript: &[u8],
) -> Result<(), String> {
    validate_store(store)?;
    let destination = store.join(digest);
    if destination.exists() {
        return Err(format!(
            "trusted-key artifact `{}` already exists",
            destination.display()
        ));
    }
    std::fs::create_dir_all(store)
        .map_err(|error| format!("cannot create trusted-key store: {error}"))?;
    let staging = store.join(format!(".{digest}.partial-{}", std::process::id()));
    std::fs::create_dir(&staging)
        .map_err(|error| format!("cannot create trusted-key staging directory: {error}"))?;
    let result = (|| {
        write_new(&staging.join(MANIFEST_FILE), manifest)?;
        write_new(&staging.join(TRANSCRIPT_FILE), transcript)?;
        copy_new(zkey, &staging.join(ZKEY_FILE))?;
        std::fs::rename(&staging, &destination)
            .map_err(|error| format!("cannot install trusted-key artifact: {error}"))
    })();
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&staging);
    }
    result
}

fn validate_store(store: &Path) -> Result<(), String> {
    if !store.exists() {
        return Ok(());
    }
    let metadata = std::fs::symlink_metadata(store)
        .map_err(|error| format!("cannot inspect trusted-key store: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err("trusted-key store must be a directory, not a symlink".to_string());
    }
    Ok(())
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("cannot create `{}`: {error}", path.display()))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| format!("cannot write `{}`: {error}", path.display()))
}

fn copy_new(source: &Path, destination: &Path) -> Result<(), String> {
    let mut input = open_regular_file(source, "final proving key")?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(|error| format!("cannot create `{}`: {error}", destination.display()))?;
    std::io::copy(&mut input, &mut output)
        .and_then(|_| output.sync_all())
        .map_err(|error| format!("cannot copy final proving key: {error}"))
}

pub(super) fn json_bytes(value: &impl serde::Serialize) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("cannot serialize trusted-key metadata: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}
