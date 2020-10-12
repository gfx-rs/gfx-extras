# Change Log

## memory-0.2.1 (12-10-2020)
  - remove `hibitset` dependency
  - fix overallocating memory when nothing is allocated yet
  - fix overallocating memory after a few cycles of allocation

## memory-0.2, descriptor-0.2
  - update to gfx-hal-0.6
  - remove `colorful` dependency

## memory-0.1.3 (07-04-2020)
  - fix chunk refcounting in General allocator for dedicated blocks

## memory-0.1.2 (05-04-2020)
  - fix freeing from General allocator

## memory-0.1.1 (31-03-2020)
  - fall back from linear to dedicated kind

## descriptor-0.1 (26-03-2020)
  - original port from Rendy, partially rewritten
  - sub-allocating from pools

## memory-0.1 (26-03-2020)
  - original port from Rendy, partially rewritten
  - simplified `MemoryUsage`
  - explicit `Kind` passed on allocation (Linear/General/Dedicated)
