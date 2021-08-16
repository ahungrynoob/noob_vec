use std::alloc::{alloc, dealloc, handle_alloc_error, realloc, Layout};
use std::mem;
use std::ptr::Unique;

pub struct RawVec<T> {
    pub ptr: Unique<T>,
    pub cap: usize,
}

impl<T> RawVec<T> {
    pub fn new() -> Self {
        assert!(mem::size_of::<T>() != 0, "TODO:实现零尺寸类型的支持");
        RawVec {
            ptr: Unique::dangling(),
            cap: 0,
        }
    }

    pub fn grow(&mut self) {
        unsafe {
            // 获取到T类型的对齐方式
            let align = mem::align_of::<T>();
            let elem_size = mem::size_of::<T>();
            let layout: Layout;

            let (new_cap, ptr) = if self.cap == 0 {
                layout = Layout::from_size_align_unchecked(elem_size, align);
                let ptr = alloc(layout);
                (1, ptr)
            } else {
                let new_cap = self.cap * 2;
                let old_num_bytes = self.cap * elem_size;
                assert!(
                    old_num_bytes <= (isize::MAX as usize) / 2,
                    "capacity overflow"
                );
                let new_num_bytes = old_num_bytes * 2;
                layout = Layout::from_size_align_unchecked(new_num_bytes, align);
                let ptr = realloc(self.ptr.as_ptr() as *mut _, layout, new_num_bytes);
                (new_cap, ptr)
            };

            if ptr.is_null() {
                handle_alloc_error(layout)
            }

            if let Some(ptr) = Unique::new(ptr as *mut _) {
                self.ptr = ptr;
            } else {
                panic!("error");
            }

            self.cap = new_cap;
        }
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        if self.cap != 0 {
            let align = mem::align_of::<T>();
            let elem_size = mem::size_of::<T>();
            let num_bytes = elem_size * self.cap;

            unsafe {
                let layout: Layout = Layout::from_size_align_unchecked(num_bytes, align);
                dealloc(self.ptr.as_ptr() as *mut _, layout);
            }
        }
    }
}
