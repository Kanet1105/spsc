use std::{
    mem::MaybeUninit,
    ptr::NonNull,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

pub struct RingBuffer<T: Clone + Copy> {
    inner: Arc<RingBufferInner<T>>,
}

struct RingBufferInner<T: Clone + Copy> {
    buffer: NonNull<Vec<T>>,
    capacity: usize,
    head: AtomicUsize,
    tail: AtomicUsize,
}

unsafe impl<T: Clone + Copy> Send for RingBuffer<T> {}

impl<T: Clone + Copy> std::fmt::Debug for RingBuffer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "capacity: {:?}, head: {:?}, tail: {:?}",
            self.inner.capacity, self.inner.head, self.inner.tail,
        )
    }
}

impl<T: Clone + Copy> Clone for RingBuffer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Clone + Copy> RingBuffer<T> {
    pub fn new(capacity: usize) -> Result<(BufferWriter<T>, BufferReader<T>), RingBufferError> {
        let mut buffer = Vec::<T>::with_capacity(capacity);
        let t = unsafe { MaybeUninit::<T>::zeroed().assume_init() };
        (0..capacity).for_each(|_| buffer.push(t));
        let buffer_ptr = Box::into_raw(Box::new(buffer));

        let ring_buffer = Self {
            inner: RingBufferInner {
                buffer: NonNull::new(buffer_ptr).ok_or(RingBufferError::Initialize)?,
                capacity,
                head: 0.into(),
                tail: 0.into(),
            }
            .into(),
        };
        let writer = BufferWriter::new(ring_buffer.clone());
        let reader = BufferReader::new(ring_buffer);

        Ok((writer, reader))
    }

    #[inline(always)]
    fn as_ptr(&self) -> *mut Vec<T> {
        self.inner.buffer.as_ptr()
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.inner.capacity
    }

    #[inline(always)]
    pub fn head_index(&self) -> usize {
        self.inner.head.load(Ordering::SeqCst)
    }

    #[inline(always)]
    pub fn tail_index(&self) -> usize {
        self.inner.tail.load(Ordering::SeqCst)
    }

    #[inline(always)]
    fn advance_head_index(&self, offset: usize) {
        self.inner.head.fetch_add(offset, Ordering::SeqCst);
    }

    #[inline(always)]
    fn advance_tail_index(&self, offset: usize) {
        self.inner.tail.fetch_add(offset, Ordering::SeqCst);
    }
}

pub struct BufferWriter<T: Clone + Copy> {
    ring_buffer: RingBuffer<T>,
}

impl<T: Clone + Copy> BufferWriter<T> {
    fn new(ring_buffer: RingBuffer<T>) -> Self {
        Self { ring_buffer }
    }

    pub fn write(&self, buffer: &[T]) -> usize {
        let capacity = self.ring_buffer.capacity();
        let head_index = self.ring_buffer.head_index();
        let tail_index = self.ring_buffer.tail_index();
        let available = capacity - tail_index.wrapping_sub(head_index);
        let burst_size = buffer.len();

        if available >= burst_size {
            let ring_buffer = unsafe { self.ring_buffer.as_ptr().as_mut().unwrap() };
            for offset in 0..burst_size {
                let index = tail_index.wrapping_add(offset) % capacity;
                ring_buffer[index] = buffer[offset];
            }
            self.ring_buffer.advance_tail_index(burst_size);

            burst_size
        } else {
            0
        }
    }

    pub fn info(&self) -> &RingBuffer<T> {
        &self.ring_buffer
    }

    pub fn ring_buffer(&self) -> &Vec<T> {
        unsafe { self.ring_buffer.as_ptr().as_ref().unwrap() }
    }
}

pub struct BufferReader<T: Clone + Copy> {
    ring_buffer: RingBuffer<T>,
}

impl<T: Clone + Copy> BufferReader<T> {
    fn new(ring_buffer: RingBuffer<T>) -> Self {
        Self { ring_buffer }
    }

    pub fn read(&self, buffer: &mut [T]) -> usize {
        let capacity = self.ring_buffer.capacity();
        let head_index = self.ring_buffer.head_index();
        let tail_index = self.ring_buffer.tail_index();
        let filled = tail_index.wrapping_sub(head_index);
        let burst_size = buffer.len();

        if filled >= burst_size {
            let ring_buffer = unsafe { self.ring_buffer.as_ptr().as_ref().unwrap() };
            for offset in 0..burst_size {
                let index = head_index.wrapping_add(offset) % capacity;
                buffer[offset] = ring_buffer[index];
            }
            self.ring_buffer.advance_head_index(burst_size);

            burst_size
        } else {
            0
        }
    }

    pub fn info(&self) -> &RingBuffer<T> {
        &self.ring_buffer
    }

    pub fn ring_buffer(&self) -> &Vec<T> {
        unsafe { self.ring_buffer.as_ptr().as_ref().unwrap() }
    }
}

#[derive(Debug)]
pub enum RingBufferError {
    Initialize,
}
