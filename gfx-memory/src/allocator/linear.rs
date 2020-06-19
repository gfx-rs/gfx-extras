use crate::{
    allocator::{Allocator, Kind},
    block::Block,
    mapping::MappedRange,
    memory::Memory,
    AtomSize, Size,
};
use hal::{device::Device as _, Backend};
use std::{collections::VecDeque, ops::Range, ptr::NonNull, sync::Arc};

/// Memory block allocated from `LinearAllocator`.
#[derive(Debug)]
pub struct LinearBlock<B: Backend> {
    memory: Arc<Memory<B>>,
    line_index: Size,
    ptr: Option<NonNull<u8>>,
    range: Range<Size>,
}

unsafe impl<B: Backend> Send for LinearBlock<B> {}
unsafe impl<B: Backend> Sync for LinearBlock<B> {}

impl<B: Backend> LinearBlock<B> {
    /// Get the size of this block.
    pub fn size(&self) -> Size {
        self.range.end - self.range.start
    }
}

impl<B: Backend> Block<B> for LinearBlock<B> {
    fn properties(&self) -> hal::memory::Properties {
        self.memory.properties()
    }

    fn memory(&self) -> &B::Memory {
        self.memory.raw()
    }

    fn segment(&self) -> hal::memory::Segment {
        hal::memory::Segment {
            offset: self.range.start,
            size: Some(self.range.end - self.range.start),
        }
    }

    fn map<'a>(
        &'a mut self,
        _device: &B::Device,
        segment: hal::memory::Segment,
    ) -> Result<MappedRange<'a, B>, hal::device::MapError> {
        let requested_range = crate::segment_to_sub_range(segment, &self.range)?;

        let mapping_range = match self.memory.non_coherent_atom_size {
            Some(atom) => crate::align_range(&requested_range, atom),
            None => requested_range.clone(),
        };

        Ok(unsafe {
            MappedRange::from_raw(
                &self.memory,
                self.ptr
                    //TODO: https://github.com/gfx-rs/gfx/issues/3182
                    .ok_or(hal::device::MapError::MappingFailed)?
                    .as_ptr()
                    .offset((mapping_range.start - self.range.start) as isize),
                mapping_range,
                requested_range,
            )
        })
    }
}

/// Config for [`LinearAllocator`].
/// Refer to documentation on [`LinearAllocator`] to better understand what the configuration options mean.
#[derive(Clone, Copy, Debug)]
pub struct LinearConfig {
    /// Size in bytes of each `Line`.
    /// If you try to create an allocation larger then this your allocation will fall back to the general allocator.
    pub line_size: Size,
}

/// The `LinearAllocator` is best suited for short-lived allocations.
/// The allocation strategy has minimal overhead and the implementation is fast.
/// But holding a single block will completely stop memory recycling.
///
/// The linear allocator will internally create multiple lines.
/// Each line is a `gfx_hal::Backend::Memory` from which multiple [`LinearBlock`]s are linearly allocated.
///
/// A new line is created if there is insufficient space to create a [`LinearBlock`] from the current line.
#[derive(Debug)]
pub struct LinearAllocator<B: Backend> {
    memory_type: hal::MemoryTypeId,
    memory_properties: hal::memory::Properties,
    line_size: Size,
    finished_lines_count: Size,
    lines: VecDeque<Line<B>>,
    non_coherent_atom_size: Option<AtomSize>,
}

/// If freed >= allocated it is safe to free the line.
#[derive(Debug)]
struct Line<B: Backend> {
    /// Points to the last allocated byte in the line. Only ever increases.
    allocated: Size,
    /// Points to the last freed byte in the line. Only ever increases.
    freed: Size,
    memory: Arc<Memory<B>>,
    ptr: Option<NonNull<u8>>,
}

unsafe impl<B: Backend> Send for Line<B> {}
unsafe impl<B: Backend> Sync for Line<B> {}

