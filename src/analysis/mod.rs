use std::{collections::{HashMap, HashSet}, rc::Rc, cell::RefCell, fs::{File, self}, env, ffi::OsString};

use log::info;
use rustc_hir::def_id::{LOCAL_CRATE, LocalDefId, DefId};
use rustc_middle::{ty::{TyCtxt, Ty, TyKind::FnDef}, mir::{Body, Local, Terminator, StatementKind, TerminatorKind}};
use serde::{Serialize, Deserialize};

use crate::analysis::call_graph::analyze_callgraph;

use self::{ty::{Lifetimes, Lifetime}, lifetime::analyze_lifetimes, lock::parse_lockguard_type, call_graph::CallSite, range::parse_span};


mod ty;
mod lifetime;
mod lock;
mod call_graph;
mod range;

#[derive(Hash, Eq, PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum CriticalSectionCall {
    ChSend,
    ChRecv,
    CondVarWait
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HighlightArea {
    // filename, start line & col, end line & col
    pub ranges: Vec<(String, u32, u32, u32, u32)>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallInCriticalSection {
    // filename, start line & col, end line & col
    pub callchains: Vec<(String, u32, u32, u32, u32)>,
    pub ty: CriticalSectionCall,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub calls: Vec<CallInCriticalSection>,
    pub critical_sections: Vec<HighlightArea>
}

fn callchains_to_spans<'tcx>(callchains:& Vec<CallSite<'tcx>>) -> Vec<(String, u32, u32, u32, u32)> {
    callchains.iter()
    .map(|c| {
        let (filename, rg) = parse_span(&c.span);
        return (filename, rg.0.0, rg.0.1, rg.1.0, rg.1.1)
    })
    .collect()
}

fn lifetime_to_highlight_area(l: &Lifetime) -> HighlightArea {
    HighlightArea {
        ranges:l.live_span.iter()
        .map(|c| {
            let (filename, rg) = parse_span(&c);
            return (filename, rg.0.0, rg.0.1, rg.1.0, rg.1.1)
        })
        .collect(),
    }
}

pub fn filter_body_locals(body: &Body, filter: fn(Ty) -> bool) -> Vec<Local> {
    body.local_decls
    .iter_enumerated()
    .filter_map(|(local, decl)|{
        if filter(decl.ty) {
            return Some(local)
        } 

        None
    })
    .collect()
}

type CSCallFilter<'tcx> = dyn Fn(TyCtxt<'tcx>, &CallSite) -> bool;
type CSCallFilterSet<'tcx> = HashMap<CriticalSectionCall, &'tcx CSCallFilter<'tcx>>;

pub fn find_in_lifetime<'tcx, 'a>(tcx: TyCtxt<'tcx>, body: &'a Body<'tcx>, lt:&Lifetime, callgraph: &call_graph::CallGraph<'tcx>, cs_calls:& mut Vec<CallInCriticalSection>, callchains:Vec<CallSite<'tcx>>, filter_set:&CSCallFilterSet<'tcx>) {
    let callsites = &callgraph.callsites[&body.source.def_id()];
    for cs in callsites {
        let callee_id = cs.callee.def_id();
        let mut cs_call_type: Option<CriticalSectionCall> = None;
        for (t, f) in filter_set {
            if f(tcx, cs) {
                cs_call_type = Some(*t);
                break;
            }
        }
        // if callchain is 0, means it is not in the critical section yet
        if callchains.len() == 0 {
            for loc in &lt.live_locs {
                if cs.location != *loc {
                    continue
                }
                
                if cs_call_type != None {
                    // if this call is in critical section and is our interests
                    let mut new_cc = callchains.clone();
                    new_cc.push(cs.clone());
                    cs_calls.push(CallInCriticalSection{
                        callchains: callchains_to_spans(&new_cc),
                        ty:cs_call_type.unwrap(),
                    })
                } else {
                    // if this call is not in critical section and is not our interests

                    let mut new_cc = callchains.clone();
                    new_cc.push(cs.clone());
                    let callee_body = tcx.optimized_mir(callee_id);
                    find_in_lifetime(tcx, callee_body, lt, callgraph, cs_calls, new_cc, filter_set)
                }
            }
        } else {
            let mut new_cc = callchains.clone();
            new_cc.push(cs.clone());
            if cs_call_type != None {
                // if this call is in critical section and is our interests
                cs_calls.push(CallInCriticalSection{
                    callchains: callchains_to_spans(&new_cc),
                    ty: cs_call_type.unwrap(),
                })
            }
        }
        
    }
}


