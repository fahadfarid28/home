pub use b_x::*;
pub type Result<T> = std::result::Result<T, BS>;

#[macro_export]
macro_rules! eyre {
    ($($tt:tt)*) => {
        ::noteyre::BS::from_string(format!($($tt)*))
    };
}

#[macro_export]
macro_rules! bail {
    ($($tt:tt)*) => {
        return Err(::noteyre::eyre!($($tt)*))
    };
}
