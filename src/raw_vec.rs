use std::alloc::{alloc, dealloc, handle_alloc_error, realloc, Layout};
use std::mem;
use std::ptr::Unique;

pub struct RawVec<T> {
    pub ptr: Unique<T>,
    pub cap: usize,
}

impl<T> RawVec<T> {
    pub fn new() -> Self {
        // !0就是usize::MAX。这段分支代码在编译期就可以计算出结果。
        let cap = if mem::size_of::<T>() == 0 { !0 } else { 0 };
        RawVec {
            ptr: Unique::dangling(),
            cap,
        }
    }

    pub fn grow(&mut self) {
        unsafe {
            // 获取到T类型的对齐方式
            let elem_size = mem::size_of::<T>();

            // 因为当elem_size为0时我们设置了cap为usize::MAX，
            // 这一步成立意味着Vec的容量溢出了
            assert!(elem_size != 0, "capacity overflow");

            let align = mem::align_of::<T>();
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
        let elem_size = mem::size_of::<T>();
        // 不要释放零尺寸空间，因为它根本就没有分配过
        if self.cap != 0 && elem_size != 0 {
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
