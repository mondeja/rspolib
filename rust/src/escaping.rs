use crate::errors::EscapingError;

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

struct EscapedStringInterpreter<'a> {
    s: std::str::Chars<'a>,
}

impl<'a> Iterator for EscapedStringInterpreter<'a> {
    type Item = Result<char, EscapingError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.s.next().map(|c| match c {
            '\\' => match self.s.next() {
                None => Err(EscapingError::EscapeAtEndOfString {
                    text: self.s.as_str().to_string(),
                }),
                Some('"') => Ok('"'),
                Some('n') => Ok('\n'),
                Some('r') => Ok('\r'),
                Some('t') => Ok('\t'),
                Some('b') => Ok('\u{8}'),
                Some('v') => Ok('\u{11}'),
                Some('f') => Ok('\u{12}'),
                Some('\\') => Ok('\\'),
                Some(c) => {
                    Err(EscapingError::InvalidEscapedCharacter {
                        text: self.s.as_str().to_string(),
                        character: c,
                    })
                }
            },
            c => Ok(c),
        })
    }
}

pub fn unescape(s: &str) -> Result<String, EscapingError> {
    (EscapedStringInterpreter { s: s.chars() }).collect()
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
    fn test_unescape() -> Result<(), EscapingError> {
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
            assert_eq!(unescape(&value)?, expected);
        }

        Ok(())
    }

    #[test]
    fn test_unescape_all() -> Result<(), EscapingError> {
        let (expected, escapes) = ESCAPES_EXPECTED;
        assert_eq!(unescape(escapes)?, expected);

        Ok(())
    }
}
