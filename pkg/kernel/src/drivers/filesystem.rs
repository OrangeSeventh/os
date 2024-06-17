use super::ata::*;
use alloc::format;
use alloc::boxed::Box;
use chrono::DateTime;
use storage::fat16::Fat16;
use storage::mbr::*;
use storage::*;
use alloc::string::ToString;

pub static ROOTFS: spin::Once<Mount> = spin::Once::new();

pub fn get_rootfs() -> &'static Mount {
    ROOTFS.get().unwrap()
}

pub fn init() {
    info!("Opening disk device...");

    let drive = AtaDrive::open(0, 0).expect("Failed to open disk device");

    // only get the first partition
    let part = MbrTable::parse(drive)
        .expect("Failed to parse MBR")
        .partitions()
        .expect("Failed to get partitions")
        .remove(0);

    info!("Mounting filesystem...");

    ROOTFS.call_once(|| Mount::new(Box::new(Fat16::new(part)), "/".into()));

    trace!("Root filesystem: {:#?}", ROOTFS.get().unwrap());

    info!("Initialized Filesystem.");
}

pub fn ls(root_path: &str) {
    let iter: Box<dyn Iterator<Item = Metadata> + Send> = match get_rootfs().read_dir(root_path) {
        Ok(iter) => iter,
        Err(err) => {
            warn!("{:?}", err);
            return;
        }
    };
    println!("{:<30} {:>10} {:>20} {:<5}", "Name", "Size", "Date", "Type");
    // FIXME: format and print the file metadata
    //      - use `for meta in iter` to iterate over the entries
    for meta in iter {
        //      - use `crate::humanized_size_short` for file size
        let (size, unit) = crate::humanized_size_short(meta.len as u64);
        
        //      - add '/' to the end of directory names
        //      - format the date as you like
        let date = meta
            .modified
            .map(|t| t
            .format("%Y/%m/%d %H:%M:%S").to_string()).unwrap_or("1970/01/01 00:00:00".to_string());
        let file_type = if meta.entry_type == FileType::Directory {
            "Dir"
        } else {
            "File"
        };
        //      - do not forget to print the table header
        let name = format!("{}{}", meta.name, if file_type == "Dir" { "/" } else { "" });
        println!("{:<30} {:>10.2} {:<3} {:>20} {:<5}", name, size, unit, date, file_type);
    }
}
