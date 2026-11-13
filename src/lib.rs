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

pub mod utils;
pub mod checks;
pub mod os;
pub mod output;
