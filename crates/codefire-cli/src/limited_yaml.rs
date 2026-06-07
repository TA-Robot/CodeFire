use crate::CliError;

pub(crate) fn strip_comment(line: &str) -> String {
    let mut quote = None;
    let mut previous_escape = false;
    for (index, byte) in line.bytes().enumerate() {
        match (byte, quote, previous_escape) {
            (b'\\', Some(b'"'), false) => {
                previous_escape = true;
                continue;
            }
            (b'"' | b'\'', None, _) => quote = Some(byte),
            (b'"' | b'\'', Some(current), false) if current == byte => quote = None,
            (b'#', None, _) => return line[..index].to_string(),
            _ => {}
        }
        previous_escape = false;
    }
    line.to_string()
}

pub(crate) fn parse_key_value(
    line: &str,
    line_number: usize,
    context: &str,
) -> Result<(String, String), CliError> {
    let colon = line.find(':').ok_or_else(|| {
        CliError::Usage(format!(
            "invalid {context}: expected key: value at line {line_number}"
        ))
    })?;
    let key = line[..colon].trim();
    if key.is_empty() {
        return Err(CliError::Usage(format!(
            "invalid {context}: empty key at line {line_number}"
        )));
    }
    Ok((key.to_string(), parse_scalar(&line[colon + 1..])))
}

pub(crate) fn parse_scalar(raw: &str) -> String {
    let value = raw.trim();
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        let quote = bytes[0];
        if (quote == b'"' || quote == b'\'') && bytes[value.len() - 1] == quote {
            return value[1..value.len() - 1]
                .replace("\\\"", "\"")
                .replace("\\\\", "\\");
        }
    }
    value.to_string()
}

pub(crate) fn parse_u64(
    value: &str,
    field: &str,
    line_number: usize,
    context: &str,
) -> Result<u64, CliError> {
    value.parse::<u64>().map_err(|_| {
        CliError::Usage(format!(
            "invalid {context}: {field} must be an integer at line {line_number}"
        ))
    })
}

pub(crate) fn parse_usize(
    value: &str,
    field: &str,
    line_number: usize,
    context: &str,
) -> Result<usize, CliError> {
    value.parse::<usize>().map_err(|_| {
        CliError::Usage(format!(
            "invalid {context}: {field} must be an integer at line {line_number}"
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_comment_removes_only_unquoted_hashes() {
        assert_eq!(strip_comment("version: 1 # trailing"), "version: 1 ");
        assert_eq!(
            strip_comment("reason: \"keep # hash\" # drop"),
            "reason: \"keep # hash\" "
        );
        assert_eq!(
            strip_comment("reason: 'keep # hash' # drop"),
            "reason: 'keep # hash' "
        );
    }

    #[test]
    fn strip_comment_preserves_escaped_quotes_inside_double_quotes() {
        assert_eq!(
            strip_comment("reason: \"quoted \\\"#\\\" value\" # drop"),
            "reason: \"quoted \\\"#\\\" value\" "
        );
    }

    #[test]
    fn parse_key_value_trims_key_and_unquotes_value() {
        assert_eq!(
            parse_key_value(" label : \"hello \\\"world\\\"\" ", 7, "test YAML").unwrap(),
            ("label".to_string(), "hello \"world\"".to_string())
        );
    }

    #[test]
    fn parse_key_value_rejects_missing_colon_with_context() {
        let error = parse_key_value("label", 3, "test YAML")
            .expect_err("missing colon should be invalid")
            .to_string();
        assert_eq!(error, "invalid test YAML: expected key: value at line 3");
    }

    #[test]
    fn integer_parsers_use_contextual_errors() {
        assert_eq!(parse_u64("42", "version", 2, "test YAML").unwrap(), 42);
        assert_eq!(parse_usize("9", "size", 4, "test YAML").unwrap(), 9);
        assert_eq!(
            parse_u64("x", "version", 2, "test YAML")
                .expect_err("invalid integer should fail")
                .to_string(),
            "invalid test YAML: version must be an integer at line 2"
        );
    }
}
