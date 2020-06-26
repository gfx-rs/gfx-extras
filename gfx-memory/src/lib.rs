//! GPU memory management
//!

#![warn(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications
)]
#[allow(clippy::new_without_default)]
mod allocator;
mod block;
mod heaps;
mod mapping;
mod memory;
mod stats;
mod usage;

pub use crate::{
    allocator::*,
    block::Block,
    heaps::{Heaps, HeapsError, MemoryBlock},
    mapping::{MappedRange, Writer},
    memory::Memory,
    stats::*,
    usage::MemoryUsage,
};

use std::ops::Range;

/// Type for any memory sizes.
/// Used in ranges, stats, and internally.
pub type RawSize = u64;
/// Type for non-zero memory size.
pub type Size = std::num::NonZeroU64;

fn is_non_coherent_visible(properties: hal::memory::Properties) -> bool {
    properties.contains(hal::memory::Properties::CPU_VISIBLE)
        && !properties.contains(hal::memory::Properties::COHERENT)
}

fn align_range(range: &Range<RawSize>, align: Size) -> Range<RawSize> {
    let start = range.start - range.start % align.get();
    let end = ((range.end - 1) / align.get() + 1) * align.get();
    start..end
}

fn align_size(size: Size, align: Size) -> Size {
    Size::new(((size.get() - 1) / align.get() + 1) * align.get()).unwrap()
}

fn align_offset(value: RawSize, align: Size) -> RawSize {
    debug_assert_eq!(align.get().count_ones(), 1);
    if value == 0 {
        0
    } else {
        1 + ((value - 1) | (align.get() - 1))
    }
}

fn segment_to_sub_range(
    segment: hal::memory::Segment,
    whole: &Range<RawSize>,
) -> Result<Range<RawSize>, hal::device::MapError> {
    let start = whole.start + segment.offset;
    match segment.size {
        Some(s) if start + s <= whole.end => Ok(start..start + s),
        None if start < whole.end => Ok(start..whole.end),
        _ => Err(hal::device::MapError::OutOfBounds),
    }
}

fn is_sub_range(sub: &Range<RawSize>, range: &Range<RawSize>) -> bool {
    sub.start >= range.start && sub.end <= range.end
}
