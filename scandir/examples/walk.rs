use std::env;
use std::io::Error;
use std::result::Result;

use scandir::{self, ReturnType, Walk};

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let mut instance = Walk::new(&args[1])?;
    if args.len() > 2 {
        instance = instance.return_type(ReturnType::Ext);
    }
    instance.collect()?;
    println!("{}", &format!("{:#?}", instance.collect())[..2000]);
    println!("{:?}", instance.results(true).len());
    println!("{:?}", instance.finished());
    println!("{:?}", instance.has_errors());
    println!("{:?}", instance.duration());
    Ok(())
}
