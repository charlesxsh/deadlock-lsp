use std::{error::Error, time::Instant, path::PathBuf};

use lsp_types::{
    request::DocumentHighlightRequest, InitializeParams, notification::DidSaveTextDocument,
};

use lsp_server::{Connection, Message};
use deadlock_lsp::{lsp::{global_ctxt, get_capabilities, cast_notification, cast_request}, utils::{run_analysis_in_dir, cargo_clean}};
fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    // Note that  we must have our logging only write out to stderr.
    eprintln!("starting generic LSP server");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    
    let initialization_params = connection.initialize(get_capabilities())?;
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
    if let Some(client_info) = _params.client_info {
        eprintln!("client_info: {:?}", client_info);
    }

    let workspace_roots: Vec<PathBuf> = _params.workspace_folders.iter()
    .flat_map(|folders| folders.iter())
    .flat_map(|folder|folder.uri.to_file_path().ok())
    .collect();

    eprintln!("workspace roots: {:?}", workspace_roots);

    for workspace in &workspace_roots {
        let start = Instant::now();

        eprintln!("init at workspace {:?}", workspace);

        let analysis_out = workspace.join(".rda/a.json")
        .to_str().unwrap().to_string();
        let _ = std::fs::remove_file(&analysis_out);
        let wsstr = workspace.to_str().unwrap();
        cargo_clean(wsstr);
        run_analysis_in_dir(wsstr, &analysis_out);
        ctx.update_from_json(&analysis_out);
        ctx.send_diagnoistic();

        let elapsed_time = start.elapsed().as_millis();
        eprintln!("init at workspace {:?} took {}ms", workspace, elapsed_time);

    }

    for msg in &connection.receiver {
        // eprintln!("got msg: {:?}", msg);
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                
                match cast_request::<DocumentHighlightRequest>(req) {
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
                match cast_notification::<DidSaveTextDocument>(not.clone()) {
                    Ok(params) => {
                        eprintln!("{:?} saved!", params.text_document);
                        eprintln!("connection.sender {:?}", connection.sender.len());
                        match &_params.workspace_folders {
                            Some(workspaces) => {
                                let start = Instant::now();

                                let ws = &workspaces[0];
                                let analysis_out = ws.uri.to_file_path().unwrap().join(".rda/a.json")
                                .to_str().unwrap().to_string();
                                let _ = std::fs::remove_file(&analysis_out);
                                run_analysis_in_dir(ws.uri.to_file_path().unwrap().to_str().unwrap(), &analysis_out);
                                ctx.update_from_json(&analysis_out);
                                ctx.send_diagnoistic();

                                let elapsed_time = start.elapsed().as_millis();
                                eprintln!("DidSaveTextDocument took {}ms", elapsed_time);

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

