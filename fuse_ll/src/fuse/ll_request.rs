//! Low-level filesystem operation request.
//!
//! A request represents information about a filesystem operation the kernel driver wants us to
//! perform.

use std::convert::TryFrom;
use std::ffi::OsStr;
use std::{error, fmt, mem};

use super::abi::*;
use super::argument::ArgumentIterator;

/// Error that may occur while reading and parsing a request from the kernel driver.
#[derive(Debug)]
pub enum RequestError {
    /// Not enough data for parsing header (short read).
    ShortReadHeader(usize),
    /// Kernel requested an unknown operation.
    UnknownOperation(u32),
    /// Not enough data for arguments (short read).
    ShortRead(usize, usize),
    /// Insufficient argument data.
    InsufficientData,
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestError::ShortReadHeader(len) => write!(
                f,
                "Short read of FUSE request header ({} < {})",
                len,
                mem::size_of::<fuse_in_header>()
            ),
            RequestError::UnknownOperation(opcode) => write!(f, "Unknown FUSE opcode ({})", opcode),
            RequestError::ShortRead(len, total) => {
                write!(f, "Short read of FUSE request ({} < {})", len, total)
            }
            RequestError::InsufficientData => write!(f, "Insufficient argument data"),
        }
    }
}

impl error::Error for RequestError {}

/// Filesystem operation (and arguments) the kernel driver wants us to perform. The fields of each
/// variant needs to match the actual arguments the kernel driver sends for the specific operation.
#[derive(Debug)]
pub enum Operation<'a> {
    Lookup {
        name: &'a OsStr,
    },
    Forget {
        arg: &'a fuse_forget_in,
    },
    GetAttr,
    SetAttr {
        arg: &'a fuse_setattr_in,
    },
    ReadLink,
    SymLink {
        name: &'a OsStr,
        link: &'a OsStr,
    },
    MkNod {
        arg: &'a fuse_mknod_in,
        name: &'a OsStr,
    },
    MkDir {
        arg: &'a fuse_mkdir_in,
        name: &'a OsStr,
    },
    Unlink {
        name: &'a OsStr,
    },
    RmDir {
        name: &'a OsStr,
    },
    Rename {
        arg: &'a fuse_rename_in,
        name: &'a OsStr,
        newname: &'a OsStr,
    },
    Link {
        arg: &'a fuse_link_in,
        name: &'a OsStr,
    },
    Open {
        arg: &'a fuse_open_in,
    },
    Read {
        arg: &'a fuse_read_in,
    },
    Write {
        arg: &'a fuse_write_in,
        data: &'a [u8],
    },
    StatFs,
    Release {
        arg: &'a fuse_release_in,
    },
    FSync {
        arg: &'a fuse_fsync_in,
    },
    SetXAttr {
        arg: &'a fuse_setxattr_in,
        name: &'a OsStr,
        value: &'a [u8],
    },
    GetXAttr {
        arg: &'a fuse_getxattr_in,
        name: &'a OsStr,
    },
    ListXAttr {
        arg: &'a fuse_getxattr_in,
    },
    RemoveXAttr {
        name: &'a OsStr,
    },
    Flush {
        arg: &'a fuse_flush_in,
    },
    Init {
        arg: &'a fuse_init_in,
    },
    OpenDir {
        arg: &'a fuse_open_in,
    },
    ReadDir {
        arg: &'a fuse_read_in,
    },
    ReleaseDir {
        arg: &'a fuse_release_in,
    },
    FSyncDir {
        arg: &'a fuse_fsync_in,
    },
    GetLk {
        arg: &'a fuse_lk_in,
    },
    SetLk {
        arg: &'a fuse_lk_in,
    },
    SetLkW {
        arg: &'a fuse_lk_in,
    },
    Access {
        arg: &'a fuse_access_in,
    },
    Create {
        arg: &'a fuse_create_in,
        name: &'a OsStr,
    },
    Interrupt {
        arg: &'a fuse_interrupt_in,
    },
    BMap {
        arg: &'a fuse_bmap_in,
    },
    Destroy,
    // TODO: FUSE_IOCTL since ABI 7.11
    // IoCtl {
    //     arg: &'a fuse_ioctl_in,
    //     data: &'a [u8],
    // },
    // TODO: FUSE_POLL since ABI 7.11
    // Poll {
    //     arg: &'a fuse_poll_in,
    // },
    // TODO: FUSE_NOTIFY_REPLY since ABI 7.15
    // NotifyReply {
    //     data: &'a [u8],
    // },
    // TODO: FUSE_BATCH_FORGET since ABI 7.16
    // BatchForget {
    //     arg: &'a fuse_forget_in,
    //     nodes: &'a [fuse_forget_one],
    // },
    // TODO: FUSE_FALLOCATE since ABI 7.19
    // FAllocate {
    //     arg: &'a fuse_fallocate_in,
    // },
    #[cfg(target_os = "macos")]
    SetVolName {
        name: &'a OsStr,
    },
    #[cfg(target_os = "macos")]
    GetXTimes,
    #[cfg(target_os = "macos")]
    Exchange {
        arg: &'a fuse_exchange_in,
        oldname: &'a OsStr,
        newname: &'a OsStr,
    },
    // TODO: CUSE_INIT since ABI 7.12
    // CuseInit {
    //     arg: &'a fuse_init_in,
    // },
}

