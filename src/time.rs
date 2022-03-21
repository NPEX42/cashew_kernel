use crate::{
    arch::cmos::{self, CMOS},
    csh::{ExitCode, ShellArgs},
    println,
};

const DAYS_BEFORE_MONTH: [u64; 13] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];

#[derive(Debug)]
pub struct TimeStamp(u64);

// NOTE: This clock is not monotonic
pub fn realtime() -> TimeStamp {
    let rtc = CMOS::new().rtc(); // Assuming GMT

    let timestamp = 86400 * days_before_year(rtc.year as u64)
        + 86400 * days_before_month(rtc.year as u64, rtc.month as u64)
        + 86400 * (rtc.day - 1) as u64
        + 3600 * rtc.hour as u64
        + 60 * rtc.minute as u64
        + rtc.second as u64;
    TimeStamp(timestamp)
}

fn days_before_year(year: u64) -> u64 {
    (1970..year).fold(0, |days, y| days + if is_leap_year(y) { 366 } else { 365 })
}

fn days_before_month(year: u64, month: u64) -> u64 {
    let leap_day = is_leap_year(year) && month > 2;
    DAYS_BEFORE_MONTH[(month as usize) - 1] + if leap_day { 1 } else { 0 }
}

fn is_leap_year(year: u64) -> bool {
    if year % 4 != 0 {
        false
    } else if year % 100 != 0 {
        true
    } else if year % 400 != 0 {
        false
    } else {
        true
    }
}

pub fn time(_: ShellArgs) -> ExitCode {
    let rt = CMOS::new().rtc();
    println!(
        "Time: {}:{}:{} - {}/{}/{} - Uptime: {}",
        rt.hour,
        rt.minute,
        rt.second,
        rt.day,
        rt.month,
        rt.year,
        seconds()
    );
    ExitCode::Ok
}

static mut RTC_TICKS: usize = 0;
static mut UPDATE_FREQ: usize = 1;

pub fn rtc_tick() {
    unsafe { RTC_TICKS += 1 }
}

pub fn ticks() -> usize {
    unsafe { RTC_TICKS }
}

pub fn seconds() -> f64 {
    unsafe { (RTC_TICKS as f64) / (UPDATE_FREQ as f64) }
}

pub fn set_rate(rate: usize) {
    unsafe {
        UPDATE_FREQ = 8 << 2u32.pow(rate as u32);
        cmos::set_rate(rate as u8);
    }
}
