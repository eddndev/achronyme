pub(crate) fn validate_groth16_metadata(
    value: &serde_json::Value,
    label: &str,
    accepted_curves: &[&str],
) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("{label} JSON root must be an object"))?;
    let protocol = object
        .get("protocol")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("{label} protocol must be a string"))?;
    if protocol != "groth16" {
        return Err(format!("{label} protocol must be groth16"));
    }
    let curve = object
        .get("curve")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("{label} curve must be a string"))?;
    if !accepted_curves.contains(&curve) {
        return Err(format!(
            "{label} curve `{curve}` does not match selected curve"
        ));
    }
    Ok(())
}

pub(crate) fn declared_public_inputs(value: &serde_json::Value) -> Result<usize, String> {
    let count = value
        .get("nPublic")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "vkey nPublic must be a non-negative integer".to_string())?;
    usize::try_from(count).map_err(|_| "vkey nPublic exceeds platform limits".to_string())
}
