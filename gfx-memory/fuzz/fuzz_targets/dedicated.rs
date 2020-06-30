#![no_main]

use gfx_backend_empty::Backend;
use gfx_memory::{Allocator, DedicatedAllocator, DedicatedBlock};
use gfx_memory_fuzz::{create_device, Allocation};
use hal::{memory::Properties as MemoryProperties, MemoryTypeId};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|allocations: Vec<Allocation>| {
    let device = create_device();

    let mut allocator = DedicatedAllocator::new(MemoryTypeId(0), MemoryProperties::DEVICE_LOCAL, 1);

    let mut blocks = Vec::with_capacity(allocations.len());

    // Allocate all blocks, in order.
    for allocation in allocations.iter() {
        let size = allocation.size as u64;
        let alignment = allocation.alignment as u64;
        let (block, _size): (DedicatedBlock<Backend>, u64) = allocator
            .alloc(&device, size, alignment)
            .expect("Failed to allocate memory");
        blocks.push(block);
    }

    // Deallocate all blocks.
    for block in blocks.drain(..) {
        allocator.free(&device, block);
    }
});
