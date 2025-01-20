use crate::errors::EscapingError;
use std::borrow::Cow;

/// Escape characters in a PO string field
pub fn escape(text: &str) -> Cow<'_, str> {
    let mut ret: String = String::with_capacity(text.len());
    for char in text.chars() {
        match char {
            '"' => ret.push_str(r#"\""#),
            '\n' => ret.push_str(r#"\n"#),
            '\r' => ret.push_str(r#"\r"#),
            '\t' => ret.push_str(r#"\t"#),
            '\u{11}' => ret.push_str(r#"\v"#),
            '\u{8}' => ret.push_str(r#"\b"#),
            '\u{12}' => ret.push_str(r#"\f"#),
            '\\' => ret.push_str(r#"\\"#),
            c => ret.push(c),
        }
    }
    ret.into()
}

struct EscapedStringInterpreter<'a> {
    characters: std::str::Chars<'a>,
}

#[allow(clippy::needless_lifetimes)]
impl<'a> Iterator for EscapedStringInterpreter<'a> {
    type Item = Result<char, EscapingError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.characters.next().map(|c| match c {
            '\\' => match self.characters.next() {
                None => Err(EscapingError::EscapeAtEndOfString {
                    text: self.characters.as_str().to_string(),
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
                        text: self.characters.as_str().to_string(),
                        character: c,
                    })
                }
            },
            c => Ok(c),
        })
    }
}

/// Unescape characters in a PO string field
pub fn unescape(text: &str) -> Result<Cow<'_, str>, EscapingError> {
    if text.contains('\\') {
        (EscapedStringInterpreter {
            characters: text.chars(),
        })
        .collect()
    } else {
        Ok(text.into())
    }
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
