//! Implementation of physical and virtual address and page number.

use super::PageTableEntry;
use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};
use core::fmt::{self, Debug, Formatter};

// RV64: SV39 paging mode
#[cfg(target_pointer_width = "64")]
const PA_WIDTH_SV39: usize = 56;
#[cfg(target_pointer_width = "64")]
const VA_WIDTH_SV39: usize = 39;
#[cfg(target_pointer_width = "64")]
const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;
#[cfg(target_pointer_width = "64")]
const VPN_WIDTH_SV39: usize = VA_WIDTH_SV39 - PAGE_SIZE_BITS;

// RV32: SV32 paging mode
#[cfg(target_pointer_width = "32")]
#[allow(dead_code)]
const PA_WIDTH_SV32: usize = 32;
#[cfg(target_pointer_width = "32")]
#[allow(dead_code)]
const VA_WIDTH_SV32: usize = 32;
#[cfg(target_pointer_width = "32")]
const PPN_WIDTH_SV32: usize = 22;
#[cfg(target_pointer_width = "32")]
const VPN_WIDTH_SV32: usize = 20;

// Architecture-specific constants
#[cfg(target_pointer_width = "64")]
const PPN_WIDTH: usize = PPN_WIDTH_SV39;
#[cfg(target_pointer_width = "32")]
const PPN_WIDTH: usize = PPN_WIDTH_SV32;

#[cfg(target_pointer_width = "64")]
const VPN_WIDTH: usize = VPN_WIDTH_SV39;
#[cfg(target_pointer_width = "32")]
const VPN_WIDTH: usize = VPN_WIDTH_SV32;

/// Number of PTEs per page
#[cfg(target_pointer_width = "64")]
pub const PTES_PER_PAGE: usize = 512;
#[cfg(target_pointer_width = "32")]
pub const PTES_PER_PAGE: usize = 1024;

/// Physical address
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub usize);

/// Virtual address
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub usize);

/// Physical page number
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);

/// Virtual page number
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPageNum(pub usize);

/// Debugging

impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.0))
    }
}
impl Debug for VirtPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("VPN:{:#x}", self.0))
    }
}
impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.0))
    }
}
impl Debug for PhysPageNum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("PPN:{:#x}", self.0))
    }
}

/// T: {PhysAddr, VirtAddr, PhysPageNum, VirtPageNum}
/// T -> usize: T.0
/// usize -> T: usize.into()

impl From<usize> for PhysAddr {
    fn from(v: usize) -> Self {
        #[cfg(target_pointer_width = "64")]
        {
            Self(v & ((1 << PA_WIDTH_SV39) - 1))
        }
        #[cfg(target_pointer_width = "32")]
        {
            Self(v)
        }
    }
}
impl From<usize> for PhysPageNum {
    fn from(v: usize) -> Self {
        Self(v & ((1 << PPN_WIDTH) - 1))
    }
}
impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self {
        #[cfg(target_pointer_width = "64")]
        {
            Self(v & ((1 << VA_WIDTH_SV39) - 1))
        }
        #[cfg(target_pointer_width = "32")]
        {
            Self(v)
        }
    }
}
impl From<usize> for VirtPageNum {
    fn from(v: usize) -> Self {
        Self(v & ((1 << VPN_WIDTH) - 1))
    }
}
impl From<PhysAddr> for usize {
    fn from(v: PhysAddr) -> Self {
        v.0
    }
}
impl From<PhysPageNum> for usize {
    fn from(v: PhysPageNum) -> Self {
        v.0
    }
}
impl From<VirtAddr> for usize {
    fn from(v: VirtAddr) -> Self {
        #[cfg(target_pointer_width = "64")]
        {
            if v.0 >= (1 << (VA_WIDTH_SV39 - 1)) {
                v.0 | (!((1 << VA_WIDTH_SV39) - 1))
            } else {
                v.0
            }
        }
        #[cfg(target_pointer_width = "32")]
        {
            v.0
        }
    }
}
impl From<VirtPageNum> for usize {
    fn from(v: VirtPageNum) -> Self {
        v.0
    }
}

