#![warn(
    clippy::unwrap_used,
    missing_docs,
    rust_2018_idioms,
    unused_lifetimes,
    unused_qualifications
)]
#![doc = include_str!("../README.md")]

#[cfg(not(target_os = "linux"))]
compile_error!(
    "Crate anonymous-mmap is Linux specific dependency but is used in non-linux system."
);

/// Error
#[derive(Debug)]
pub enum AnonymousMmapError {
    /// Call to mmap failed with errno
    MmapFailed(std::io::Error),
    /// Call to munmap failed with errno with the non-dropped Self given back.
    MunmapFailed(AnonymousMmap, std::io::Error),
}

impl core::fmt::Display for AnonymousMmapError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MmapFailed(e) => write!(f, "mmap Failed: {}", e),
            Self::MunmapFailed(tlb, e) => write!(f, "Drop / munmap on {:?} Failed: {}", tlb, e),
        }
    }
}

impl core::error::Error for AnonymousMmapError {}

/// An anonymous region of memory mapped using `mmap(2)`, not backed by a file
/// but that is guaranteed to be page-aligned and zero-filled.
#[derive(Debug)]
pub struct AnonymousMmap {
    addr: core::ptr::NonNull<libc::c_void>,
    len: usize,
}

impl AnonymousMmap {
    /// Construct a new AnonymousMmap with the given len of size.
    #[inline]
    pub fn new(len: usize) -> Result<Self, AnonymousMmapError> {
        let p = unsafe {
            libc::mmap(
                core::ptr::null_mut(),
                len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_ANONYMOUS | libc::MAP_SHARED | libc::MAP_POPULATE,
                -1,
                0,
            )
        };

        if p == libc::MAP_FAILED {
            let os_err = std::io::Error::last_os_error();
            return Err(AnonymousMmapError::MmapFailed(os_err));
        }

        Ok(Self {
            // SAFETY: We've checked the error
            addr: unsafe { core::ptr::NonNull::new_unchecked(p) },
            len,
        })
    }
    /// Provide the raw mutable ptr
    /// Same warnigns apply as [`slice::as_mut_ptr`](https://doc.rust-lang.org/std/primitive.slice.html#method.as_mut_ptr).
    #[inline]
    pub fn as_ptr_mut(&self) -> *mut libc::c_void {
        self.addr.as_ptr()
    }
    /// Provide the raw ptr
    #[inline]
    pub fn as_ptr(&self) -> *const libc::c_void {
        self.addr.as_ptr()
    }
    /// Provide ptr with an added offset without bounds checking.
    #[inline]
    pub unsafe fn offset_unchecked_as_ptr(&self, offset: u32) -> *const libc::c_void {
        self.as_ptr().add(offset as usize)
    }
    /// Get a mut pointer to the data at the given offset without bounds checking.
    #[inline]
    pub unsafe fn offset_unchecked_as_ptr_mut(&self, offset: u32) -> *mut libc::c_void {
        self.as_ptr_mut().add(offset as usize)
    }
    /// Given Drop may fail, the consumer is responsible manually handling the drop of the construct.
    #[inline]
    pub unsafe fn try_drop(self) -> Result<(), AnonymousMmapError> {
        // SAFETY: Construct assumes valid construction and initialization with the given capacity.
        let p = unsafe { libc::munmap(self.addr.as_ptr(), self.len) };
        if p != 0 {
            let os_err = std::io::Error::last_os_error();
            return Err(AnonymousMmapError::MunmapFailed(self, os_err));
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(128)]
    fn choices(#[case] len: usize) {
        AnonymousMmap::new(len).unwrap();
    }
}
