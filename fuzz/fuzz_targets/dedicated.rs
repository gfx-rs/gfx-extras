#![no_main]

use gfx_fuzz::*;
use gfx_memory::DedicatedAllocator;
use hal::{memory::Properties, MemoryTypeId};

libfuzzer_sys::fuzz_target!(|allocations: Vec<Allocation>| {
    let allocator = DedicatedAllocator::new(MemoryTypeId(0), Properties::DEVICE_LOCAL, 1);
    perform_allocations(allocator, allocations);
});
