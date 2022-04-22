use std::sync;
use parking_lot;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

fn std_mutex() {
    let mu1 = sync::Mutex::new(1);
    let (tx, rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();
    match *mu1.lock().ok().unwrap() {
        1 => {},
        _ => { 
            tx.send(32).unwrap();
        } 
    };
}

fn std_rwlock() {
    let rw1 = sync::RwLock::new(1);
    let mut a = 0;
    match *rw1.read().unwrap() {
        1 => { *rw1.write().unwrap() += 1; },
        _ => { a = *rw1.read().unwrap(); },
    };
}

fn parking_lot_mutex() {
    let mu1 = parking_lot::Mutex::new(1);
    match *mu1.lock() {
        1 => {},
        _ => { *mu1.lock() += 1; },
    };
}

fn parking_lot_rwlock() {
    let rw1 = parking_lot::RwLock::new(1);
    let mut a = 0;
    match *rw1.read() {
        1 => { *rw1.write() += 1; },
        _ => { a = *rw1.read(); }, 
    };
}

fn main() {
    std_mutex();
    std_rwlock();
    parking_lot_mutex();
    parking_lot_rwlock();
}
