use std::collections::HashMap;

use unicode_linebreak::{
    linebreaks as unicode_linebreaks, BreakOpportunity,
};
use unicode_width::UnicodeWidthChar;

#[allow(clippy::mut_range_bound)]
fn get_linebreaks(
    linebreaks: &[(usize, BreakOpportunity)],
    text: &str,
    wrapwidth: usize,
) -> Vec<usize> {
    let char_indices_widths: HashMap<usize, usize> = text
        .char_indices()
        .map(|(i, c)| (i, UnicodeWidthChar::width(c).unwrap_or(0)))
        .collect();
    let mut ret = vec![];

    let mut accum_char_bindex = 0;
    let mut accum_char_width = 0; // bindex, width
    let mut last_break_width = 0;

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

        let mut partial_accum_width = accum_char_width;
        for i in accum_char_bindex..next_lb {
            if let Some(width) = char_indices_widths.get(&i) {
                partial_accum_width += width;
            }
        }
        let width = partial_accum_width - last_break_width;
        if width > wrapwidth {
            ret.push(*lb);
            last_break_width = accum_char_width;
        }
    }

    ret
}

/// Wrap a text in lines using Unicode Line Breaking algorithm
///
/// - `text` - Text to wrap in lines
/// - `wrapwidth` - Maximum width of a line
pub(crate) fn wrap(text: &str, wrapwidth: usize) -> Vec<String> {
    let linebreaks = get_linebreaks(
        &unicode_linebreaks(text).collect::<Vec<(usize, unicode_linebreak::BreakOpportunity)>>(),
        text,
        wrapwidth,
    );

    let mut ret: Vec<String> =
        Vec::with_capacity(linebreaks.len() + 1);
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
                "This is ",
                "a test ",
                "of the ",
                "emergency ",
                "broadcast ",
                "system."
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
                "Thislineisverylongbutmustnotbebroken ",
                "breaks ",
                "should ",
                "be ",
                "here."
            ]
        );
    }

    #[test]
    fn unicode_characters() {
        let text = "123Ááé aabbcc ÁáééÚí aabbcc";
        let wrapped = wrap(text, 7);
        assert_eq!(
            wrapped,
            vec!["123Ááé ", "aabbcc ", "ÁáééÚí ", "aabbcc"]
        );
    }
}
