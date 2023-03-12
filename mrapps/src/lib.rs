pub mod mr;
pub mod wc;

use std::env;

use mr::*;
use wc::*;

pub fn mrsequential() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        println!("Usage: mrsequential xxx.so inputfiles...");
        return;
    }


}

pub fn load_plugin(filename: String) {}
