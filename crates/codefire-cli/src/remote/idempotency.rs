use serde_json::Value;

pub(crate) fn remote_idempotency_key(request: &Value) -> Option<String> {
    request
        .get("idempotency_key")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}
