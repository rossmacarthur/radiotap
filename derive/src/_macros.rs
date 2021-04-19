macro_rules! error {
    ($node:expr, $msg:expr) => {
        Error::new_spanned($node, $msg)
    };
    ($node:expr, $fmt:expr, $($arg:tt)+) => {
        Error::new_spanned($node, format!($fmt, $($arg)+))
    }
}

macro_rules! bail {
    ($node:expr, $msg:expr) => {
        return Err(Error::new_spanned($node, $msg));
    };
}