impl<B: Backend> LinearAllocator<B> {
    /// Create new `LinearAllocator`
    /// for `memory_type` with `memory_properties` specified,
    /// with `config`.
    pub fn new(
        memory_type: hal::MemoryTypeId,
        memory_properties: hal::memory::Properties,
        config: &LinearConfig,
        non_coherent_atom_size: Size,
    ) -> Self {
        log::trace!(
            "Create new 'linear' allocator: type: '{:?}', properties: '{:#?}' config: '{:#?}'",
            memory_type,
            memory_properties,
            config
        );
        let (line_size, non_coherent_atom_size) =
            if crate::is_non_coherent_visible(memory_properties) {
                let atom = AtomSize::new(non_coherent_atom_size);
                (crate::align_size(config.line_size, atom.unwrap()), atom)
            } else {
                (config.line_size, None)
            };

        LinearAllocator {
            memory_type,
            memory_properties,
            line_size,
            finished_lines_count: 0,
            lines: VecDeque::new(),
            non_coherent_atom_size,
        }
    }

    /// Maximum allocation size.
    pub fn max_allocation(&self) -> Size {
        self.line_size / 2
    }

    fn cleanup(&mut self, device: &B::Device, off: usize) -> Size {
        let mut freed = 0;
        while self.lines.len() > off {
            if self.lines[0].allocated > self.lines[0].freed {
                break;
            }

            let line = self.lines.pop_front().unwrap();
            self.finished_lines_count += 1;

            match Arc::try_unwrap(line.memory) {
                Ok(mem) => unsafe {
                    log::trace!("Freed 'Line' of size of {}", mem.size());
                    if mem.is_mappable() {
                        device.unmap_memory(mem.raw());
                    }
                    freed += mem.size();
                    device.free_memory(mem.into_raw());
                },
                Err(_) => {
                    log::error!("Allocated `Line` was freed, but memory is still shared and never will be destroyed.");
                }
            }
        }
        freed
    }

    /// Perform full cleanup of the allocated memory.
    pub fn clear(&mut self, device: &B::Device) {
        let _ = self.cleanup(device, 0);
        if !self.lines.is_empty() {
            log::error!(
                "Lines are not empty during allocator disposal. Lines: {:#?}",
                self.lines
            );
        }
    }
}

impl<B: Backend> Allocator<B> for LinearAllocator<B> {
    type Block = LinearBlock<B>;

    const KIND: Kind = Kind::Linear;

    fn alloc(
        &mut self,
        device: &B::Device,
        size: Size,
        align: Size,
    ) -> Result<(LinearBlock<B>, Size), hal::device::AllocationError> {
        let (size, align) = match self.non_coherent_atom_size {
            Some(atom) => (
                crate::align_size(size, atom),
                crate::align_size(align, atom),
            ),
            None => (size, align),
        };

        if size > self.line_size || align > self.line_size {
            //TODO: better error here?
            return Err(hal::device::AllocationError::TooManyObjects);
        }

        let lines_count = self.lines.len() as Size;
        if let Some(line) = self.lines.back_mut() {
            let aligned_offset =
                crate::align_offset(line.allocated, unsafe { AtomSize::new_unchecked(align) });
            if aligned_offset + size <= self.line_size {
                line.freed += aligned_offset - line.allocated;
                line.allocated = aligned_offset + size;

                let block = LinearBlock {
                    line_index: self.finished_lines_count + lines_count - 1,
                    memory: line.memory.clone(),
                    ptr: line.ptr.map(|ptr| unsafe {
                        NonNull::new_unchecked(ptr.as_ptr().offset(aligned_offset as isize))
                    }),
                    range: aligned_offset..aligned_offset + size,
                };

                return Ok((block, 0));
            }
        }

        log::trace!("Allocated 'Line' of size of {}", self.line_size);
        let (memory, ptr) = unsafe {
            super::allocate_memory_helper(
                device,
                self.memory_type,
                self.line_size,
                self.memory_properties,
                self.non_coherent_atom_size,
            )?
        };

        let line = Line {
            allocated: size,
            freed: 0,
            ptr,
            memory: Arc::new(memory),
        };

        let block = LinearBlock {
            line_index: self.finished_lines_count + lines_count,
            memory: Arc::clone(&line.memory),
            ptr,
            range: 0..size,
        };

        self.lines.push_back(line);
        Ok((block, self.line_size))
    }

    fn free(&mut self, device: &B::Device, block: Self::Block) -> Size {
        let index = (block.line_index - self.finished_lines_count) as usize;
        self.lines[index].freed += block.size();
        drop(block);
        self.cleanup(device, 1)
    }
}

impl<B: Backend> Drop for LinearAllocator<B> {
    fn drop(&mut self) {
        if !self.lines.is_empty() {
            log::error!("Not all allocations from LinearAllocator were freed");
        }
    }
}
