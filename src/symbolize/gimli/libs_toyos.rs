use super::mystd::ffi::OsString;
use super::{Library, LibrarySegment};
use alloc::vec::Vec;

/// Every image of the process, from its module list (`toyos::tls`): the
/// executable and each library, named by the path it was loaded from.
pub(super) fn native_libraries() -> Vec<Library> {
    toyos::tls::modules()
        .map(|module| {
            let image = module.image;
            Library {
                name: OsString::from(module.name().to_str().unwrap_or("")),
                segments: vec![LibrarySegment {
                    stated_virtual_memory_address: image.start - image.base,
                    len: image.end - image.start,
                }],
                bias: image.base,
            }
        })
        .collect()
}
