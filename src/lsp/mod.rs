use lsp_server::{RequestId, Request, ExtractError, Notification};
use lsp_types::{ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncOptions, SelectionRangeProviderCapability, OneOf, SaveOptions};
use serde_json::Value;


pub mod global_ctxt;
pub mod lockbud_ty;


pub fn get_capabilities() -> Value {
    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    serde_json::to_value(
        &ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Options(TextDocumentSyncOptions {
                will_save: None,
                will_save_wait_until: None,
                save: Some(SaveOptions::default().into()),
                open_close: None,
                change: None,
            })),
            selection_range_provider: Some(SelectionRangeProviderCapability::Simple(true)),
            document_highlight_provider: Some(OneOf::Left(true)),
            
            ..Default::default()
        }
    ).unwrap()
}

pub fn cast_request<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}


pub fn cast_notification<R>(req: Notification) -> Result<R::Params, ExtractError<Notification>>
where
    R: lsp_types::notification::Notification,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}

#[cfg(test)]
mod tests {

    use lsp_types::{request::DocumentHighlightRequest, notification::DidSaveTextDocument};

    use super::*;

    #[test]
    fn test_get_capabilities() {
        let res = get_capabilities();
        assert!(res.get("documentHighlightProvider").is_some());

    }

    #[test]
    fn test_cast_request_failed() {
        let res = cast_request::<DocumentHighlightRequest>(Request { id: RequestId::from(123), method: "adfasdf".to_string(), params: Default::default() });
        assert!(res.is_err());
    }

    #[test]
    fn test_cast_notification_failed() {
        let res = cast_notification::<DidSaveTextDocument>(Notification { method: "adfasdf".to_string(), params: Default::default() } );
        assert!(res.is_err());
    }
}