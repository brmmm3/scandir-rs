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
    instance.collect()?;
    println!("{:#?}", instance.options());
    println!("{}", &format!("{:#?}", instance.results(true))[..2000]);
    println!("{:?}", instance.finished());
    println!("{:?}", instance.has_entries(true));
    println!("{:?}", instance.has_errors());
    println!("{:?}", instance.duration());
    Ok(())
}
