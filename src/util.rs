pub trait BoolExt {
    fn into_option<T>(self, f: impl FnOnce() -> T) -> Option<T>;
}

impl BoolExt for bool {
    fn into_option<T>(self, f: impl FnOnce() -> T) -> Option<T> {
        if self {
            Some(f())
        } else {
            None
        }
    }
}
