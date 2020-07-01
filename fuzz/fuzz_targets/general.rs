#![no_main]

use arbitrary::{Arbitrary, Error, Unstructured};
use gfx_fuzz::*;
use gfx_memory::{GeneralAllocator, GeneralConfig};
use hal::{memory::Properties, MemoryTypeId};

#[derive(Debug)]
struct FuzzingInput {
    config: GeneralConfig,
    allocations: Vec<Allocation>,
}

impl Arbitrary for FuzzingInput {
    fn arbitrary(u: &mut Unstructured) -> Result<Self, Error> {
        let config = GeneralConfig {
            block_size_granularity: *u.choose(POWERS_OF_TWO)?,
            max_chunk_size: *u.choose(POWERS_OF_TWO)?,
            min_device_allocation: *u.choose(POWERS_OF_TWO)?,
        };
        // Need to make sure the values make some sense
        if config.min_device_allocation > config.max_chunk_size {
            return Err(Error::IncorrectFormat);
        }
        let allocations = u.arbitrary()?;
        let input = Self {
            config,
            allocations,
        };
        Ok(input)
    }
}

libfuzzer_sys::fuzz_target!(|input: FuzzingInput| {
    let allocator = GeneralAllocator::new(MemoryTypeId(0), Properties::empty(), input.config, 1);
    perform_allocations(allocator, input.allocations);
});
