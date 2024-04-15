use std::{ fs, path::Path, time::Duration };
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

#[cfg(windows)]
use std::path::PathBuf;

use criterion::{ criterion_group, criterion_main, Criterion };

#[cfg(unix)]
#[derive(Debug, Clone)]
pub struct MetaDataExt {
    pub st_mode: u32,
    pub st_ino: u64,
    pub st_dev: u64,
    pub st_nlink: u64,
    pub st_blksize: u64,
    pub st_blocks: u64,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_rdev: u64,
}

#[cfg(windows)]
#[derive(Debug, Clone)]
pub struct MetaDataExt {
    pub file_attributes: u32,
    pub volume_serial_number: Option<u32>,
    pub number_of_links: Option<u32>,
    pub file_index: Option<u64>,
}

#[inline]
pub fn get_metadata_ext(metadata: &fs::Metadata) -> MetaDataExt {
    #[cfg(unix)]
    {
        MetaDataExt {
            st_mode: metadata.mode(),
            st_ino: metadata.ino(),
            st_dev: metadata.dev(),
            st_nlink: metadata.nlink(),
            st_blksize: metadata.blksize(),
            st_blocks: metadata.blocks(),
            st_uid: metadata.uid(),
            st_gid: metadata.gid(),
            st_rdev: metadata.rdev(),
        }
    }
    #[cfg(windows)]
    {
        MetaDataExt {
            file_attributes: metadata.file_attributes(),
            volume_serial_number: metadata.volume_serial_number(),
            number_of_links: metadata.number_of_links(),
            file_index: metadata.file_index(),
        }
    }
}

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
    let mut group = c.benchmark_group(format!("Walk {dir}"));
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(20);
    group.bench_function("walkdir.WalkDir", |b|
        b.iter(|| {
            let _ = walkdir::WalkDir::new(&path).into_iter().collect::<Vec<_>>();
        })
    );
    group.bench_function("walkdir.WalkDir(Ext)", |b|
        b.iter(|| {
            let _ = walkdir::WalkDir
                ::new(&path)
                .into_iter()
                .map(|result| {
                    match result {
                        Ok(entry) => {
                            if let Ok(metadata) = fs::metadata(entry.path()) {
                                Ok((entry.metadata().unwrap(), Some(get_metadata_ext(&metadata))))
                            } else {
                                Ok((entry.metadata().unwrap(), None))
                            }
                        }
                        Err(e) => Err(e),
                    }
                })
                .collect::<Vec<_>>();
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