impl VirtAddr {
    /// `VirtAddr` -> `VirtPageNum` (floor)
    pub fn floor(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }
    /// `VirtAddr` -> `VirtPageNum` (ceil)
    pub fn ceil(&self) -> VirtPageNum {
        VirtPageNum((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
    }
    /// Get page offset
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    /// Check page aligned
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}
impl From<VirtAddr> for VirtPageNum {
    fn from(v: VirtAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}
impl From<VirtPageNum> for VirtAddr {
    fn from(v: VirtPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}
impl PhysAddr {
    /// `PhysAddr` -> `PhysPageNum` (floor)
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }
    /// `PhysAddr` -> `PhysPageNum` (ceil)
    pub fn ceil(&self) -> PhysPageNum {
        PhysPageNum((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
    }
    /// Get page offset
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    /// Check page aligned
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
    /// Get mutable reference to `PhysAddr` value
    pub fn get_mut<T>(&self) -> &'static mut T {
        unsafe { (self.0 as *mut T).as_mut().unwrap() }
    }
}
impl From<PhysAddr> for PhysPageNum {
    fn from(v: PhysAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}
impl From<PhysPageNum> for PhysAddr {
    fn from(v: PhysPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}

impl VirtPageNum {
    /// Return VPN indexes for page table walk
    /// RV64 SV39: 3-level page table, each level 9 bits
    /// RV32 SV32: 2-level page table, each level 10 bits
    #[cfg(target_pointer_width = "64")]
    pub fn indexes(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idx = [0usize; 3];
        for i in (0..3).rev() {
            idx[i] = vpn & 511;
            vpn >>= 9;
        }
        idx
    }

    #[cfg(target_pointer_width = "32")]
    pub fn indexes(&self) -> [usize; 2] {
        let mut vpn = self.0;
        let mut idx = [0usize; 2];
        for i in (0..2).rev() {
            idx[i] = vpn & 1023;
            vpn >>= 10;
        }
        idx
    }
}

impl PhysPageNum {
    /// Get `PageTableEntry` array on `PhysPageNum`
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let pa: PhysAddr = (*self).into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut PageTableEntry, PTES_PER_PAGE) }
    }
    /// Get byte array
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhysAddr = (*self).into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096) }
    }
    /// Get mutable reference
    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: PhysAddr = (*self).into();
        pa.get_mut()
    }
}

/// Step by one trait
pub trait StepByOne {
    /// Step
    fn step(&mut self);
}
impl StepByOne for VirtPageNum {
    fn step(&mut self) {
        self.0 += 1;
    }
}

#[derive(Copy, Clone)]
/// A simple range structure for type T
pub struct SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    l: T,
    r: T,
}
impl<T> SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    /// Create new range
    pub fn new(start: T, end: T) -> Self {
        assert!(start <= end, "start {:?} > end {:?}!", start, end);
        Self { l: start, r: end }
    }
    /// Get start
    pub fn get_start(&self) -> T {
        self.l
    }
    /// Get end
    pub fn get_end(&self) -> T {
        self.r
    }
}
impl<T> IntoIterator for SimpleRange<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    type Item = T;
    type IntoIter = SimpleRangeIterator<T>;
    fn into_iter(self) -> Self::IntoIter {
        SimpleRangeIterator::new(self.l, self.r)
    }
}

/// Iterator for the simple range structure
pub struct SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    current: T,
    end: T,
}
impl<T> SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    /// Create new iterator
    pub fn new(l: T, r: T) -> Self {
        Self { current: l, end: r }
    }
}
impl<T> Iterator for SimpleRangeIterator<T>
where
    T: StepByOne + Copy + PartialEq + PartialOrd + Debug,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            None
        } else {
            let t = self.current;
            self.current.step();
            Some(t)
        }
    }
}

/// A simple range structure for virtual page number
pub type VPNRange = SimpleRange<VirtPageNum>;
