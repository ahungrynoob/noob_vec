use super::raw_val_iter::RawValIter;

pub struct Drain<T> {
    pub iter: RawValIter<T>,
}

impl<T> Iterator for Drain<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T> DoubleEndedIterator for Drain<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl<T> Drop for Drain<T> {
    fn drop(&mut self) {
        for _ in &mut self.iter {}
    }
}
