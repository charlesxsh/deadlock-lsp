use std::{ collections::HashMap};

use lsp_types::{ Range, DocumentHighlightKind, Position, DocumentHighlight,DocumentHighlightParams, DiagnosticRelatedInformation, Location, Diagnostic, DiagnosticSeverity, PublishDiagnosticsParams};

use lsp_server::Message;
use lsp_server::{RequestId};
use crossbeam_channel::{Sender};

use super::lockbud_ty::{AnalysisResult, HighlightArea, RangeInFile, SuspiciousCall};

pub struct DocHighlightsWithTrigger {
    areas: Vec<DocumentHighlight>,
    triggers: Vec<RangeInFile>
}

type IndexedHighlights = HashMap<String, Vec< DocHighlightsWithTrigger > >;
type IndexedDiagnostics = HashMap<String, Vec< Diagnostic > >;

pub struct GlobalCtxt {
    pub result: Option<AnalysisResult>,
    pub sender: Sender<Message>,
    pub file_highlights: IndexedHighlights
}


impl GlobalCtxt {
    pub fn new(sender: Sender<Message>) -> Self {
        Self {
            result: None,
            sender,
            file_highlights: HashMap::new()
        }
    }
    pub fn update_from_json(&mut self, p:&str) {

        eprintln!("update analysis result: {}", p);

        match AnalysisResult::from_file(p) {
            Ok(result) => {
                self.update_from_analysis_result(result)
            },
            Err(err) => {
                eprintln!("update analysis result: {}", err)
            },
        }
    }

    fn update_from_analysis_result(&mut self, result: AnalysisResult) {
        self.file_highlights = raw_highlight_to_doc_highlights(&result.critical_sections);
        self.result = Some(result)
    }



    fn get_highlights(&self, file:&str, pos:&Position) -> Option<Vec<DocumentHighlight>> {
        eprintln!("finding highlight at {:?} {:?}", file, pos);

        match self.file_highlights.get(file) {
            Some(areas) => {

                for area in areas {
                    for h in &area.triggers {
                        if h.1 <= pos.line && 
                        h.2 <= pos.character &&
                        h.3 >= pos.line &&
                        h.4 >= pos.character {
                            eprintln!("found highlight with {:?}", area.triggers);

                            return Some(area.areas.to_vec());
                        }
                    }
                }
            },
            None => {},
        }
        eprintln!("no found highlight");

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
        
        let result = suspicious_calls_to_diagnostics(&analysis.calls);
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
            },
            Err(err) => {
                eprintln!("send noti errorr: {}", err);
            },
        }
    }

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

        let zero_based_triggers: Vec<RangeInFile> = r.triggers.iter().map(|t| (t.0.clone(), t.1-1, t.2-1, t.3-1, t.4-1)).collect();
        ih.get_mut(filename).unwrap().push(DocHighlightsWithTrigger { areas: highlights, triggers: zero_based_triggers } );
        
    }
    return ih;
}


