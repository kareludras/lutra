use std::io::{stdin, stdout};

fn main() {
    let stdin = stdin();
    let stdout = stdout();
    lutra_uci::run(stdin.lock(), stdout.lock());
}
