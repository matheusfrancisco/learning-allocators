# Arena

An arena allocator (also known as a region allocator) is a memory management technique 
that allows for efficient allocation and deallocation of memory in a contiguous block. It is 
often used in scenarios where there are many small allocations and deallocations, 
such as in game development or real-time applications.

1. Single large block allocation
2. Bump pointer allocation: O(1) time complexity for allocation
3. Bulk deallocation - All memory is freed at once when the arena is dropped (no individual frees)


# Use cases:
- Parser AST nodes
- Game entity systems
- Request handler in web servers
- Any scenario with many small, same-lifetime allocations
 
