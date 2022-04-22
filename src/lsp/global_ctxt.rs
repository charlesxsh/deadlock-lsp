use std::{io::BufReader, error::Error, fs::File, collections::HashMap};

use lsp_types::{Position, DocumentHighlight, Diagnostic, DiagnosticSeverity, notification::PublishDiagnostics, PublishDiagnosticsParams};
use serde::{Serialize, Deserialize};

use crate::analysis::AnalysisResult;
use lsp_server::Message;
use lsp_server::Connection;
use crossbeam_channel::{Sender};


pub struct GlobalCtxt {
    pub result: Option<AnalysisResult>,
    pub sender: Sender<Message>
}



fn get_analysis_result(p: &String)->Result<AnalysisResult, Box<dyn Error>> {
    let file = File::open(p)?;
    let reader = BufReader::new(file);
    let r = serde_json::from_reader(reader)?;
    Ok(r)
}

impl GlobalCtxt {
    pub fn new(sender: Sender<Message>) -> Self {
        Self {
            result: None,
            sender,
        }
    }
    pub fn update_from_json(&mut self, p:&String) {
        match get_analysis_result(p) {
            Ok(result) => {
                self.result = Some(result)
            },
            Err(err) => {
                eprintln!("update analysis result: {}", err)
            },
        }
    }

    fn get_highlight(&self, file:&String, pos:&Position) -> Option<DocumentHighlight> {
        return None
    }
    fn get_diagnoistics(&self) -> Option<HashMap<String, Vec<Diagnostic>>> {
        let analysis = match &self.result {
            Some(r) => r,
            None => return None,
        };
        let mut result = HashMap::new();
        for call in &analysis.calls {
            for cc in &call.callchains {

                let d = Diagnostic {
                    range: lsp_types::Range { 
                        start: Position { line: cc.1, character: cc.2}, 
                        end: Position { line: cc.3, character: cc.4 }
                    },
                    severity: Some(DiagnosticSeverity::INFORMATION),
                    code: None,
                    code_description: None,
                    source: Some("rust deadlock analyzer".to_string()),
                    message: "call in critical section".to_string(),
                    related_information: None,
                    tags: None,
                    data: None,
                };

                result.get_mut(&cc.0).get_or_insert(&mut Vec::new())
                .push(d)
            }
            

        }
        return Some(result);
    }

    pub fn send_diagnoistic(&mut self) {
        match self.get_diagnoistics() {
            Some(file_diags) => {
                for (f, d) in file_diags {
                    let uri = lsp_types::Url::from_file_path(f).unwrap();
                    let params = PublishDiagnosticsParams::new(uri, d, Some(0));
                    self.send_notification::<lsp_types::notification::PublishDiagnostics>(params);
                }
            },
            None => {},
        }
    }

    pub fn send_notification<N: lsp_types::notification::Notification>(
        &mut self,
        params: N::Params,
    ) {
        let not = lsp_server::Notification::new(N::METHOD.to_string(), params);
        match self.sender.send(not.into()) {
            Ok(_) => {},
            Err(err) => {
                eprintln!("send noti: {}", err);
            },
        }
    }

}