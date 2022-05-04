use std::{error::Error, process::{Command, Stdio}, env, ffi::OsString, fs};

use log::info;
use lsp_types::{
    request::{GotoDefinition, DocumentHighlightRequest}, GotoDefinitionResponse, InitializeParams, ServerCapabilities, OneOf, SelectionRangeProviderCapability, HoverProviderCapability, TextDocumentSyncCapability, TextDocumentSyncOptions, SaveOptions, notification::DidSaveTextDocument, DocumentHighlight, Range, Position, DocumentHighlightKind, DocumentHighlightParams,
};

use lsp_server::{Connection, Message, Request, RequestId, Response, Notification, ExtractError};
use serde::Deserialize;
use deadlock_lsp::lsp::global_ctxt;
fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    // Note that  we must have our logging only write out to stderr.
    eprintln!("starting generic LSP server");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(
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
    ).unwrap();
    let initialization_params = connection.initialize(server_capabilities)?;
    main_loop(connection, initialization_params)?;
    io_threads.join()?;

    // Shut down gracefully.
    eprintln!("shutting down server");
    Ok(())
}

fn main_loop(
    connection: Connection,
    params: serde_json::Value,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    let mut ctx = global_ctxt::GlobalCtxt::new(connection.sender.clone());
    
    for msg in &connection.receiver {
        // eprintln!("got msg: {:?}", msg);
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                // eprintln!("got request: {:?}", req);
                // match cast::<GotoDefinition>(&req) {
                //     Ok((id, params)) => {
                //         eprintln!("got goto def request #{}: {:?}", id, params);
                //         let result = Some(GotoDefinitionResponse::Array(Vec::new()));
                //         let result = serde_json::to_value(&result).unwrap();
                //         let resp = Response { id, result: Some(result), error: None };
                //         connection.sender.send(Message::Response(resp))?;
                //         continue;
                //     }
                //     Err(err) => {},
                // };
                
                match cast::<DocumentHighlightRequest>(req) {
                    Ok((id, params)) => {
                        ctx.send_highlight(id, params);
                    },
                    Err(err) => {
                        eprintln!("parse highlight error: {:?}", err);
                    },
                }
                // ...
            }
            Message::Response(resp) => {
                eprintln!("got response: {:?}", resp);
            }
            Message::Notification(not) => {
                eprintln!("got notification: {:?}", not);
                // match cast_notification::<lsp_types::notification::DidOpenTextDocument>(not.clone()) {
                //     Ok(params) => {
                //         eprintln!("DidOpenTextDocument {:?} ", params);
                //     }
                //     Err(_) => {}
                // }
                match cast_notification::<DidSaveTextDocument>(not.clone()) {
                    Ok(params) => {
                        eprintln!("{:?} saved!", params.text_document);
                        eprintln!("connection.sender {:?}", connection.sender.len());
                        match &_params.workspace_folders {
                            Some(workspaces) => {
                                
                                let ws = &workspaces[0];
                                let analysis_out = ws.uri.to_file_path().unwrap().join(".rda/a.json")
                                .to_str().unwrap().to_string();
                                std::fs::remove_file(&analysis_out);
                                run_analysis_in_dir(ws.uri.to_file_path().unwrap().to_str().unwrap(), &analysis_out);
                                ctx.update_from_json(&analysis_out);
                                ctx.send_diagnoistic();
                            },
                            None => {},
                        }
                        
                    },
                    Err(_) => {}
                }
            }
        }
    }
    Ok(())
}

fn cast<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}


fn cast_notification<R>(req: Notification) -> Result<R::Params, ExtractError<Notification>>
where
    R: lsp_types::notification::Notification,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}

fn cargo() -> Command {
    Command::new(env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo")))
}

#[derive(Debug, Deserialize)]
struct Cargo {
    package: CargoPackage,
}

#[derive(Debug, Deserialize)]
struct CargoPackage {
    name: String
}


fn run_analysis_in_dir(dir: &str, out: &String) {
    let ws_dir = std::path::Path::new(dir);
    eprintln!("running analysis in directory: {}", dir);
    let mut crate_name = None;
    let cargo_toml_fp = ws_dir.join("Cargo.toml");
    match fs::read_to_string(&cargo_toml_fp) {
        Ok(toml_str) => {
            let decoded: Cargo = toml::from_str(&toml_str).unwrap();
            crate_name = Some(decoded.package.name)
        },
        Err(err) => {
            eprintln!("read {:?}: {}", cargo_toml_fp, err)
        },
    }

    let mut cmd = cargo();

    let dl_rustc = env::var_os("__DL_RUSTC").unwrap_or(OsString::from(""));
    cmd.env("RUSTC", dl_rustc);
    cmd.env("__DL_CRATE", crate_name.unwrap_or("".to_string()));
    cmd.env("__DL_OUT", out);
    cmd.arg("check");
    cmd.current_dir(ws_dir);
    cmd.stdout(Stdio::null());
    eprintln!("{:?} in {:?}", cmd, ws_dir);
    let exit_status = cmd
        .spawn()
        .expect("could not run cargo")
        .wait()
        .expect("failed to wait for cargo?");
    if !exit_status.success() {
        eprintln!("cargo error code: {}", exit_status.code().unwrap_or(-1));
    };
}