impl<'a> fmt::Display for Operation<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operation::Lookup { name } => write!(f, "LOOKUP name {:?}", name),
            Operation::Forget { arg } => write!(f, "FORGET nlookup {}", arg.nlookup),
            Operation::GetAttr => write!(f, "GETATTR"),
            Operation::SetAttr { arg } => write!(f, "SETATTR valid {:#x}", arg.valid),
            Operation::ReadLink => write!(f, "READLINK"),
            Operation::SymLink { name, link } => write!(f, "SYMLINK name {:?}, link {:?}", name, link),
            Operation::MkNod { arg, name } => write!(f, "MKNOD name {:?}, mode {:#05o}, rdev {}", name, arg.mode, arg.rdev),
            Operation::MkDir { arg, name } => write!(f, "MKDIR name {:?}, mode {:#05o}", name, arg.mode),
            Operation::Unlink { name } => write!(f, "UNLINK name {:?}", name),
            Operation::RmDir { name } => write!(f, "RMDIR name {:?}", name),
            Operation::Rename { arg, name, newname } => write!(f, "RENAME name {:?}, newdir {:#018x}, newname {:?}", name, arg.newdir, newname),
            Operation::Link { arg, name } => write!(f, "LINK name {:?}, oldnodeid {:#018x}", name, arg.oldnodeid),
            Operation::Open { arg } => write!(f, "OPEN flags {:#x}", arg.flags),
            Operation::Read { arg } => write!(f, "READ fh {}, offset {}, size {}", arg.fh, arg.offset, arg.size),
            Operation::Write { arg, .. } => write!(f, "WRITE fh {}, offset {}, size {}, write flags {:#x}", arg.fh, arg.offset, arg.size, arg.write_flags),
            Operation::StatFs => write!(f, "STATFS"),
            Operation::Release { arg } => write!(f, "RELEASE fh {}, flags {:#x}, release flags {:#x}, lock owner {}", arg.fh, arg.flags, arg.release_flags, arg.lock_owner),
            Operation::FSync { arg } => write!(f, "FSYNC fh {}, fsync flags {:#x}", arg.fh, arg.fsync_flags),
            Operation::SetXAttr { arg, name, .. } => write!(f, "SETXATTR name {:?}, size {}, flags {:#x}", name, arg.size, arg.flags),
            Operation::GetXAttr { arg, name } => write!(f, "GETXATTR name {:?}, size {}", name, arg.size),
            Operation::ListXAttr { arg } => write!(f, "LISTXATTR size {}", arg.size),
            Operation::RemoveXAttr { name } => write!(f, "REMOVEXATTR name {:?}", name),
            Operation::Flush { arg } => write!(f, "FLUSH fh {}, lock owner {}", arg.fh, arg.lock_owner),
            Operation::Init { arg } => write!(f, "INIT kernel ABI {}.{}, flags {:#x}, max readahead {}", arg.major, arg.minor, arg.flags, arg.max_readahead),
            Operation::OpenDir { arg } => write!(f, "OPENDIR flags {:#x}", arg.flags),
            Operation::ReadDir { arg } => write!(f, "READDIR fh {}, offset {}, size {}", arg.fh, arg.offset, arg.size),
            Operation::ReleaseDir { arg } => write!(f, "RELEASEDIR fh {}, flags {:#x}, release flags {:#x}, lock owner {}", arg.fh, arg.flags, arg.release_flags, arg.lock_owner),
            Operation::FSyncDir { arg } => write!(f, "FSYNCDIR fh {}, fsync flags {:#x}", arg.fh, arg.fsync_flags),
            Operation::GetLk { arg } => write!(f, "GETLK fh {}, lock owner {}", arg.fh, arg.owner),
            Operation::SetLk { arg } => write!(f, "SETLK fh {}, lock owner {}", arg.fh, arg.owner),
            Operation::SetLkW { arg } => write!(f, "SETLKW fh {}, lock owner {}", arg.fh, arg.owner),
            Operation::Access { arg } => write!(f, "ACCESS mask {:#05o}", arg.mask),
            Operation::Create { arg, name } => write!(f, "CREATE name {:?}, mode {:#05o}, flags {:#x}", name, arg.mode, arg.flags),
            Operation::Interrupt { arg } => write!(f, "INTERRUPT unique {}", arg.unique),
            Operation::BMap { arg } => write!(f, "BMAP blocksize {}, ids {}", arg.blocksize, arg.block),
            Operation::Destroy => write!(f, "DESTROY"),

            #[cfg(target_os = "macos")]
            Operation::SetVolName { name } => write!(f, "SETVOLNAME name {:?}", name),
            #[cfg(target_os = "macos")]
            Operation::GetXTimes => write!(f, "GETXTIMES"),
            #[cfg(target_os = "macos")]
            Operation::Exchange { arg, oldname, newname } => write!(f, "EXCHANGE olddir {:#018x}, oldname {:?}, newdir {:#018x}, newname {:?}, options {:#x}", arg.olddir, oldname, arg.newdir, newname, arg.options),
        }
    }
}

