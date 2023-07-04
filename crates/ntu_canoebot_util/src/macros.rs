//! Utility macros

/// Prints as per normal when in debug mode.
/// Does not print when in release mode.
#[macro_export]
macro_rules! debug_println {
    ($($args:tt)*) => {
        #[cfg(debug_assertions)]
        println!($($args)*);

        #[cfg(not(debug_assertions))]
        ()
    };
}

#[macro_export]
macro_rules! debug_print {
    ($($args:tt)*) => {
        #[cfg(debug_assertions)]
        print!($($args)*);

        #[cfg(not(debug_assertions))]
        ()
    };
}

#[cfg(test)]
mod test {

    #[test]
    fn test_with_debug_assertions() {
        debug_println!("hello world, {}, {}, {}", 1, 2, 3);
    }

    #[test]
    fn test_without_debug_assertions() {
        #[cfg(not(debug_assertions))]
        debug_println!("hello world, {}, {}, {}", 1, 2, 3);
    }
}
