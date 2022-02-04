#[macro_export]
macro_rules! klog {
    ($fmt:expr, $($args:tt)*) => {
        //$crate::terminal::write_fmt(format_args!(concat!("[LOG|{}:{}:{}]: ", $fmt), file!(), line!(), column!(), $($args)*));
        $crate::serial::_print(format_args!(concat!("[LOG|{}:{}:{}]: ", $fmt), file!(), line!(), column!(), $($args)*));
        //$crate::terminal::swap();
    };

    ($fmt:expr) => {
        //$crate::terminal::write_fmt(format_args!(concat!("[LOG|{}:{}:{}]: ", $fmt), file!(), line!(), column!()));
        $crate::serial::_print(format_args!(concat!("[LOG|{}:{}:{}]: ", $fmt), file!(), line!(), column!()));
        //$crate::terminal::swap();
    };
}

#[macro_export]
macro_rules! kwarn {
    ($fmt:expr, $($args:tt)*) => {
        let old_color = $crate::terminal::get_fg();
        $crate::terminal::set_fg(C64_PALLETE[10]);
        $crate::terminal::write_fmt(format_args!(concat!("[WARN]: ", $fmt), $($args)*));
        $crate::terminal::set_fg(old_color);
    };

    ($fmt:expr) => {
        let old_color = $crate::terminal::get_fg();
        $crate::terminal::set_fg(C64_PALLETE[10]);
        $crate::terminal::write_fmt(format_args!(concat!("[WARN]: ", $fmt)));
        $crate::terminal::set_fg(old_color);
    };
}

#[macro_export]
macro_rules! kerr {
    ($fmt:expr, $($args:tt)*) => {
        let old_fg_color = $crate::terminal::get_fg();
        let old_bg_color = $crate::terminal::get_bg();
        $crate::terminal::set_fg(C64_PALLETE[2]);
        $crate::terminal::set_bg(C64_PALLETE[10]);
        $crate::terminal::write_fmt(format_args!(concat!("[ERROR]: ", $fmt), $($args)*));
        $crate::terminal::set_fg(old_fg_color);
        $crate::terminal::set_bg(old_bg_color);
    };

    ($fmt:expr) => {
        let old_fg_color = $crate::terminal::get_fg();
        let old_bg_color = $crate::terminal::get_bg();
        let old_color = $crate::terminal::get_fg();
        $crate::terminal::set_fg(C64_PALLETE[2]);
        $crate::terminal::set_bg(C64_PALLETE[10]);
        $crate::terminal::print_custom(&$crate::fonts::SAD_TRIANGLE);
        $crate::terminal::print_custom(&$crate::fonts::SAD_TRIANGLE);
        $crate::terminal::print_custom(&$crate::fonts::SAD_TRIANGLE);
        $crate::terminal::write_fmt(format_args!(concat!("[ERROR]: ", $fmt)));
        $crate::terminal::set_fg(old_fg_color);
        $crate::terminal::set_bg(old_bg_color);
    };
}
