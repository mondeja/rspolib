use std::collections::HashMap;

use unicode_linebreak::{
    linebreaks as unicode_linebreaks, BreakOpportunity,
};
use unicode_width::UnicodeWidthStr;

#[allow(clippy::mut_range_bound)]
fn get_linebreaks(
    linebreaks: &Vec<(usize, BreakOpportunity)>,
    text: &str,
    wrapwidth: usize,
) -> Vec<usize> {
    let char_indices_widths: HashMap<usize, usize> = text
        .char_indices()
        .map(|(i, c)| {
            (i, UnicodeWidthStr::width(c.to_string().as_str()))
        })
        .collect();
    let mut ret = vec![];

    let mut accum_char_bindex = 0;
    let mut accum_char_width = 0; // bindex, width
    let mut last_break_width = 0;
    let mut prev_end_whitespaces = 0;

    for (lbi, (lb, _)) in linebreaks.iter().enumerate() {
        let range = accum_char_width..*lb;
        for bindex in range {
            accum_char_width +=
                char_indices_widths.get(&bindex).unwrap_or(&0);
            accum_char_bindex = bindex;
        }
        if lbi == linebreaks.len() - 1 {
            continue;
        }
        let (next_lb, _) = linebreaks[lbi + 1];

        let mut lb_bindex = *lb;

        let mut partial_accum_width = accum_char_width;
        let mut partial_accum_bindex = accum_char_bindex;
        for i in accum_char_bindex..next_lb {
            if let Some(width) = char_indices_widths.get(&i) {
                partial_accum_width += width;
                partial_accum_bindex = i;
            }
        }
        if lbi + 1 == linebreaks.len() - 1 {
            if prev_end_whitespaces > 0 {
                lb_bindex -= prev_end_whitespaces;
                partial_accum_width -= prev_end_whitespaces;
            }
        } else {
            // Don't break on whitespace, keep it in the next line
            // This is the same behaviour that gettext has
            if text[partial_accum_bindex..].starts_with(' ') {
                prev_end_whitespaces = 0;
                for ch in
                    text[..partial_accum_bindex + 1].chars().rev()
                {
                    if ch.is_ascii_whitespace() {
                        partial_accum_width -= 1;
                        lb_bindex -= 1;
                        prev_end_whitespaces += 1;
                    } else {
                        break;
                    }
                }
            }
        }
        let width = partial_accum_width - last_break_width;
        if width > wrapwidth {
            ret.push(lb_bindex);
            last_break_width = accum_char_width;
        }
    }

    ret
}

/// Wrap a text in lines using Unicode Line Breaking algorithm
pub(crate) fn wrap(text: &str, wrapwidth: usize) -> Vec<String> {
    let linebreaks = get_linebreaks(
        &unicode_linebreaks(text).collect(),
        text,
        wrapwidth,
    );

    let mut ret: Vec<String> = vec![];
    let mut prev_lb = 0;
    for lb in linebreaks {
        ret.push(text[prev_lb..lb].to_string());
        prev_lb = lb;
    }
    ret.push(text[prev_lb..].to_string());
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let text =
            "This is a test of the emergency broadcast system.";
        let wrapped = wrap(text, 10);
        assert_eq!(
            wrapped,
            vec![
                "This is a",
                " test of",
                " the",
                " emergency",
                " broadcast",
                " system."
            ]
        );
    }

    #[test]
    fn long_wrapwidth() {
        let text =
            "This is a test of the emergency broadcast system.";
        let wrapped = wrap(text, 100);
        assert_eq!(wrapped, vec![text]);
    }

    #[test]
    fn unbreakable_line() {
        let text = "Thislineisverylongbutmustnotbebroken breaks should be here.";
        let wrapped = wrap(text, 5);

        assert_eq!(
            wrapped,
            vec![
                "Thislineisverylongbutmustnotbebroken",
                " breaks",
                " should",
                " be",
                " here."
            ]
        );
    }

    #[test]
    fn unicode_characters() {
        let text = "123Ááé aabbcc ÁáééÚí aabbcc";
        let wrapped = wrap(text, 7);
        assert_eq!(
            wrapped,
            vec!["123Ááé", " aabbcc", " ÁáééÚí", " aabbcc"]
        );
    }
}
