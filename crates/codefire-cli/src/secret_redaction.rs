#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Redacted<T> {
    pub(crate) value: T,
    pub(crate) count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NextValue {
    None,
    Secret,
    Authorization,
}

pub(crate) fn redact_text(input: &str) -> Redacted<String> {
    let mut output = String::with_capacity(input.len());
    let mut count = 0usize;
    let mut next = NextValue::None;
    for segment in input.split_inclusive(char::is_whitespace) {
        let token_len = segment.trim_end_matches(char::is_whitespace).len();
        let (token, whitespace) = segment.split_at(token_len);
        let redacted = redact_token(token, &mut next);
        count += redacted.count;
        output.push_str(&redacted.value);
        output.push_str(whitespace);
    }
    if input.is_empty() {
        return Redacted {
            value: String::new(),
            count: 0,
        };
    }
    Redacted {
        value: output,
        count,
    }
}

pub(crate) fn redact_argv(argv: &[String]) -> Redacted<Vec<String>> {
    let mut output = Vec::with_capacity(argv.len());
    let mut count = 0usize;
    let mut next = NextValue::None;
    for token in argv {
        let redacted = redact_token(token, &mut next);
        count += redacted.count;
        output.push(redacted.value);
    }
    Redacted {
        value: output,
        count,
    }
}

pub(crate) fn redact_literals(input: &str, literals: &[String]) -> Redacted<String> {
    let mut output = input.to_string();
    let mut count = 0usize;
    let mut literals = literals
        .iter()
        .filter(|literal| literal.len() >= 3)
        .cloned()
        .collect::<Vec<_>>();
    literals.sort_by_key(|literal| std::cmp::Reverse(literal.len()));
    literals.dedup();
    for literal in literals {
        let matches = output.matches(&literal).count();
        if matches > 0 {
            output = output.replace(&literal, "<redacted-path>");
            count += matches;
        }
    }
    Redacted {
        value: output,
        count,
    }
}

fn redact_token(token: &str, next: &mut NextValue) -> Redacted<String> {
    if token.is_empty() {
        return Redacted {
            value: String::new(),
            count: 0,
        };
    }
    match *next {
        NextValue::Secret => {
            *next = NextValue::None;
            return Redacted {
                value: "<redacted>".to_string(),
                count: 1,
            };
        }
        NextValue::Authorization if token.eq_ignore_ascii_case("bearer") => {
            *next = NextValue::Secret;
            return Redacted {
                value: token.to_string(),
                count: 0,
            };
        }
        NextValue::Authorization => {
            *next = NextValue::None;
            return Redacted {
                value: "<redacted>".to_string(),
                count: 1,
            };
        }
        NextValue::None => {}
    }

    if let Some((index, separator)) = assignment_separator(token) {
        let key = &token[..index];
        if is_sensitive_key(key) {
            let value_start = index + separator.len_utf8();
            let value = &token[value_start..];
            if value.is_empty() {
                *next = if normalized_key(key) == "authorization" {
                    NextValue::Authorization
                } else {
                    NextValue::Secret
                };
                return Redacted {
                    value: token.to_string(),
                    count: 0,
                };
            }
            return Redacted {
                value: format!("{}{}<redacted>", &token[..index], separator),
                count: 1,
            };
        }
    }

    if is_sensitive_key(token) {
        *next = if normalized_key(token) == "authorization" {
            NextValue::Authorization
        } else {
            NextValue::Secret
        };
    }
    redact_embedded_credentials(token)
}

fn redact_embedded_credentials(token: &str) -> Redacted<String> {
    let mut output = token.to_string();
    let mut count = 0usize;
    if let Some(scheme_end) = output.find("://") {
        let authority_start = scheme_end + 3;
        let authority_end = output[authority_start..]
            .find(['/', '?', '#'])
            .map(|offset| authority_start + offset)
            .unwrap_or(output.len());
        if let Some(at_offset) = output[authority_start..authority_end].rfind('@') {
            let at = authority_start + at_offset;
            output.replace_range(authority_start..at, "<redacted>");
            count += 1;
        }
    }

    let mut cursor = 0usize;
    while let Some(relative) = output[cursor..].find('=') {
        let separator = cursor + relative;
        let key_start = output[..separator]
            .rfind(['?', '&', ';', ',', '#'])
            .map_or(0, |index| index + 1);
        if !is_sensitive_key(&output[key_start..separator]) {
            cursor = separator + 1;
            continue;
        }
        let value_start = separator + 1;
        let value_end = output[value_start..]
            .find(['&', ';', ',', '#'])
            .map(|offset| value_start + offset)
            .unwrap_or(output.len());
        output.replace_range(value_start..value_end, "<redacted>");
        count += 1;
        cursor = value_start + "<redacted>".len();
    }
    Redacted {
        value: output,
        count,
    }
}

fn assignment_separator(token: &str) -> Option<(usize, char)> {
    token
        .char_indices()
        .find(|(_, character)| matches!(character, '=' | ':'))
}

fn is_sensitive_key(key: &str) -> bool {
    let key = normalized_key(key);
    matches!(
        key.as_str(),
        "token"
            | "api_token"
            | "access_token"
            | "secret"
            | "client_secret"
            | "password"
            | "passwd"
            | "api_key"
            | "apikey"
            | "authorization"
            | "auth"
            | "credential"
            | "credentials"
            | "signature"
            | "sig"
            | "private_key"
            | "access_key"
    ) || key.ends_with("_token")
        || key.ends_with("_secret")
        || key.ends_with("_password")
        || key.ends_with("_key")
        || key.ends_with("_credential")
        || key.ends_with("_signature")
        || key.ends_with("_auth")
}

fn normalized_key(key: &str) -> String {
    key.trim_matches(['-', '"', '\'', '{', '[', '('])
        .trim_end_matches(['"', '\'', '}', ']', ')'])
        .replace('-', "_")
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_assignment_option_and_bearer_values() {
        let redacted = redact_text(
            "API_TOKEN=alpha --password bravo Authorization: Bearer charlie safe=value\n",
        );
        assert_eq!(
            redacted.value,
            "API_TOKEN=<redacted> --password <redacted> Authorization: Bearer <redacted> safe=value\n"
        );
        assert_eq!(redacted.count, 3);
    }

    #[test]
    fn redacts_argv_without_changing_non_secret_arguments() {
        let argv = vec![
            "tool".to_string(),
            "--token".to_string(),
            "alpha".to_string(),
            "--mode=safe".to_string(),
        ];
        let redacted = redact_argv(&argv);
        assert_eq!(
            redacted.value,
            vec!["tool", "--token", "<redacted>", "--mode=safe"]
        );
        assert_eq!(redacted.count, 1);
    }

    #[test]
    fn redacts_sensitive_path_literals_longest_first() {
        let redacted = redact_literals(
            "/home/alice/project /home/alice token",
            &["/home/alice".to_string(), "/home/alice/project".to_string()],
        );
        assert_eq!(redacted.value, "<redacted-path> <redacted-path> token");
        assert_eq!(redacted.count, 2);
    }

    #[test]
    fn redacts_uri_userinfo_and_query_credentials() {
        let redacted = redact_text(
            "https://alice:pass@example.test/path?mode=safe&token=alpha&api_key=bravo&X-Amz-Signature=charlie",
        );
        assert_eq!(
            redacted.value,
            "https://<redacted>@example.test/path?mode=safe&token=<redacted>&api_key=<redacted>&X-Amz-Signature=<redacted>"
        );
        assert_eq!(redacted.count, 4);
    }
}
