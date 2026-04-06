use std::io;

pub fn none_if_not_found<T>(result: io::Result<T>) -> io::Result<Option<T>> {
    match result {
        Ok(value) =>
            Ok(Some(value)),
        Err(err) if err.kind() == io::ErrorKind::NotFound =>
            Ok(None),
        Err(err) =>
            Err(err),
    }
}
