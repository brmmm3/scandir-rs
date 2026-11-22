pub mod def;
pub use def::{ReturnType, timeout_from_secs};

pub fn root_path(path: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<std::path::PathBuf> {
    use pyo3::prelude::*;

    let decoded = path.py().import("os")?.call_method1("fsdecode", (path,))?;
    Ok(decoded.extract::<std::ffi::OsString>()?.into())
}

pub fn paths_to_os_strings(paths: Vec<std::path::PathBuf>) -> Vec<std::ffi::OsString> {
    paths
        .into_iter()
        .map(|path| path.into_os_string())
        .collect()
}

pub fn errors_to_os_strings(errors: scandir::ErrorsType) -> Vec<(std::ffi::OsString, String)> {
    errors
        .into_iter()
        .map(|(path, error)| (path.into_os_string(), error))
        .collect()
}
