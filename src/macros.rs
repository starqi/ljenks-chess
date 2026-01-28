pub mod global_macros {

    #[macro_export]
    macro_rules! custom_log {
        ($log_func:expr, $($t:tt)*) => ($log_func(&format_args!($($t)*).to_string()))
    }

    #[cfg(feature = "my_debug")]
    #[macro_export]
    macro_rules! console_log {
        ($($t:tt)*) => (crate::custom_log!(crate::extern_funcs::log, $($t)*))
    }

    #[cfg(not(feature = "my_debug"))]
    #[macro_export]
    macro_rules! console_log {
        ($($t:tt)*) => ()
    }

    #[macro_export]
    macro_rules! console_error {
        ($($t:tt)*) => (crate::custom_log!(crate::extern_funcs::error, $($t)*))
    }

    #[macro_export]
    macro_rules! branchless_mask {
        ($cond:expr, $val:expr) => (-($cond as i32) & $val)
    }
}
