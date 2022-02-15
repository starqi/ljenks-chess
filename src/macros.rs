pub mod global_macros {

    #[macro_export]
    macro_rules! custom_log {
        ($log_func:expr, $($t:tt)*) => ($log_func(&format_args!($($t)*).to_string()))
    }

    #[macro_export]
    macro_rules! console_log {
        ($($t:tt)*) => (crate::custom_log!(crate::extern_funcs::log, $($t)*))
    }

    #[macro_export]
    macro_rules! console_error {
        ($($t:tt)*) => (crate::custom_log!(crate::extern_funcs::error, $($t)*))
    }

    #[macro_export]
    macro_rules! bitboard_union {
        ($e:expr) => ($e);
        ($e:expr, $($e2:expr),+) => (Bitboard($e.0 | bitboard_union!($($e2),+).0))
    }
}
