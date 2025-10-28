#![warn(
    clippy::unwrap_used,
    missing_docs,
    rust_2018_idioms,
    unused_lifetimes,
    unused_qualifications
)]
#![doc = include_str!("../README.md")]

// /sys/kernel/mm/hugepages/hugepages-
// hugepages-1 048 576kB/  hugepages-2048kB/     hugepages-32768kB/    hugepages-64kB/

#[cfg(not(target_os = "linux"))]
compile_error!("Crate hugepages is Linux specific dependency but is used in non-linux system.");

/// Error
#[derive(Debug)]
pub enum HugePageBytesError {
    /// Call to mmap failed with errno
    MmapFailed(std::io::Error),
    /// Call to munmap failed with errno with the non-dropped Self given back.
    MunmapFailed(HugePageBytes, std::io::Error),
}

impl core::fmt::Display for HugePageBytesError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MmapFailed(e) => write!(f, "mmap Failed: {}", e),
            Self::MunmapFailed(tlb, e) => write!(f, "Drop / munmap on {:?} Failed: {}", tlb, e),
        }
    }
}

impl core::error::Error for HugePageBytesError {}

/// HugePage Bytes constructs large size continuous byteslices through mmap() using Linux HugeTLB feature.
/// See https://www.kernel.org/doc/Documentation/admin-guide/mm/hugetlbpage.rst
#[derive(Debug)]
pub struct HugePageBytes {
    addr: *mut u8,
    tlb_choice: HugePageChoice,
}

/// It is the responsibility of the application to understand which sizes are both configured and supported in the kernel.
#[derive(Copy, Clone, Debug)]
#[allow(missing_docs)]
#[allow(non_camel_case_types)]
pub enum HugePageChoice {
    HUGE_64KB,
    HUGE_512KB,
    HUGE_1MB,
    HUGE_2MB,
    HUGE_8MB,
    HUGE_16MB,
    HUGE_32MB,
    HUGE_256MB,
    HUGE_512MB,
    HUGE_1GB,
    HUGE_2GB,
    HUGE_16GB,
}

//fn align_to(size: usize, align: usize) -> usize {
//    (size + align - 1) & !(align - 1)
//}

impl HugePageChoice {
    // Encode the base-2 logarithm of the desired page size in the six bits at the offset libc::MAP_HUGE_SHIFT.
    // However libc has direct mapping of the well known which we just relay
    #[inline]
    fn as_libc_flag(&self) -> libc::c_int {
        match self {
            Self::HUGE_64KB => libc::MAP_HUGE_64KB,
            Self::HUGE_512KB => libc::MAP_HUGE_512KB,
            Self::HUGE_1MB => libc::MAP_HUGE_1MB,
            Self::HUGE_2MB => libc::MAP_HUGE_2MB,
            Self::HUGE_8MB => libc::MAP_HUGE_8MB,
            Self::HUGE_16MB => libc::MAP_HUGE_16MB,
            Self::HUGE_32MB => libc::MAP_HUGE_32MB,
            Self::HUGE_256MB => libc::MAP_HUGE_256MB,
            Self::HUGE_512MB => libc::MAP_HUGE_512MB,
            Self::HUGE_1GB => libc::MAP_HUGE_1GB,
            #[cfg(not(target_pointer_width = "32"))]
            Self::HUGE_2GB => libc::MAP_HUGE_2GB,
            #[cfg(not(target_pointer_width = "32"))]
            Self::HUGE_16GB => libc::MAP_HUGE_16GB,
        }
    }
    #[inline]
    fn as_libc_usize(&self) -> usize {
        match self {
            Self::HUGE_64KB => 65_536,
            Self::HUGE_512KB => 524_288,
            Self::HUGE_1MB => 1_048_576,
            Self::HUGE_2MB => 2_097_152,
            Self::HUGE_8MB => 8_388_608,
            Self::HUGE_16MB => 16_777_216,
            Self::HUGE_32MB => 33_554_432,
            Self::HUGE_256MB => 268_435_456,
            Self::HUGE_512MB => 536_870_912,
            Self::HUGE_1GB => 1_073_741_824,
            #[cfg(not(target_pointer_width = "32"))]
            Self::HUGE_2GB => 2_147_483_648,
            #[cfg(not(target_pointer_width = "32"))]
            Self::HUGE_16GB => 17_179_869_184,
        }
    }
}

impl HugePageBytes {
    /// The range of HugeTLB page sixes can be discovered from /sys/kernel/mm/hugepages.
    /// It is the responsibility of the application to know which sizes are supported on
    /// the running system.  See mmap(2) man page for details.
    #[inline]
    pub fn new(tlb_choice: HugePageChoice) -> Result<Self, HugePageBytesError> {
        let p = unsafe {
            libc::mmap(
                core::ptr::null_mut(),
                tlb_choice.as_libc_usize(),
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE
                    | libc::MAP_ANONYMOUS
                    | libc::MAP_HUGETLB
                    | libc::MAP_POPULATE
                    | tlb_choice.as_libc_flag(),
                -1,
                0,
            )
        };

        if p == libc::MAP_FAILED {
            let os_err = std::io::Error::last_os_error();
            return Err(HugePageBytesError::MmapFailed(os_err));
        }

        Ok(Self {
            addr: p as *mut u8,
            tlb_choice,
        })
    }
    /// Provide the capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        self.tlb_choice.as_libc_usize()
    }
    /// As a mut bytes slice.
    #[inline]
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        // SAFETY: Construct assumes valid construction and initialization with the given capacity.
        unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr(), self.capacity()) }
    }
    /// Provide the raw unsafe mutable ptr of the allocated TLB.
    /// Same warnigns apply as [`slice::as_mut_ptr`](https://doc.rust-lang.org/std/primitive.slice.html#method.as_mut_ptr).    
    #[inline]
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.addr
    }
    /// Given Drop may fail, the consumer is responsible manually handling the drop of the construct.
    #[inline]
    pub fn try_drop(self) -> Result<(), HugePageBytesError> {
        // SAFETY: Construct assumes valid construction and initialization with the given capacity.
        let p = unsafe {
            libc::munmap(
                self.addr as *mut libc::c_void,
                self.tlb_choice.as_libc_usize(),
            )
        };
        if p != 0 {
            let os_err = std::io::Error::last_os_error();
            return Err(HugePageBytesError::MunmapFailed(self, os_err));
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use rstest::rstest;

    #[rstest]
    //#[case(HugePageChoice::HUGE_64KB)]
    //#[case(HugePageChoice::HUGE_512KB)]
    //#[case(HugePageChoice::HUGE_1MB)]
    #[case(HugePageChoice::HUGE_2MB)]
    //#[case(HugePageChoice::HUGE_8MB)]
    //#[case(HugePageChoice::HUGE_16MB)]
    //#[case(HugePageChoice::HUGE_32MB)]
    //#[case(HugePageChoice::HUGE_256MB)]
    //#[case(HugePageChoice::HUGE_512MB)]
    //#[case(HugePageChoice::HUGE_1GB)]
    //#[case(HugePageChoice::HUGE_2GB)]
    //#[case(HugePageChoice::HUGE_16GB)]
    fn choices(#[case] tlb_choice: HugePageChoice) {
        HugePageBytes::new(tlb_choice).unwrap();
    }
}
