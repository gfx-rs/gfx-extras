use super::{BlockFlavor, HeapsConfig};
use crate::{allocator::*, stats::*, usage::MemoryUsage, Size};
use hal::memory::Properties;

#[derive(Debug)]
pub(super) struct MemoryType<B: hal::Backend> {
    heap_index: usize,
    properties: Properties,
    dedicated: DedicatedAllocator,
    general: Option<GeneralAllocator<B>>,
    linear: Option<LinearAllocator<B>>,
    used: Size,
    effective: Size,
}

impl<B: hal::Backend> MemoryType<B> {
    pub(super) fn new(
        memory_type: hal::MemoryTypeId,
        heap_index: usize,
        properties: Properties,
        config: HeapsConfig,
        non_coherent_atom_size: Size,
    ) -> Self {
        MemoryType {
            properties,
            heap_index,
            dedicated: DedicatedAllocator::new(memory_type, properties, non_coherent_atom_size),
            linear: config.linear.map(|config| {
                LinearAllocator::new(memory_type, properties, config, non_coherent_atom_size)
            }),
            general: config.general.map(|config| {
                GeneralAllocator::new(memory_type, properties, config, non_coherent_atom_size)
            }),
            used: 0,
            effective: 0,
        }
    }

    pub(super) fn properties(&self) -> Properties {
        self.properties
    }

    pub(super) fn heap_index(&self) -> usize {
        self.heap_index
    }

    pub(super) fn alloc(
        &mut self,
        device: &B::Device,
        usage: MemoryUsage,
        size: Size,
        align: Size,
    ) -> Result<(BlockFlavor<B>, Size), hal::device::AllocationError> {
        let (block, allocated) = self.alloc_impl(device, usage, size, align)?;
        self.effective += block.size();
        self.used += allocated;
        Ok((block, allocated))
    }

    fn alloc_impl(
        &mut self,
        device: &B::Device,
        usage: MemoryUsage,
        size: Size,
        align: Size,
    ) -> Result<(BlockFlavor<B>, Size), hal::device::AllocationError> {
        match (self.general.as_mut(), self.linear.as_mut()) {
            (Some(general), Some(linear)) => {
                if general.max_allocation() >= size
                    && usage.allocator_fitness(Kind::General)
                        > usage.allocator_fitness(Kind::Linear)
                {
                    general
                        .alloc(device, size, align)
                        .map(|(block, size)| (BlockFlavor::General(block), size))
                } else if linear.max_allocation() >= size
                    && usage.allocator_fitness(Kind::Linear) > 0
                {
                    linear
                        .alloc(device, size, align)
                        .map(|(block, size)| (BlockFlavor::Linear(block), size))
                } else {
                    self.dedicated
                        .alloc(device, size, align)
                        .map(|(block, size)| (BlockFlavor::Dedicated(block), size))
                }
            }
            (Some(general), None) => {
                if general.max_allocation() >= size && usage.allocator_fitness(Kind::General) > 0 {
                    general
                        .alloc(device, size, align)
                        .map(|(block, size)| (BlockFlavor::General(block), size))
                } else {
                    self.dedicated
                        .alloc(device, size, align)
                        .map(|(block, size)| (BlockFlavor::Dedicated(block), size))
                }
            }
            (None, Some(linear)) => {
                if linear.max_allocation() >= size && usage.allocator_fitness(Kind::Linear) > 0 {
                    linear
                        .alloc(device, size, align)
                        .map(|(block, size)| (BlockFlavor::Linear(block), size))
                } else {
                    self.dedicated
                        .alloc(device, size, align)
                        .map(|(block, size)| (BlockFlavor::Dedicated(block), size))
                }
            }
            (None, None) => self
                .dedicated
                .alloc(device, size, align)
                .map(|(block, size)| (BlockFlavor::Dedicated(block), size)),
        }
    }

    pub(super) fn free(&mut self, device: &B::Device, block: BlockFlavor<B>) -> Size {
        match block {
            BlockFlavor::Dedicated(block) => self.dedicated.free(device, block),
            BlockFlavor::General(block) => self.general.as_mut().unwrap().free(device, block),
            BlockFlavor::Linear(block) => self.linear.as_mut().unwrap().free(device, block),
        }
    }

    pub(super) fn clear(&mut self, device: &B::Device) {
        log::trace!("Dispose memory allocators");
        if let Some(mut linear) = self.linear.take() {
            linear.clear(device);
        }
        if let Some(mut general) = self.general.take() {
            general.clear(device);
        }
    }

    pub(super) fn utilization(&self) -> MemoryTypeUtilization {
        MemoryTypeUtilization {
            utilization: MemoryUtilization {
                used: self.used,
                effective: self.effective,
            },
            properties: self.properties,
            heap_index: self.heap_index,
        }
    }
}
