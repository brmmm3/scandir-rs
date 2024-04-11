use std::{ path::Path, time::Duration };

use criterion::{ criterion_group, criterion_main, Criterion };

fn create_test_data() -> String {
    let temp_dir;
    let linux_dir;
    let kernel_path;
    #[cfg(unix)]
    {
        temp_dir = expanduser::expanduser("~/Rust/_Data/benches").unwrap();
        linux_dir = expanduser::expanduser("~/Rust/_Data/benches/linux-5.9").unwrap();
        kernel_path = expanduser::expanduser("~/Rust/_Data/benches/linux-5.9.tar.gz").unwrap();
    }
    #[cfg(windows)]
    {
        temp_dir = PathBuf::from("C:/Workspace/benches");
        linux_dir = PathBuf::from("C:/Workspace/benches/linux-5.9");
        kernel_path = PathBuf::from("C:/Workspace/benches/linux-5.9.tar.gz");
    }
    if !temp_dir.exists() {
        std::fs::create_dir_all(&temp_dir).unwrap();
    }
    if !kernel_path.exists() {
        // Download kernel
        println!("Downloading linux-5.9.tar.gz...");
        let resp = reqwest::blocking
            ::get("https://cdn.kernel.org/pub/linux/kernel/v5.x/linux-5.9.tar.gz")
            .expect("request failed");
        let body = resp.text().expect("body invalid");
        let mut out = std::fs::File::create(&kernel_path).expect("failed to create file");
        std::io::copy(&mut body.as_bytes(), &mut out).expect("failed to copy content");
    }
    if !linux_dir.exists() {
        println!("Extracting linux-5.9.tar.gz...");
        let tar_gz = std::fs::File::open(&kernel_path).unwrap();
        let tar = flate2::read::GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(tar);
        archive.unpack(&linux_dir).unwrap();
    }
    linux_dir.to_str().unwrap().to_string()
}

fn benchmark_dir(c: &mut Criterion, path: &str) {
    println!("Running benchmarks for {path}...");
    let dir = Path::new(path).file_name().unwrap().to_str().unwrap();
    // Count
    let mut group = c.benchmark_group(format!("Count {dir}"));
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(20);
    group.bench_function("scandir.Count (collect)", |b|
        b.iter(|| {
            let mut instance = scandir::Count
                ::new(&path)
                .expect(&format!("Failed to create Count instance for {path}"));
            instance.collect().unwrap();
        })
    );
    group.bench_function("scandir.Count(Ext) (collect)", |b|
        b.iter(|| {
            let mut instance = scandir::Count
                ::new(&path)
                .expect(&format!("Failed to create Count instance for {path}"))
                .extended(true);
            instance.collect().unwrap();
        })
    );
    group.finish();
    // Walk
    let mut group = c.benchmark_group(format!("Walk {dir}"));
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(20);
    group.bench_function("walkdir.WalkDir", |b|
        b.iter(|| {
            let _ = walkdir::WalkDir::new(&path).into_iter().collect::<Vec<_>>();
        })
    );
    group.bench_function("scandir.Walk (collect)", |b|
        b.iter(|| {
            let mut instance = scandir::Walk
                ::new(&path, Some(true))
                .expect(&format!("Failed to create Walk instance for {path}"));
            instance.collect().unwrap();
        })
    );
    group.bench_function("scandir.Walk(Ext) (collect)", |b|
        b.iter(|| {
            let mut instance = scandir::Walk
                ::new(&path, Some(true))
                .expect(&format!("Failed to create Walk instance for {path}"))
                .return_type(scandir::ReturnType::Ext);
            instance.collect().unwrap();
        })
    );
    group.finish();
    // Scandir
    let mut group = c.benchmark_group(format!("Scandir {dir}"));
    group.measurement_time(Duration::from_secs(60));
    group.sample_size(20);
    group.bench_function("scan_dir.ScanDir", |b|
        b.iter(|| {
            let mut entries = Vec::new();
            let _ = scan_dir::ScanDir::all().walk("/usr", |iter| {
                for (entry, _name) in iter {
                    entries.push(entry.metadata().unwrap());
                }
            });
        })
    );
    group.bench_function("scandir.Scandir (collect)", |b|
        b.iter(|| {
            let mut instance = scandir::Scandir
                ::new(&path, Some(true))
                .expect(&format!("Failed to create Scandir instance for {path}"));
            instance.collect().unwrap();
        })
    );
    group.bench_function("scandir.Scandir(Ext) (collect)", |b|
        b.iter(|| {
            let mut instance = scandir::Scandir
                ::new(&path, Some(true))
                .expect(&format!("Failed to create Scandir instance for {path}"))
                .return_type(scandir::ReturnType::Ext);
            instance.collect().unwrap();
        })
    );
    group.finish();
}

fn benchmarks(c: &mut Criterion) {
    benchmark_dir(c, &create_test_data());
    #[cfg(unix)]
    let path = "/usr";
    #[cfg(windows)]
    let path = "C:/Windows";
    benchmark_dir(c, &path);
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
