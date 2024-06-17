use alloc::{collections::BTreeMap, string::String};
use pc_keyboard::DecodedKey;
use spin::Mutex;
use storage::FileHandle;

use crate::input::try_pop_key;

#[derive(Debug, Clone)]
pub enum StdIO {
    Stdin,
    Stdout,
    Stderr,
}

#[derive(Debug)]
pub struct ResourceSet {
    pub handles: BTreeMap<u8, Mutex<Resource>>,
}

impl Default for ResourceSet {
    fn default() -> Self {
        let mut res = Self {
            handles: BTreeMap::new(),
        };

        res.open(Resource::Console(StdIO::Stdin));
        res.open(Resource::Console(StdIO::Stdout));
        res.open(Resource::Console(StdIO::Stderr));

        res
    }
}

impl ResourceSet {
    pub fn open(&mut self, res: Resource) -> u8 {
        let fd = self.handles.len() as u8;
        self.handles.insert(fd, Mutex::new(res));
        fd
    }

    pub fn close(&mut self, fd: u8) -> bool {
        self.handles.remove(&fd).is_some()
    }

    pub fn read(&self, fd: u8, buf: &mut [u8]) -> isize {
        if let Some(count) = self.handles.get(&fd).and_then(|h| h.lock().read(buf)) {
            count as isize
        } else {
            -1
        }
    }

    pub fn write(&self, fd: u8, buf: &[u8]) -> isize {
        if let Some(count) = self.handles.get(&fd).and_then(|h| h.lock().write(buf)) {
            count as isize
        } else {
            -1
        }
    }
}

#[derive(Debug)]
pub enum Resource {
    File(FileHandle),
    Console(StdIO),
    Null,
}

impl Resource {
    pub fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        match self {
            Resource::Console(stdio) => match stdio {
                StdIO::Stdin => {
                    // FIXME: just read from kernel input buffer
                    if buf.len() < 4{ // 没有读取任何数据
                        return Some(0);
                    } else if let Some(DecodedKey:: Unicode(k))=try_pop_key() {
                        return Some(k.encode_utf8(buf).len()); // 使用 Unicode 字符 k 的 encode_utf8 方法将字符编码为 UTF-8 并存储到 buf 中，然后返回实际写入的字节数
                    } else {
                        return Some(0);
                    }
                }
                _ => None,
            },
            Resource::File(file) => {
                let ret = file.read(buf);
                if let Err(e) = ret {
                    error!("Failed to read file: {:?}", e);
                    None
                } else {
                    Some(ret.unwrap())
                }
            }
            Resource::Null => Some(0),
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> Option<usize> {
        match self {
            Resource::Console(stdio) => match *stdio {
                StdIO::Stdin => None,
                StdIO::Stdout => {
                    print!("{}", String::from_utf8_lossy(buf));
                    Some(buf.len())
                }
                StdIO::Stderr => {
                    warn!("{}", String::from_utf8_lossy(buf));
                    Some(buf.len())
                }
            },
            Resource::File(_) => todo!(),
            Resource::Null => Some(buf.len()),
        }
    }
}
