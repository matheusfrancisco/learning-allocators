use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct ArenaOffset {
    pub data: Vec<u8>,
    pub offset: AtomicUsize,
}

impl ArenaOffset {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![0u8; capacity],
            offset: AtomicUsize::new(0),
        }
    }

    pub fn alloc<T>(&self, value: T) -> usize {
        let size = std::mem::size_of::<T>();
        let offset = self.offset.fetch_add(size, Ordering::Relaxed);
        // explaining this for future me
        // on the stack you have
        // [ptr, len, capacity] for the Vec when you do .as_ptr() you get the ptr to the data
        // the data is on the Heap
        //`self.data` is a `Vec<u8>`. A `Vec` is a **three-word struct on the stack**:
        //``` Stack                    Heap
        //┌─────────────┐         ┌────────────────────────────┐
        //│  ptr ───────┼────────▶│  u8 u8 u8 u8 u8 u8 u8 ... │
        //│  len        │         └────────────────────────────┘
        //│  capacity   │
        //└─────────────┘
        // .as_ptr() returns a raw pointer (*const u8) to the heap buffer — the actual bytes.
        // as *mut T this tells the compiler to treat as T instead of u8
        //
        let ptr = self.data.as_ptr() as *mut T;
        unsafe {
            std::ptr::write(ptr.add(offset), value);
        }
        offset
    }

    pub fn get<T>(&self, offset: usize) -> &T {
        let ptr = self.data.as_ptr() as *const T;
        unsafe { &*ptr.add(offset) }
    }

    pub fn get_mut<T>(&self, offset: usize) -> &mut T {
        let ptr = self.data.as_ptr() as *mut T;
        unsafe { &mut *ptr.add(offset) }
    }
}
