const REDACTION: &str = "<redacted>";

pub fn redact_sensitive(input: &str) -> String {
    let mut output = String::from(input);

    for prefix in [
        "access_token=",
        "refresh_token=",
        "token=",
        "password=",
        "masterpasswordhash=",
        "master_password_hash=",
        "\"access_token\":\"",
        "\"refresh_token\":\"",
        "\"token\":\"",
        "\"password\":\"",
        "\"masterpasswordhash\":\"",
        "\"master_password_hash\":\"",
        "access_token: \"",
        "refresh_token: \"",
        "token: \"",
        "password: \"",
        "masterpasswordhash: \"",
        "master_password_hash: \"",
        "Bearer ",
        "bearer ",
    ] {
        output = redact_after_prefix(&output, prefix);
    }

    output
}

fn redact_after_prefix(input: &str, prefix: &str) -> String {
    if input.is_empty() || prefix.is_empty() {
        return String::from(input);
    }

    let mut result = String::with_capacity(input.len());
    let mut cursor = 0usize;
    let prefix_len = prefix.len();
    let prefix_lower = prefix.to_ascii_lowercase();

    while cursor < input.len() {
        let remaining = &input[cursor..];
        let remaining_lower = remaining.to_ascii_lowercase();
        let Some(found_at) = remaining_lower.find(&prefix_lower) else {
            result.push_str(remaining);
            break;
        };

        let abs_start = cursor + found_at;
        result.push_str(&input[cursor..abs_start]);
        result.push_str(&input[abs_start..abs_start + prefix_len]);

        let mut value_end = abs_start + prefix_len;
        while value_end < input.len() {
            let ch = input[value_end..].chars().next().unwrap_or_default();
            if is_value_delimiter(ch) {
                break;
            }
            value_end += ch.len_utf8();
        }

        result.push_str(REDACTION);
        cursor = value_end;
    }

    result
}

fn is_value_delimiter(ch: char) -> bool {
    matches!(
        ch,
        '&' | '?' | ' ' | '\t' | '\n' | '\r' | '"' | '\'' | ',' | ';' | ')' | ']' | '}'
    )
}

#[cfg(test)]
mod tests {
    use super::redact_sensitive;

    #[test]
    fn redacts_access_token_in_query() {
        let input = "wss://example/ws?access_token=abc123&v=1";
        let output = redact_sensitive(input);
        assert_eq!(output, "wss://example/ws?access_token=<redacted>&v=1");
    }

    #[test]
    fn redacts_password_like_fields() {
        let input = r#"payload={"password":"secret","master_password_hash":"hash"}"#;
        let output = redact_sensitive(input);
        assert_eq!(
            output,
            r#"payload={"password":"<redacted>","master_password_hash":"<redacted>"}"#
        );
    }

    #[test]
    fn redacts_bearer_token() {
        let input = "Authorization: Bearer abc.def.ghi";
        let output = redact_sensitive(input);
        assert_eq!(output, "Authorization: Bearer <redacted>");
    }
}
