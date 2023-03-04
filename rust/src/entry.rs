use std::collections::HashMap;
use std::fmt;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::escaping::{escape, unescape_except_double_quotes};
use crate::traits::Merge;
use crate::twrapper::wrap;

pub trait Translated {
    fn translated(&self) -> bool;
}

pub trait MsgidEotMsgctxt {
    fn msgid_eot_msgctxt(&self) -> String;
}

// From the MO files spec:
//
// Contexts are stored by storing the concatenation of the
// context, a EOT byte, and the original string
fn maybe_msgid_msgctxt_eot_split(
    msgid: &str,
    msgctxt: &Option<String>,
) -> String {
    if let Some(ctx) = msgctxt {
        let mut ret = String::from(ctx);
        ret.push('\u{4}');
        ret.push_str(msgid);
        ret
    } else {
        msgid.to_string()
    }
}

fn metadata_msgstr_formatter(
    msgstr: &str,
    _: &str,
    _: usize,
) -> String {
    let mut ret = String::from("msgstr \"\"\n");
    for line in msgstr.lines() {
        ret.push('"');
        ret.push_str(line);
        ret.push_str(r"\n");
        ret.push('"');
        ret.push('\n');
    }
    ret
}

fn default_mo_entry_msgstr_formatter(
    msgstr: &str,
    delflag: &str,
    wrapwidth: usize,
) -> String {
    POStringField::new(
        "msgstr",
        delflag,
        msgstr.trim_end(),
        "",
        wrapwidth,
    )
    .to_string()
}

fn mo_entry_to_string_with_msgstr_formatter(
    entry: &MOEntry,
    wrapwidth: usize,
    delflag: &str,
    msgstr_formatter: &dyn Fn(&str, &str, usize) -> String,
) -> String {
    let mut ret = String::new();

    if let Some(msgctxt) = &entry.msgctxt {
        ret.push_str(
            &POStringField::new(
                "msgctxt", delflag, msgctxt, "", wrapwidth,
            )
            .to_string(),
        );
    }

    ret.push_str(
        &POStringField::new(
            "msgid",
            delflag,
            &entry.msgid,
            "",
            wrapwidth,
        )
        .to_string(),
    );

    if let Some(msgid_plural) = &entry.msgid_plural {
        ret.push_str(
            &POStringField::new(
                "msgid_plural",
                delflag,
                msgid_plural,
                "",
                wrapwidth,
            )
            .to_string(),
        );
    }

    if let Some(msgstr_plural) = &entry.msgstr_plural {
        let mut indexes =
            msgstr_plural.keys().collect::<Vec<&String>>();
        indexes.sort();

        for index in indexes.iter() {
            let msgstr = match msgstr_plural.get(*index) {
                Some(msgstr) => msgstr,
                None => "",
            };
            ret.push_str(
                &POStringField::new(
                    "msgstr", delflag, msgstr, index, wrapwidth,
                )
                .to_string(),
            );
        }
    } else {
        let msgstr = match &entry.msgstr {
            Some(msgstr) => msgstr,
            None => "",
        };
        let formatted_msgstr =
            msgstr_formatter(msgstr, delflag, wrapwidth);
        ret.push_str(&formatted_msgstr);
    }

    ret
}

fn mo_entry_to_string(
    entry: &MOEntry,
    wrapwidth: usize,
    delflag: &str,
) -> String {
    mo_entry_to_string_with_msgstr_formatter(
        entry,
        wrapwidth,
        delflag,
        &default_mo_entry_msgstr_formatter,
    )
}

/// Converts a metadata wrapped by a [MOEntry] to a string
/// representation.
///
/// ```rust
/// use rspolib::{
///     mofile,
///     mo_metadata_entry_to_string,
/// };
///
/// let file = mofile("tests-data/all.mo").unwrap();
/// let entry = file.metadata_as_entry();
/// let entry_str = mo_metadata_entry_to_string(&entry);
///
/// assert!(entry_str.starts_with("msgid \"\"\nmsgstr \"\""));
/// ```
pub fn mo_metadata_entry_to_string(entry: &MOEntry) -> String {
    mo_entry_to_string_with_msgstr_formatter(
        entry,
        78,
        "",
        &metadata_msgstr_formatter,
    )
}

