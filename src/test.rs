#[cfg(test)]
mod tests {
    use pyo3::prelude::*;

    use crate::count::*;

    #[test]
    fn test_count() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        #[cfg(unix)]
        println!(
            "{:#?}",
            count(
                py,
                String::from("/usr"),
                Some(false),
                Some(false),
                None,
                None,
                None,
                None,
                None,
                Some(false)
            )
            .unwrap()
        );
        #[cfg(windows)]
        println!(
            "{:#?}",
            count(
                py,
                String::from("C:/Windows"),
                Some(false),
                Some(false),
                None,
                None,
                None,
                None,
                None,
                Some(false)
            )
            .unwrap()
        );
    }

    #[test]
    fn test_count_skip_hidden() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        #[cfg(unix)]
        count(
            py,
            String::from("/usr"),
            Some(true),
            Some(false),
            None,
            None,
            None,
            None,
            None,
            Some(false),
        )
        .unwrap();
        #[cfg(windows)]
        count(
            py,
            String::from("C:/Windows"),
            Some(false),
            Some(false),
            None,
            None,
            None,
            None,
            None,
            Some(false),
        )
        .unwrap();
    }

    #[test]
    fn test_count_skip_hidden_metadata() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        #[cfg(unix)]
        println!(
            "{:#?}",
            count(
                py,
                String::from("/usr"),
                Some(true),
                Some(false),
                None,
                None,
                None,
                None,
                None,
                Some(false)
            )
            .unwrap()
        );
        #[cfg(windows)]
        println!(
            "{:#?}",
            count(
                py,
                String::from("C:/Windows"),
                Some(true),
                Some(false),
                None,
                None,
                None,
                None,
                None,
                Some(false)
            )
            .unwrap()
        );
    }

    #[test]
    fn test_count_skip_hidden_metadata_ext() {
        let gil = Python::acquire_gil();
        let py = gil.python();

        #[cfg(unix)]
        count(
            py,
            String::from("/usr"),
            Some(true),
            Some(true),
            None,
            None,
            None,
            None,
            None,
            Some(false),
        )
        .unwrap();
        #[cfg(windows)]
        count(
            py,
            String::from("C:/Windows"),
            Some(true),
            Some(true),
            None,
            None,
            None,
            None,
            None,
            Some(false),
        )
        .unwrap();
    }
}
