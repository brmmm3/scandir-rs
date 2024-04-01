use pyo3::prelude::*;

mod def;
mod pycount;
mod pyscandir;
mod pywalk;

/// scandir_rs is a directory iteration module like os.walk(), but with more features and higher speed. Depending on the function call
/// it yields a list of paths, tuple of lists grouped by their entry type or DirEntry objects that include file type and stat information along
/// with the name. Using scandir_rs is about 2-17 times faster than os.walk() (depending on the platform, file system and file tree structure)
/// by parallelizing the iteration in background.
#[pymodule]
#[pyo3(name = "scandir_rs")]
fn init(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_class::<def::ReturnType>()?;
    m.add_class::<pycount::Count>()?;
    m.add_class::<pywalk::Walk>()?;
    m.add_class::<pyscandir::Scandir>()?;
    Ok(())
}
