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

    let mut accum_char_index = 0;
    let mut last_break_width = 0;

    for (lbi, (lb, _)) in linebreaks.iter().enumerate() {
        for c_width in accum_char_index..*lb {
            accum_char_index +=
                char_indices_widths.get(&c_width).unwrap_or(&0);
        }
        if lbi == linebreaks.len() - 1 {
            continue;
        }
        let (next_lb, _) = linebreaks[lbi + 1];

        let partial_accum_char_index = (char_indices_widths
            .iter()
            .filter(|(i, _)| **i >= accum_char_index && **i < next_lb)
            .map(|(_, w)| *w)
            .sum::<usize>())
            + accum_char_index;
        let width = partial_accum_char_index - last_break_width;
        if width > wrapwidth {
            ret.push(*lb);
            last_break_width = accum_char_index;
        }
    }

    ret
}

pub fn wrap(text: &str, wrapwidth: usize) -> Vec<String> {
    // linebreaks with accumulated width
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
                "This is a ",
                "test of ",
                "the ",
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