fn suspicious_calls_to_diagnostics(calls: &Vec<SuspiciousCall>) ->IndexedDiagnostics {
    let mut result: IndexedDiagnostics = HashMap::new();
    for call in calls {
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
            message: format!("{:?} in critical section", call.ty),
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

    result
}

#[cfg(test)]
mod tests {
    use std::{fs, error::Error};

    use crossbeam_channel::unbounded;

    use crate::lsp::lockbud_ty::Suspicious;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;


    #[test]
    fn test_global_ctx_init() {
        let (s1, _) = unbounded();
        let ctx = GlobalCtxt::new(s1);
        assert!(ctx.result.is_none());
        assert!(ctx.file_highlights.is_empty());
    }


    #[test]
    fn test_global_ctx_update_from_file() -> Result<(),Box<dyn Error>>  {
        fs::create_dir_all(".tmp")?;

        let tmp_result_file = ".tmp/_test_result1.json";
    
        let mut calls: Vec<SuspiciousCall> = Vec::new();
        calls.push(SuspiciousCall { callchains: vec![
            ("/some/file1.rs".to_string(), 4, 5, 6, 7)
        ], ty: Suspicious::DoubleLock });
        calls.push(SuspiciousCall { callchains: vec![
            ("/some/file1.rs".to_string(), 6, 7, 8, 9)
        ], ty: Suspicious::ConflictLock });

        calls.push(SuspiciousCall { callchains: vec![
            ("/some/file2.rs".to_string(), 6, 7, 8, 9)
        ], ty: Suspicious::ChRecv });

        calls.push(SuspiciousCall { callchains: vec![
            ("/some/file3.rs".to_string(), 6, 7, 8, 9)
        ], ty: Suspicious::ChSend });

        calls.push(SuspiciousCall { callchains: vec![
            ("/some/file4.rs".to_string(), 6, 7, 8, 9)
        ], ty: Suspicious::CondVarWait });

        let result = AnalysisResult {
            calls,
            critical_sections: vec![
                HighlightArea { triggers: vec![
                    ("file1.rs".to_string(), 1, 2, 3, 4)
                ], ranges: vec![
                    ("file1.rs".to_string(), 5, 6, 7, 8)
                ] },
    
                HighlightArea { triggers: vec![
                    ("file2.rs".to_string(), 1, 2, 3, 4)
                ], ranges: vec![
                    ("file2.rs".to_string(), 5, 6, 7, 8)
                ] }
            ],
        };

        result.to_file(tmp_result_file).unwrap();
        let (s1, _) = unbounded();
        let mut ctx = GlobalCtxt::new(s1);
        assert!(ctx.result.is_none());
        assert!(ctx.file_highlights.is_empty());

        ctx.update_from_json(tmp_result_file);

        assert!(ctx.result.is_some());
        assert!(!ctx.file_highlights.is_empty());


        
        Ok(())
    }

    #[test]
    fn test_global_ctx_get_heightlights() -> Result<(),Box<dyn Error>>  {

        let result = AnalysisResult {
            calls:Vec::new(),
            critical_sections: vec![
                HighlightArea { triggers: vec![
                    ("file1.rs".to_string(), 1, 2, 3, 4)
                ], ranges: vec![
                    ("file1.rs".to_string(), 5, 6, 7, 8)
                ] },
    
                HighlightArea { triggers: vec![
                    ("file2.rs".to_string(), 1, 2, 3, 4)
                ], ranges: vec![
                    ("file2.rs".to_string(), 5, 6, 7, 8)
                ] }
            ],
        };

        let (s1, _) = unbounded();
        let mut ctx = GlobalCtxt::new(s1);
        ctx.update_from_analysis_result(result);

        let areas = ctx.get_highlights("file1.rs", &Position { line: 1, character: 3 });
        
        assert!(areas.is_some());
        assert_eq!(areas.unwrap().len(), 1);
        Ok(())
    }


    /// 
    /// The test make sures the files and their highlight areas are properly
    /// handled.
    ///
    #[test]
    fn test_raw_highlight_to_doc_highlights_file_index() {
        
        
        let raw_highlights = vec![
            HighlightArea { triggers: vec![
                ("file1.rs".to_string(), 1, 2, 3, 4)
            ], ranges: vec![
                ("file1.rs".to_string(), 5, 6, 7, 8)
            ] },

            HighlightArea { triggers: vec![
                ("file2.rs".to_string(), 1, 2, 3, 4)
            ], ranges: vec![
                ("file2.rs".to_string(), 5, 6, 7, 8)
            ] }
        ];
        let res = raw_highlight_to_doc_highlights(&raw_highlights);

        assert_eq!(2, res.len());
        assert_eq!(1, res.get("file1.rs").unwrap().len());
        assert_eq!(1, res.get("file2.rs").unwrap().len());
        
        
    }

    /// 
    /// The test make sures the line and column numbers are properly
    /// handled considered to the difference of source code (1 based) and LSP (0 based)
    ///
    #[test]
    fn test_raw_highlight_to_doc_highlights_linecol() {
        
        
        let raw_highlights = vec![
            HighlightArea { triggers: vec![
                ("file1.rs".to_string(), 1, 2, 3, 4)
            ], ranges: vec![
                ("file1.rs".to_string(), 5, 6, 7, 8)
            ] },

            HighlightArea { triggers: vec![
                ("file2.rs".to_string(), 1, 2, 3, 4)
            ], ranges: vec![
                ("file2.rs".to_string(), 5, 6, 7, 8)
            ] }
        ];
        let res = raw_highlight_to_doc_highlights(&raw_highlights);

        let highlight_with_trigger = res.get("file1.rs").unwrap();
        let trigger = highlight_with_trigger.first().unwrap().triggers.first().unwrap();
        assert_eq!(trigger.0, "file1.rs");
        assert_eq!(trigger.1, 0);
        assert_eq!(trigger.2, 1);
        assert_eq!(trigger.3, 2);
        assert_eq!(trigger.4, 3);
        
        let area = highlight_with_trigger.first().unwrap().areas.first().unwrap();

        assert_eq!(area.range.start.line, 4);
        assert_eq!(area.range.start.character, 5);
        assert_eq!(area.range.end.line, 6);
        assert_eq!(area.range.end.character, 7);
        
    }

   
    ///
    /// Test suspicious calls (with callchain depth 1) dump by luckbud can be properly transfered to LSP's diagnostic.
    /// 
    #[test]
    fn test_suspicious_calls_to_diagnostics_one_callchain() {
        let mut calls: Vec<SuspiciousCall> = Vec::new();
        calls.push(SuspiciousCall { callchains: vec![
            ("/some/file1.rs".to_string(), 4, 5, 6, 7)
        ], ty: Suspicious::DoubleLock });
        calls.push(SuspiciousCall { callchains: vec![
            ("/some/file1.rs".to_string(), 6, 7, 8, 9)
        ], ty: Suspicious::ConflictLock });
        let result = suspicious_calls_to_diagnostics(&calls);
        
        assert_eq!(result.len(), 1);
        assert_eq!(result.get("/some/file1.rs").unwrap().len(), 2);

        let diags = result.get("/some/file1.rs").unwrap();
        let first_diag = diags.first().unwrap();
        assert_eq!(first_diag.range.start.line, 3);
        assert_eq!(first_diag.range.start.character, 4);
        assert_eq!(first_diag.range.end.line, 5);
        assert_eq!(first_diag.range.end.character, 6);
    }

    ///
    /// Test suspicious calls (with callchain depth more than 1) dump by luckbud can be properly transfered to LSP's diagnostic.
    /// 
    #[test]
    fn test_suspicious_calls_to_diagnostics() {
        let mut calls: Vec<SuspiciousCall> = Vec::new();
        calls.push(SuspiciousCall { callchains: vec![
            ("/some/file1.rs".to_string(), 1, 2, 3, 4),
            ("/some/file1.rs".to_string(), 4, 5, 6, 7)
        ], ty: Suspicious::DoubleLock });
        calls.push(SuspiciousCall { callchains: vec![
            ("/some/file1.rs".to_string(), 2, 3, 4, 5),
            ("/some/file1.rs".to_string(), 6, 7, 8, 9)
        ], ty: Suspicious::ConflictLock });
        let result = suspicious_calls_to_diagnostics(&calls);
        
        assert_eq!(result.len(), 1);
        assert_eq!(result.get("/some/file1.rs").unwrap().len(), 2);

        let diags = result.get("/some/file1.rs").unwrap();
        let first_diag = diags.first().unwrap();
        assert_eq!(first_diag.range.start.line, 3);
        assert_eq!(first_diag.range.start.character, 4);
        assert_eq!(first_diag.range.end.line, 5);
        assert_eq!(first_diag.range.end.character, 6);

        let first_related:&Vec<DiagnosticRelatedInformation> = first_diag.related_information.as_ref().unwrap();
        let first_loc = &first_related.first().unwrap().location;
        assert_eq!(first_loc.range.start.line, 0);
        assert_eq!(first_loc.range.start.character, 1);
        assert_eq!(first_loc.range.end.line, 2);
        assert_eq!(first_loc.range.end.character, 3);
    }
    
}