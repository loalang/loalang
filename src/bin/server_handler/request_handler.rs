use crate::server_handler::ServerContext;
use lsp_types::request::Request;

pub trait RequestHandler {
    type R: Request;

    fn handle(
        context: &mut ServerContext,
        params: <Self::R as Request>::Params,
    ) -> <Self::R as Request>::Result;
}
