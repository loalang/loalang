use crate::server_handler::*;

pub struct FormattingRequestHandler;

impl RequestHandler for FormattingRequestHandler {
    type R = request::Formatting;

    fn handle(
        context: &mut ServerContext,
        params: DocumentFormattingParams,
    ) -> Option<Vec<TextEdit>> {
        let uri = convert::from_lsp::url_to_uri(&params.text_document.uri);
        let cell = context.server.module_cells.get(&uri)?;
        let mut indent;

        if params.options.insert_spaces {
            indent = String::new();
            for _ in 0..params.options.tab_size {
                indent.push(' ');
            }
        } else {
            indent = "\t".into();
        }

        let new_text = format::Formatter::format(cell.tree.as_ref(), indent.as_ref());

        Some(vec![TextEdit {
            range: convert::from_loa::span_to_range(Span::all_of(&cell.source)),
            new_text,
        }])
    }
}
