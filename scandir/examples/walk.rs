use std::env;
use std::io::Error;
use std::result::Result;

use scandir::{self, Walk};

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    //let args: Vec<String> = vec!["".to_owned(), "../../_Data".to_owned()];
    let mut instance = Walk::new(&args[1])?;
    instance.start()?;
    instance.join();
    println!("{}", &format!("{:#?}", instance.collect())[..2000]);
    println!("{:?}", instance.results(true).len());
    println!("{:?}", instance.finished());
    println!("{:?}", instance.has_errors());
    println!("{:?}", instance.duration());
    Ok(())
}
