use std::env;
use std::io::Error;
use std::result::Result;

use scandir::{self, ReturnType, Scandir};

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let mut instance = Scandir::new(&args[1])?;
    instance = instance.max_file_cnt(100);
    if args.len() > 2 {
        instance = instance.return_type(ReturnType::Ext);
    }
    let (_results, _errors) = instance.collect(true)?;
    let mut result = format!("{:#?}", instance.results(true, true));
    if result.len() > 2000 {
        result = result[..2000].to_string();
    }
    println!("options {:#?}", instance.options());
    println!("result {}", &format!("{:#?}", result));
    println!("finished {:?}", instance.finished());
    println!("has more entries {:?}", instance.has_entries(true));
    println!("has more errors {:?}", instance.has_errors());
    println!("duration {:?}", instance.duration());
    Ok(())
}
