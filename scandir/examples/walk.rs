use std::env;
use std::io::Error;
use std::thread;
use std::time::Duration;

use scandir::{ReturnType, Walk};

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let root_dir = &args[1];
    //let root_dir = "/tmp/1".to_owned();
    let mut instance = Walk::new(&root_dir, Some(true))?.max_file_cnt(100);
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
    let mut result = format!("{:#?}", instance.collect()?);
    if result.len() > 2000 {
        result = result[..2000].to_string();
    }
    println!("options {:#?}", instance.options());
    println!("result {}", &format!("{:#?}", result));
    println!("result_cnt {}", instance.results(true).len());
    println!("finished {:?}", instance.finished());
    println!("has more errors {:?}", instance.has_errors());
    println!("duration {:?}", instance.duration());
    Ok(())
}
