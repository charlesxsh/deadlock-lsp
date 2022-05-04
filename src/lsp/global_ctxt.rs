use std::{io::BufReader, error::Error, fs::File, collections::HashMap};

use lsp_types::{ Range, DocumentHighlightKind, Position, DocumentHighlight,DocumentHighlightParams, DiagnosticRelatedInformation, Location, Diagnostic, DiagnosticSeverity, notification::PublishDiagnostics, PublishDiagnosticsParams};
use serde::{Serialize, Deserialize};

use crate::analysis::AnalysisResult;
use crate::analysis::HighlightArea;
use lsp_server::Message;
use lsp_server::Connection;
use lsp_server::{RequestId};
use crossbeam_channel::{Sender};

type IndexedHighlights = HashMap<String, Vec<Vec<DocumentHighlight>>>;
pub struct GlobalCtxt {
    pub result: Option<AnalysisResult>,
    pub sender: Sender<Message>,
    pub file_highlights: IndexedHighlights
}



fn get_analysis_result(p: &String)->Result<AnalysisResult, Box<dyn Error>> {
    let file = File::open(p)?;
    let reader = BufReader::new(file);
    let r = serde_json::from_reader(reader)?;
    Ok(r)
}

fn raw_highlight_to_doc_highlights(raw: &Vec<HighlightArea>) -> IndexedHighlights {
    let mut ih: IndexedHighlights = HashMap::new();
    for r in raw {
        let mut highlights: Vec<DocumentHighlight> = Vec::new();
        let filename = &r.ranges.first().unwrap().0;
        for h in &r.ranges {
            let h: DocumentHighlight = DocumentHighlight {
                range: Range {
                    start: Position {
                        line: h.1-1,
                        character: h.2-1,
                    },
                    end: Position {
                        line: h.3-1,
                        character: h.4-1,
                    },
                },
                kind: Some(DocumentHighlightKind::TEXT)
            };
            highlights.push(h);
        }
        
        if !ih.contains_key(filename) {
            ih.insert(filename.to_string(), Vec::new());
        }
        ih.get_mut(filename).unwrap().push(highlights);
        
    }
    return ih;
}

impl GlobalCtxt {
    pub fn new(sender: Sender<Message>) -> Self {
        Self {
            result: None,
            sender,
            file_highlights: HashMap::new()
        }
    }
    pub fn update_from_json(&mut self, p:&String) {

        eprintln!("update analysis result: {}", p);

        match get_analysis_result(p) {
            Ok(result) => {
                self.file_highlights = raw_highlight_to_doc_highlights(&result.critical_sections);
                self.result = Some(result)
            },
            Err(err) => {
                eprintln!("update analysis result: {}", err)
            },
        }
    }



    fn get_highlights(&self, file:&String, pos:&Position) -> Option<Vec<DocumentHighlight>> {
        match self.file_highlights.get(file) {
            Some(areas) => {
                for area in areas {
                    for h in area {
                        if h.range.start.line <= pos.line && 
                        h.range.start.character <= pos.character &&
                        h.range.end.line >= pos.line &&
                        h.range.end.character >= pos.character {
                            return Some(area.to_vec());
                        }
                    }
                }
            },
            None => {},
        }

        return None
    }

    pub fn send_highlight(&mut self, id: RequestId, params: DocumentHighlightParams ) {
        match self.get_highlights(
            &params.text_document_position_params.text_document.uri.to_file_path().unwrap().to_str().unwrap().to_string(),
            &params.text_document_position_params.position,
        ) {
            Some(h) => {
                let res =  lsp_server::Response::new_ok(id, h);
        
                match self.sender.send(res.into()) {
                    Ok(()) => {
                        eprintln!("sent highlight!")
                    },
                    Err(err) => {
                        eprintln!("send highlight error: {:?}", err);
                    }
                }
            },
            None => {

            },
        }

        
    }
    fn get_diagnoistics(&self) -> Option<HashMap<String, Vec<Diagnostic>>> {
        let analysis = match &self.result {
            Some(r) => r,
            None => return None,
        };
        let mut result: HashMap<String, Vec<Diagnostic>> = HashMap::new();
        for call in &analysis.calls {
            if call.callchains.len() == 0 {
                eprintln!("unexpected callchain found {:?}", call);
            }
            let target = call.callchains.last().unwrap();
            let relateds = &call.callchains[..call.callchains.len()-1];
            let mut d = Diagnostic {
                range: lsp_types::Range { 
                    start: Position { line: target.1-1, character: target.2-1}, 
                    end: Position { line: target.3-1, character: target.4-1 }
                },
                severity: Some(DiagnosticSeverity::INFORMATION),
                code: None,
                code_description: None,
                source: Some("rust-deadlock-detector".to_string()),
                message: format!("{} in critical section", call.ty),
                related_information: None,
                tags: None,
                data: None,
            };

          
            let drelateds:Vec<DiagnosticRelatedInformation> = relateds.iter().map(|r|{
                let uri = lsp_types::Url::from_file_path(&r.0).unwrap();

                DiagnosticRelatedInformation {
                    location: Location {
                        uri,
                        range: lsp_types::Range { 
                            start: Position { line: r.1-1, character: r.2-1}, 
                            end: Position { line: r.3-1, character: r.4-1 }
                        }
                    },
                    message: "may contains blocking call in critical section".to_string(),
                }
            })
            .collect();

            if drelateds.len() > 0 {
                d.related_information = Some(drelateds);
            }


            if !result.contains_key(&target.0) {
                result.insert(target.0.clone(), Vec::new());
            }
            result.get_mut(&target.0).unwrap().push(d);
            

        }
        return Some(result);
    }
    pub fn send_message(&mut self) {
        self.send_notification::<lsp_types::notification::ShowMessage>(
            lsp_types::ShowMessageParams { typ: lsp_types::MessageType::INFO, message: "yooooo".to_string() },
        );
    }
    pub fn send_diagnoistic(&mut self) {
        
        match self.get_diagnoistics() {
            Some(file_diags) => {
                eprintln!("found diags {}", file_diags.len());

                for (f, d) in file_diags {
                    eprintln!("found {} diags for file {}",d.len(), f);
                    let uri = lsp_types::Url::from_file_path(f).unwrap();
                    let params = PublishDiagnosticsParams::new(uri, d, None);
                    self.send_notification::<lsp_types::notification::PublishDiagnostics>(params);
                }
            },
            None => {
                eprintln!("no diagnostic found {:?}", self.result);
            },
        }
    }

    pub fn send_notification<N: lsp_types::notification::Notification>(
        &mut self,
        params: N::Params,
    ) {
        let not = lsp_server::Notification::new(N::METHOD.to_string(), params);
        match self.sender.send(not.into()) {
            Ok(_) => {
                eprintln!("send noti successfully");
            },
            Err(err) => {
                eprintln!("send noti errorr: {}", err);
            },
        }
    }

}