#![no_main]

use arbitrary::{Arbitrary, Error, Unstructured};
use gfx_backend_empty::Backend;
use gfx_descriptor::{DescriptorAllocator, DescriptorCounts};
use gfx_fuzz::*;
use hal::{
    device::Device,
    pso::{DescriptorSetLayoutBinding, ShaderStageFlags},
};

#[derive(Debug)]
struct FuzzingInput {
    bindings: Vec<DescriptorSetLayoutBinding>,
    repeats: u8,
    count: u32,
}

impl Arbitrary for FuzzingInput {
    fn arbitrary(u: &mut Unstructured) -> Result<Self, Error> {
        let num_bindings = u.arbitrary_len::<u128>()?;
        let bindings = (0..num_bindings)
            .map(|index| arbitrary_binding(index as u32, u))
            .collect::<Result<Vec<_>, Error>>()?;

        let input = FuzzingInput {
            bindings,
            // Allocate the same layout multiple times
            repeats: u.int_in_range(1..=8)?,
            // Generate a reasonable count of descriptor sets.
            count: u.int_in_range(1..=1024)?,
        };
        Ok(input)
    }
}

// TODO: it would be more elegant to add a new feature to `gfx-backend-empty`,
// which would generate `Arbitrary` implementations for this type.
// For now, this is much more convenient.
fn arbitrary_binding(
    binding: u32,
    u: &mut Unstructured,
) -> Result<DescriptorSetLayoutBinding, Error> {
    let tys = &gfx_descriptor::DESCRIPTOR_TYPES;
    Ok(DescriptorSetLayoutBinding {
        binding,
        ty: u.choose(tys)?.clone(),
        count: u.int_in_range(0..=512)?,
        stage_flags: ShaderStageFlags::ALL,
        immutable_samplers: false,
    })
}

libfuzzer_sys::fuzz_target!(|input: FuzzingInput| {
    let mut allocator = unsafe { DescriptorAllocator::<Backend>::new() };
    let device = create_device();

    // TODO: also generate some random samplers
    let samplers = [];
    let layout = unsafe {
        device
            .create_descriptor_set_layout(&input.bindings, &samplers)
            .unwrap()
    };
    let mut layout_counts = DescriptorCounts::EMPTY;
    for binding in input.bindings {
        layout_counts.add_binding(binding);
    }

    for _ in 0..input.repeats {
        let mut sets = Vec::new();

        allocator
            .allocate(&device, &layout, &layout_counts, input.count, &mut sets)
            .unwrap();

        let actual = sets.len();
        let expected = input.count as usize;
        assert_eq!(actual, expected);

        unsafe {
            allocator.free(sets);
        }
    }

    allocator.cleanup(&device);
});
