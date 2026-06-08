mod util;
use util::parser;
use util::structures::*;

use std::env;
use std::path::PathBuf;



fn main() -> Result<(), ()> {
    let mut cli_args = env::args();
    cli_args.next().expect("Wasn't passed filepath? This shouldn't happen");
    let program_path: PathBuf;
    if let Some(p) = cli_args.next() {
        program_path = PathBuf::from(p);
    } else {
        println!("XMLangEE Interpreter (https://github.com/GlineYT/XMLangEE)
Usage: <path> [arg1] [arg2] [arg3]...");
        return Ok(());
    }
    
    // File extension advisory note
    if let Some(ext) = program_path.extension() {
        let ext_str = ext.to_string_lossy();
        if ext_str != "xee" && ext_str != "xml" {
            eprintln!("Note: File extension '.{}' is not standard for XMLangEE. Expected .xee or .xml", ext_str);
            eprintln!("      The program will still attempt to run, but consider using .xee for XMLangEE files.");
        }
    } else {
        eprintln!("Note: File has no extension. Consider using .xee for XMLangEE files.");
    }
    
    let args: Vec<String> = cli_args.collect();
    let raw_program = match std::fs::read_to_string(&program_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read program: {e}");
            return Err(())
        }
    };
    let tree = match parser::parse(raw_program) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to parse program: {e}");
            return Err(());
        }
    };
    let path = std::env::current_dir().expect("Need read permission on current directory");
    let mut intr = Interpreter::new(path.into(), program_path);
    intr.variables.insert(
        "!args".into(),
        Value::Array(
            args.into_iter().map(|s| Value::String(s)).collect()
        )
    );
    match util::run(&tree, &mut intr) {
        Ok(_) => {
            return Ok(());
        }
        Err(e) => {
            eprintln!("{e}");
            return Err(());
        }
    }
}
