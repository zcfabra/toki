use crate::ast::AstNode;
use crate::parser::ParseErr;

pub fn report<'src>(
    parsed: Result<AstNode<'src>, ParseErr>,
    src: &'src str,
) -> Result<AstNode<'src>, String> {
    let err = match parsed {
        Err(e) => e,
        Ok(r) => return Ok(r),
    };

    Err(match err {
        ParseErr::InvalidExpressionStart(ix) => {
            let line = extract_line(src, ix);
            let line_w_highlight = highlight_line(line, ix);
            format!(
                "Invalid Expression Start at Position {}\n\n{}\n{}\n\n",
                ix, line, line_w_highlight
            )
        }
        _ => todo!(),
    })
}

fn extract_line(s: &str, ix: usize) -> &str {
    // Find the slice before the index
    let before_ix = &s[..ix];

    // Find the slice after the index
    let after_ix = &s[ix..];

    // Find the last newline before the index, or the start of the string
    let start = before_ix.rfind('\n').map_or(0, |n| n + 1);

    // Find the first newline after the index, or the end of the string
    let end = after_ix.find('\n').map_or(s.len(), |n| ix + n);

    // Return the slice between the found newlines
    &s[start..end]
}

fn highlight_line(line: &str, ix: usize) -> String {
    let mut s = std::iter::repeat(" ").take(line.len()).collect::<String>();
    s.replace_range(ix..ix + 1, "^");
    return s;
}
