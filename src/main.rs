#![feature(box_syntax)]

use clap::Parser;
use std::fs;
use std::time::Instant;
mod types;
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    file: String,

    #[clap(short, long)]
    verbose: bool,
}

mod interpreter;
use interpreter::Interpreter;

fn main() {
    std::panic::set_hook(
        box |info| {
            eprint!("{msg}", msg = match info.message() {
                None => "Program panicked!".to_owned(),
                Some(x) => x.to_string()
            });
        }
    );
    
    let start = Instant::now();

    let args = Args::parse();
    if args.verbose == true {
        println!("Running: {}\n", args.file);
    }
    let file_input = fs::read_to_string(args.file).expect("File reading error");
    let mut onint = Interpreter::new(file_input);

    onint.run();

    if args.verbose == true {
        println!("\nElapsed: {:?}", start.elapsed());
    }
}
