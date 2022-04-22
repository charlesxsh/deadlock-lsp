use std::{fmt, collections::{HashSet, HashMap}};

use log::info;
use rustc_hir::def_id::{LocalDefId, DefId};
use rustc_middle::mir::{Local, Location};
use rustc_span::Span;

#[derive(Clone, Debug)]
pub struct Lifetime {
    pub body_id: DefId,
    pub live_locs: HashSet<Location>,
    pub live_span: Vec<Span>
}

impl Lifetime {
    pub fn new(body_id: DefId) -> Self {
        return Self {
            body_id,
            live_locs: HashSet::new(),
            live_span: Vec::new()
        }
    }
}
pub struct Lifetimes {
    pub body_local_lifetimes: HashMap<DefId, HashMap<Local, Lifetime>>
}



impl Lifetimes {
    pub fn new() -> Self {
        Self {
            body_local_lifetimes: HashMap::new()
        }
    }

    pub fn add_live_loc(&mut self, body_id: DefId, local: Local, loc: Location, span: Span) {
        if !self.body_local_lifetimes.contains_key(&body_id) {
            self.body_local_lifetimes.insert(body_id, HashMap::new());
        }

        let loc2life = self.body_local_lifetimes.get_mut(&body_id).unwrap();
        if !loc2life.contains_key(&local) {
            loc2life.insert( local, Lifetime::new(body_id));
        }

        let mut ll = loc2life.get_mut(&local).unwrap();
        ll.live_locs.insert(loc);
        ll.live_span.push(span);
    }
}
