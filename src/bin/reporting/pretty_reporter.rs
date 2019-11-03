use crate::repl::highlight;
use colored::*;
use loa::semantics::Navigator;
use loa::*;

const GUTTER_SEPARATOR: &str = "|";
const FOLDED_GUTTER_SEPARATOR: &str = "╎";
const FOLDED_BEGINNING_GUTTER_SEPARATOR: &str = "↑";
const FOLDED_ENDING_GUTTER_SEPARATOR: &str = "↓";

pub struct PrettyReporter;

impl PrettyReporter {
    fn header(uri: &URI) -> String {
        let uri_row = format!("{}:\n", uri);

        uri_row.bright_black().to_string()
    }

    fn color_of_diagnostic(diagnostic: &Diagnostic) -> Color {
        match diagnostic.level() {
            DiagnosticLevel::Error => Color::BrightRed,
            DiagnosticLevel::Warning => Color::Yellow,
            DiagnosticLevel::Info => Color::Cyan,
        }
    }

    fn code_frames(diagnostics: Vec<Diagnostic>, source: Arc<Source>) -> String {
        let markers = diagnostics
            .iter()
            .map(|d| (Self::color_of_diagnostic(d), d.span().clone()))
            .collect::<Vec<_>>();

        let code = highlight(source.clone(), markers);
        let lines: Vec<_> = code.split("\n").collect();
        let lines_count = lines.len();

        let line_should_fold = |n: usize| {
            for diagnostic in diagnostics.iter() {
                let s = diagnostic.span();

                let mut start_line = s.start.line;
                let mut end_line = s.end.line;

                for _ in 0..3 {
                    if start_line > 0 {
                        start_line -= 1;
                    }
                    if end_line < lines_count {
                        end_line += 1;
                    }
                }

                if n >= start_line && n <= end_line {
                    return false;
                }
            }
            true
        };

        let mut formatted_lines = String::new();

        let gutter_width = lines_count.to_string().len() + 1;

        let mut previous_line_was_folded = false;
        for (i, line) in lines.iter().enumerate() {
            let line_number = i + 1;

            if line_should_fold(line_number) {
                previous_line_was_folded = true;
                continue;
            }

            if previous_line_was_folded {
                formatted_lines.push_str(
                    format!(
                        "{:gutter$}{pipe}\n",
                        "",
                        pipe = if formatted_lines.is_empty() {
                            FOLDED_BEGINNING_GUTTER_SEPARATOR.bright_black()
                        } else {
                            FOLDED_GUTTER_SEPARATOR.bright_black()
                        },
                        gutter = gutter_width,
                    )
                    .as_str(),
                )
            }

            previous_line_was_folded = false;

            let mut color = Color::BrightBlack;
            for d in diagnostics.iter() {
                if d.span().start.line == line_number {
                    color = Self::color_of_diagnostic(d);
                    break;
                }
            }

            formatted_lines
                .push_str(Self::code_frame_line(line, line_number, lines_count, color).as_str());
            formatted_lines.push('\n');

            for diagnostic in diagnostics.iter().rev() {
                let d = diagnostic.span();
                if line_number == d.end.line {
                    formatted_lines.push_str(
                        format!(
                            "{:gutter$}{pipe}{:space$}{}\n",
                            "",
                            "",
                            format!("↑ {:?}", diagnostic)
                                .color(Self::color_of_diagnostic(diagnostic)),
                            pipe = FOLDED_GUTTER_SEPARATOR.bright_black(),
                            gutter = gutter_width,
                            space = d.start.character
                        )
                        .as_str(),
                    );
                }
            }
        }

        if previous_line_was_folded {
            formatted_lines.push_str(
                format!(
                    "{:gutter$}{pipe}\n",
                    "",
                    pipe = FOLDED_ENDING_GUTTER_SEPARATOR.bright_black(),
                    gutter = gutter_width,
                )
                .as_str(),
            );
        }

        format!("{}", formatted_lines)
    }

    fn code_frame_line(line: &str, n: usize, end_line: usize, color: Color) -> String {
        let line_number_column = format!("{:>width$}", n, width = end_line.to_string().len());

        format!(
            "{} {} {}",
            line_number_column.color(color),
            GUTTER_SEPARATOR.bright_black(),
            line
        )
    }
}

impl Reporter for PrettyReporter {
    fn report(diagnostics: Vec<Diagnostic>, navigator: &Navigator) {
        let mut by_uri = HashMap::new();
        for diagnostic in diagnostics {
            let uri = &diagnostic.span().start.uri;
            if !by_uri.contains_key(uri) {
                by_uri.insert(uri.clone(), vec![]);
            }
            by_uri.get_mut(uri).unwrap().push(diagnostic);
        }

        for (uri, diagnostics) in by_uri {
            let mut result = String::new();
            result.push_str(Self::header(&uri).as_str());
            if let Some(source) = navigator.source(&uri) {
                result.push_str(Self::code_frames(diagnostics, source).as_str());
            }
            print!("{}", result);
        }
    }
}
