use arbitrary::Arbitrary;
use gfx_backend_empty as backend;
use hal::{adapter::PhysicalDevice, Features, Instance};

/// Structure describing an allocation to be performed.
#[derive(Arbitrary, Debug)]
pub struct Allocation {
    /// The size of this allocated block.
    // This is currently set to `u8` to ensure the allocations don't get too big.
    pub size: u8,
    /// Required alignment for this allocation.
    pub alignment: u8,
}

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
