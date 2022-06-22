
/// Alias for `panic`.
#[macro_export]
macro_rules! fatal {
    ($fmt:expr, $($args:tt)*) => {
        panic!($fmt, $($args)*);
    };

    ($fmt:expr) => {
        panic!($fmt);
    };

    () => {
        panic!();
    };
}

/// Log To Serial.
#[macro_export]
macro_rules! klog {
    ($fmt:expr, $($args:tt)*) => {
        $crate::serial::_print(format_args!(concat!("[LOG - {}:{}:{}]: ", $fmt, "\n"), file!(), line!(), column!(), $($args)*));
    };

    ($fmt:expr) => {
        $crate::serial::_print(format_args!(concat!("[LOG - {}:{}:{}]: ", $fmt, "\n"), file!(), line!(), column!()))
    };
}

/// Write A Short Progress Message To Serial.
#[macro_export]
macro_rules! kprog {
    ($fmt:expr, $($args:tt)*) => {
        $crate::serial::_print(format_args!(concat!("[{}] ", $fmt, "\n"), module_path!(),  $($args)*));
    };

    ($fmt:expr) => {
        //$crate::terminal::write_fmt(format_args!(concat!("[LOG|{}:{}:{}]: ", $fmt), file!(), line!(), column!()));
        $crate::serial::_print(format_args!(concat!("[{}] ", $fmt, "\n"), module_path!()));
        //$crate::terminal::swap();
    };
}

/// Print A Error Message Over Serial.
/// 
/// ! USED BY PANIC !
#[macro_export]
macro_rules! kerr {
    ($fmt:expr, $($args:tt)*) => {
        $crate::terminal::write_fmt(format_args!(concat!("[ERROR]: ", $fmt), $($args)*))
    };

    ($fmt:expr) => {
        $crate::terminal::write_fmt(format_args!(concat!("[ERROR]: ", $fmt, "\n")))
    };
}
#[cfg(feature = "debug")]
#[macro_export]
macro_rules! debug {
    
    ($fmt:expr, $($args:tt)*) => {
        $crate::terminal::write_fmt(format_args!(concat!("[DEBUG]: ", $fmt, "\n"), $($args)*))
    };

    ($fmt:expr) => {
        $crate::terminal::write_fmt(format_args!(concat!("[DEBUG]: ", $fmt, "\n")))
    };
}




#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! debug {
    
    ($fmt:expr, $($args:tt)*) => {
    };

    ($fmt:expr) => {
    };
}




#[macro_export]
macro_rules! trace_enter {
    () => {
        $crate::sprint!("ENTERING -> [{}:{}:{}]\n", file!(), line!(), column!());
    }
}

#[macro_export]
macro_rules! trace_exit {
    () => {
        $crate::sprint!("EXITING -> [{}:{}:{}]\n", file!(), line!(), column!());
    }
}