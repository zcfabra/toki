use crate::ast::AstBlock;
use crate::parser::ParseErr;

// TODO: Extract the print formatting stuff

pub fn report<'src>(
    parsed: Result<AstBlock<'src>, ParseErr>,
    src: &'src str,
) -> Result<AstBlock<'src>, String> {
    let err = match parsed {
        Err(e) => e,
        Ok(r) => return Ok(r),
    };

    Err(match err {
        ParseErr::InvalidExpressionStart(ix, len) => {
            let (line, line_no, ix_in_line) = extract_line(src, ix);

            let line_w_underline = underline_line(&line, ix_in_line, len);
            let highlight_line = highlight_line(&line, ix_in_line, len);

            format!(
                "\n\x1b[1mError: Expected Expression at Position {}:{}:\x1b[0m\n\n\t{}\n\t{}\n\n",
                line_no, ix, highlight_line, line_w_underline
            )
        }
        ParseErr::ExpectedTypeAnnotation(ix, len) => {
            let (line, line_no, ix_in_line) = extract_line(src, ix);

            let line_w_underline = underline_line(&line, ix_in_line, len);
            let highlight_line = highlight_line(&line, ix_in_line, len);

            format!(
                "\n\x1b[1mError: Expected Valid Type In Annotation at Position {}:{}:\x1b[0m\n\n\t{}\n\t{}\n\n",
                line_no, ix, highlight_line, line_w_underline
            )
        }
        _ => todo!(),
    })
}

fn extract_line(s: &str, ix: usize) -> (&str, usize, usize) {
    // Find the slice before the index
    let before_ix = &s[..ix];

    // Find the slice after the index
    let after_ix = &s[ix..];

    // Find the last newline before the index, or the start of the string
    let start = before_ix.rfind('\n').map_or(0, |n| n + 1);

    // Find the first newline after the index, or the end of the string
    let end = after_ix.find('\n').map_or(s.len(), |n| ix + n);

    let line_no = before_ix.split('\n').collect::<Vec<&str>>().len() - 1;

    // Return the slice between the found newlines
    let line = &s[start..end];
    (line, line_no, ix - start)
}

fn highlight_line(line: &str, ix: usize, token_len: usize) -> String {
    let mut s = line.to_string();

    let start = line
        .char_indices()
        .nth(ix)
        .map(|(pos, _)| pos)
        .unwrap_or(ix);

    let end = line
        .char_indices()
        .nth(ix + token_len)
        .map(|(pos, _)| pos)
        .unwrap_or(line.len());

    s.replace_range(
        start..end,
        format!("\x1b[91m{}\x1b[0m", &line[start..end]).as_str(),
    );
    return s;
}

fn underline_line(line: &str, ix: usize, token_len: usize) -> String {
    let mut s = std::iter::repeat(" ")
        .take(line.chars().count())
        .collect::<String>();
    let highlight = std::iter::repeat("^").take(token_len).collect::<String>();
    let red = format!("\x1b[91m{}\x1b[0m", highlight);

    let start = line
        .char_indices()
        .nth(ix)
        .map(|(pos, _)| pos)
        .unwrap_or(ix);

    let end = line
        .char_indices()
        .nth(ix + token_len)
        .map(|(pos, _)| pos)
        .unwrap_or(line.len());

    s.replace_range(start..end, red.as_str());
    return s;
}
