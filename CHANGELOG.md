# Change Log

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