pub fn analyze(tcx: TyCtxt) -> Result<AnalysisResult, Box<dyn std::error::Error>>  {
    let crate_name = tcx.crate_name(LOCAL_CRATE).to_string();
    let trimmed_name = crate_name.trim_matches('\"');
    let dl_crate = env::var_os("__DL_CRATE").unwrap_or(OsString::from(""));
    let dl_out = env::var_os("__DL_OUT").unwrap_or(OsString::from(""));
    let mut result = AnalysisResult {
        calls: Vec::new(),
        critical_sections: Vec::new(),
    };
    if dl_crate != trimmed_name {
        return Ok(result)
    }
    info!("deadlock analyzing crate {:?}", trimmed_name);
    
    
    let fn_ids: Vec<LocalDefId> = tcx.mir_keys(())
    .iter()
    .filter(|id| {
        let hir = tcx.hir();
        hir.body_owner_kind(hir.local_def_id_to_hir_id(**id))
            .is_fn_or_closure()
    })
    .copied()
    .collect();

    info!("functions: {}", fn_ids.len());
    let lifetimes = Rc::new(RefCell::new(Lifetimes::new()));
    let mut callgraph = call_graph::CallGraph::new();
    fn_ids
    .clone()
    .into_iter()
    .for_each(|fn_id| {
        // println!("{:?}", fn_id);
        let body = tcx.optimized_mir(fn_id);
        analyze_lifetimes(tcx, body, lifetimes.clone());
        analyze_callgraph(tcx, body, &mut callgraph);
    });

    // fill critical section into result
    let mut all_lifetime:Vec<Lifetime> = (&lifetimes.borrow().body_local_lifetimes).values().into_iter().map(|hm|hm.values()).flatten().map(|l| l.clone()).collect();
    let mut areas:Vec<HighlightArea> = all_lifetime.iter().map(|l| lifetime_to_highlight_area(l)).collect();
    result.critical_sections.append(&mut areas);
    
    let mut filter_set: CSCallFilterSet = HashMap::new();

    filter_set.insert(CriticalSectionCall::ChSend, &|tcx, cs|{
        match cs.call_by_type {
            Some(caller_ty) => {
                let fname = tcx.item_name(cs.callee.def_id());
                let caller_ty_name = caller_ty.to_string();
                return caller_ty_name.contains("std::sync::mpsc::Sender") && fname.to_string() == "send";
            },
            None => false,
        }
    } );

    filter_set.insert(CriticalSectionCall::ChRecv, &|tcx, cs|{
        match cs.call_by_type {
            Some(caller_ty) => {
                let fname = tcx.item_name(cs.callee.def_id());
                let caller_ty_name = caller_ty.to_string();
                return caller_ty_name.contains("std::sync::mpsc::Receiver") && fname.to_string() == "recv";
            },
            None => false,
        }
    } );

    filter_set.insert(CriticalSectionCall::CondVarWait, &|tcx, cs|{
        match cs.call_by_type {
            Some(caller_ty) => {
                let fname = tcx.item_name(cs.callee.def_id());
                let caller_ty_name = caller_ty.to_string();
                return caller_ty_name.contains("std::sync::Condvar") && fname.to_string() == "wait";
            },
            None => false,
        }
    } );
    
    for (fn_id, local_lifetimes) in &lifetimes.borrow().body_local_lifetimes {
        let body = tcx.optimized_mir(*fn_id);
        let interested_locals = filter_body_locals(body, |ty| {
            match parse_lockguard_type(&ty) {
                Some(guard) => {
                    return true;
                },
                None => {},
            }
            false
        });


        for il in interested_locals {
            let lft = &local_lifetimes[&il];
            find_in_lifetime(tcx, body, lft, &callgraph, &mut result.calls, vec![], &filter_set);
        }


    }
    println!("output to {:?}", dl_out);
    if dl_out != "" {
        let out_file = std::path::Path::new(&dl_out);
        fs::create_dir_all(out_file.parent().unwrap())?;
        serde_json::to_writer(&File::create(out_file)?, &result)?;

    }
    //info!("results: {:?}", result);

    // for sec in &result.critical_sections {
    //     info!("body Id {:?}", sec.body_id);
    //     for sp in &sec.live_span {
    //         info!("span {:?}", sp);
    //     }
    // }

    return Ok(result)
}
