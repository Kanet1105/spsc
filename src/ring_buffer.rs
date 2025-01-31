use std::{
    mem::MaybeUninit,
    ptr::NonNull,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};

pub trait BufferWriter<T: Clone + Copy> {
    fn available(&self) -> u32;

    fn get_mut(&mut self, index: u32) -> &mut T;

    fn advance_index(&mut self, offset: u32);

    fn write(&mut self, buffer: &[T]) -> u32;
}

pub trait BufferReader<T: Clone + Copy> {
    fn available(&self) -> u32;

    fn get(&self, index: u32) -> &T;

    fn advance_index(&mut self, offset: u32);

    fn read(&mut self, buffer: &mut [T]) -> u32;
}

pub struct RingBuffer<T: Clone + Copy> {
    inner: Arc<RingBufferInner<T>>,
}

struct RingBufferInner<T: Clone + Copy> {
    buffer: NonNull<Vec<T>>,
    capacity: u32,
    head: AtomicU32,
    tail: AtomicU32,
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
    pub fn new(capacity: usize) -> Result<(Writer<T>, Reader<T>), RingBufferError> {
        let mut buffer = Vec::<T>::with_capacity(capacity);
        let t = unsafe { MaybeUninit::<T>::zeroed().assume_init() };
        (0..capacity).for_each(|_| buffer.push(t));
        let buffer_ptr = Box::into_raw(Box::new(buffer));

        let ring_buffer = Self {
            inner: RingBufferInner {
                buffer: NonNull::new(buffer_ptr).ok_or(RingBufferError::Initialize)?,
                capacity: capacity.try_into().unwrap(),
                head: 0.into(),
                tail: 0.into(),
            }
            .into(),
        };
        let writer = Writer::new(ring_buffer.clone());
        let reader = Reader::new(ring_buffer);

        Ok((writer, reader))
    }

    #[inline(always)]
    pub fn as_mut(&self) -> &mut Vec<T> {
        unsafe { self.inner.buffer.as_ptr().as_mut().unwrap() }
    }

    #[inline(always)]
    pub fn as_ref(&self) -> &Vec<T> {
        unsafe { self.inner.buffer.as_ref() }
    }

    #[inline(always)]
    pub fn capacity(&self) -> u32 {
        self.inner.capacity
    }

    #[inline(always)]
    pub fn head_index(&self) -> u32 {
        self.inner.head.load(Ordering::SeqCst)
    }

    #[inline(always)]
    pub fn tail_index(&self) -> u32 {
        self.inner.tail.load(Ordering::SeqCst)
    }

    #[inline(always)]
    fn advance_head_index(&self, offset: u32) -> u32 {
        self.inner.head.fetch_add(offset, Ordering::SeqCst)
    }

    #[inline(always)]
    fn advance_tail_index(&self, offset: u32) -> u32 {
        self.inner.tail.fetch_add(offset, Ordering::SeqCst)
    }
}

pub struct Writer<T: Clone + Copy> {
    ring_buffer: RingBuffer<T>,
    current_index: u32,
}

impl<T: Clone + Copy> BufferWriter<T> for Writer<T> {
    #[inline(always)]
    fn available(&self) -> u32 {
        let head_index = self.ring_buffer.head_index();
        let tail_index = self.ring_buffer.tail_index();
        let capacity = self.ring_buffer.capacity();

        capacity - tail_index.wrapping_sub(head_index)
    }

    #[inline(always)]
    fn get_mut(&mut self, index: u32) -> &mut T {
        let index = self.current_index.wrapping_add(index) % self.ring_buffer.capacity();
        let ring_buffer = self.ring_buffer.as_mut();

        ring_buffer.get_mut(index as usize).unwrap()
    }

    #[inline(always)]
    fn advance_index(&mut self, offset: u32) {
        let previous_index = self.ring_buffer.advance_tail_index(offset);
        self.current_index = previous_index.wrapping_add(offset);
    }

    #[inline(always)]
    fn write(&mut self, buffer: &[T]) -> u32 {
        let available = self.available();

        if available > 0 {
            let burst_size = std::cmp::min(available, buffer.len() as u32);
            for index in 0..burst_size {
                let data = self.get_mut(index);
                *data = buffer[index as usize];
            }
            self.advance_index(burst_size);

            burst_size
        } else {
            0
        }
    }
}

impl<T: Clone + Copy> Writer<T> {
    fn new(ring_buffer: RingBuffer<T>) -> Self {
        let current_index = ring_buffer.tail_index();

        Self {
            ring_buffer,
            current_index,
        }
    }
}

pub struct Reader<T: Clone + Copy> {
    ring_buffer: RingBuffer<T>,
    current_index: u32,
}

impl<T: Clone + Copy> BufferReader<T> for Reader<T> {
    #[inline(always)]
    fn available(&self) -> u32 {
        let head_index = self.ring_buffer.head_index();
        let tail_index = self.ring_buffer.tail_index();

        tail_index.wrapping_sub(head_index)
    }

    #[inline(always)]
    fn get(&self, index: u32) -> &T {
        let index = self.current_index.wrapping_add(index) % self.ring_buffer.capacity();
        let ring_buffer = self.ring_buffer.as_ref();

        ring_buffer.get(index as usize).unwrap()
    }

    #[inline(always)]
    fn advance_index(&mut self, offset: u32) {
        let previous_index = self.ring_buffer.advance_head_index(offset);
        self.current_index = previous_index.wrapping_add(offset);
    }

    #[inline(always)]
    fn read(&mut self, buffer: &mut [T]) -> u32 {
        let available = self.available();

        if available > 0 {
            let burst_size = std::cmp::min(available, buffer.len() as u32);
            for index in 0..burst_size {
                buffer[index as usize] = *self.get(index);
            }
            self.advance_index(burst_size);

            burst_size
        } else {
            0
        }
    }
}

impl<T: Clone + Copy> Reader<T> {
    fn new(ring_buffer: RingBuffer<T>) -> Self {
        let current_index = ring_buffer.head_index();

        Self {
            ring_buffer,
            current_index,
        }
    }
}

#[derive(Debug)]
pub enum RingBufferError {
    Initialize,
}
