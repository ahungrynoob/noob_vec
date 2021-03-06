#![feature(ptr_internals)]
use std::marker::PhantomData;
use std::ptr;
use std::{mem, panic}; // 定义一个 unique类型，满足如下条件： 为了锁定内存的内容和owner, 让裸指针拥有数据
                       // 对T可变;
                       // 拥有类型T的值
                       // 如果T是Send/Sync，那么Unique也是Send/Sync
                       // 指针永远不为空
use std::ops::{Deref, DerefMut};
use std::slice;
mod raw_val_iter;
mod raw_vec;
use raw_val_iter::RawValIter;
use raw_vec::RawVec;
struct MyVec<T> {
    pub buf: RawVec<T>,
    pub len: usize,
}

impl<T> MyVec<T> {
    fn new() -> Self {
        MyVec {
            buf: RawVec::new(),
            len: 0,
        }
    }

    fn ptr(&self) -> *mut T {
        self.buf.ptr.as_ptr()
    }

    fn cap(&self) -> usize {
        self.buf.cap
    }

    fn push(&mut self, elem: T) {
        if self.len == self.cap() {
            self.buf.grow();
        }

        unsafe {
            ptr::write(self.ptr().offset(self.len as isize), elem);
        }

        self.len += 1;
    }

    fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(ptr::read(self.ptr().offset(self.len as isize))) }
        }
    }

    fn insert(&mut self, index: usize, elem: T) {
        assert!(index <= self.len, "overflow");
        if self.cap() == self.len {
            self.buf.grow();
        }
        unsafe {
            if index < self.len {
                ptr::copy(
                    self.ptr().offset(index as isize),
                    self.ptr().offset(index as isize + 1),
                    self.len - index,
                );
            }
            ptr::write(self.ptr().offset(index as isize), elem);
            self.len += 1;
        }
    }

    fn remove(&mut self, index: usize) -> T {
        assert!(index <= self.len, "overflow");
        unsafe {
            self.len -= 1;
            let result = ptr::read(self.ptr().offset(index as isize));

            ptr::copy(
                self.ptr().offset(index as isize + 1),
                self.ptr().offset(index as isize),
                self.len - index,
            );

            result
        }
    }

    fn into_iter(self) -> IntoIter<T> {
        unsafe {
            // 需要使用ptr::read非安全地把buf移出，因为它不是Copy，
            // 而且Vec实现了Drop（所以我们不能销毁它）
            let buf = ptr::read(&self.buf);
            let iter = RawValIter::new(&self);

            mem::forget(self);

            IntoIter {
                // start: buf.ptr.as_ptr(),
                // end: buf.ptr.as_ptr().offset(len as isize),
                _buf: buf,
                iter,
            }
        }
    }

    pub fn drain(&mut self) -> Drain<T> {
        unsafe {
            let iter = RawValIter::new(&self);

            // 这一步是为了mem::forget的安全。如果Drain被forget，我们会泄露整个Vec的内容
            // 同时，既然我们无论如何都会做这一步，为什么不现在做呢？
            self.len = 0;

            Drain {
                iter,
                vec: PhantomData,
            }
        }
    }
}

impl<T> Drop for MyVec<T> {
    fn drop(&mut self) {
        if self.cap() != 0 {
            while let Some(_) = self.pop() {}
            // 释放空间由RawVec负责
        }
    }
}

impl<T> Deref for MyVec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr(), self.len) }
    }
}

impl<T> DerefMut for MyVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.ptr(), self.len) }
    }
}

struct IntoIter<T> {
    _buf: RawVec<T>,
    iter: RawValIter<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.iter.next_back()
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        // 只需要保证所有的元素都被读到了
        // 缓存会在随后自己清理自己
        for _ in &mut self.iter {}
    }
}

struct Drain<'a, T: 'a> {
    // 这里需要限制生命周期。我们使用&'a mut Vec<T>，因为这就是语义上我们包含的东西。
    iter: RawValIter<T>,
    vec: PhantomData<&'a mut MyVec<T>>,
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for Drain<'a, T> {
    fn next_back(&mut self) -> Option<T> {
        self.iter.next_back()
    }
}

impl<'a, T> Drop for Drain<'a, T> {
    fn drop(&mut self) {
        for _ in &mut self.iter {}
    }
}

fn main() {}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    pub fn create_push_pop() {
        let mut v = MyVec::new();
        v.push(1);
        assert_eq!(1, v.len());
        assert_eq!(1, v[0]);
        for i in v.iter_mut() {
            *i += 1;
        }
        v.insert(0, 5);
        let x = v.pop();
        assert_eq!(Some(2), x);
        assert_eq!(1, v.len());
        v.push(10);
        let x = v.remove(0);
        assert_eq!(5, x);
        assert_eq!(1, v.len());
    }
    #[test]
    pub fn iter_test() {
        let mut v = Vec::new();
        for i in 0..10 {
            v.push(Box::new(i))
        }
        let mut iter = v.into_iter();
        let first = iter.next().unwrap();
        let last = iter.next_back().unwrap();
        drop(iter);
        assert_eq!(0, *first);
        assert_eq!(9, *last);
    }

    #[test]
    pub fn test_drain() {
        let mut v = MyVec::new();
        for i in 0..10 {
            v.push(Box::new(i))
        }
        {
            let mut drain = v.drain();
            let first = drain.next().unwrap();
            let last = drain.next_back().unwrap();
            assert_eq!(0, *first);
            assert_eq!(9, *last);
        }
        assert_eq!(0, v.len());
        v.push(Box::new(1));
        assert_eq!(1, *v.pop().unwrap());
    }

    #[test]
    pub fn test_zst() {
        let mut v = MyVec::new();
        for _i in 0..10 {
            v.push(())
        }

        let mut count = 0;

        for _ in v.into_iter() {
            count += 1
        }

        assert_eq!(10, count);
    }
}