impl<'a> Operation<'a> {
    fn parse(opcode: &fuse_opcode, data: &mut ArgumentIterator<'a>) -> Option<Self> {
        unsafe {
            Some(match opcode {
                fuse_opcode::FUSE_LOOKUP => Operation::Lookup {
                    name: data.fetch_str()?,
                },
                fuse_opcode::FUSE_FORGET => Operation::Forget { arg: data.fetch()? },
                fuse_opcode::FUSE_GETATTR => Operation::GetAttr,
                fuse_opcode::FUSE_SETATTR => Operation::SetAttr { arg: data.fetch()? },
                fuse_opcode::FUSE_READLINK => Operation::ReadLink,
                fuse_opcode::FUSE_SYMLINK => Operation::SymLink {
                    name: data.fetch_str()?,
                    link: data.fetch_str()?,
                },
                fuse_opcode::FUSE_MKNOD => Operation::MkNod {
                    arg: data.fetch()?,
                    name: data.fetch_str()?,
                },
                fuse_opcode::FUSE_MKDIR => Operation::MkDir {
                    arg: data.fetch()?,
                    name: data.fetch_str()?,
                },
                fuse_opcode::FUSE_UNLINK => Operation::Unlink {
                    name: data.fetch_str()?,
                },
                fuse_opcode::FUSE_RMDIR => Operation::RmDir {
                    name: data.fetch_str()?,
                },
                fuse_opcode::FUSE_RENAME => Operation::Rename {
                    arg: data.fetch()?,
                    name: data.fetch_str()?,
                    newname: data.fetch_str()?,
                },
                fuse_opcode::FUSE_LINK => Operation::Link {
                    arg: data.fetch()?,
                    name: data.fetch_str()?,
                },
                fuse_opcode::FUSE_OPEN => Operation::Open { arg: data.fetch()? },
                fuse_opcode::FUSE_READ => Operation::Read { arg: data.fetch()? },
                fuse_opcode::FUSE_WRITE => Operation::Write {
                    arg: data.fetch()?,
                    data: data.fetch_all(),
                },
                fuse_opcode::FUSE_STATFS => Operation::StatFs,
                fuse_opcode::FUSE_RELEASE => Operation::Release { arg: data.fetch()? },
                fuse_opcode::FUSE_FSYNC => Operation::FSync { arg: data.fetch()? },
                fuse_opcode::FUSE_SETXATTR => Operation::SetXAttr {
                    arg: data.fetch()?,
                    name: data.fetch_str()?,
                    value: data.fetch_all(),
                },
                fuse_opcode::FUSE_GETXATTR => Operation::GetXAttr {
                    arg: data.fetch()?,
                    name: data.fetch_str()?,
                },
                fuse_opcode::FUSE_LISTXATTR => Operation::ListXAttr { arg: data.fetch()? },
                fuse_opcode::FUSE_REMOVEXATTR => Operation::RemoveXAttr {
                    name: data.fetch_str()?,
                },
                fuse_opcode::FUSE_FLUSH => Operation::Flush { arg: data.fetch()? },
                fuse_opcode::FUSE_INIT => Operation::Init { arg: data.fetch()? },
                fuse_opcode::FUSE_OPENDIR => Operation::OpenDir { arg: data.fetch()? },
                fuse_opcode::FUSE_READDIR => Operation::ReadDir { arg: data.fetch()? },
                fuse_opcode::FUSE_RELEASEDIR => Operation::ReleaseDir { arg: data.fetch()? },
                fuse_opcode::FUSE_FSYNCDIR => Operation::FSyncDir { arg: data.fetch()? },
                fuse_opcode::FUSE_GETLK => Operation::GetLk { arg: data.fetch()? },
                fuse_opcode::FUSE_SETLK => Operation::SetLk { arg: data.fetch()? },
                fuse_opcode::FUSE_SETLKW => Operation::SetLkW { arg: data.fetch()? },
                fuse_opcode::FUSE_ACCESS => Operation::Access { arg: data.fetch()? },
                fuse_opcode::FUSE_CREATE => Operation::Create {
                    arg: data.fetch()?,
                    name: data.fetch_str()?,
                },
                fuse_opcode::FUSE_INTERRUPT => Operation::Interrupt { arg: data.fetch()? },
                fuse_opcode::FUSE_BMAP => Operation::BMap { arg: data.fetch()? },
                fuse_opcode::FUSE_DESTROY => Operation::Destroy,

                #[cfg(target_os = "macos")]
                fuse_opcode::FUSE_SETVOLNAME => Operation::SetVolName {
                    name: data.fetch_str()?,
                },
                #[cfg(target_os = "macos")]
                fuse_opcode::FUSE_GETXTIMES => Operation::GetXTimes,
                #[cfg(target_os = "macos")]
                fuse_opcode::FUSE_EXCHANGE => Operation::Exchange {
                    arg: data.fetch()?,
                    oldname: data.fetch_str()?,
                    newname: data.fetch_str()?,
                },
            })
        }
    }
}

