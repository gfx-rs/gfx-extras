use arbitrary::Arbitrary;
use gfx_backend_empty as backend;
use gfx_memory::Allocator;
use hal::{adapter::PhysicalDevice, Features, Instance};

/// Structure describing an allocation to be performed.
#[derive(Debug)]
pub struct Allocation {
    /// The size of this allocated block.
    pub size: u64,
    /// Required alignment for this allocation.
    pub alignment: u64,
}

impl Arbitrary for Allocation {
    fn arbitrary(u: &mut arbitrary::Unstructured) -> Result<Self, arbitrary::Error> {
        let allocation = Allocation {
            // Set a limit to ensure the allocations don't get too big.
            size: u.int_in_range(1..=4096)?,
            alignment: *u.choose(POWERS_OF_TWO)?,
        };
        Ok(allocation)
    }
}

/// A lot of the allocation functions require powers of two to work.
///
/// This constant array has some hard-coded values which can be used.
pub const POWERS_OF_TWO: &[u64] = &[1, 2, 4, 8, 16, 32, 64, 128, 256];

/// Creates a new mock device.
pub fn create_device() -> backend::Device {
    let instance =
        backend::Instance::create("gfx-memory fuzzer", 1).expect("Failed to create instance");
    let mut adapters = instance.enumerate_adapters();
    let adapter = adapters.remove(0);
    let family = &adapter.queue_families[0];
    let gpu = unsafe {
        adapter
            .physical_device
            .open(&[(family, &[1.0])], Features::empty())
            .expect("Failed to open logical device")
    };
    gpu.device
}

/// Takes an allocator and tests it with the given allocations.
pub fn perform_allocations(
    mut allocator: impl Allocator<backend::Backend>,
    allocations: Vec<Allocation>,
) {
    let device = create_device();

    let mut blocks = Vec::with_capacity(allocations.len());

    // Allocate all blocks, in order.
    for allocation in allocations.iter() {
        let size = allocation.size as u64;
        let alignment = allocation.alignment as u64;
        // Ensure the allocator either successfully allocates a block of memory,
        // or safely returns an error.
        if let Ok((block, _size)) = allocator.alloc(&device, size, alignment) {
            blocks.push(block);
        }
    }

    // Deallocate all blocks.
    for block in blocks.drain(..) {
        allocator.free(&device, block);
    }
}
