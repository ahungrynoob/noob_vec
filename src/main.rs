#![feature(ptr_internals)]
use std::ptr::{self, Unique};
use std::{mem, panic}; // 定义一个 unique类型，满足如下条件： 为了锁定内存的内容和owner, 让裸指针拥有数据
                       // 对T可变;
                       // 拥有类型T的值
                       // 如果T是Send/Sync，那么Unique也是Send/Sync
                       // 指针永远不为空
use std::alloc::{alloc, handle_alloc_error, realloc, Layout};
struct MyVec<T> {
    ptr: Unique<T>,
    cap: usize,
    len: usize,
}

impl<T> MyVec<T> {
    fn new() -> Self {
        assert!(mem::size_of::<T>() != 0, "先不处理零尺寸大小的类型");
        MyVec {
            ptr: Unique::dangling(),
            cap: 0,
            len: 0,
        }
    }

    fn grow(&mut self) {
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

    fn push(&mut self, elem: T) {
        if self.len == self.cap {
            self.grow();
        }

        unsafe {
            ptr::write(self.ptr.as_ptr().offset(self.len as isize), elem);
        }

        self.len += 1;
    }

    fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(ptr::read(self.ptr.as_ptr().offset(self.len as isize))) }
        }
    }
}

fn main() {
    let mut vec: MyVec<i32> = MyVec::new();
    vec.push(1);
    if let Some(v) = vec.pop() {
        println!("v == {}", v);
    }
}
