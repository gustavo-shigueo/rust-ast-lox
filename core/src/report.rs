use crate::Error;
use color_eyre::owo_colors::OwoColorize;
use std::error::Error as ErrorTrait;

/// How many lines before and after the line containing the error
/// should be displayed
const LINE_PADDING: usize = 2;

const SEPARATOR: &str = " | ";

pub fn report<E: ErrorTrait>(source: &str, error: &Error<E>) {
    let line = error.line + 1;
    let column = error.column + 1;

    eprintln!();
    eprintln!(
        "{}: {} at {line}:{column}.",
        "Error".red().bold(),
        error.source
    );
    eprintln!();

    let offset = line.saturating_sub(LINE_PADDING + 1);
    let take = line.saturating_add(LINE_PADDING).min(2 * LINE_PADDING + 1);
    let chunk = source.lines().skip(offset).take(take);

    let align =
        // Length of the error line number
        usize::ilog10(line) as usize + 1

        // Add 1 if one of the next `LINE_PADDING` line numbers is one
        // digit longer than the error line's number.
        // This happens when the last digit of `line` (`line % 10`) is greater
        // than or equal to 10 - `LINE_PADDING`
        + usize::saturating_sub(line % 10, 9 - LINE_PADDING).min(1);

    for (i, code) in chunk.enumerate() {
        let line_indicator = format!("{:align$}{SEPARATOR}", offset + i + 1);
        eprint!("{}", line_indicator.blue().bold());

        if i == usize::min(line - 1, LINE_PADDING) {
            eprintln!("{}", code.red());
            eprintln!(
                "{}{}",
                " ".repeat(SEPARATOR.len() + align + column - 1),
                "^--- Here".yellow(),
            );
        } else {
            eprintln!("{code}");
        }
    }

    eprintln!()
}
