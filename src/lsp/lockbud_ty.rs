// FIXME: this file should be synced with luckbud's src/cs/diagnostics.rs file.
// Ideally this project should either put together with luckbud or add luckbud as dependency at Cargo.toml, or extract interaces as independent crate.

use std::{error::Error, fs::File, io::BufReader};

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
pub type RangeInFile = (String, u32, u32, u32, u32);


#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct SuspiciousCall {
    pub callchains: Vec<RangeInFile>,
    pub ty: Suspicious,
}

#[derive(Debug, Serialize, Deserialize,  PartialEq, Eq)]
pub struct HighlightArea {
    pub triggers: Vec<RangeInFile>,
    // filename, start line & col, end line & col
    pub ranges: Vec<RangeInFile>
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnalysisResult {
    pub calls: Vec<SuspiciousCall>,
    pub critical_sections: Vec<HighlightArea>
}

impl AnalysisResult {

    pub fn from_file(p: &str)->Result<AnalysisResult, Box<dyn Error>> {
        let file = File::open(p)?;
        let reader = BufReader::new(file);
        let r = serde_json::from_reader(reader)?;
        Ok(r)
    }

    pub fn to_file(&self, output_path: &str) -> Result<(),Box<dyn Error>> {
        std::fs::write(
            output_path,
            serde_json::to_string_pretty(&self).unwrap(),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{fs};

    use super::*;

    /// 
    /// The test make sures the files and their highlight areas are properly
    /// handled.
    ///
    #[test]
    fn test_analysis_result_from_should_fail() {
        let res = AnalysisResult::from_file(&"not a valid path".to_string());
        assert_eq!(res.is_err(), true);

    }

    #[test]
    fn test_analysis_result_from_should_success() -> Result<(),Box<dyn Error>> {
        fs::create_dir_all(".tmp")?;

        let tmp_result_file = ".tmp/_test_result.json";
    
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

        let res = AnalysisResult::from_file(tmp_result_file);
        assert_eq!(res.is_err(), false);

        let parsed_res = res.unwrap();
        assert_eq!(parsed_res, result);

        Ok(())
    }

}