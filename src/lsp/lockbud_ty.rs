// FIXME: this file should be synced with luckbud's src/cs/diagnostics.rs file.
// Ideally this project should either put together with luckbud or add luckbud as dependency at Cargo.toml, or extract interaces as independent crate.

use std::collections::HashSet;
use serde::{Serialize, Deserialize};



#[derive(Hash, Eq, PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum Suspicious {
    ChSend,
    ChRecv,
    CondVarWait,
    DoubleLock,
    ConflictLock
}

// filename, start line & col, end line & col
type RangeInFile = (String, u32, u32, u32, u32);


#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct SuspiciousCall {
    pub callchains: Vec<RangeInFile>,
    pub ty: Suspicious,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HighlightArea {
    pub triggers: Vec<RangeInFile>,
    // filename, start line & col, end line & col
    pub ranges: Vec<RangeInFile>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub calls: HashSet<SuspiciousCall>,
    pub critical_sections: Vec<HighlightArea>
}