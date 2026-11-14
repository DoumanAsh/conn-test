#[cfg(debug_assertions)]
macro_rules! unreach {
    () => {
        unreachable!()
    }
}

#[cfg(not(debug_assertions))]
macro_rules! unreach {
    () => {
        unsafe {
            core::hint::unreachable_unchecked()
        }
    }
}

const APP_NAME: &'static str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub mod utils;
pub mod checks;
pub mod os;
pub mod output;
