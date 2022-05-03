#[macro_export]
macro_rules! klog {
    ($fmt:expr, $($args:tt)*) => {
        //$crate::terminal::write_fmt(format_args!(concat!("[LOG|{}:{}:{}]: ", $fmt), file!(), line!(), column!(), $($args)*));
        $crate::serial::_print(format_args!(concat!("[LOG|{}:{}:{}]: ", $fmt, "\n"), file!(), line!(), column!(), $($args)*));
        //$crate::terminal::swap();
    };

    ($fmt:expr) => {
        //$crate::terminal::write_fmt(format_args!(concat!("[LOG|{}:{}:{}]: ", $fmt), file!(), line!(), column!()));
        $crate::serial::_print(format_args!(concat!("[LOG|{}:{}:{}]: ", $fmt, "\n"), file!(), line!(), column!()))
        //$crate::terminal::swap();
    };
}

#[macro_export]
macro_rules! kwarn {
    ($fmt:expr, $($args:tt)*) => {
        let old_color = $crate::terminal::get_fg();
        $crate::terminal::set_fg(C64_PALLETE[10]);
        $crate::terminal::write_fmt(format_args!(concat!("[WARN]: ", $fmt, "\n"), $($args)*));
        $crate::terminal::set_fg(old_color);
    };

    ($fmt:expr) => {
        let old_color = $crate::terminal::get_fg();
        $crate::terminal::set_fg(C64_PALLETE[10]);
        $crate::terminal::write_fmt(format_args!(concat!("[WARN]: , ", $fmt)));
        $crate::terminal::set_fg(old_color);
    };
}

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
macro_rules! trace {
    () => {
        $crate::sprint!("[{}:{}:{}]\n", file!(), line!(), column!());
    }
}