pub(crate) struct POStringField<'a> {
    fieldname: &'a str,
    delflag: &'a str,
    value: &'a str,
    plural_index: &'a str,
    wrapwidth: usize,
}

impl<'a> POStringField<'a> {
    pub fn new(
        fieldname: &'a str,
        delflag: &'a str,
        value: &'a str,
        plural_index: &'a str,
        wrapwidth: usize,
    ) -> Self {
        Self {
            fieldname,
            delflag,
            value,
            plural_index,
            wrapwidth,
        }
    }
}

impl<'a> fmt::Display for POStringField<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut lines = vec!["".to_string()];
        let escaped_value = escape(self.value);

        let repr_plural_index = match self.plural_index.is_empty() {
            false => format!("[{}]", self.plural_index),
            true => "".to_string(),
        };

        // +1 here because of the space between fieldname and value
        let real_width =
            UnicodeWidthStr::width(escaped_value.as_str())
                + UnicodeWidthStr::width(self.fieldname)
                + 1;
        if real_width > self.wrapwidth {
            let new_lines = wrap(&escaped_value, self.wrapwidth);
            lines.extend(new_lines);
        } else {
            lines = vec![escaped_value];
        }

        // format first line
        let mut ret = format!(
            "{}{}{} \"{}\"\n",
            self.delflag,
            self.fieldname,
            repr_plural_index,
            unescape_except_double_quotes(&lines.remove(0)),
        );

        // format other lines
        for line in lines {
            ret.push_str(&format!(
                "{}\"{}\"\n",
                self.delflag,
                unescape_except_double_quotes(&line)
            ));
        }

        write!(f, "{}", ret)
    }
}

/// MO file entry representing a message
///
/// Unlike PO files, MO files contain only the content
/// needed to translate a program at runtime, so this
/// is struct optimized as saves much more memory
/// than [POEntry].
///
/// MO entries ieally contain `msgstr` or the fields
/// `msgid_plural` and `msgstr_plural as not being `None`.
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
    pub msgstr_plural: Option<HashMap<String, String>>,
    /// context
    pub msgctxt: Option<String>,
}

impl MOEntry {
    pub fn new(
        msgid: String,
        msgstr: Option<String>,
        msgid_plural: Option<String>,
        msgstr_plural: Option<HashMap<String, String>>,
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

    pub fn to_string_with_wrapwidth(
        &self,
        wrapwidth: usize,
    ) -> String {
        mo_entry_to_string(self, wrapwidth, "")
    }
}

impl MsgidEotMsgctxt for MOEntry {
    /// Returns msgid + EOT + msgctxt
    ///
    /// Function required to generate MO files as
    /// msgid + EOT + msgctxt is used as the key
    /// for the hash table.
    fn msgid_eot_msgctxt(&self) -> String {
        maybe_msgid_msgctxt_eot_split(&self.msgid, &self.msgctxt)
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

        if let Some(msgstr_plural) = &self.msgstr_plural {
            if msgstr_plural.is_empty() {
                return false;
            }
            for msgstr in msgstr_plural.values() {
                if msgstr.is_empty() {
                    return false;
                }
            }
            return true;
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
        MOEntry::new(s.to_string(), None, None, None, None)
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
            msgstr_plural: match entry.msgstr_plural.is_empty() {
                true => None,
                false => Some(entry.msgstr_plural.clone()),
            },
            msgctxt: entry.msgctxt.clone(),
        }
    }
}

/// PO file entry representing a message
///
/// This struct contains all the information that is stored
/// in PO files.
///
/// PO entries can contain `msgstr` or the fields
/// `msgid_plural` and `msgstr_plural as not being `None`.
/// The logic would be:
///
/// - If `msgstr` is not `None`, then the entry is a
///   translation of a singular form.
/// - If `msgid_plural` is not `None`, then the entry
///   is a translation of a plural form contained in
///   `msgstr_plural`.
///
/// The `previous_msgid` and `previous_msgid_plural` fields
/// are used to store the previous msgid/msgid_plural values
/// when the entry is obsolete.
///
/// The `previous_msgctxt` field is used to store the previous
/// msgctxt value when the entry is obsolete.
#[derive(Default, Clone, PartialEq, Debug)]
pub struct POEntry {
    /// untranslated string
    pub msgid: String,
    /// translated string
    pub msgstr: Option<String>,
    /// untranslated string for plural form
    pub msgid_plural: Option<String>,
    /// translated strings for plural form
    pub msgstr_plural: HashMap<String, String>,
    /// context
    pub msgctxt: Option<String>,
    /// the entry is marked as obsolete
    pub obsolete: bool,
    /// generated comments for machines
    pub comment: Option<String>,
    /// generated comments for translators
    pub tcomment: Option<String>,
    /// files and lines from which the translations are taken
    pub occurrences: Vec<(String, String)>,
    /// flags indicating the state, i.e. fuzzy
    pub flags: Vec<String>,
    /// previous untranslated string
    pub previous_msgid: Option<String>,
    /// previous untranslated string for plural form
    pub previous_msgid_plural: Option<String>,
    /// previous context
    pub previous_msgctxt: Option<String>,
    /// line number in the file or content
    pub linenum: usize,
}

impl POEntry {
    /// Creates a new POEntry
    ///
    /// It just creates the entry with a given line number.
    /// This function is used by the parser to initialize new
    /// entries. Use the `From` traits instead to initialize
    /// [POEntry]s from strings.
    pub fn new(linenum: usize) -> Self {
        Self {
            msgid: String::new(),
            linenum,

            ..Default::default()
        }
    }

