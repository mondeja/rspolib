use std::cmp::Ordering;
use std::fmt;

use unicode_width::UnicodeWidthStr;

use crate::entry::{
    maybe_msgid_msgctxt_eot_split, mo_entry_to_string,
    EntryCmpByOptions, MOEntry, MsgidEotMsgctxt, POStringField,
    Translated,
};
use crate::errors::EscapingError;
use crate::escaping::unescape;
use crate::traits::Merge;
use crate::twrapper::wrap;

/// PO file entry representing a message
///
/// This struct contains all the information that is stored
/// in PO files.
///
/// PO entries can contain `msgstr` or the fields
/// `msgid_plural` and `msgstr_plural` as not being `None`.
/// The logic would be:
///
/// - If `msgstr` is not `None`, then the entry is a
///   translation of a singular form.
/// - If `msgid_plural` is not `None`, then the entry
///   is a translation of a plural form contained in
///   `msgstr_plural`.
///
/// The `previous_msgid` and `previous_msgid_plural` fields
/// are used to store the previous `msgid` and `msgid_plural`
/// values when the entry is obsolete.
///
/// The `previous_msgctxt` field is used to store the previous
/// `msgctxt` value when the entry is obsolete.
#[derive(Default, Clone, PartialEq, Debug)]
pub struct POEntry {
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
            if UnicodeWidthStr::width(line) + 2 > wrapwidth {
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
                tcomment, "# ", wrapwidth, &mut ret,
            );
        }

        // comments
        if let Some(comment) = &self.comment {
            self.format_comment_inplace(
                comment, "#. ", wrapwidth, &mut ret,
            );
        }

        // occurrences
        if !self.obsolete && !self.occurrences.is_empty() {
            let whitespace_sep_occurrences = self
                .occurrences
                .iter()
                .map(|(fpath, lineno)| {
                    if lineno.is_empty() {
                        return fpath.clone();
                    }
                    format!("{fpath}:{lineno}")
                })
                .collect::<Vec<String>>();

            let mut files_repr: Vec<String> = vec![];

            let mut current_line_occs: Vec<&str> = vec![];
            let mut current_width = 2;
            for occ in &whitespace_sep_occurrences {
                let occ_width = UnicodeWidthStr::width(occ.as_str());
                let width = current_width + occ_width + 1;
                if width > wrapwidth && !current_line_occs.is_empty()
                {
                    let curr_line =
                        format!("#: {}", current_line_occs.join(" "));
                    files_repr.push(curr_line);
                    current_width = occ_width + 2;
                } else {
                    current_line_occs.push(occ);
                    current_width += occ_width + 1;
                }
            }
            if !current_line_occs.is_empty() {
                let curr_line =
                    format!("#: {}", current_line_occs.join(" "));
                files_repr.push(curr_line);
            }
            ret.push_str(&files_repr.join("\n"));
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

    pub fn unescaped(&self) -> Result<Self, EscapingError> {
        let mut entry = self.clone();
        entry.msgid = unescape(&self.msgid)?.to_string();

        if let Some(msgstr) = &self.msgstr {
            entry.msgstr = Some(unescape(msgstr)?.to_string());
        }

        if let Some(msgid_plural) = &self.msgid_plural {
            entry.msgid_plural =
                Some(unescape(msgid_plural)?.to_string());
        }

        for msgstr_plural in &mut entry.msgstr_plural {
            *msgstr_plural = unescape(msgstr_plural)?.to_string();
        }

        if let Some(msgctxt) = &self.msgctxt {
            entry.msgctxt = Some(unescape(msgctxt)?.to_string());
        }

        if let Some(previous_msgid) = &self.previous_msgid {
            entry.previous_msgid =
                Some(unescape(previous_msgid)?.to_string());
        }

        if let Some(previous_msgid_plural) =
            &self.previous_msgid_plural
        {
            entry.previous_msgid_plural =
                Some(unescape(previous_msgid_plural)?.to_string());
        }

        if let Some(previous_msgctxt) = &self.previous_msgctxt {
            entry.previous_msgctxt =
                Some(unescape(previous_msgctxt)?.to_string());
        }

        Ok(entry)
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
        if options.by_obsolete && self.obsolete != other.obsolete {
            match self.obsolete {
                true => return Ordering::Less,
                false => return Ordering::Greater,
            }
        }

        if options.by_occurrences {
            let mut occ1 = self.occurrences.clone();
            occ1.sort();
            let mut occ2 = other.occurrences.clone();
            occ2.sort();
            if occ1 > occ2 {
                return Ordering::Greater;
            } else if occ1 < occ2 {
                return Ordering::Less;
            }
        }

        if options.by_flags {
            let mut flags1 = self.flags.clone();
            flags1.sort();
            let mut flags2 = other.flags.clone();
            flags2.sort();
            if flags1 > flags2 {
                return Ordering::Greater;
            } else if flags1 < flags2 {
                return Ordering::Less;
            }
        }

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

impl MsgidEotMsgctxt for POEntry {
    fn msgid_eot_msgctxt(&self) -> String {
        maybe_msgid_msgctxt_eot_split(&self.msgid, &self.msgctxt)
            .to_string()
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
        for msgstr in &self.msgstr_plural {
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
        entry.msgstr_plural = mo_entry.msgstr_plural.clone();
        entry.msgctxt = mo_entry.msgctxt.as_ref().cloned();
        entry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pofile;

    #[test]
    fn constructor() {
        let poentry = POEntry::new(7);

        assert_eq!(poentry.linenum, 7);
        assert_eq!(poentry.msgid, "");
        assert_eq!(poentry.msgstr, None);
        assert_eq!(poentry.msgid_plural, None);
        assert_eq!(poentry.msgstr_plural, vec![] as Vec<String>);
        assert_eq!(poentry.msgctxt, None);
    }

    #[test]
    fn fuzzy() {
        let non_fuzzy_entry = POEntry::new(0);
        assert_eq!(non_fuzzy_entry.fuzzy(), false);

        let mut fuzzy_entry = POEntry::new(0);
        fuzzy_entry.flags.push("fuzzy".to_string());
        assert_eq!(fuzzy_entry.fuzzy(), true);
    }

    #[test]
    fn translated() {
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
        empty_msgstr_plural_entry.msgstr_plural = vec![];
        assert_eq!(empty_msgstr_plural_entry.translated(), false);

        // with empty msgstr_plural means untranslated
        let mut empty_msgstr_plural_entry = POEntry::new(0);
        empty_msgstr_plural_entry.msgstr_plural =
            vec!["".to_string()];
        assert_eq!(empty_msgstr_plural_entry.translated(), false);

        // with msgstr_plural means translated
        let mut translated_plural_entry = POEntry::new(0);
        translated_plural_entry.msgstr_plural =
            vec!["msgstr_plural".to_string()];
        assert_eq!(translated_plural_entry.translated(), true);
    }

    #[test]
    fn merge() {
        let mut poentry = POEntry::new(0);
        poentry.msgid = "msgid".to_string();
        poentry.msgstr = Some("msgstr".to_string());
        poentry.msgid_plural = Some("msgid_plural".to_string());
        poentry.msgstr_plural = vec!["msgstr_plural".to_string()];

        let mut other = POEntry::new(0);
        other.msgid = "other_msgid".to_string();
        other.msgstr = Some("other_msgstr".to_string());
        other.msgid_plural = Some("other_msgid_plural".to_string());
        other.msgstr_plural = vec!["other_msgstr_plural".to_string()];

        poentry.merge(other);

        assert_eq!(poentry.msgid, "other_msgid");
        assert_eq!(poentry.msgstr, Some("other_msgstr".to_string()));
        assert_eq!(
            poentry.msgid_plural,
            Some("other_msgid_plural".to_string())
        );
        assert_eq!(
            poentry.msgstr_plural,
            vec!["other_msgstr_plural".to_string()]
        );
    }

    #[test]
    fn to_string() {
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
        entry.msgstr_plural =
            vec!["plural 1".to_string(), "plural 2".to_string()];

        let expected = concat!(
            "msgid \"msgid\"\nmsgid_plural \"msgid_plural\"\n",
            "msgstr[0] \"plural 1\"\nmsgstr[1] \"plural 2\"\n",
        );
        assert_eq!(entry.to_string(), expected);

        // msgctxt
        entry.msgctxt = Some("msgctxt".to_string());
        let expected = concat!(
            "msgctxt \"msgctxt\"\nmsgid \"msgid\"\n",
            "msgid_plural \"msgid_plural\"\n",
            "msgstr[0] \"plural 1\"\n",
            "msgstr[1] \"plural 2\"\n"
        );
        assert_eq!(entry.to_string(), expected);

        // flags
        entry.flags.push("fuzzy".to_string());
        let expected = concat!(
            "#, fuzzy\n",
            "msgctxt \"msgctxt\"\nmsgid \"msgid\"\n",
            "msgid_plural \"msgid_plural\"\n",
            "msgstr[0] \"plural 1\"\n",
            "msgstr[1] \"plural 2\"\n"
        );
        assert_eq!(entry.to_string(), expected);

        entry.flags.push("python-format".to_string());
        let expected = concat!(
            "#, fuzzy, python-format\nmsgctxt \"msgctxt\"\n",
            "msgid \"msgid\"\nmsgid_plural \"msgid_plural\"\n",
            "msgstr[0] \"plural 1\"\nmsgstr[1] \"plural 2\"\n"
        );
        assert_eq!(entry.to_string(), expected);

        // comments
        entry.comment = Some("comment".to_string());
        let expected = concat!(
            "#. comment\n#, fuzzy, python-format\n",
            "msgctxt \"msgctxt\"\nmsgid \"msgid\"\n",
            "msgid_plural \"msgid_plural\"\n",
            "msgstr[0] \"plural 1\"\nmsgstr[1] \"plural 2\"\n"
        );
        assert_eq!(entry.to_string(), expected);

        entry.tcomment = Some("translator comment".to_string());
        let expected = concat!(
            "# translator comment\n#. comment\n",
            "#, fuzzy, python-format\nmsgctxt \"msgctxt\"\n",
            "msgid \"msgid\"\nmsgid_plural \"msgid_plural\"\n",
            "msgstr[0] \"plural 1\"\nmsgstr[1] \"plural 2\"\n"
        );
        assert_eq!(entry.to_string(), expected);

        // obsolete
        entry.obsolete = true;
        let expected = concat!(
            "# translator comment\n#. comment\n",
            "#, fuzzy, python-format\n#~ msgctxt \"msgctxt\"\n",
            "#~ msgid \"msgid\"\n",
            "#~ msgid_plural \"msgid_plural\"\n",
            "#~ msgstr[0] \"plural 1\"\n",
            "#~ msgstr[1] \"plural 2\"\n"
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
            "# translator comment\n#. comment\n",
            "#, fuzzy, python-format\n",
            "#~ msgctxt \"msgctxt\"\n",
            "#~ msgid \"msgid\"\n",
            "#~ msgid_plural \"msgid_plural\"\n",
            "#~ msgstr[0] \"plural 1\"\n",
            "#~ msgstr[1] \"plural 2\"\n"
        );
        assert_eq!(entry.to_string(), expected);

        entry.obsolete = false;
        let expected = concat!(
            "# translator comment\n#. comment\n",
            "#: file1.rs:1 file2.rs:2\n",
            "#, fuzzy, python-format\n",
            "msgctxt \"msgctxt\"\nmsgid \"msgid\"\n",
            "msgid_plural \"msgid_plural\"\n",
            "msgstr[0] \"plural 1\"\n",
            "msgstr[1] \"plural 2\"\n"
        );
        assert_eq!(entry.to_string(), expected);

        // Basic complete example
        entry.msgstr = Some("msgstr".to_string());
        entry.comment = Some("comment".to_string());
        entry.tcomment = Some("translator comment".to_string());
        entry.flags.push("rspolib".to_string());
        let expected = concat!(
            "# translator comment\n#. comment\n",
            "#: file1.rs:1 file2.rs:2\n",
            "#, fuzzy, python-format, rspolib\n",
            "msgctxt \"msgctxt\"\nmsgid \"msgid\"\n",
            "msgid_plural \"msgid_plural\"\n",
            "msgstr[0] \"plural 1\"\n",
            "msgstr[1] \"plural 2\"\n"
        );
        assert_eq!(entry.to_string(), expected);

        // previous msgctxt
        entry.previous_msgctxt =
            Some("A previous msgctxt".to_string());
        let expected = concat!(
            "# translator comment\n#. comment\n",
            "#: file1.rs:1 file2.rs:2\n",
            "#, fuzzy, python-format, rspolib\n",
            "#| msgctxt \"A previous msgctxt\"\n",
            "msgctxt \"msgctxt\"\n",
            "msgid \"msgid\"\n",
            "msgid_plural \"msgid_plural\"\n",
            "msgstr[0] \"plural 1\"\n",
            "msgstr[1] \"plural 2\"\n"
        );
        assert_eq!(entry.to_string(), expected);

        // previous msgid
        entry.previous_msgid = Some("A previous msgid".to_string());
        let expected = concat!(
            "# translator comment\n#. comment\n",
            "#: file1.rs:1 file2.rs:2\n",
            "#, fuzzy, python-format, rspolib\n",
            "#| msgctxt \"A previous msgctxt\"\n",
            "#| msgid \"A previous msgid\"\n",
            "msgctxt \"msgctxt\"\n",
            "msgid \"msgid\"\n",
            "msgid_plural \"msgid_plural\"\n",
            "msgstr[0] \"plural 1\"\n",
            "msgstr[1] \"plural 2\"\n"
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
            " long long long long long \"\n",
            "\"long msgid\"\n",
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
            " long long long long long long long long long \"\n",
            "\"long long long long long long long long long long",
            " long long long long long \"\n\"long long long long",
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
            "long long long\\nlong long long \"\n",
            "\"long lo\\nng long msgid\"\n",
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
            "msgid \"aa\\\\bb\"\nmsgstr \"\"\n",
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
            "msgstr \"\"\n",
            "\nmsgid \"\"\n",
            "\"Some line that contain special characters",
            " \\\" and that \\t is very, very, very \"\n",
            "\"long...: %s \\n\"\n",
            "msgstr \"\"\n",
            "\nmsgid \"\"\n",
            "\"Some line that contain special characters",
            " \\\"foobar\\\" and that contains \"\n",
            "\"whitespace at the end          \"\n",
            "msgstr \"\"\n"
        );
        assert_eq!(file.to_string(), expected);
    }

    #[test]
    fn cmp_by_obsolete() {
        let mut entry1 = POEntry::new(0);
        let mut entry2 = POEntry::new(0);

        // options
        let by_obsolete =
            EntryCmpByOptions::new().by_all(false).by_obsolete(true);
        let by_nothing = EntryCmpByOptions::new().by_all(false);

        entry1.obsolete = true;
        entry2.obsolete = false;

        assert_eq!(
            entry1.cmp_by(&entry2, &by_obsolete),
            Ordering::Less,
        );
        assert_eq!(
            entry1.cmp_by(&entry2, &by_nothing),
            Ordering::Equal,
        );
        assert_eq!(
            entry2.cmp_by(&entry1, &by_obsolete),
            Ordering::Greater,
        );

        entry1.obsolete = false;
        assert_eq!(
            entry1.cmp_by(&entry2, &by_obsolete),
            Ordering::Equal,
        );
    }

    #[test]
    fn cmp_by_msgid() {
        let mut entry1 = POEntry::new(0);
        let mut entry2 = POEntry::new(0);

        // options
        let by_msgid =
            EntryCmpByOptions::new().by_all(false).by_msgid(true);
        let by_nothing = EntryCmpByOptions::new().by_all(false);

        entry1.msgid = "a".to_string();
        entry2.msgid = "b".to_string();

        assert_eq!(entry1.cmp_by(&entry2, &by_msgid), Ordering::Less);
        assert_eq!(
            entry2.cmp_by(&entry1, &by_msgid),
            Ordering::Greater,
        );
        assert_eq!(
            entry2.cmp_by(&entry1, &by_nothing),
            Ordering::Equal,
        );
    }

    #[test]
    fn cmp_by_msgstr() {
        let mut entry1 = POEntry::new(0);
        let mut entry2 = POEntry::new(0);

        // options
        let by_msgstr =
            EntryCmpByOptions::new().by_all(false).by_msgstr(true);
        let by_nothing = EntryCmpByOptions::new().by_all(false);

        entry1.msgstr = Some("a".to_string());
        entry2.msgstr = Some("b".to_string());

        assert_eq!(
            entry1.cmp_by(&entry2, &by_msgstr),
            Ordering::Less
        );
        assert_eq!(
            entry2.cmp_by(&entry1, &by_msgstr),
            Ordering::Greater,
        );
        assert_eq!(
            entry2.cmp_by(&entry1, &by_nothing),
            Ordering::Equal,
        );
    }

    #[test]
    fn cmp_by_msgctxt() {
        let mut entry1 = POEntry::new(0);
        let mut entry2 = POEntry::new(0);

        // options
        let by_msgctxt =
            EntryCmpByOptions::new().by_all(false).by_msgctxt(true);
        let by_nothing = EntryCmpByOptions::new().by_all(false);

        entry1.msgctxt = Some("a".to_string());
        entry2.msgctxt = Some("b".to_string());

        assert_eq!(
            entry1.cmp_by(&entry2, &by_msgctxt),
            Ordering::Less
        );
        assert_eq!(
            entry2.cmp_by(&entry1, &by_msgctxt),
            Ordering::Greater,
        );
        assert_eq!(
            entry2.cmp_by(&entry1, &by_nothing),
            Ordering::Equal,
        );
    }

    #[test]
    fn cmp_by_msgid_plural() {
        let mut entry1 = POEntry::new(0);
        let mut entry2 = POEntry::new(0);

        // options
        let by_msgid_plural = EntryCmpByOptions::new()
            .by_all(false)
            .by_msgid_plural(true);
        let by_nothing = EntryCmpByOptions::new().by_all(false);

        entry1.msgid_plural = Some("a".to_string());
        entry2.msgid_plural = Some("b".to_string());

        assert_eq!(
            entry1.cmp_by(&entry2, &by_msgid_plural),
            Ordering::Less,
        );
        assert_eq!(
            entry2.cmp_by(&entry1, &by_msgid_plural),
            Ordering::Greater,
        );
        assert_eq!(
            entry2.cmp_by(&entry1, &by_nothing),
            Ordering::Equal,
        );
    }

    #[test]
    fn cmp_by_msgstr_plural() {
        let mut entry1 = POEntry::new(0);
        let mut entry2 = POEntry::new(0);

        // options
        let by_msgstr_plural = EntryCmpByOptions::new()
            .by_all(false)
            .by_msgstr_plural(true);
        let by_nothing = EntryCmpByOptions::new().by_all(false);

        entry1.msgstr_plural.insert(0, "a".to_string());
        entry2.msgstr_plural.insert(0, "b".to_string());

        assert_eq!(
            entry1.cmp_by(&entry2, &by_msgstr_plural),
            Ordering::Less,
        );
        assert_eq!(
            entry2.cmp_by(&entry1, &by_msgstr_plural),
            Ordering::Greater,
        );
        assert_eq!(
            entry2.cmp_by(&entry1, &by_nothing),
            Ordering::Equal,
        );
    }

    #[test]
    fn cmp_by_flags() {
        let mut entry1 = POEntry::new(0);
        let mut entry2 = POEntry::new(0);

        // options
        let by_flags =
            EntryCmpByOptions::new().by_all(false).by_flags(true);
        let by_nothing = EntryCmpByOptions::new().by_all(false);

        entry1.flags.push("a".to_string());
        entry2.flags.push("b".to_string());

        assert_eq!(entry1.cmp_by(&entry2, &by_flags), Ordering::Less);
        assert_eq!(
            entry2.cmp_by(&entry1, &by_flags),
            Ordering::Greater,
        );
        assert_eq!(
            entry2.cmp_by(&entry1, &by_nothing),
            Ordering::Equal,
        );
    }

    #[test]
    fn cmp_by_occurrences() {
        let mut entry1 = POEntry::new(0);
        let mut entry2 = POEntry::new(0);

        // options
        let by_occurrences = EntryCmpByOptions::new()
            .by_all(false)
            .by_occurrences(true);
        let by_nothing = EntryCmpByOptions::new().by_all(false);

        entry1.occurrences.push(("a".to_string(), "30".to_string()));
        entry2.occurrences.push(("a".to_string(), "40".to_string()));

        assert_eq!(
            entry1.cmp_by(&entry2, &by_occurrences),
            Ordering::Less,
        );
        assert_eq!(
            entry2.cmp_by(&entry1, &by_occurrences),
            Ordering::Greater,
        );
        assert_eq!(
            entry2.cmp_by(&entry1, &by_nothing),
            Ordering::Equal,
        );

        entry2.occurrences[0] = ("a".to_string(), "30".to_string());
        assert_eq!(
            entry2.cmp_by(&entry1, &by_nothing),
            Ordering::Equal,
        );
    }
}