/// Low-level request of a filesystem operation the kernel driver wants to perform.
#[derive(Debug)]
pub struct Request<'a> {
    header: &'a fuse_in_header,
    operation: Operation<'a>,
}

impl<'a> fmt::Display for Request<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "FUSE({:3}) ino {:#018x}: {}",
            self.header.unique, self.header.nodeid, self.operation
        )
    }
}

impl<'a> TryFrom<&'a [u8]> for Request<'a> {
    type Error = RequestError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        // Parse a raw packet as sent by the kernel driver into typed data. Every request always
        // begins with a `fuse_in_header` struct followed by arguments depending on the opcode.
        let data_len = data.len();
        let mut data = ArgumentIterator::new(data);
        // Parse header
        let header: &fuse_in_header =
            unsafe { data.fetch() }.ok_or_else(|| RequestError::ShortReadHeader(data.len()))?;
        // Parse/check opcode
        let opcode = fuse_opcode::try_from(header.opcode)
            .map_err(|_: InvalidOpcodeError| RequestError::UnknownOperation(header.opcode))?;
        // Check data size
        if data_len < header.len as usize {
            return Err(RequestError::ShortRead(data_len, header.len as usize));
        }
        // Parse/check operation arguments
        let operation =
            Operation::parse(&opcode, &mut data).ok_or_else(|| RequestError::InsufficientData)?;
        Ok(Self { header, operation })
    }
}

impl<'a> Request<'a> {
    /// Returns the unique identifier of this request.
    ///
    /// The FUSE kernel driver assigns a unique id to every concurrent request. This allows to
    /// distinguish between multiple concurrent requests. The unique id of a request may be
    /// reused in later requests after it has completed.
    #[inline]
    pub fn unique(&self) -> u64 {
        self.header.unique
    }

    /// Returns the node id of the inode this request is targeted to.
    #[inline]
    pub fn nodeid(&self) -> u64 {
        self.header.nodeid
    }

    /// Returns the UID that the process that triggered this request runs under.
    #[inline]
    pub fn uid(&self) -> u32 {
        self.header.uid
    }

    /// Returns the GID that the process that triggered this request runs under.
    #[inline]
    pub fn gid(&self) -> u32 {
        self.header.gid
    }

