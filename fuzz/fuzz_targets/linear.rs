#![no_main]

use arbitrary::{Arbitrary, Error, Unstructured};
use gfx_fuzz::*;
use gfx_memory::{LinearAllocator, LinearConfig};
use hal::{memory::Properties, MemoryTypeId};

#[derive(Debug)]
struct FuzzingInput {
    config: LinearConfig,
    allocations: Vec<Allocation>,
}

impl Arbitrary for FuzzingInput {
    fn arbitrary(u: &mut Unstructured) -> Result<Self, Error> {
        let config = LinearConfig {
            line_size: u.int_in_range(1..=4096)?,
        };
        let allocations = u.arbitrary()?;
        let input = Self {
            config,
            allocations,
        };
        Ok(input)
    }
}

libfuzzer_sys::fuzz_target!(|input: FuzzingInput| {
    let allocator = LinearAllocator::new(MemoryTypeId(0), Properties::empty(), input.config, 1);
    perform_allocations(allocator, input.allocations);
});
