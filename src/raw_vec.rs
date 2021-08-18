use std::alloc::{alloc, dealloc, handle_alloc_error, realloc, Layout};
use std::mem;
use std::ptr::Unique;

pub struct RawVec<T> {
    pub ptr: Unique<T>,
    pub cap: usize,
}

impl<T> RawVec<T> {
    pub fn new() -> Self {
        let cap = if mem::size_of::<T>() == 0 { !0 } else { 0 };
        RawVec {
            ptr: Unique::dangling(),
            cap,
        }
    }

    /**
     * use alloc and realloc to enlarge capacity
     */
    pub fn grow(&mut self) {
        let elem_size = mem::size_of::<T>();
        assert!(elem_size != 0, "capacity overflow");
        let align = mem::align_of::<T>();
        let layout: Layout;
        unsafe {
            let (new_cap, ptr) = if self.cap == 0 {
                layout = Layout::from_size_align(elem_size, align).unwrap();
                let ptr = alloc(layout);
                (1, ptr)
            } else {
                let new_cap = self.cap * 2;
                let old_num_bytes = elem_size * self.cap;
                assert!(old_num_bytes <= isize::MAX as usize / 2, "capcity overflow");

                let new_num_bytes = elem_size * new_cap;
                layout = Layout::from_size_align(new_num_bytes, align).unwrap();
                let ptr = realloc(self.ptr.as_ptr() as *mut _, layout, new_num_bytes);
                (new_cap, ptr)
            };

            if ptr.is_null() {
                handle_alloc_error(layout)
            }

            self.ptr = Unique::new(ptr as *mut T).unwrap();
            self.cap = new_cap;
        }
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        let elem_size = mem::size_of::<T>();
        if self.cap != 0 && elem_size != 0 {
            let align = mem::align_of::<T>();
            let num_bytes = elem_size * self.cap;
            let layout = Layout::from_size_align(num_bytes, align).unwrap();
            unsafe {
                dealloc(self.ptr.as_ptr() as *mut _, layout);
            }
        }
    }
}