    /// Returns `true` the entry has the `fuzzy` flag
    pub fn fuzzy(&self) -> bool {
        self.flags.contains(&"fuzzy".to_string())
    }

    fn format_comment_inplace(
        &self,
        comment: &str,
        prefix: &str,
        wrapwidth: usize,
        target: &mut String,
    ) {
        for line in comment.lines() {
            if line.graphemes(true).count() + 2 > wrapwidth {
                target.push_str(&wrap(line, wrapwidth - 2).join("\n"))
            } else {
                target.push_str(prefix);
                target.push_str(line);
            }
            target.push('\n');
        }
    }

    /// Convert to string with a given wrap width
    pub fn to_string_with_wrapwidth(
        &self,
        wrapwidth: usize,
    ) -> String {
        let mut ret = String::new();

        // translator comments
        if let Some(tcomment) = &self.tcomment {
            self.format_comment_inplace(
                tcomment, "#. ", wrapwidth, &mut ret,
            );
        }

        // comments
        if let Some(comment) = &self.comment {
            self.format_comment_inplace(
                comment, "# ", wrapwidth, &mut ret,
            );
        }

        // occurrences
        if !self.obsolete && !self.occurrences.is_empty() {
            let files_repr = self
                .occurrences
                .iter()
                .map(|(fpath, lineno)| {
                    if lineno.is_empty() {
                        return fpath.clone();
                    }
                    format!("{}:{}", fpath, lineno)
                })
                .collect::<Vec<String>>()
                .join(" ");
            if files_repr.graphemes(true).count() + 3 > wrapwidth {
                let wrapped = wrap(&files_repr, wrapwidth - 3)
                    .iter()
                    .map(|s| format!("#: {}", s))
                    .collect::<Vec<String>>()
                    .join("\n");
                ret.push_str(&wrapped);
            } else {
                ret.push_str("#: ");
                ret.push_str(&files_repr);
            }
            ret.push('\n');
        }

        // flags
        if !self.flags.is_empty() {
            ret.push_str(&format!("#, {}\n", self.flags.join(", ")));
        }

        // previous context and previous msgid/msgid_plural
        let mut prefix = String::from("#");
        if self.obsolete {
            prefix.push('~');
        }
        prefix.push_str("| ");

        if let Some(previous_msgctxt) = &self.previous_msgctxt {
            ret.push_str(
                &POStringField::new(
                    "msgctxt",
                    &prefix,
                    previous_msgctxt,
                    "",
                    wrapwidth,
                )
                .to_string(),
            );
        }

        if let Some(previous_msgid) = &self.previous_msgid {
            ret.push_str(
                &POStringField::new(
                    "msgid",
                    &prefix,
                    previous_msgid,
                    "",
                    wrapwidth,
                )
                .to_string(),
            );
        }

        if let Some(previous_msgid_plural) =
            &self.previous_msgid_plural
        {
            ret.push_str(
                &POStringField::new(
                    "msgid",
                    &prefix,
                    previous_msgid_plural,
                    "",
                    wrapwidth,
                )
                .to_string(),
            );
            ret.push('\n');
        }

        ret.push_str(&mo_entry_to_string(
            &MOEntry::from(self),
            wrapwidth,
            match self.obsolete {
                true => "#~ ",
                false => "",
            },
        ));
        ret
    }
}

impl MsgidEotMsgctxt for POEntry {
    fn msgid_eot_msgctxt(&self) -> String {
        maybe_msgid_msgctxt_eot_split(&self.msgid, &self.msgctxt)
    }
}

impl Translated for POEntry {
    fn translated(&self) -> bool {
        if self.obsolete || self.fuzzy() {
            return false;
        }

        if let Some(msgstr) = &self.msgstr {
            return !msgstr.is_empty();
        }

        if self.msgstr_plural.is_empty() {
            return false;
        }
        for msgstr in self.msgstr_plural.values() {
            if msgstr.is_empty() {
                return false;
            }
        }

        true
    }
}

impl Merge for POEntry {
    fn merge(&mut self, other: Self) {
        self.msgid = other.msgid;
        self.msgstr = other.msgstr;
        self.msgid_plural = other.msgid_plural;
        self.msgstr_plural = other.msgstr_plural;
        self.msgctxt = other.msgctxt;
        self.obsolete = other.obsolete;
        self.comment = other.comment;
        self.tcomment = other.tcomment;
        self.occurrences = other.occurrences;
        self.flags = other.flags;
        self.previous_msgctxt = other.previous_msgctxt;
        self.previous_msgid = other.previous_msgid;
        self.previous_msgid_plural = other.previous_msgid_plural;
        self.linenum = other.linenum;
    }
}

impl fmt::Display for POEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_with_wrapwidth(78))
    }
}

