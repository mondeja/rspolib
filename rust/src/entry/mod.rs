use std::fmt;

use unicode_width::UnicodeWidthStr;

use crate::escaping::escape;
use crate::twrapper::wrap;

pub mod moentry;
pub mod poentry;

pub use moentry::MOEntry;
pub use poentry::POEntry;

/// Provides a function `translated` to represent
/// if some struct is translated
pub trait Translated {
    fn translated(&self) -> bool;
}

/// Concatenates `msgid` + `EOT` + `msgctxt`
///
/// The MO files spec indicates:
///
/// > Contexts are stored (in MO files) by storing
/// the concatenation of the context, a EOT byte,
/// and the original string.
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

pub(crate) fn maybe_msgid_msgctxt_eot_split(
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
    let mut ret = String::new();
    ret.push_str(&mo_entry_to_string_with_msgstr_formatter(
        entry,
        78,
        "",
        &metadata_msgstr_formatter,
    ));
    ret
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
            &lines.remove(0),
        );

        // format other lines
        for line in lines {
            ret.push_str(&format!("{}\"{}\"\n", self.delflag, &line));
        }

        write!(f, "{}", ret)
    }
}
