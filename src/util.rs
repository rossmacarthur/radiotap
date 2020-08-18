pub trait BoolExt {
    fn some<T>(self, f: impl FnOnce() -> T) -> Option<T>;
}

impl BoolExt for bool {
    fn some<T>(self, f: impl FnOnce() -> T) -> Option<T> {
        if self {
            Some(f())
        } else {
            None
        }
    }
}
