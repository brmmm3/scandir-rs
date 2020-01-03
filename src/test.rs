
#[cfg(test)]
mod tests {
    use pyo3::prelude::*;

    use crate::count::*;
    
    #[test]
    fn test_count() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        #[cfg(unix)]
        println!("{:#?}", count(py, "/usr", Some(false), Some(false), Some(false)).unwrap());
        #[cfg(windows)]
        println!("{:#?}", count(py, "C:/Windows", Some(false), Some(false), Some(false)).unwrap());
    }

    #[test]
    fn test_count_skip_hidden() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        #[cfg(unix)]
        count(py, "/usr", Some(true), Some(false), Some(false)).unwrap();
        #[cfg(windows)]
        count(py, "C:/temp", Some(true), Some(false), Some(false)).unwrap();
    }

    #[test]
    fn test_count_skip_hidden_metadata() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        #[cfg(unix)]
        println!("{:#?}", count(py, "/usr", Some(true), Some(true), Some(false)).unwrap());
        #[cfg(windows)]
        println!("{:#?}", count(py, "C:/temp", Some(true), Some(true), Some(false)).unwrap());
    }

    #[test]
    fn test_count_skip_hidden_metadata_ext() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        #[cfg(unix)]
        count(py, "/usr", Some(true), Some(true), Some(true)).unwrap();
        #[cfg(windows)]
        count(py, "C:/temp", Some(true), Some(true), Some(true)).unwrap();
    }
}