impl From<&str> for POEntry {
    fn from(s: &str) -> Self {
        let mut entry = POEntry::new(0);
        entry.msgid = s.to_string();
        entry
    }
}

impl From<usize> for POEntry {
    fn from(linenum: usize) -> Self {
        Self::new(linenum)
    }
}

impl From<(&str, &str)> for POEntry {
    fn from((msgid, msgstr): (&str, &str)) -> Self {
        let mut entry = POEntry::new(0);
        entry.msgid = msgid.to_string();
        entry.msgstr = Some(msgstr.to_string());
        entry
    }
}

impl From<&MOEntry> for POEntry {
    fn from(mo_entry: &MOEntry) -> Self {
        let mut entry = POEntry::new(0);
        entry.msgid = mo_entry.msgid.clone();
        entry.msgstr = mo_entry.msgstr.as_ref().cloned();
        entry.msgid_plural = mo_entry.msgid_plural.as_ref().cloned();
        entry.msgstr_plural = match mo_entry.msgstr_plural {
            Some(ref plural) => plural.clone(),
            None => HashMap::new(),
        };
        entry.msgctxt = mo_entry.msgctxt.as_ref().cloned();
        entry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::pofile::pofile;

    #[test]
    fn moentry_constructor() {
        let moentry = MOEntry::new(
            "msgid".to_string(),
            Some("msgstr".to_string()),
            None,
            None,
            None,
        );

        assert_eq!(moentry.msgid, "msgid");
        assert_eq!(moentry.msgstr, Some("msgstr".to_string()));
        assert_eq!(moentry.msgid_plural, None);
        assert_eq!(moentry.msgstr_plural, None);
        assert_eq!(moentry.msgctxt, None);
    }

    #[test]
    fn moentry_translated() {
        // empty msgstr means untranslated
        let moentry = MOEntry::new(
            "msgid".to_string(),
            Some("".to_string()),
            None,
            None,
            None,
        );
        assert_eq!(moentry.translated(), false);

        let moentry = MOEntry::new(
            "msgid".to_string(),
            Some("msgstr".to_string()),
            None,
            None,
            None,
        );
        assert_eq!(moentry.translated(), true);

        // empty msgstr_plural means untranslated
        let moentry = MOEntry::new(
            "msgid".to_string(),
            None,
            None,
            Some(HashMap::new()),
            None,
        );
        assert_eq!(moentry.translated(), false);

        // empty msgstr in msgstr_plural means untranslated
        let moentry = MOEntry::new(
            "msgid".to_string(),
            None,
            None,
            Some(HashMap::from([("0".to_string(), "".to_string())])),
            None,
        );
        assert_eq!(moentry.translated(), false);

        let moentry = MOEntry::new(
            "msgid".to_string(),
            None,
            None,
            Some(
                // doesn't matter if has an invalid index
                HashMap::from([(
                    "4".to_string(),
                    "msgstr_plural".to_string(),
                )]),
            ),
            None,
        );
        assert_eq!(moentry.translated(), true);
    }

    #[test]
    fn moentry_merge() {
        let mut moentry = MOEntry::new(
            "msgid".to_string(),
            Some("msgstr".to_string()),
            Some("msgid_plural".to_string()),
            Some(HashMap::from([(
                "0".to_string(),
                "msgstr_plural".to_string(),
            )])),
            Some("msgctxt".to_string()),
        );
        let other = MOEntry::new(
            "other_msgid".to_string(),
            Some("other_msgstr".to_string()),
            Some("other_msgid_plural".to_string()),
            Some(HashMap::from([(
                "4".to_string(),
                "other_msgstr_plural".to_string(),
            )])),
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
            Some(HashMap::from([(
                "4".to_string(),
                "other_msgstr_plural".to_string()
            )]))
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
            Some(HashMap::from([(
                "0".to_string(),
                "msgstr_plural".to_string(),
            )])),
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
            None,
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
        let msgstr_plural = HashMap::from([(
            "0".to_string(),
            "msgstr_plural".to_string(),
        )]);

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
        assert_eq!(moentry.msgstr_plural, Some(msgstr_plural));
        assert_eq!(moentry.msgctxt, Some("msgctxt".to_string()));
    }

    #[test]
    fn poentry_constructor() {
        let poentry = POEntry::new(7);

        assert_eq!(poentry.linenum, 7);
        assert_eq!(poentry.msgid, "");
        assert_eq!(poentry.msgstr, None);
        assert_eq!(poentry.msgid_plural, None);
        assert_eq!(poentry.msgstr_plural, HashMap::new());
        assert_eq!(poentry.msgctxt, None);
    }

    #[test]
    fn poentry_fuzzy() {
        let non_fuzzy_entry = POEntry::new(0);
        assert_eq!(non_fuzzy_entry.fuzzy(), false);

        let mut fuzzy_entry = POEntry::new(0);
        fuzzy_entry.flags.push("fuzzy".to_string());
        assert_eq!(fuzzy_entry.fuzzy(), true);
    }

    #[test]
    fn poentry_translated() {
        // obsolete means untranslated
        let mut obsolete_entry = POEntry::new(0);
        obsolete_entry.obsolete = true;
        assert_eq!(obsolete_entry.translated(), false);

        // fuzzy means untranslated
        let mut fuzzy_entry = POEntry::new(0);
        fuzzy_entry.flags.push("fuzzy".to_string());
        assert_eq!(fuzzy_entry.translated(), false);

        // no msgstr means untranslated
        let no_msgstr_entry = POEntry::new(0);
        assert_eq!(no_msgstr_entry.translated(), false);

        // empty msgstr means untranslated
        let mut empty_msgstr_entry = POEntry::new(0);
        empty_msgstr_entry.msgstr = Some("".to_string());
        assert_eq!(empty_msgstr_entry.translated(), false);

        // with msgstr means translated
        let mut translated_entry = POEntry::new(0);
        translated_entry.msgstr = Some("msgstr".to_string());
        assert_eq!(translated_entry.translated(), true);

        // empty msgstr_plural means untranslated
        let mut empty_msgstr_plural_entry = POEntry::new(0);
        empty_msgstr_plural_entry.msgstr_plural = HashMap::new();
        assert_eq!(empty_msgstr_plural_entry.translated(), false);

        // with empty msgstr_plural means untranslated
        let mut empty_msgstr_plural_entry = POEntry::new(0);
        empty_msgstr_plural_entry.msgstr_plural =
            HashMap::from([("0".to_string(), "".to_string())]);
        assert_eq!(empty_msgstr_plural_entry.translated(), false);

        // with msgstr_plural means translated
        let mut translated_plural_entry = POEntry::new(0);
        translated_plural_entry.msgstr_plural = HashMap::from([(
            "0".to_string(),
            "msgstr_plural".to_string(),
        )]);
        assert_eq!(translated_plural_entry.translated(), true);
    }

    #[test]
    fn poentry_merge() {
        let mut poentry = POEntry::new(0);
        poentry.msgid = "msgid".to_string();
        poentry.msgstr = Some("msgstr".to_string());
        poentry.msgid_plural = Some("msgid_plural".to_string());
        poentry.msgstr_plural = HashMap::from([(
            "0".to_string(),
            "msgstr_plural".to_string(),
        )]);

        let mut other = POEntry::new(0);
        other.msgid = "other_msgid".to_string();
        other.msgstr = Some("other_msgstr".to_string());
        other.msgid_plural = Some("other_msgid_plural".to_string());
        other.msgstr_plural = HashMap::from([(
            "0".to_string(),
            "other_msgstr_plural".to_string(),
        )]);

        poentry.merge(other);

        assert_eq!(poentry.msgid, "other_msgid");
        assert_eq!(poentry.msgstr, Some("other_msgstr".to_string()));
        assert_eq!(
            poentry.msgid_plural,
            Some("other_msgid_plural".to_string())
        );
        assert_eq!(
            poentry.msgstr_plural,
            HashMap::from([(
                "0".to_string(),
                "other_msgstr_plural".to_string()
            )])
        );
    }

    #[test]
    fn poentry_to_string() {
        let mut entry = POEntry::new(0);

        // empty
        let expected = "msgid \"\"\nmsgstr \"\"\n".to_string();
        assert_eq!(entry.to_string(), expected);

        // msgid
        entry.msgid = "msgid".to_string();
        let expected = "msgid \"msgid\"\nmsgstr \"\"\n".to_string();
        assert_eq!(entry.to_string(), expected);

        // msgstr
        entry.msgstr = Some("msgstr".to_string());
        let expected =
            concat!("msgid \"msgid\"\n", "msgstr \"msgstr\"\n");
        assert_eq!(entry.to_string(), expected);

        // msgid_plural
        entry.msgid_plural = Some("msgid_plural".to_string());
        let expected = concat!(
            "msgid \"msgid\"\n",
            "msgid_plural \"msgid_plural\"\n",
            "msgstr \"msgstr\"\n",
        );
        assert_eq!(entry.to_string(), expected);

        // msgid_plural (no msgstr)
        entry.msgstr = None;
        let expected = concat!(
            "msgid \"msgid\"\n",
            "msgid_plural \"msgid_plural\"\n",
            "msgstr \"\"\n",
        );
        assert_eq!(entry.to_string(), expected);

        // msgstr_plural
        entry.msgstr_plural = HashMap::from([
            ("1".to_string(), "plural 2".to_string()),
            ("0".to_string(), "plural 1".to_string()),
        ]);
        let expected = concat!(
            "msgid \"msgid\"\nmsgid_plural \"msgid_plural\"\n",
            "msgstr[0] \"plural 1\"\nmsgstr[1] \"plural 2\"\n",
        );
        assert_eq!(entry.to_string(), expected);

        // all indexes are allowed
        entry.msgstr_plural = HashMap::from([
            ("5".to_string(), "plural 2".to_string()),
            ("3".to_string(), "plural 1".to_string()),
        ]);

        // msgctxt
        entry.msgctxt = Some("msgctxt".to_string());
        let expected = concat!(
            "msgctxt \"msgctxt\"\nmsgid \"msgid\"\n",
            "msgid_plural \"msgid_plural\"\n",
            "msgstr[3] \"plural 1\"\n",
            "msgstr[5] \"plural 2\"\n"
        );
        assert_eq!(entry.to_string(), expected);

        // flags
        entry.flags.push("fuzzy".to_string());
        let expected = concat!(
            "#, fuzzy\n",
            "msgctxt \"msgctxt\"\nmsgid \"msgid\"\n",
            "msgid_plural \"msgid_plural\"\n",
            "msgstr[3] \"plural 1\"\n",
            "msgstr[5] \"plural 2\"\n"
        );
        assert_eq!(entry.to_string(), expected);

        entry.flags.push("python-format".to_string());
        let expected = concat!(
            "#, fuzzy, python-format\nmsgctxt \"msgctxt\"\n",
            "msgid \"msgid\"\nmsgid_plural \"msgid_plural\"\n",
            "msgstr[3] \"plural 1\"\nmsgstr[5] \"plural 2\"\n"
        );
        assert_eq!(entry.to_string(), expected);

        // comments
        entry.comment = Some("comment".to_string());
        let expected = concat!(
            "# comment\n#, fuzzy, python-format\n",
            "msgctxt \"msgctxt\"\nmsgid \"msgid\"\n",
            "msgid_plural \"msgid_plural\"\n",
            "msgstr[3] \"plural 1\"\nmsgstr[5] \"plural 2\"\n"
        );
        assert_eq!(entry.to_string(), expected);

        entry.tcomment = Some("extracted_comment".to_string());
        let expected = concat!(
            "#. extracted_comment\n# comment\n",
            "#, fuzzy, python-format\nmsgctxt \"msgctxt\"\n",
            "msgid \"msgid\"\nmsgid_plural \"msgid_plural\"\n",
            "msgstr[3] \"plural 1\"\nmsgstr[5] \"plural 2\"\n"
        );
        assert_eq!(entry.to_string(), expected);

        // obsolete
        entry.obsolete = true;
        let expected = concat!(
            "#. extracted_comment\n# comment\n",
            "#, fuzzy, python-format\n#~ msgctxt \"msgctxt\"\n",
            "#~ msgid \"msgid\"\n",
            "#~ msgid_plural \"msgid_plural\"\n",
            "#~ msgstr[3] \"plural 1\"\n",
            "#~ msgstr[5] \"plural 2\"\n"
        );
        assert_eq!(entry.to_string(), expected);

        // occurrences
        //
        // when obsolete, occurrences are not included
        entry
            .occurrences
            .push(("file1.rs".to_string(), "1".to_string()));
        entry
            .occurrences
            .push(("file2.rs".to_string(), "2".to_string()));
        let expected = concat!(
            "#. extracted_comment\n# comment\n",
            "#, fuzzy, python-format\n",
            "#~ msgctxt \"msgctxt\"\n",
            "#~ msgid \"msgid\"\n",
            "#~ msgid_plural \"msgid_plural\"\n",
            "#~ msgstr[3] \"plural 1\"\n",
            "#~ msgstr[5] \"plural 2\"\n"
        );
        assert_eq!(entry.to_string(), expected);

        entry.obsolete = false;
        let expected = concat!(
            "#. extracted_comment\n# comment\n",
            "#: file1.rs:1 file2.rs:2\n",
            "#, fuzzy, python-format\n",
            "msgctxt \"msgctxt\"\nmsgid \"msgid\"\n",
            "msgid_plural \"msgid_plural\"\n",
            "msgstr[3] \"plural 1\"\n",
            "msgstr[5] \"plural 2\"\n"
        );
        assert_eq!(entry.to_string(), expected);

        // Basic complete example
        entry.msgstr = Some("msgstr".to_string());
        entry.comment = Some("comment".to_string());
        entry.tcomment = Some("extracted_comment".to_string());
        entry.flags.push("rspolib".to_string());
        let expected = concat!(
            "#. extracted_comment\n# comment\n",
            "#: file1.rs:1 file2.rs:2\n",
            "#, fuzzy, python-format, rspolib\n",
            "msgctxt \"msgctxt\"\nmsgid \"msgid\"\n",
            "msgid_plural \"msgid_plural\"\n",
            "msgstr[3] \"plural 1\"\n",
            "msgstr[5] \"plural 2\"\n"
        );
        assert_eq!(entry.to_string(), expected);

        // previous msgctxt
        entry.previous_msgctxt =
            Some("A previous msgctxt".to_string());
        let expected = concat!(
            "#. extracted_comment\n# comment\n",
            "#: file1.rs:1 file2.rs:2\n",
            "#, fuzzy, python-format, rspolib\n",
            "#| msgctxt \"A previous msgctxt\"\n",
            "msgctxt \"msgctxt\"\n",
            "msgid \"msgid\"\n",
            "msgid_plural \"msgid_plural\"\n",
            "msgstr[3] \"plural 1\"\n",
            "msgstr[5] \"plural 2\"\n"
        );
        assert_eq!(entry.to_string(), expected);

        // previous msgid
        entry.previous_msgid = Some("A previous msgid".to_string());
        let expected = concat!(
            "#. extracted_comment\n# comment\n",
            "#: file1.rs:1 file2.rs:2\n",
            "#, fuzzy, python-format, rspolib\n",
            "#| msgctxt \"A previous msgctxt\"\n",
            "#| msgid \"A previous msgid\"\n",
            "msgctxt \"msgctxt\"\n",
            "msgid \"msgid\"\n",
            "msgid_plural \"msgid_plural\"\n",
            "msgstr[3] \"plural 1\"\n",
            "msgstr[5] \"plural 2\"\n"
        );
        assert_eq!(entry.to_string(), expected);
    }

    #[test]
    fn multiline_format() {
        let mut entry = POEntry::new(0);

        // simple msgid wrapping
        entry.msgid = concat!(
            "  A long long long long long long long long",
            " long long long long long long long msgid",
        )
        .to_string();
        let expected = concat!(
            "msgid \"\"\n",
            "\"  A long long long long long long long long long",
            " long long long long long\"\n",
            "\" long msgid\"\n",
            "msgstr \"\"\n",
        );
        assert_eq!(entry.to_string(), expected);

        entry.msgid = concat!(
            "A long long long long long long long long",
            " long long long long long long long long long",
            " long long long long long long long long long",
            " long long long long long long long long long",
            " long long long long long long long long long",
            " msgid",
        )
        .to_string();
        let expected = concat!(
            "msgid \"\"\n",
            "\"A long long long long long long",
            " long long long long long long long long long\"\n",
            "\" long long long long long long long long long long",
            " long long long long long\"\n\" long long long long",
            " long long long long long long long long long long",
            " msgid\"\n",
            "msgstr \"\"\n",
        );
        assert_eq!(entry.to_string(), expected);

        // include newlines in msgid
        entry.msgid = concat!(
            "A long long long long\nlong long long long\n",
            "long long long\nlong long long long lo\nng long",
            " msgid",
        )
        .to_string();
        let expected = concat!(
            "msgid \"\"\n",
            "\"A long long long long\\nlong long long long\\n",
            "long long long\\nlong long long\"\n",
            "\" long lo\\nng long msgid\"\n",
            "msgstr \"\"\n"
        );
        assert_eq!(entry.to_string(), expected);
    }

    #[test]
    fn format_escapes() {
        let mut entry = POEntry::new(0);

        // "
        entry.msgid = "aa\"bb".to_string();
        assert_eq!(
            entry.to_string(),
            "msgid \"aa\\\"bb\"\nmsgstr \"\"\n",
        );

        // \n
        entry.msgid = "aa\nbb".to_string();
        assert_eq!(
            entry.to_string(),
            "msgid \"aa\\nbb\"\nmsgstr \"\"\n",
        );

        // \t
        entry.msgid = "aa\tbb".to_string();
        assert_eq!(
            entry.to_string(),
            "msgid \"aa\\tbb\"\nmsgstr \"\"\n",
        );

        // \r
        entry.msgid = "aa\rbb".to_string();
        assert_eq!(
            entry.to_string(),
            "msgid \"aa\\rbb\"\nmsgstr \"\"\n",
        );

        // \\
        entry.msgid = "aa\\bb".to_string();
        assert_eq!(
            entry.to_string(),
            "msgid \"aa\\bb\"\nmsgstr \"\"\n",
        );
    }

    #[test]
    fn format_wrapping() {
        let path = "tests-data/wrapping.po";
        let file = pofile(path).unwrap();

        let expected = concat!(
            "# test wrapping\n",
            "msgid \"\"\n",
            "msgstr \"\"\n",
            "\n",
            "msgid \"This line will not be wrapped\"\n",
            "msgstr \"\"\n\nmsgid \"\"\n",
            "\"Some line that contain special characters",
            " \\\" and that \\t is very, very, very\"\n",
            "\" long...: %s \\n\"\n",
            "msgstr \"\"\n",
            "\n",
            "msgid \"\"\n",
            "\"Some line that contain special characters",
            " \\\"foobar\\\" and that contains\"\n",
            "\" whitespace at the end          \"\n",
            "msgstr \"\"\n"
        );
        assert_eq!(file.to_string(), expected);
    }
}
