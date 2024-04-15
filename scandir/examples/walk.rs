use std::{ env, time::Instant };
use std::io::Error;
use std::thread;
use std::time::Duration;

use scandir::{ ReturnType, Walk };

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let default_dir = "/usr".to_string();
    let root_dir = &args.get(1).unwrap_or(&default_dir);
    let mut instance = Walk::new(&root_dir, Some(true))?.max_file_cnt(100);
    if args.contains(&"--ext".to_string()) {
        instance = instance.return_type(ReturnType::Ext);
    }
    println!("options {:#?}", instance.options());
    instance.start()?;
    let now = Instant::now();
    loop {
        if !instance.busy() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    let result = format!("{:#?}", instance.collect()?);
    println!("dt={}", now.elapsed().as_secs_f64());
    let result_str = format!("{result:#?}");
    println!("result {}", &result_str[..std::cmp::min(result_str.len(), 500)]);
    let results = instance.results(false);
    println!("result_cnt {}", results.len());
    println!("result_cnt {}", instance.results_cnt(false));
    println!("finished {:?}", instance.finished());
    println!("has_errors {:?}", instance.has_errors());
    println!("error_cnt {}", instance.has_errors());
    println!("statistics {:#?}", instance.statistics());
    println!("duration {:?}", instance.duration());
    Ok(())
}
