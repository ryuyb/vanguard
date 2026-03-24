use serde_json::Value;

/// Converts camelCase to snake_case
pub fn camel_to_snake(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

/// Converts snake_case to camelCase
pub fn snake_to_camel(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

/// Recursively converts all keys in a JSON Value from camelCase to snake_case
pub fn convert_keys_to_snake_case(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let converted = map
                .into_iter()
                .map(|(k, v)| (camel_to_snake(&k), convert_keys_to_snake_case(v)))
                .collect();
            Value::Object(converted)
        }
        Value::Array(arr) => {
            let converted = arr.into_iter().map(convert_keys_to_snake_case).collect();
            Value::Array(converted)
        }
        other => other,
    }
}

/// Recursively converts all keys in a JSON Value from snake_case to camelCase
pub fn convert_keys_to_camel_case(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let converted = map
                .into_iter()
                .map(|(k, v)| (snake_to_camel(&k), convert_keys_to_camel_case(v)))
                .collect();
            Value::Object(converted)
        }
        Value::Array(arr) => {
            let converted = arr.into_iter().map(convert_keys_to_camel_case).collect();
            Value::Array(converted)
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_camel_to_snake() {
        assert_eq!(camel_to_snake("camelCase"), "camel_case");
        assert_eq!(camel_to_snake("simple"), "simple");
        assert_eq!(camel_to_snake("XMLHttpRequest"), "x_m_l_http_request");
        assert_eq!(camel_to_snake("camelCaseString"), "camel_case_string");
    }

    #[test]
    fn test_snake_to_camel() {
        assert_eq!(snake_to_camel("snake_case"), "snakeCase");
        assert_eq!(snake_to_camel("simple"), "simple");
        assert_eq!(snake_to_camel("snake_case_string"), "snakeCaseString");
    }

    #[test]
    fn test_convert_keys_to_snake_case() {
        let input = json!({
            "camelCase": "value",
            "nested": {
                "innerKey": "innerValue"
            },
            "array": [
                {"arrayKey": "arrayValue"}
            ]
        });

        let expected = json!({
            "camel_case": "value",
            "nested": {
                "inner_key": "innerValue"
            },
            "array": [
                {"array_key": "arrayValue"}
            ]
        });

        assert_eq!(convert_keys_to_snake_case(input), expected);
    }

    #[test]
    fn test_convert_keys_to_camel_case() {
        let input = json!({
            "snake_case": "value",
            "nested": {
                "inner_key": "innerValue"
            },
            "array": [
                {"array_key": "arrayValue"}
            ]
        });

        let expected = json!({
            "snakeCase": "value",
            "nested": {
                "innerKey": "innerValue"
            },
            "array": [
                {"arrayKey": "arrayValue"}
            ]
        });

        assert_eq!(convert_keys_to_camel_case(input), expected);
    }

    #[test]
    fn test_roundtrip_conversion() {
        let original = json!({
            "camelCase": "value",
            "nestedObject": {
                "innerKey": "innerValue"
            }
        });

        let snake = convert_keys_to_snake_case(original.clone());
        let camel = convert_keys_to_camel_case(snake);
        assert_eq!(original, camel);
    }
}
