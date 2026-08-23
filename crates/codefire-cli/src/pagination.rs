use super::CliError;
use serde_json::{json, Value};

pub(crate) const DEFAULT_PAGE_LIMIT: usize = 100;
pub(crate) const MAX_PAGE_LIMIT: usize = 1_000;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct PageRequest {
    pub(crate) limit: Option<usize>,
    pub(crate) cursor: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct Page {
    pub(crate) items: Vec<Value>,
    pub(crate) metadata: Value,
}

pub(crate) fn parse_page_limit(value: &str) -> Result<usize, CliError> {
    let limit = value
        .parse::<usize>()
        .map_err(|_| CliError::Usage(format!("invalid page limit: {value}")))?;
    if limit == 0 || limit > MAX_PAGE_LIMIT {
        return Err(CliError::Usage(format!(
            "page limit must be between 1 and {MAX_PAGE_LIMIT}"
        )));
    }
    Ok(limit)
}

pub(crate) fn paginate(
    collection: &str,
    items: Vec<Value>,
    request: &PageRequest,
) -> Result<Page, CliError> {
    let snapshot = format!(
        "sha256:{}",
        codefire_util::sha256_hex(&codefire_store::canonical_json(&Value::Array(
            items.clone()
        ))?)
    );
    let (offset, cursor_limit) = match request.cursor.as_deref() {
        Some(cursor) => decode_cursor(cursor, collection, &snapshot)?,
        None => (0, None),
    };
    let limit = request
        .limit
        .or(cursor_limit)
        .unwrap_or_else(|| items.len().max(1));
    if limit == 0 || limit > MAX_PAGE_LIMIT && request.limit.is_some() {
        return Err(CliError::Usage(format!(
            "page limit must be between 1 and {MAX_PAGE_LIMIT}"
        )));
    }
    if request.limit.is_some() && cursor_limit.is_some() && request.limit != cursor_limit {
        return Err(CliError::Usage(
            "page cursor limit does not match the requested limit".to_string(),
        ));
    }
    if offset > items.len() {
        return Err(CliError::Usage(
            "page cursor offset exceeds the collection snapshot".to_string(),
        ));
    }
    let end = offset.saturating_add(limit).min(items.len());
    let next_cursor = (end < items.len())
        .then(|| encode_cursor(collection, &snapshot, end, limit))
        .transpose()?;
    let page_items = items[offset..end].to_vec();
    Ok(Page {
        metadata: json!({
            "type": "codefire_page",
            "version": 1,
            "collection": collection,
            "snapshot": snapshot,
            "offset": offset,
            "limit": limit,
            "returned": page_items.len(),
            "total": items.len(),
            "next_cursor": next_cursor,
        }),
        items: page_items,
    })
}

fn encode_cursor(
    collection: &str,
    snapshot: &str,
    offset: usize,
    limit: usize,
) -> Result<String, CliError> {
    let value = json!({
        "type": "codefire_page_cursor",
        "version": 1,
        "collection": collection,
        "snapshot": snapshot,
        "offset": offset,
        "limit": limit,
    });
    Ok(format!(
        "cfcur1.{}",
        hex_encode(&codefire_store::canonical_json(&value)?)
    ))
}

fn decode_cursor(
    cursor: &str,
    collection: &str,
    snapshot: &str,
) -> Result<(usize, Option<usize>), CliError> {
    if cursor.len() > 4_096 {
        return Err(CliError::Usage(
            "page cursor exceeds 4096 bytes".to_string(),
        ));
    }
    let encoded = cursor
        .strip_prefix("cfcur1.")
        .ok_or_else(|| CliError::Usage("unsupported page cursor version".to_string()))?;
    let bytes = hex_decode(encoded)?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|_| CliError::Usage("page cursor is not valid JSON".to_string()))?;
    let valid = value.get("type").and_then(Value::as_str) == Some("codefire_page_cursor")
        && value.get("version").and_then(Value::as_u64) == Some(1)
        && value.get("collection").and_then(Value::as_str) == Some(collection);
    if !valid {
        return Err(CliError::Usage(
            "page cursor does not belong to this collection".to_string(),
        ));
    }
    if value.get("snapshot").and_then(Value::as_str) != Some(snapshot) {
        return Err(CliError::Usage(
            "page cursor snapshot is stale; restart pagination".to_string(),
        ));
    }
    let offset = value
        .get("offset")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| CliError::Usage("page cursor offset is invalid".to_string()))?;
    let limit = value
        .get("limit")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .filter(|value| *value > 0 && *value <= MAX_PAGE_LIMIT)
        .ok_or_else(|| CliError::Usage("page cursor limit is invalid".to_string()))?;
    Ok((offset, Some(limit)))
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn hex_decode(value: &str) -> Result<Vec<u8>, CliError> {
    if value.len() % 2 != 0 {
        return Err(CliError::Usage("page cursor hex is truncated".to_string()));
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = hex_nibble(pair[0])?;
            let low = hex_nibble(pair[1])?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn hex_nibble(byte: u8) -> Result<u8, CliError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(CliError::Usage(
            "page cursor contains non-canonical hex".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_replays_exact_snapshot_and_rejects_drift() {
        let first = paginate(
            "branches:fixture",
            vec![json!({"id": 1}), json!({"id": 2}), json!({"id": 3})],
            &PageRequest {
                limit: Some(2),
                cursor: None,
            },
        )
        .unwrap();
        let cursor = first.metadata["next_cursor"].as_str().unwrap();
        let second = paginate(
            "branches:fixture",
            vec![json!({"id": 1}), json!({"id": 2}), json!({"id": 3})],
            &PageRequest {
                limit: Some(2),
                cursor: Some(cursor.to_string()),
            },
        )
        .unwrap();
        assert_eq!(second.items, vec![json!({"id": 3})]);

        let error = paginate(
            "branches:fixture",
            vec![json!({"id": 1}), json!({"id": 4}), json!({"id": 3})],
            &PageRequest {
                limit: Some(2),
                cursor: Some(cursor.to_string()),
            },
        )
        .unwrap_err();
        assert!(error.to_string().contains("snapshot is stale"));
    }
}
