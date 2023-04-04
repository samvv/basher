
use std::collections::VecDeque;

pub trait Stream {
    type Item;
    fn peek(&mut self, offset: usize) -> Self::Item;
    fn get(&mut self) -> Self::Item;
}

pub struct BufferStream<S: Stream> {
    buffer: VecDeque<S::Item>,
    parent: S,
}

impl <T: Clone, S: Stream<Item = T>> Stream for BufferStream<S> {

    type Item = S::Item;

    fn get(&mut self) -> Self::Item {
        match self.buffer.pop_front() {
            None => self.parent.get(),
            Some(item) => item,
        }
    }

    fn peek(&mut self, offset: usize) -> Self::Item {
        for _ in self.buffer.len()..offset {
            self.buffer.push_back(self.parent.get());
        }
        self.buffer.iter().nth(offset).unwrap().clone()
    }

}
