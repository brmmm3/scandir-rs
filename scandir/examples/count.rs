use std::env;
use std::io::Error;
use std::result::Result;

use scandir::{self, Count};

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    //let args: Vec<String> = vec!["".to_owned(), "../../_Data".to_owned()];
    let mut instance = Count::new(&args[1], false, false, 0, 0, None, None, None, None, false)?;
    instance.start();
    instance.join();
    println!("{:?}", instance.results());
    println!("{:?}", instance.finished());
    println!("{:?}", instance.has_errors());
    println!("{:?}", instance.duration());
    Ok(())
}
