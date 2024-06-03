use std::env;
use std::path::Path;

mod analyzer;
fn main() {
    let args: Vec<String> = env::args().collect();
    let dir = &args[1];
    println!("Reading files from: [{}]", dir);

    let src_directory = Path::new(dir);
    analyzer::read_directory(src_directory);
}
