#![allow(unused)]

struct NoDebug {}

#[derive(Debug)]
struct YesDebug {}

fn main() {
    let yes_debug = YesDebug {};
    let no_debug = NoDebug {};
}