    /// Returns the PID of the process that triggered this request.
    #[inline]
    pub fn pid(&self) -> u32 {
        self.header.pid
    }

    /// Returns the filesystem operation (and its arguments) of this request.
    #[inline]
    pub fn operation(&self) -> &Operation<'_> {
        &self.operation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_endian = "big")]
    const INIT_REQUEST: [u8; 56] = [
        0x00, 0x00, 0x00, 0x38, 0x00, 0x00, 0x00, 0x1a, // len, opcode
        0xde, 0xad, 0xbe, 0xef, 0xba, 0xad, 0xd0, 0x0d, // unique
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, // nodeid
        0xc0, 0x01, 0xd0, 0x0d, 0xc0, 0x01, 0xca, 0xfe, // uid, gid
        0xc0, 0xde, 0xba, 0x5e, 0x00, 0x00, 0x00, 0x00, // pid, padding
        0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x08, // major, minor
        0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, // max_readahead, flags
    ];

    #[cfg(target_endian = "little")]
    const INIT_REQUEST: [u8; 56] = [
        0x38, 0x00, 0x00, 0x00, 0x1a, 0x00, 0x00, 0x00, // len, opcode
        0x0d, 0xf0, 0xad, 0xba, 0xef, 0xbe, 0xad, 0xde, // unique
        0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // nodeid
        0x0d, 0xd0, 0x01, 0xc0, 0xfe, 0xca, 0x01, 0xc0, // uid, gid
        0x5e, 0xba, 0xde, 0xc0, 0x00, 0x00, 0x00, 0x00, // pid, padding
        0x07, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, // major, minor
        0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // max_readahead, flags
    ];

    #[cfg(target_endian = "big")]
    const MKNOD_REQUEST: [u8; 56] = [
        0x00, 0x00, 0x00, 0x38, 0x00, 0x00, 0x00, 0x08, // len, opcode
        0xde, 0xad, 0xbe, 0xef, 0xba, 0xad, 0xd0, 0x0d, // unique
        0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, // nodeid
        0xc0, 0x01, 0xd0, 0x0d, 0xc0, 0x01, 0xca, 0xfe, // uid, gid
        0xc0, 0xde, 0xba, 0x5e, 0x00, 0x00, 0x00, 0x00, // pid, padding
        0x00, 0x00, 0x01, 0xa4, 0x00, 0x00, 0x00, 0x00, // mode, rdev
        0x66, 0x6f, 0x6f, 0x2e, 0x74, 0x78, 0x74, 0x00, // name
    ];

    #[cfg(target_endian = "little")]
    const MKNOD_REQUEST: [u8; 56] = [
        0x38, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, // len, opcode
        0x0d, 0xf0, 0xad, 0xba, 0xef, 0xbe, 0xad, 0xde, // unique
        0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, // nodeid
        0x0d, 0xd0, 0x01, 0xc0, 0xfe, 0xca, 0x01, 0xc0, // uid, gid
        0x5e, 0xba, 0xde, 0xc0, 0x00, 0x00, 0x00, 0x00, // pid, padding
        0xa4, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mode, rdev
        0x66, 0x6f, 0x6f, 0x2e, 0x74, 0x78, 0x74, 0x00, // name
    ];

    #[test]
    fn setattr() {
        let bit = 1 << 7;
        let v = 268435456;
        println!(
            "{:#x}={:#b}={} {:b} {:x}",
            v, v, v, bit, 18446744071626706816u64
        );

        let reqs: Vec<[u8; 168]> = vec![
            [
                168, 0, 0, 0, 4, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0, 0, 0, 0, 0, 246, 1,
                0, 0, 20, 0, 0, 0, 200, 92, 0, 0, 168, 225, 255, 21, 0, 0, 0, 16, 160, 225, 255,
                21, 2, 130, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 58, 8, 0, 0, 0, 0, 0, 0, 246,
                1, 0, 0, 20, 0, 0, 0, 164, 129, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 72, 0, 4, 2, 8,
                0, 10, 34, 8, 0, 8, 32, 40, 0, 5, 0, 8, 0, 13, 40, 8, 0, 108, 111, 103, 46, 116,
                120, 116, 0, 150, 179, 82, 2, 1, 0, 0, 0, 40, 126, 2, 0, 0, 0, 0, 0, 0, 16, 3, 0,
                128, 79, 218, 131, 255, 255, 255, 255, 0, 0, 0, 0, 188, 116, 5, 0, 0, 0, 0, 0, 126,
                150, 20, 0,
            ], /*
               [
                   168, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 246, 1,
                   0, 0, 20, 0, 0, 0, 120, 83, 0, 0, 221, 218, 255, 21, 0, 0, 0, 16, 203, 218, 255,
                   21, 2, 128, 2, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 105, 25, 0, 0, 0, 0, 0, 0,
                   246, 1, 0, 0, 20, 0, 0, 0, 164, 129, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 64, 0, 4,
                   2, 8, 0, 10, 34, 8, 0, 8, 32, 40, 0, 5, 0, 8, 0, 108, 111, 103, 46, 116, 120, 116,
                   0, 52, 99, 119, 2, 1, 0, 0, 0, 128, 219, 1, 0, 0, 0, 0, 0, 0, 16, 2, 0, 0, 0, 0, 0,
                   128, 79, 218, 131, 255, 255, 255, 255, 160, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0,
               ],
               [
                   168, 0, 0, 0, 4, 0, 0, 0, 17, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 246, 1,
                   0, 0, 20, 0, 0, 0, 92, 83, 0, 0, 101, 0, 118, 0, 0, 0, 0, 16, 116, 0, 114, 0, 97,
                   0, 99, 0, 101, 0, 104, 0, 101, 0, 108, 0, 112, 0, 101, 0, 114, 0, 0, 0, 248, 140,
                   143, 29, 0, 0, 0, 0, 144, 161, 124, 94, 0, 0, 0, 0, 248, 140, 143, 29, 0, 0, 0, 0,
                   144, 161, 124, 94, 0, 0, 0, 0, 248, 140, 143, 29, 0, 0, 0, 0, 246, 1, 0, 0, 20, 0,
                   0, 0, 237, 65, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0, 0, 128, 79, 218, 131, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   175, 149, 230, 150,
               ],
               [
                   168, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 246, 1,
                   0, 0, 20, 0, 0, 0, 120, 83, 0, 0, 221, 218, 255, 21, 0, 0, 0, 16, 203, 218, 255,
                   21, 2, 128, 2, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 105, 25, 0, 0, 0, 0, 0, 0,
                   246, 1, 0, 0, 20, 0, 0, 0, 164, 129, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 64, 0, 4,
                   2, 8, 0, 10, 34, 8, 0, 8, 32, 40, 0, 5, 0, 8, 0, 108, 111, 103, 46, 116, 120, 116,
                   0, 52, 99, 119, 2, 1, 0, 0, 0, 128, 219, 1, 0, 0, 0, 0, 0, 0, 16, 2, 0, 0, 0, 0, 0,
                   128, 79, 218, 131, 255, 255, 255, 255, 160, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0,
               ],
               [
                   168, 0, 0, 0, 4, 0, 0, 0, 15, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 246, 1,
                   0, 0, 20, 0, 0, 0, 88, 81, 0, 0, 62, 218, 255, 21, 0, 0, 0, 16, 52, 218, 255, 21,
                   2, 130, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 232, 2, 0, 0, 0, 0, 0, 0, 246, 1,
                   0, 0, 20, 0, 0, 0, 164, 129, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 72, 0, 4, 2, 8, 0,
                   10, 34, 8, 0, 8, 32, 40, 0, 5, 0, 8, 0, 13, 40, 8, 0, 108, 111, 103, 46, 116, 120,
                   116, 0, 150, 179, 82, 2, 1, 0, 0, 0, 113, 206, 1, 0, 0, 0, 0, 0, 0, 16, 2, 0, 128,
                   79, 218, 131, 255, 255, 255, 255, 0, 0, 0, 0, 110, 204, 1, 0, 0, 0, 0, 0, 4, 0, 0,
                   0,
               ],
               [
                   168, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 246, 1,
                   0, 0, 20, 0, 0, 0, 219, 55, 0, 0, 0, 0, 0, 0, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0, 0, 0, 0, 98, 175, 54, 128, 255, 255, 255, 215, 120, 38, 0, 0, 0, 0, 0, 239,
                   190, 237, 254, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255,
                   255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 128, 129, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0, 0, 0, 0, 0, 0, 0, 0, 25, 0, 0, 0, 6, 0, 0, 0, 31, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
               ],
               [
                   168, 0, 0, 0, 4, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 246, 1,
                   0, 0, 20, 0, 0, 0, 219, 55, 0, 0, 0, 0, 0, 0, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0, 0, 0, 0, 126, 200, 58, 128, 255, 255, 255, 102, 167, 34, 0, 0, 0, 0, 0, 239,
                   190, 237, 254, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0, 0, 0, 0, 0, 0, 0, 0, 164, 129, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0, 0, 5, 0, 0, 0, 5, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 248, 255,
                   255, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0,
               ],
               [
                   168, 0, 0, 0, 4, 0, 0, 0, 17, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 246, 1,
                   0, 0, 20, 0, 0, 0, 219, 55, 0, 0, 0, 0, 0, 0, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 248, 21, 94, 38, 128, 255, 255,
                   255, 1, 0, 0, 0, 164, 129, 0, 0, 120, 206, 180, 30, 128, 255, 255, 255, 0, 176,
                   182, 32, 128, 255, 255, 255, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 16,
                   197, 115, 71, 128, 255, 255, 255, 1, 0, 0, 0, 98, 219, 27, 0, 0, 0, 0, 0, 0, 0, 0,
                   0,
               ],
               [
                   168, 0, 0, 0, 4, 0, 0, 0, 24, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 246, 1,
                   0, 0, 20, 0, 0, 0, 219, 55, 0, 0, 0, 0, 0, 0, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   66, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 56, 92, 48, 50, 128, 255, 255,
                   255, 1, 0, 0, 0, 164, 129, 0, 0, 120, 206, 180, 30, 128, 255, 255, 255, 8, 96, 221,
                   14, 129, 255, 255, 255, 66, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 176, 183,
                   178, 73, 128, 255, 255, 255, 1, 0, 0, 0, 26, 103, 31, 0, 0, 0, 0, 0, 0, 0, 0, 0,
               ],
               [
                   168, 0, 0, 0, 4, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 246, 1,
                   0, 0, 20, 0, 0, 0, 246, 30, 0, 0, 0, 0, 0, 0, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   164, 129, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0, 0, 0, 0, 0,
               ],
               [
                   168, 0, 0, 0, 4, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 246, 1,
                   0, 0, 20, 0, 0, 0, 246, 30, 0, 0, 0, 0, 0, 0, 65, 0, 0, 0, 128, 255, 255, 255, 0,
                   0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 184, 62, 189, 45, 128,
                   255, 255, 255, 1, 0, 0, 0, 164, 129, 0, 0, 240, 172, 196, 49, 128, 255, 255, 255,
                   128, 125, 202, 237, 254, 127, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
                   0, 112, 149, 197, 37, 128, 255, 255, 255, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0,
               ],
               [
                   168, 0, 0, 0, 4, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 246, 1,
                   0, 0, 20, 0, 0, 0, 246, 30, 0, 0, 0, 0, 0, 0, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   164, 129, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11, 0, 0, 0, 6, 0,
                   0, 0, 17, 0, 0, 0, 0, 0, 0, 0, 0, 208, 75, 32, 128, 255, 255, 255, 0, 0, 0, 0, 0,
                   0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
               ],
               [
                   168, 0, 0, 0, 4, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 246, 1,
                   0, 0, 20, 0, 0, 0, 246, 30, 0, 0, 160, 198, 255, 21, 65, 0, 0, 0, 200, 188, 255,
                   21, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 103, 2, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0, 80, 0, 0, 0, 164, 129, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 80, 0, 4, 2, 25,
                   0, 8, 32, 40, 0, 5, 0, 8, 0, 128, 129, 0, 0, 48, 48, 48, 48, 48, 48, 48, 48, 48,
                   52, 52, 49, 46, 116, 114, 97, 99, 101, 118, 51, 0, 0, 0, 0, 0, 0, 0, 0, 248, 146,
                   156, 0, 0, 0, 0, 0, 0, 16, 160, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 248, 210,
                   220, 0,
               ],*/
        ];
        for sr in reqs {
            let req = Request::try_from(&sr[..]).unwrap();
            debug_attr(&req);
        }

        // assert_eq!(req.header.len, 168);
        // assert_eq!(req.header.opcode, 4);
        // assert_eq!(req.unique(), 7);
        // assert_eq!(req.nodeid(), 8);
        // assert_eq!(req.uid(), 502);
        // assert_eq!(req.gid(), 20);
        // match req.operation() {
        //     Operation::SetAttr{ arg } => {
        //         assert_eq!(arg.valid, 65);
        //         assert_eq!(arg.fh, 0);
        //         assert_eq!(arg.size, 1);
        //         // assert_eq!(arg.lock_owner, 0);
        //         assert_eq!(arg.atime, 343597383680);
        //         assert_eq!(arg.mtime, 33188);
        //         assert_eq!(arg.atimensec, 1638916);
        //         assert_eq!(arg.mtimensec, 2629640);
        //         assert_eq!(arg.mode, 33152);
        //         assert_eq!(arg.bkuptime, 863397219);
        //         assert_eq!(arg.chgtime, 44071690216407040);
        //         assert_eq!(arg.crtime, 45053588459749376);
        //         assert_eq!(arg.bkuptimensec, 0);
        //         assert_eq!(arg.chgtimensec, 0);
        //         assert_eq!(arg.crtimensec, 0);
        //         assert_eq!(arg.flags, 14471928);
        //     }
        //     _ => panic!("Unexpected request operation"),
        // }

        fn debug_attr(req: &Request) {
            dbg!(req.header.len);
            dbg!(req.header.opcode);
            dbg!(req.unique());
            dbg!(req.nodeid());
            dbg!(req.uid());
            dbg!(req.gid());
            match req.operation() {
                Operation::SetAttr { arg } => {
                    let valid_bits = format!("{:b}", arg.valid);
                    dbg!(valid_bits);
                    dbg!(arg.valid);
                    dbg!(arg.fh);
                    dbg!(arg.size);
                    // dbg!(arg.lock_owner);
                    dbg!(arg.atime);
                    dbg!(arg.mtime);
                    dbg!(arg.atimensec);
                    dbg!(arg.mtimensec);
                    dbg!(arg.mode);
                    #[cfg(target_os = "macos")]
                    {
                        dbg!(arg.bkuptime);
                        dbg!(arg.chgtime);
                        let crtime_hex = format!("{:x}", arg.crtime);
                        dbg!(crtime_hex);
                        dbg!(arg.crtime as i64);
                        dbg!(arg.crtime);
                        dbg!(arg.bkuptimensec);
                        dbg!(arg.chgtimensec);
                        dbg!(arg.crtimensec);
                        dbg!(arg.flags);
                    }
                }
                _ => panic!("Unexpected request operation"),
            }
        }
    }

    #[test]
    fn short_read_header() {
        match Request::try_from(&INIT_REQUEST[..20]) {
            Err(RequestError::ShortReadHeader(20)) => (),
            _ => panic!("Unexpected request parsing result"),
        }
    }

    #[test]
    fn short_read() {
        match Request::try_from(&INIT_REQUEST[..48]) {
            Err(RequestError::ShortRead(48, 56)) => (),
            _ => panic!("Unexpected request parsing result"),
        }
    }

    #[test]
    fn init() {
        let req = Request::try_from(&INIT_REQUEST[..]).unwrap();
        assert_eq!(req.header.len, 56);
        assert_eq!(req.header.opcode, 26);
        assert_eq!(req.unique(), 0xdead_beef_baad_f00d);
        assert_eq!(req.nodeid(), 0x1122_3344_5566_7788);
        assert_eq!(req.uid(), 0xc001_d00d);
        assert_eq!(req.gid(), 0xc001_cafe);
        assert_eq!(req.pid(), 0xc0de_ba5e);
        match req.operation() {
            Operation::Init { arg } => {
                assert_eq!(arg.major, 7);
                assert_eq!(arg.minor, 8);
                assert_eq!(arg.max_readahead, 4096);
            }
            _ => panic!("Unexpected request operation"),
        }
    }

    #[test]
    fn mknod() {
        let req = Request::try_from(&MKNOD_REQUEST[..]).unwrap();
        assert_eq!(req.header.len, 56);
        assert_eq!(req.header.opcode, 8);
        assert_eq!(req.unique(), 0xdead_beef_baad_f00d);
        assert_eq!(req.nodeid(), 0x1122_3344_5566_7788);
        assert_eq!(req.uid(), 0xc001_d00d);
        assert_eq!(req.gid(), 0xc001_cafe);
        assert_eq!(req.pid(), 0xc0de_ba5e);
        match req.operation() {
            Operation::MkNod { arg, name } => {
                assert_eq!(arg.mode, 0o644);
                assert_eq!(*name, "foo.txt");
            }
            _ => panic!("Unexpected request operation"),
        }
    }
}