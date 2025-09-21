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

#[cfg(not(target_os ="linux"))]
compile_error!("Crate hugepages is Linux specific dependency but is used in non-linux system.");

/// Error
#[derive(Debug)]
pub enum HugePageBytesError {
    /// Call to mmap failed with errno
    MmapFailed(std::io::Error),
}

/// HugePage Bytes constructs large size continuous byteslices through mmap() using Linux HugeTLB feature.
/// See https://www.kernel.org/doc/Documentation/admin-guide/mm/hugetlbpage.rst
#[derive(Debug)]
pub struct HugePageBytes {
    addr: *mut u8,
    tlb_choice: HugePageChoice,
}

/// It is the responsibility of the application to understand which sizes are both configured and supported in the kernel.
#[derive(Debug)]
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
            Self::HUGE_2GB => libc::MAP_HUGE_2GB,
            Self::HUGE_16GB => libc::MAP_HUGE_16GB,
        }
    }
    fn as_libc_usize(&self) -> usize {
        match self {
            Self::HUGE_64KB => 64_000,
            Self::HUGE_512KB => 512_000,
            Self::HUGE_1MB => 1_000_000,
            Self::HUGE_2MB => 2_000_000,
            Self::HUGE_8MB => 8_000_000,
            Self::HUGE_16MB => 16_000_000,
            Self::HUGE_32MB => 32_000_000,
            Self::HUGE_256MB => 256_000_000,
            Self::HUGE_512MB => 512_000_000,
            Self::HUGE_1GB => 1_000_000_000,
            Self::HUGE_2GB => 2_000_000_000,
            Self::HUGE_16GB => 16_000_000_000,
        }
    }
}

impl HugePageBytes {
    /// The range of HugeTLB page sixes can be discovered from /sys/kernel/mm/hugepages.
    /// It is the responsibility of the application to know which sizes are supported on
    /// the running system.  See mmap(2) man page for details.
    pub fn new(tlb_choice: HugePageChoice) -> Result<Self, HugePageBytesError> {

        let p = unsafe { libc::mmap(
            core::ptr::null_mut(),
            tlb_choice.as_libc_usize(),
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_HUGETLB | libc::MAP_POPULATE | tlb_choice.as_libc_flag(),
            -1,
            0,
        ) };

        if p == libc::MAP_FAILED {

            let os_err = std::io::Error::last_os_error();
            return Err(HugePageBytesError::MmapFailed(os_err));
        }

        Ok ( Self { addr: p as *mut u8, tlb_choice } )
    }
    /// Provide the raw ptr of the allocated TLB.
    pub fn as_ptr(&self) -> *mut u8 {
        self.addr
    }
}

impl Drop for HugePageBytes {
    fn drop(&mut self) {
        unsafe { libc::munmap(self.addr as *mut libc::c_void, self.tlb_choice.as_libc_usize()) };
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
