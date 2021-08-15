#![feature(ptr_internals)]
use std::mem;
use std::ptr::{self, Unique}; // 定义一个 unique类型，满足如下条件： 为了锁定内存的内容和owner, 让裸指针拥有数据
                              // 对T可变;
                              // 拥有类型T的值
                              // 如果T是Send/Sync，那么Unique也是Send/Sync
                              // 指针永远不为空
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
}

fn main() {
    let vec: MyVec<i32> = MyVec::new();
}
