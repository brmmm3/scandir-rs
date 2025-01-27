#![cfg_attr(windows, feature(junction_point))]

use std::cmp::min;
use std::fs::{create_dir_all, hard_link, File};
use std::io::{Error, Write};
#[cfg(unix)]
use std::os::unix::fs::symlink;
#[cfg(windows)]
use std::os::windows::fs::junction_point;

use tempfile::TempDir;

pub fn setup() -> TempDir {
    let base_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let _ = std::env::set_current_dir(&base_dir);
    TempDir::new().unwrap()
}

#[allow(dead_code)]
pub fn cleanup(temp_dir: TempDir) -> Result<(), Error> {
    temp_dir.close()
}

fn get_filename(i: u32) -> String {
    if i < 3 {
        format!(".file{i}")
    } else {
        format!("file{i}")
    }
}

pub fn create_temp_file_tree(
    depth: u32,
    dircnt: u32,
    filecnt: u32,
    hlinkcnt: u32,
    #[cfg(windows)] jcnt: u32, // Number of junctions to create
    #[cfg(unix)] slinkcnt: u32,
    #[cfg(unix)] pipecnt: u32,
) -> Result<TempDir, Error> {
    let temp_dir = setup();
    #[cfg(windows)]
    let junc_dir = temp_dir.path().join("junc_dir");
    #[cfg(windows)]
    {
        if jcnt > 0 {
            create_dir_all(&junc_dir)?;
            for i in 1..=filecnt {
                let mut file = File::create(junc_dir.join("junc_".to_string() + &get_filename(i)))?;
                file.write_all(format!("HELLO{i}").as_bytes())?;
            }
        }
    }
    for i in 1..=dircnt {
        let mut dir = temp_dir.path().join(format!("dir{i}"));
        for d in 1..=depth {
            dir = dir.join(format!("dir{i}_{d}"));
            create_dir_all(&dir)?;
            for i in 1..=filecnt {
                let mut file = File::create(dir.join(get_filename(i)))?;
                file.write_all(format!("HELLO{i}").as_bytes())?;
            }
            for i in 1..=hlinkcnt {
                let filenum = min(i, filecnt);
                hard_link(
                    dir.join(get_filename(filenum)),
                    dir.join(format!("hardlink{i}")),
                )?;
            }
            #[cfg(windows)]
            for i in 1..=jcnt {
                junction_point(&junc_dir, &dir.join(format!("junction{i}")))?;
            }
            #[cfg(unix)]
            for i in 1..=slinkcnt {
                let filenum = min(i, filecnt);
                symlink(
                    dir.join(get_filename(filenum)),
                    dir.join(format!("symlink{i}")),
                )?;
            }
            #[cfg(unix)]
            for i in 1..=pipecnt {
                unix_named_pipe::create(dir.join(format!("pipe{i}")), None)?;
            }
        }
    }
    Ok(temp_dir)
}
