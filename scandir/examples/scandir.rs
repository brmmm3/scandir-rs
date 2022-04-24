use std::env;
use std::io::Error;
use std::result::Result;

use scandir::{self, ReturnType, Scandir};

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    //let args: Vec<String> = vec!["".to_owned(), "../../_Data".to_owned()];
    let mut instance = Scandir::new(
        &args[1],
        false,
        false,
        0,
        0,
        None,
        None,
        None,
        None,
        false,
        ReturnType::Fast,
    )?;
    instance.start();
    instance.join();
    println!("{:#?}", instance.results(true));
    println!("{:?}", instance.finished());
    println!("{:?}", instance.has_entries());
    println!("{:?}", instance.has_errors());
    println!("{:?}", instance.duration());
    Ok(())
}
