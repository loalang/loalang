use crate::repl::highlight;
use colored::*;
use loa::semantics::Navigator;
use loa::*;

pub struct PrettyReporter;

impl PrettyReporter {
    fn header(diagnostic: &Diagnostic) -> String {
        let uri_row = format!("{}:", diagnostic.span().start.uri);
        let message_row = format!("{:?}", diagnostic);

        format!("{}\n{}\n", message_row.red(), uri_row.bright_black())
    }

    fn code_frame(diagnostic: &Diagnostic, source: Arc<Source>) -> String {
        let span = diagnostic.span();
        let mut start_line = span.start.line;
        let mut end_line = span.end.line;

        let code = highlight(source.clone(), vec![
            (match diagnostic.level() {
                DiagnosticLevel::Error => Color::BrightRed,
                DiagnosticLevel::Warning => Color::Yellow,
                DiagnosticLevel::Info => Color::Cyan,
            }, span.clone())
        ]);
        let lines: Vec<_> = code.split("\n").collect();

        for _ in 0..4 {
            if start_line > 0 {
                start_line -= 1;
            }
            if end_line < lines.len() {
                end_line += 1;
            }
        }

        let mut formatted_lines = String::new();

        for (i, line) in lines[start_line..end_line].iter().enumerate() {
            let line_number = start_line + i + 1;
            formatted_lines.push_str(Self::code_frame_line(line, line_number).as_str());
            formatted_lines.push('\n');
        }

        format!("{}", formatted_lines)
    }

    fn code_frame_line(line: &str, n: usize) -> String {
        let line_number_column = format!("{:>3} |", n);

        format!("{} {}", line_number_column.bright_black(), line)
    }
}

impl Reporter for PrettyReporter {
    fn report(diagnostic: Diagnostic, navigator: &Navigator) {
        let mut result = String::new();
        result.push_str(Self::header(&diagnostic).as_str());
        if let Some(source) = navigator.source(&diagnostic.span().start.uri) {
            result.push_str(Self::code_frame(&diagnostic, source).as_str());
        }
        println!("{}", result);
    }
}
