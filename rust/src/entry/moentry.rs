use std::cmp::Ordering;
use std::fmt;

use crate::entry::{
    maybe_msgid_msgctxt_eot_split, mo_entry_to_string,
    EntryCmpByOptions, MsgidEotMsgctxt, POEntry, Translated,
};
use crate::traits::Merge;

/// MO file entry representing a message
///
/// Unlike PO files, MO files contain only the content
/// needed to translate a program at runtime, so this
/// is struct optimized as saves much more memory
/// than [POEntry].
///
/// MO entries ieally contain `msgstr` or the fields
/// `msgid_plural` and `msgstr_plural` as not being `None`.
/// The logic would be:
///
/// - If `msgstr` is not `None`, then the entry is a
///   translation of a singular form.
/// - If `msgid_plural` is not `None`, then the entry
///   is a translation of a plural form contained in
///   `msgstr_plural`.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct MOEntry {
    /// untranslated string
    pub msgid: String,
    /// translated string
    pub msgstr: Option<String>,
    /// untranslated string for plural form
    pub msgid_plural: Option<String>,
    /// translated strings for plural form
    pub msgstr_plural: Vec<String>,
    /// context
    pub msgctxt: Option<String>,
}

impl MOEntry {
    pub fn new(
        msgid: String,
        msgstr: Option<String>,
        msgid_plural: Option<String>,
        msgstr_plural: Vec<String>,
        msgctxt: Option<String>,
    ) -> MOEntry {
        MOEntry {
            msgid,
            msgstr,
            msgid_plural,
            msgstr_plural,
            msgctxt,
        }
    }

    /// Convert to a string representation with a given wrap width
    pub fn to_string_with_wrapwidth(
        &self,
        wrapwidth: usize,
    ) -> String {
        mo_entry_to_string(self, wrapwidth, "")
    }

    /// Compare the current entry with other entry
    ///
    /// You can disable some comparison options by setting the corresponding
    /// field in `options` to `false`. See [EntryCmpByOptions].
    pub fn cmp_by(
        &self,
        other: &Self,
        options: &EntryCmpByOptions,
    ) -> Ordering {
        let placeholder = &"\0".to_string();

        if options.by_msgctxt {
            let msgctxt = self
                .msgctxt
                .as_ref()
                .unwrap_or(placeholder)
                .to_string();
            let other_msgctxt = other
                .msgctxt
                .as_ref()
                .unwrap_or(placeholder)
                .to_string();
            if msgctxt > other_msgctxt {
                return Ordering::Greater;
            } else if msgctxt < other_msgctxt {
                return Ordering::Less;
            }
        }

        if options.by_msgid_plural {
            let msgid_plural = self
                .msgid_plural
                .as_ref()
                .unwrap_or(placeholder)
                .to_string();
            let other_msgid_plural = other
                .msgid_plural
                .as_ref()
                .unwrap_or(placeholder)
                .to_string();
            if msgid_plural > other_msgid_plural {
                return Ordering::Greater;
            } else if msgid_plural < other_msgid_plural {
                return Ordering::Less;
            }
        }

        if options.by_msgstr_plural {
            let mut msgstr_plural = self.msgstr_plural.clone();
            msgstr_plural.sort();
            let mut other_msgstr_plural = other.msgstr_plural.clone();
            other_msgstr_plural.sort();
            if msgstr_plural > other_msgstr_plural {
                return Ordering::Greater;
            } else if msgstr_plural < other_msgstr_plural {
                return Ordering::Less;
            }
        }

        if options.by_msgid {
            if self.msgid > other.msgid {
                return Ordering::Greater;
            } else if self.msgid < other.msgid {
                return Ordering::Less;
            }
        }

        if options.by_msgstr {
            let msgstr = self
                .msgstr
                .as_ref()
                .unwrap_or(placeholder)
                .to_string();
            let other_msgstr = other
                .msgstr
                .as_ref()
                .unwrap_or(placeholder)
                .to_string();
            if msgstr > other_msgstr {
                return Ordering::Greater;
            } else if msgstr < other_msgstr {
                return Ordering::Less;
            }
        }

        Ordering::Equal
    }
}

impl MsgidEotMsgctxt for MOEntry {
    fn msgid_eot_msgctxt(&self) -> String {
        maybe_msgid_msgctxt_eot_split(&self.msgid, &self.msgctxt)
            .to_string()
    }
}

impl Translated for MOEntry {
    /// Returns `true` if the entry is translated
    ///
    /// Really, MO files has only translated entries,
    /// but this function is here to be consistent
    /// with the PO implementation and to be used
    /// when manipulating MOEntry directly.
    fn translated(&self) -> bool {
        if let Some(msgstr) = &self.msgstr {
            return !msgstr.is_empty();
        }

        if self.msgstr_plural.is_empty() {
            return false;
        } else {
            for msgstr_plural in &self.msgstr_plural {
                if !msgstr_plural.is_empty() {
                    return true;
                }
            }
        }

        false
    }
}

impl Merge for MOEntry {
    fn merge(&mut self, other: Self) {
        self.msgid = other.msgid;
        self.msgstr = other.msgstr;
        self.msgid_plural = other.msgid_plural;
        self.msgstr_plural = other.msgstr_plural;
        self.msgctxt = other.msgctxt;
    }
}

