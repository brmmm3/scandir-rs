use std::io::Error;
use std::time::Duration;
use std::{env, time::Instant};

use scandir::{ReturnType, Scandir};

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let default_dir = "/usr".to_string();
    let root_dir = &args.get(1).unwrap_or(&default_dir);
    let mut instance = Scandir::new(root_dir, Some(true))?;
    //instance = instance.max_file_cnt(100);
    if args.contains(&"--ext".to_string()) {
        instance = instance.return_type(ReturnType::Ext);
    }
    println!("options {:#?}", instance.options());
    instance.start()?;
    let now = Instant::now();
    std::thread::sleep(Duration::from_millis(100));
    //instance.stop();
    let entries = instance.collect()?;
    println!("dt={}", now.elapsed().as_secs_f64());
    for (path, error) in entries.errors.iter() {
        println!("ERROR {path:?}: {error}");
    }
    let first_result = entries.results.first().unwrap();
    println!(
        "First file {:?} has size {}",
        first_result.path(),
        first_result.size()
    );
    let result = format!("{:#?}", instance.results(false));
    let result_str = format!("{result:#?}");
    println!(
        "result {}",
        &result_str[..std::cmp::min(result_str.len(), 500)]
    );
    println!("finished {:?}", instance.finished());
    println!("has more entries {:?}", instance.has_entries(true));
    println!("has_errors {:?}", instance.has_errors());
    println!("results {}", entries.results.len());
    println!("error_cnt {}", entries.errors.len());
    println!("duration {:?}", instance.duration());
    Ok(())
}
