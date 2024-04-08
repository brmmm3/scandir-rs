use std::env;
use std::io::Error;

use scandir::Count;

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let default_dir = "/tmp".to_string();
    let root_dir = &args.get(1).unwrap_or(&default_dir);
    let mut instance = Count::new(&root_dir)?;
    instance = instance.dir_exclude(Some(vec!["dir0".to_owned(), "dir1".to_owned()]));
    if args.len() > 2 {
        instance = instance.extended(true);
    }
    let _results = instance.collect()?;
    println!("options {:#?}", instance.options());
    println!("results {:#?}", instance.results());
    println!("finished {:?}", instance.finished());
    println!("has more errors {:?}", instance.has_errors());
    println!("duration {:?}", instance.duration());
    Ok(())
}
