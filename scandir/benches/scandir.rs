#![cfg_attr(windows, feature(windows_by_handle))]

use std::fs;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;
use std::path::Path;
use std::time::Duration;

#[cfg(windows)]
use std::path::PathBuf;

use criterion::{Criterion, criterion_group, criterion_main};

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
        let resp =
            reqwest::blocking::get("https://cdn.kernel.org/pub/linux/kernel/v5.x/linux-5.9.tar.gz")
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
    let mut group = c.benchmark_group(format!("Scandir {dir}"));
    group.measurement_time(Duration::from_secs(60));
    group.sample_size(20);
    group.bench_function("scan_dir.ScanDir", |b| {
        b.iter(|| {
            let mut entries = Vec::new();
            let _ = scan_dir::ScanDir::all().walk(path, |iter| {
                for (entry, _name) in iter {
                    entries.push(entry.metadata().unwrap());
                }
            });
        })
    });
    group.bench_function("scan_dir.ScanDir(Ext)", |b| {
        b.iter(|| {
            let mut entries = Vec::new();
            let _ = scan_dir::ScanDir::all().walk(path, |iter| {
                for (entry, _name) in iter {
                    if let Ok(metadata) = fs::metadata(entry.path()) {
                        entries
                            .push((entry.metadata().unwrap(), Some(get_metadata_ext(&metadata))));
                    } else {
                        entries.push((entry.metadata().unwrap(), None));
                    }
                }
            });
        })
    });
    group.bench_function("scandir.Scandir (collect)", |b| {
        b.iter(|| {
            let mut instance = scandir::Scandir::new(path, Some(true))
                .unwrap_or_else(|_| panic!("Failed to create Scandir instance for {path}"));
            instance.collect().unwrap();
        })
    });
    group.bench_function("scandir.Scandir(Ext) (collect)", |b| {
        b.iter(|| {
            let mut instance = scandir::Scandir::new(path, Some(true))
                .unwrap_or_else(|_| panic!("Failed to create Scandir instance for {path}"))
                .return_type(scandir::ReturnType::Ext);
            instance.collect().unwrap();
        })
    });
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
