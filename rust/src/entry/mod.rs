use std::borrow::Cow;
use std::fmt;

use unicode_width::UnicodeWidthStr;

use crate::escaping::escape;
use crate::twrapper::wrap;

pub mod moentry;
pub mod poentry;

pub use moentry::MOEntry;
pub use poentry::POEntry;

/// Provides a function `translated` to represent
/// if an entry struct is translated
pub trait Translated {
    fn translated(&self) -> bool;
}

/// Concatenates `msgid` + `EOT` + `msgctxt`
///
/// The MO files spec indicates:
///
/// > Contexts are stored (in MO files) by storing
/// > the concatenation of the context, a EOT byte,
/// > and the original string.
///
/// This trait provides a way to get the string
/// representation of `msgid` + `EOT` + `msgctxt`.
///
/// Function required to generate MO files as
/// the returned value is used as key on the
/// translations table.
pub trait MsgidEotMsgctxt {
    /// Returns `msgid` + (optionally: `EOT` + `msgctxt`)
    fn msgid_eot_msgctxt(&self) -> String;
}

pub(crate) fn maybe_msgid_msgctxt_eot_split<'a>(
    msgid: &'a str,
    msgctxt: &Option<String>,
) -> Cow<'a, str> {
    if let Some(ctx) = msgctxt {
        let mut ret = String::from(ctx);
        ret.reserve(msgid.len() + 1);
        ret.push('\u{4}');
        ret.push_str(msgid);
        ret.into()
    } else {
        msgid.into()
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
        ret.push_str(&escape(line));
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

    if entry.msgstr_plural.is_empty() {
        let msgstr = match &entry.msgstr {
            Some(msgstr) => msgstr,
            None => "",
        };
        let formatted_msgstr =
            msgstr_formatter(msgstr, delflag, wrapwidth);
        ret.push_str(&formatted_msgstr);
    } else {
        for (i, msgstr_plural) in
            entry.msgstr_plural.iter().enumerate()
        {
            ret.push_str(
                &POStringField::new(
                    "msgstr",
                    delflag,
                    msgstr_plural,
                    &i.to_string(),
                    wrapwidth,
                )
                .to_string(),
            );
        }
    }

    ret
}

pub(crate) fn mo_entry_to_string(
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

/// Converts a metadata wrapped by a [POEntry] to a string
/// representation.
///
/// ```rust
/// use rspolib::{
///     pofile,
///     po_metadata_entry_to_string,
/// };
///
/// let file = pofile("tests-data/all.po").unwrap();
/// let entry = file.metadata_as_entry();
/// let entry_str = po_metadata_entry_to_string(&entry, true);
///
/// assert!(
///     entry_str.starts_with("#, fuzzy\nmsgid \"\"\nmsgstr \"\"")
/// );
/// ```
pub fn po_metadata_entry_to_string(
    entry: &POEntry,
    metadata_is_fuzzy: bool,
) -> String {
    let mut ret = String::new();
    if metadata_is_fuzzy {
        ret.push_str("#, fuzzy\n");
    }
    ret.push_str(&mo_metadata_entry_to_string(&MOEntry::from(entry)));
    ret
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

#[allow(clippy::needless_lifetimes)]
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
            UnicodeWidthStr::width(escaped_value.as_ref())
                + UnicodeWidthStr::width(self.fieldname)
                + 1;
        if real_width > self.wrapwidth {
            let new_lines = wrap(&escaped_value, self.wrapwidth);
            lines.extend(new_lines);
        } else {
            lines = vec![escaped_value.into_owned()];
        }

        // format first line
        let mut ret = format!(
            "{}{}{} \"{}\"\n",
            self.delflag,
            self.fieldname,
            repr_plural_index,
            &lines.remove(0),
        );

        // format other lines
        for line in lines {
            ret.push_str(&format!("{}\"{}\"\n", self.delflag, &line));
        }

        write!(f, "{ret}")
    }
}

