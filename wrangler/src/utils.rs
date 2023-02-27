use cfg_if::cfg_if;

cfg_if! {
    // https://github.com/rustwasm/console_error_panic_hook#readme
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}

pub trait Flatten<T> {
    fn flatten(self) -> Option<T>;
}

impl<T, U> Flatten<T> for Result<T, U> {
    fn flatten(self) -> Option<T> {
        match self {
            Err(_) => None,
            Ok(v) => Some(v),
        }
    }
}
