use core::alloc::{GlobalAlloc, Layout};

struct BumpAllocator;

const HEAP_SIZE: usize = 2 * 1024 * 1024;

#[repr(C, align(16))]
struct Heap([u8; HEAP_SIZE]);

static mut HEAP: Heap = Heap([0; HEAP_SIZE]);
static mut NEXT: usize = 0;

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator;

// SAFETY: single-core early kernel boot allocator.
unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        // SAFETY: serialized access in this tutorial kernel context.
        unsafe {
            let base = core::ptr::addr_of_mut!(HEAP.0) as usize;
            let start = (base + NEXT + align - 1) & !(align - 1);
            let new_next = start.saturating_add(size).saturating_sub(base);
            if new_next > HEAP_SIZE {
                core::ptr::null_mut()
            } else {
                NEXT = new_next;
                start as *mut u8
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}