/// A struct to compare two entries.
///
/// ```rust
/// use std::cmp::Ordering;
/// use rspolib::{POEntry, EntryCmpByOptions};
///
/// let mut entry1 = POEntry::from("msgid 1");
/// let entry2 = POEntry::from("msgid 2");
///
/// let compare_by_all_fields = EntryCmpByOptions::new();
/// let compare_by_msgid_only = EntryCmpByOptions::new()
///     .by_all(false)
///     .by_msgid(true);
///
/// assert_eq!(entry1.cmp_by(&entry2, &compare_by_msgid_only), Ordering::Less);
/// assert_eq!(entry2.cmp_by(&entry1, &compare_by_msgid_only), Ordering::Greater);
///
/// entry1.msgid = "msgid 2".to_string();
/// assert_eq!(entry1.cmp_by(&entry2, &compare_by_msgid_only), Ordering::Equal);
///
/// entry1.msgstr = Some("msgstr 1".to_string());
/// assert_eq!(entry1.cmp_by(&entry2, &compare_by_msgid_only), Ordering::Equal);
/// assert_eq!(entry1.cmp_by(&entry2, &compare_by_all_fields), Ordering::Greater);
/// ```
pub struct EntryCmpByOptions {
    by_msgid: bool,
    by_msgstr: bool,
    by_msgctxt: bool,
    by_obsolete: bool,
    by_occurrences: bool,
    by_msgid_plural: bool,
    by_msgstr_plural: bool,
    by_flags: bool,
}

impl EntryCmpByOptions {
    /// Creates a instance of [EntryCmpByOptions] with comparisons for all fields enabled
    pub fn new() -> Self {
        Self {
            by_msgid: true,
            by_msgstr: true,
            by_msgctxt: true,
            by_obsolete: true,
            by_occurrences: true,
            by_msgid_plural: true,
            by_msgstr_plural: true,
            by_flags: true,
        }
    }

    pub fn by_msgid(mut self, by_msgid: bool) -> Self {
        self.by_msgid = by_msgid;
        self
    }

    pub fn by_msgstr(mut self, by_msgstr: bool) -> Self {
        self.by_msgstr = by_msgstr;
        self
    }

    pub fn by_msgctxt(mut self, by_msgctxt: bool) -> Self {
        self.by_msgctxt = by_msgctxt;
        self
    }

    pub fn by_obsolete(mut self, by_obsolete: bool) -> Self {
        self.by_obsolete = by_obsolete;
        self
    }

    pub fn by_occurrences(mut self, by_occurrences: bool) -> Self {
        self.by_occurrences = by_occurrences;
        self
    }

    pub fn by_msgid_plural(mut self, by_msgid_plural: bool) -> Self {
        self.by_msgid_plural = by_msgid_plural;
        self
    }

    pub fn by_msgstr_plural(
        mut self,
        by_msgstr_plural: bool,
    ) -> Self {
        self.by_msgstr_plural = by_msgstr_plural;
        self
    }

    pub fn by_flags(mut self, by_flags: bool) -> Self {
        self.by_flags = by_flags;
        self
    }

    pub fn by_all(mut self, by_all: bool) -> Self {
        self.by_msgid = by_all;
        self.by_msgstr = by_all;
        self.by_msgctxt = by_all;
        self.by_obsolete = by_all;
        self.by_occurrences = by_all;
        self.by_msgid_plural = by_all;
        self.by_msgstr_plural = by_all;
        self.by_flags = by_all;
        self
    }
}

impl Default for EntryCmpByOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&Vec<(String, bool)>> for EntryCmpByOptions {
    fn from(options: &Vec<(String, bool)>) -> Self {
        let mut ret = Self::new();
        for (key, value) in options {
            match key.as_str() {
                "msgid" => ret.by_msgid = *value,
                "msgstr" => ret.by_msgstr = *value,
                "msgctxt" => ret.by_msgctxt = *value,
                "obsolete" => ret.by_obsolete = *value,
                "occurrences" => ret.by_occurrences = *value,
                "msgid_plural" => ret.by_msgid_plural = *value,
                "msgstr_plural" => ret.by_msgstr_plural = *value,
                "flags" => ret.by_flags = *value,
                _ => {}
            }
        }
        ret
    }
}