impl fmt::Display for MOEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_with_wrapwidth(78))
    }
}

impl From<&str> for MOEntry {
    /// Generates a [MOEntry] from a string as the `msgid`
    fn from(s: &str) -> Self {
        MOEntry::new(s.to_string(), None, None, vec![], None)
    }
}

impl From<&POEntry> for MOEntry {
    /// Generates a [MOEntry] from a [POEntry]
    ///
    /// Keep in mind that this conversion loss the information
    /// that is contained in [POEntry]s but not in [MOEntry]s.
    fn from(entry: &POEntry) -> Self {
        MOEntry {
            msgid: entry.msgid.clone(),
            msgstr: entry.msgstr.clone(),
            msgid_plural: entry.msgid_plural.clone(),
            msgstr_plural: entry.msgstr_plural.clone(),
            msgctxt: entry.msgctxt.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor() {
        let moentry = MOEntry::new(
            "msgid".to_string(),
            Some("msgstr".to_string()),
            None,
            vec![],
            None,
        );

        assert_eq!(moentry.msgid, "msgid");
        assert_eq!(moentry.msgstr, Some("msgstr".to_string()));
        assert_eq!(moentry.msgid_plural, None);
        assert_eq!(moentry.msgstr_plural, vec![] as Vec<String>);
        assert_eq!(moentry.msgctxt, None);
    }

    #[test]
    fn moentry_translated() {
        // empty msgstr means untranslated
        let moentry = MOEntry::new(
            "msgid".to_string(),
            Some("".to_string()),
            None,
            vec![],
            None,
        );
        assert_eq!(moentry.translated(), false);

        let moentry = MOEntry::new(
            "msgid".to_string(),
            Some("msgstr".to_string()),
            None,
            vec![],
            None,
        );
        assert_eq!(moentry.translated(), true);

        // empty msgstr_plural means untranslated
        let moentry = MOEntry::new(
            "msgid".to_string(),
            None,
            None,
            vec![],
            None,
        );
        assert_eq!(moentry.translated(), false);

        // empty msgstr in msgstr_plural means untranslated
        let moentry = MOEntry::new(
            "msgid".to_string(),
            None,
            None,
            vec!["".to_string()],
            None,
        );
        assert_eq!(moentry.translated(), false);
    }

    #[test]
    fn moentry_merge() {
        let mut moentry = MOEntry::new(
            "msgid".to_string(),
            Some("msgstr".to_string()),
            Some("msgid_plural".to_string()),
            vec!["msgstr_plural".to_string()],
            Some("msgctxt".to_string()),
        );
        let other = MOEntry::new(
            "other_msgid".to_string(),
            Some("other_msgstr".to_string()),
            Some("other_msgid_plural".to_string()),
            vec!["other_msgstr_plural".to_string()],
            Some("other_msgctxt".to_string()),
        );

        moentry.merge(other);

        assert_eq!(moentry.msgid, "other_msgid");
        assert_eq!(moentry.msgstr, Some("other_msgstr".to_string()));
        assert_eq!(
            moentry.msgid_plural,
            Some("other_msgid_plural".to_string())
        );
        assert_eq!(
            moentry.msgstr_plural,
            vec!["other_msgstr_plural".to_string()],
        );
        assert_eq!(
            moentry.msgctxt,
            Some("other_msgctxt".to_string())
        );
    }

    #[test]
    fn moentry_to_string() {
        // with msgid_plural
        let moentry = MOEntry::new(
            "msgid".to_string(),
            Some("msgstr".to_string()),
            Some("msgid_plural".to_string()),
            vec!["msgstr_plural".to_string()],
            Some("msgctxt".to_string()),
        );

        let expected = r#"msgctxt "msgctxt"
msgid "msgid"
msgid_plural "msgid_plural"
msgstr[0] "msgstr_plural"
"#
        .to_string();

        assert_eq!(moentry.to_string(), expected);

        // with msgstr
        let moentry = MOEntry::new(
            "msgid".to_string(),
            Some("msgstr".to_string()),
            None,
            vec![],
            Some("msgctxt".to_string()),
        );

        let expected = r#"msgctxt "msgctxt"
msgid "msgid"
msgstr "msgstr"
"#
        .to_string();

        assert_eq!(moentry.to_string(), expected);
    }

    #[test]
    fn moentry_from_poentry() {
        let msgstr_plural = vec!["msgstr_plural".to_string()];

        let mut poentry = POEntry::new(0);
        poentry.msgid = "msgid".to_string();
        poentry.msgstr = Some("msgstr".to_string());
        poentry.msgid_plural = Some("msgid_plural".to_string());
        poentry.msgstr_plural = msgstr_plural.clone();
        poentry.msgctxt = Some("msgctxt".to_string());

        let moentry = MOEntry::from(&poentry);

        assert_eq!(moentry.msgid, "msgid");
        assert_eq!(moentry.msgstr, Some("msgstr".to_string()));
        assert_eq!(
            moentry.msgid_plural,
            Some("msgid_plural".to_string())
        );
        assert_eq!(moentry.msgstr_plural, msgstr_plural);
        assert_eq!(moentry.msgctxt, Some("msgctxt".to_string()));
    }
}
