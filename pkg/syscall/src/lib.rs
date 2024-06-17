#![no_std]

use num_enum::FromPrimitive;

pub mod macros;

#[repr(usize)]
#[derive(Clone, Debug, FromPrimitive)]
pub enum Syscall {
    Read = 0,
    Write = 1,
    ListDir = 2,
    Open = 3,
    Close = 4,
    GetPid = 39,
    Fork = 58,
    Spawn = 59,
    Exit = 60,
    WaitPid = 61,
    Time = 201,
    
    Sem = 44326,
    ListApp = 65531,
    Stat = 65532,
    Allocate = 65533,
    Deallocate = 65534,
    
    #[num_enum(default)]
    Unknown = 65535,
}
