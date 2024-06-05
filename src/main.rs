use std::collections::HashMap;
use std::env;
use std::path::Path;

mod analyzer;
mod reporter;
fn main() {
    let args: Vec<String> = env::args().collect();
    let dir = &args[1];
    println!("Reading files from: [{}]", dir);

    let src_directory = Path::new(dir);

    let mut results = HashMap::new();
    analyzer::read_directory(src_directory, &mut results);

    reporter::run(&results, "report.html")
        .is_ok()
        .then(|| print!("Report Ready @ "))
        .expect("Error generating report");
}
