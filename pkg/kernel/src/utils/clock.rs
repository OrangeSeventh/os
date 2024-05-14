use super::uefi;
use boot::BootInfo;
use chrono::naive::*;

pub fn init(boot_info: &'static BootInfo) {
    uefi::init(boot_info);
}

pub fn now() -> Option<NaiveDateTime> {
    let uefi_time = uefi::UEFI_RUNTIME.lock()
        .as_ref()
        .expect("UEFI Runtime not initialized")
        .get_time();

    let naive_date = NaiveDate::from_ymd_opt(
        uefi_time.year() as i32, 
        uefi_time.month() as u32, 
        uefi_time.day() as u32, 
    )?;

    let naive_time = NaiveTime::from_hms_milli_opt(
        uefi_time.hour() as u32, 
        uefi_time.minute() as u32, 
        uefi_time.second() as u32, 
        (uefi_time.nanosecond() / 1_000_000) as u32,
    )?;

    Some(naive_date.and_time(naive_time))
}