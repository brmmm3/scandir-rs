use std::env;
use std::io::Error;

use scandir::{ReturnType, Scandir};

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let mut instance = Scandir::new(&args[1], Some(true))?;
    instance = instance.max_file_cnt(100);
    if args.len() > 2 {
        instance = instance.return_type(ReturnType::Ext);
    }
    let (results, errors) = instance.collect()?;
    for (path, error) in errors {
        println!("ERROR {path}: {error}");
    }
    let first_result = results.iter().next().unwrap();
    println!(
        "First file {} has size {}",
        first_result.path(),
        first_result.size()
    );
    let mut result = format!("{:#?}", instance.results(true,));
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
