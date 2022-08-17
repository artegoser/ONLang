use clap::Parser;
use std::time::Instant;
mod types;
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    path: String,

    #[clap(short, long)]
    verbose: bool,

    #[clap(long)]
    compress: bool,

    #[clap(long)]
    convert: Option<String>,

    #[clap(short, long)]
    out: Option<String>,
}

mod interpreter;
use interpreter::Interpreter;

fn main() {
    // #[cfg(not(debug_assertions))]
    std::panic::set_hook(Box::new(|info| {
        eprint!(
            "{}",
            match info.payload().downcast_ref::<String>() {
                None => "Program panicked without a message!",
                Some(x) => x,
            }
        )
    }));

    let start = Instant::now();
    let args = Args::parse();
    if args.verbose == true {
        println!("Running: {}\n", args.path);
    }

    let mut onint = Interpreter::new(args.path);

    match args.out {
        Some(output_path) => {
            if args.compress {
                onint.compress(output_path);
            } else if let Some(format) = args.convert {
                onint.convert(format, output_path);
            } else {
                eprintln!("The file conversion format is not specified, use the flag: --compress")
            }
        }
        None => {
            onint.run();
        }
    }

    if args.verbose == true {
        println!("\nElapsed: {:?}", start.elapsed());
    }
}
