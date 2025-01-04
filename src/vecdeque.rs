use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

pub struct RingBuffer<T: Clone + Copy> {
    inner: Arc<Mutex<VecDeque<T>>>,
}

impl<T: Clone + Copy> Clone for RingBuffer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Clone + Copy> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        let buffer = VecDeque::<T>::with_capacity(capacity);

        Self {
            inner: Arc::new(Mutex::new(buffer)),
        }
    }

    pub fn write(&self, buffer: &[T]) -> usize {
        let mut ring_buffer = self.inner.lock().unwrap();
        let available = ring_buffer.capacity() - ring_buffer.len();
        let burst_size = buffer.len();

        if available >= burst_size {
            for value in buffer {
                ring_buffer.push_back(*value);
            }

            burst_size
        } else {
            0
        }
    }

    pub fn read(&self, buffer: &mut [T]) -> usize {
        let mut ring_buffer = self.inner.lock().unwrap();
        let filled = ring_buffer.len();
        let burst_size = buffer.len();

        if filled >= burst_size {
            for index in 0..burst_size {
                buffer[index] = ring_buffer.pop_front().unwrap();
            }

            burst_size
        } else {
            0
        }
    }
}
