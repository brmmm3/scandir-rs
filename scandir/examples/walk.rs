use std::env;
use std::io::Error;
use std::result::Result;
use std::thread;
use std::time::Duration;

use scandir::{self, ReturnType, Walk};

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let root_dir = &args[1];
    //let root_dir = "/tmp/1".to_owned();
    let mut instance = Walk::new(&root_dir)?.max_file_cnt(100);
    if args.len() > 2 {
        instance = instance.return_type(ReturnType::Ext);
    }
    instance.start()?;
    loop {
        if !instance.busy() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    instance.collect()?;
    println!("{}", &format!("{:#?}", instance.collect())[..200]);
    println!("{:?}", instance.results(true).len());
    println!("{:?}", instance.finished());
    println!("{:?}", instance.has_errors());
    println!("{:?}", instance.duration());
    Ok(())
}
