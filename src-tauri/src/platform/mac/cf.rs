//! The few Core Foundation calls the Accessibility code needs, and an owner that releases.

use std::ffi::c_void;
use std::ptr::{self, NonNull};

pub type CFTypeRef = *const c_void;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CFRange {
    pub location: isize,
    pub length: isize,
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: CFTypeRef);
    fn CFGetTypeID(cf: CFTypeRef) -> usize;
    fn CFStringGetTypeID() -> usize;
    fn CFStringGetLength(string: CFTypeRef) -> isize;
    fn CFStringGetCharacters(string: CFTypeRef, range: CFRange, buffer: *mut u16);
    fn CFStringCreateWithBytes(
        allocator: CFTypeRef,
        bytes: *const u8,
        len: isize,
        encoding: u32,
        is_external: u8,
    ) -> CFTypeRef;
}

const ENCODING_UTF8: u32 = 0x0800_0100;

/// A Core Foundation object we own one reference to (the result of a `Create` or `Copy` call).
pub struct Cf(NonNull<c_void>);

impl Cf {
    /// Takes over the reference; `None` for null.
    ///
    /// # Safety
    /// `ptr` is null or a Core Foundation object whose reference the caller owns and gives up.
    pub unsafe fn from_owned(ptr: CFTypeRef) -> Option<Self> {
        NonNull::new(ptr as *mut c_void).map(Self)
    }

    pub fn as_ptr(&self) -> CFTypeRef {
        self.0.as_ptr()
    }

    pub fn type_id(&self) -> usize {
        // SAFETY: the pointer is a live object we own.
        unsafe { CFGetTypeID(self.as_ptr()) }
    }

    pub fn string(text: &str) -> Option<Self> {
        // SAFETY: the bytes are valid UTF-8 of the given length and are copied; a null allocator
        // means the default one.
        unsafe {
            Self::from_owned(CFStringCreateWithBytes(
                ptr::null(),
                text.as_ptr(),
                text.len() as isize,
                ENCODING_UTF8,
                0,
            ))
        }
    }

    /// The characters as UTF-16 units, the unit Accessibility ranges count in. `None` when this
    /// is not a string.
    pub fn utf16(&self) -> Option<Vec<u16>> {
        // SAFETY: the type is checked first; `buffer` holds exactly the string's length, which is
        // the number of units the call writes.
        unsafe {
            if self.type_id() != CFStringGetTypeID() {
                return None;
            }
            let len = CFStringGetLength(self.as_ptr());
            let mut buffer = vec![0u16; usize::try_from(len).ok()?];
            CFStringGetCharacters(
                self.as_ptr(),
                CFRange {
                    location: 0,
                    length: len,
                },
                buffer.as_mut_ptr(),
            );
            Some(buffer)
        }
    }
}

impl Drop for Cf {
    fn drop(&mut self) {
        // SAFETY: we own one reference to this object, given up here exactly once.
        unsafe { CFRelease(self.as_ptr()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_string_round_trips_through_utf16() {
        let cf = Cf::string("đường Nguyễn Huệ 😀").expect("the string is created");
        let text = String::from_utf16(&cf.utf16().expect("it is a string")).unwrap();
        assert_eq!(text, "đường Nguyễn Huệ 😀");
    }
}
