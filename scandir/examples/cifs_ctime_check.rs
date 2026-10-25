// Verify scandir-rs DirEntryExt time fields on CIFS nounix,mapposix mounts:
//   st_ctime — POSIX change time (always available, matches Python os.stat().st_ctime)
//   st_btime — birth time (available on CIFS nounix SMB2/3 + kernel>=4.13 + gnu target,
//              where io_uring.rs requests STATX_BTIME mask; Steve French commit 6e70e26dc52b)
//
// Passes when every file entry has non-None st_ctime. st_btime is reported but not
// asserted (some filesystems legitimately lack btime: ext3, FAT, tmpfs, musl target).
//
// Usage: cargo run --example cifs_ctime_check -- <mount_path>
// Defaults to /mnt/smb.

use scandir::{ReturnType, Scandir, ScandirResult};
use std::time::UNIX_EPOCH;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/mnt/smb".to_string());

    let entries = Scandir::new(&path, Some(true))
        .expect("open dir")
        .return_type(ReturnType::Ext)
        .collect()
        .expect("collect");

    println!(
        "scanned {} entries, {} errors from {}",
        entries.results.len(),
        entries.errors.len(),
        path
    );

    let mut ctime_none = 0usize;
    let mut ctime_ok = 0usize;
    let mut btime_none = 0usize;
    let mut btime_ok = 0usize;
    let mut max_skew_secs: f64 = 0.0;

    for r in &entries.results {
        let d = match r {
            ScandirResult::DirEntryExt(d) => d,
            _ => continue,
        };
        let full = std::path::Path::new(&path).join(&d.path);
        let std_btime = std::fs::metadata(&full).ok().and_then(|m| m.created().ok());

        if d.is_file {
            let ctime_secs = d.ctime();
            let btime_secs = d.btime();

            match d.st_ctime {
                Some(_) => ctime_ok += 1,
                None => {
                    ctime_none += 1;
                    println!("  CTIME-NONE {:<30} <<< REGRESSION", d.path);
                }
            }
            match d.st_btime {
                Some(t) => {
                    btime_ok += 1;
                    if let (Ok(dur), Some(std_t)) = (t.duration_since(UNIX_EPOCH), std_btime)
                        && let Ok(std_dur) = std_t.duration_since(UNIX_EPOCH)
                    {
                        let skew = (dur.as_secs_f64() - std_dur.as_secs_f64()).abs();
                        if skew > max_skew_secs {
                            max_skew_secs = skew;
                        }
                    }
                }
                None => btime_none += 1,
            }

            println!(
                "  {:<30} ctime={:.6} btime={}",
                d.path,
                ctime_secs,
                match d.st_btime {
                    Some(_) => format!("{:.6}", btime_secs),
                    None => "None".to_string(),
                }
            );
        }
    }

    println!();
    println!(
        "summary: ctime {} ok / {} none, btime {} ok / {} none, max btime skew={:.3}s",
        ctime_ok, ctime_none, btime_ok, btime_none, max_skew_secs
    );
    if ctime_none > 0 {
        std::process::exit(1);
    }
}
