use std::fmt;

// todo test
// #[feature(proc_macro_hygiene)]
// #[cfg(feature = "wee_alloc")]
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[allow(unused_imports)]
#[allow(dead_code)]
#[allow(unused_macros)]
// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

//https://stackoverflow.com/questions/54042984/can-i-format-debug-output-as-binary-when-the-values-are-in-a-vector

pub struct V(pub Vec<u32>);

// custom output
impl fmt::Binary for V {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // extract the value using tuple idexing
        // and create reference to 'vec'
        let vec = &self.0;

        // @count -> the index of the value,
        // @n     -> the value
        for (count, n) in vec.iter().enumerate() {
            if count != 0 {
                write!(f, " ")?;
            }

            write!(f, "{:b}", n)?;
        }

        Ok(())
    }
}
