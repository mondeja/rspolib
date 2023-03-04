const CHARACTERS_TO_UNESCAPE: [char; 8] = [
    '"', '\\', '\t', '\r', '\n', '\u{8}',  // \b
    '\u{11}', // \v
    '\u{12}', // \f
];
const CHARACTERS_TO_UNESCAPE_NO_DOUBLE_QUOTES: [char; 7] = [
    '\\', '\t', '\r', '\n', '\u{8}',  // \b
    '\u{11}', // \v
    '\u{12}', // \f
];

/// Escape characters in a PO string field
pub fn escape(text: &str) -> String {
    text.replace('\\', r#"\\"#)
        .replace('\t', r"\t")
        .replace('\n', r#"\n"#)
        .replace('\r', r#"\r"#)
        .replace('\u{11}', r#"\v"#)
        .replace('\u{8}', r#"\b"#)
        .replace('\u{12}', r#"\f"#)
        .replace('"', r#"\""#)
}

fn unescape_characters(text: &str, characters: &[char]) -> String {
    let mut result: Vec<char> = Vec::new();

    let mut escaping = false;

    for character in text.chars() {
        if escaping {
            if !characters.contains(&character) {
                result.push('\\');
            }
            result.push(character);
            escaping = false;
        } else if character == '\\' {
            escaping = true;
        } else {
            result.push(character);
        }
    }
    result.iter().collect()
}

/// Unescape characters in a PO string field
pub fn unescape(text: &str) -> String {
    unescape_characters(text, &CHARACTERS_TO_UNESCAPE)
}

/// Unescape all characters except double quotes in a PO string field
pub fn unescape_except_double_quotes(text: &str) -> String {
    unescape_characters(
        text,
        &CHARACTERS_TO_UNESCAPE_NO_DOUBLE_QUOTES,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const ESCAPES_EXPECTED: (&str, &str) = (
        r#"foo \ \\ \t \r bar \n \v \b \f " baz"#,
        r#"foo \\ \\\\ \\t \\r bar \\n \\v \\b \\f \" baz"#,
    );

    #[test]
    fn test_escape() {
        let escapes_map: HashMap<String, &str> = HashMap::from([
            (r#"\"#.to_string(), r#"\\"#),
            (r#"\t"#.to_string(), r#"\\t"#),
            (r#"\r"#.to_string(), r#"\\r"#),
            ("\n".to_string(), "\\n"),
            (r"\n".to_string(), "\\\\n"),
            (r#"\v"#.to_string(), r#"\\v"#),
            (r#"\b"#.to_string(), r#"\\b"#),
            (r#"\f"#.to_string(), r#"\\f"#),
            (r#"""#.to_string(), r#"\""#),
        ]);

        for (value, expected) in escapes_map {
            assert_eq!(escape(&value), expected);
        }
    }

    #[test]
    fn test_escape_all() {
        let (escapes, expected) = ESCAPES_EXPECTED;
        assert_eq!(escape(escapes), expected);
    }

    #[test]
    fn test_unescape() {
        let escapes_map: HashMap<String, &str> = HashMap::from([
            (r#"\\"#.to_string(), r#"\"#),
            (r#"\\n"#.to_string(), r#"\n"#),
            (r#"\\t"#.to_string(), r#"\t"#),
            (r#"\\r"#.to_string(), r#"\r"#),
            (r#"\""#.to_string(), r#"""#),
            (r#"\\v"#.to_string(), r#"\v"#),
            (r#"\\b"#.to_string(), r#"\b"#),
            (r#"\\f"#.to_string(), r#"\f"#),
        ]);

        for (value, expected) in escapes_map {
            assert_eq!(unescape(&value), expected);
        }
    }

    #[test]
    fn test_unescape_all() {
        let (expected, escapes) = ESCAPES_EXPECTED;
        assert_eq!(unescape(escapes), expected);
    }
}
