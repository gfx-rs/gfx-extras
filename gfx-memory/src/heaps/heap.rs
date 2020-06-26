use crate::{
    stats::{MemoryHeapUtilization, MemoryUtilization},
    RawSize, Size,
};

#[derive(Debug)]
pub(super) struct MemoryHeap {
    size: Size,
    used: RawSize,
    effective: RawSize,
}

impl MemoryHeap {
    pub(super) fn new(size: Size) -> Self {
        MemoryHeap {
            size,
            used: 0,
            effective: 0,
        }
    }

    pub(super) fn available(&self) -> RawSize {
        if self.used > self.size.get() {
            log::warn!("Heap size exceeded");
            0
        } else {
            self.size.get() - self.used
        }
    }

    pub(super) fn allocated(&mut self, used: RawSize, effective: RawSize) {
        self.used += used;
        self.effective += effective;
        debug_assert!(self.used >= self.effective);
    }

    pub(super) fn freed(&mut self, used: RawSize, effective: RawSize) {
        self.used -= used;
        self.effective -= effective;
        debug_assert!(self.used >= self.effective);
    }

    pub(super) fn utilization(&self) -> MemoryHeapUtilization {
        MemoryHeapUtilization {
            utilization: MemoryUtilization {
                used: self.used,
                effective: self.effective,
            },
            size: self.size,
        }
    }
}
