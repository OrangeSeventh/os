use crate::*;
use alloc::string::{String, ToString};
use alloc::vec;

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Stdin {
    fn new() -> Self {
        Self
    }
    pub fn test(&self) -> String {
        "test".to_string()
    }
    pub fn read_char_with_buf(&self, buf: &mut[u8]) -> Option<char> {
        if let Some(bytes)= sys_read(0, buf) {
            if bytes > 0 {
                return Some(String::from_utf8_lossy(&buf[..bytes]).to_string().remove(0));
            }
        }
        None
    }
    pub fn read_line(&self) -> String {
        // FIXME: allocate string
        // FIXME: read from input buffer
        //       - maybe char by char?
        // FIXME: handle backspace / enter...
        // FIXME: return string

        // String::new()
        let mut string = String::new();
        let mut buf = vec![0; 4];
        loop {
            if let Some(k) = self.read_char_with_buf(&mut buf[..4]) {
                match k {
                    //换行
                    '\n' => {
                        stdout().write("\n");
                        break;
                    }
                    //回车
                    '\x03' => {
                        string.clear();
                        break;
                    }
                    //ctrl+d
                    '\x04' => {
                        string.clear();
                        string.push('\x04');
                        break;
                    }
                    //退格
                    '\x08' => {
                        if !string.is_empty() {
                            stdout().write("\x08");
                            string.pop();
                        }
                    }
                    '\x00'..='\x1F' => {}
                    //正常字符
                    c => {
                        self::print!("{}", k);
                        string.push(c);
                    }
                }
            }
        }
        string
    }
}

impl Stdout {
    fn new() -> Self {
        Self
    }

    pub fn write(&self, s: &str) {
        sys_write(1, s.as_bytes());
    }
}

impl Stderr {
    fn new() -> Self {
        Self
    }

    pub fn write(&self, s: &str) {
        sys_write(2, s.as_bytes());
    }
}

pub fn stdin() -> Stdin {
    Stdin::new()
}

pub fn stdout() -> Stdout {
    Stdout::new()
}

pub fn stderr() -> Stderr {
    Stderr::new()
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum FileMode {
    ReadOnly = 0,
    ReadWriteAppend = 1,
    ReadWriteTruncate = 2,
    ReadWriteCreate = 3,
    ReadWriteCreateOrTruncate = 4,
    ReadWriteCreateOrAppend = 5,
}
