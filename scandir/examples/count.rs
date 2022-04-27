use std::env;
use std::io::Error;
use std::result::Result;

use scandir::{self, Count};

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let mut instance = Count::new(&args[1])?;
    if args.len() > 2 {
        instance = instance.extended(true);
    }
    instance.collect()?;
    println!("{:#?}", instance.results());
    println!("{:?}", instance.finished());
    println!("{:?}", instance.has_errors());
    println!("{:?}", instance.duration());
    Ok(())
}
