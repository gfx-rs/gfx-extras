#![no_main]

use arbitrary::{Arbitrary, Error, Unstructured};
use gfx_fuzz::*;
use gfx_memory::{GeneralAllocator, GeneralConfig, Size};
use hal::{memory::Properties, MemoryTypeId};

#[derive(Debug)]
struct FuzzingInput {
    config: GeneralConfig,
    total_memory: Size,
    allocations: Vec<Allocation>,
}

impl Arbitrary for FuzzingInput {
    fn arbitrary(u: &mut Unstructured) -> Result<Self, Error> {
        let config = GeneralConfig {
            block_size_granularity: *u.choose(POWERS_OF_TWO)?,
            max_chunk_size_as_heap_total_fraction: *u.choose(POWERS_OF_TWO)? as usize,
            min_device_allocation: *u.choose(POWERS_OF_TWO)?,
            significant_size_bits: *u.choose(&[0, 1, 2, 3])?,
        };
        let allocations = u.arbitrary()?;
        let input = Self {
            config,
            total_memory: 1 << *u.choose(&[8, 16, 20, 24, 28])?,
            allocations,
        };
        Ok(input)
    }
}

libfuzzer_sys::fuzz_target!(|input: FuzzingInput| {
    let allocator = GeneralAllocator::new(
        MemoryTypeId(0),
        Properties::empty(),
        input.config,
        1,
        input.total_memory,
    );
    perform_allocations(allocator, input.allocations);
});
