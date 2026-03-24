use super::mystd::ffi::OsString;
use super::{Library, LibrarySegment};
use alloc::vec::Vec;

pub(super) fn native_libraries() -> Vec<Library> {
    let mut buf = vec![0u8; 4096];
    let count = loop {
        match toyos_abi::syscall::query_modules(&mut buf) {
            Ok(count) => break count,
            Err(_) => {
                buf.resize(buf.len() * 2, 0);
                if buf.len() > 1024 * 1024 {
                    return Vec::new();
                }
            }
        }
    };

    let info_size = core::mem::size_of::<toyos_abi::syscall::ModuleInfo>();
    let mut libs = Vec::with_capacity(count);

    for i in 0..count {
        let off = i * info_size;
        if off + info_size > buf.len() {
            break;
        }
        let info =
            unsafe { &*(buf.as_ptr().add(off) as *const toyos_abi::syscall::ModuleInfo) };

        let path_start = info.path_offset as usize;
        let path_end = path_start + info.path_len as usize;
        let name = if path_end <= buf.len() {
            OsString::from(
                core::str::from_utf8(&buf[path_start..path_end]).unwrap_or(""),
            )
        } else {
            OsString::new()
        };

        let size = (info.text_end - info.base) as usize;

        libs.push(Library {
            name,
            segments: vec![LibrarySegment {
                stated_virtual_memory_address: 0,
                len: size,
            }],
            bias: info.base as usize,
        });
    }

    libs
}
