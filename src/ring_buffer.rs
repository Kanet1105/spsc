use std::{
    mem::MaybeUninit,
    ptr::NonNull,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};

pub trait BufferWriter<T: Copy> {
    fn available(&self, size: u32) -> (u32, u32);

    fn get_mut(&mut self, index: u32) -> &mut T;

    fn advance_index(&mut self, offset: u32);

    fn write(&mut self, buffer: &[T]) -> u32;
}

pub trait BufferReader<T: Copy> {
    fn filled(&self, size: u32) -> (u32, u32);

    fn get(&self, index: u32) -> &T;

    fn advance_index(&mut self, offset: u32);

    fn read(&mut self, buffer: &mut [T]) -> u32;
}

pub fn ring_buffer<T: Copy>(capacity: u32) -> Result<(Writer<T>, Reader<T>), RingBufferError> {
    let mut buffer = Vec::<T>::with_capacity(capacity as usize);
    let t = unsafe { MaybeUninit::<T>::zeroed().assume_init() };
    (0..capacity).for_each(|_| buffer.push(t));
    let buffer_ptr = Box::into_raw(Box::new(buffer));

    let buffer = NonNull::new(buffer_ptr).ok_or(RingBufferError::Initialize)?;
    let head = Arc::new(AtomicU32::new(0));
    let tail = Arc::new(AtomicU32::new(0));

    let writer = Writer::new(buffer.clone(), capacity, head.clone(), tail.clone());
    let reader = Reader::new(buffer, capacity, head, tail);

    Ok((writer, reader))
}

pub struct Writer<T: Copy> {
    buffer: NonNull<Vec<T>>,
    capacity: u32,
    head: Arc<AtomicU32>,
    tail: Arc<AtomicU32>,
}

unsafe impl<T: Copy> Send for Writer<T> {}

impl<T: Copy> BufferWriter<T> for Writer<T> {
    #[inline(always)]
    fn available(&self, size: u32) -> (u32, u32) {
        let head_index = self.head.load(Ordering::SeqCst);
        let tail_index = self.tail.load(Ordering::SeqCst);

        let available = self.capacity - tail_index.wrapping_sub(head_index);
        if available >= size {
            (size, tail_index)
        } else {
            (0, tail_index)
        }
    }

    #[inline(always)]
    fn get_mut(&mut self, index: u32) -> &mut T {
        let index = index % self.capacity;
        let buffer = unsafe { self.buffer.as_mut() };

        &mut buffer[index as usize]
    }

    #[inline(always)]
    fn advance_index(&mut self, offset: u32) {
        self.tail.fetch_add(offset, Ordering::SeqCst);
    }

    #[inline(always)]
    fn write(&mut self, buffer: &[T]) -> u32 {
        let (available, index) = self.available(buffer.len() as u32);

        if available > 0 {
            for offset in 0..available {
                let data = self.get_mut(index + offset);
                *data = buffer[offset as usize];
            }
            self.advance_index(available);

            available
        } else {
            0
        }
    }
}

impl<T: Copy> Writer<T> {
    fn new(
        buffer: NonNull<Vec<T>>,
        capacity: u32,
        head: Arc<AtomicU32>,
        tail: Arc<AtomicU32>,
    ) -> Self {
        Self {
            buffer,
            capacity,
            head,
            tail,
        }
    }
}

pub struct Reader<T: Copy> {
    buffer: NonNull<Vec<T>>,
    capacity: u32,
    head: Arc<AtomicU32>,
    tail: Arc<AtomicU32>,
}

unsafe impl<T: Copy> Send for Reader<T> {}

impl<T: Copy> BufferReader<T> for Reader<T> {
    #[inline(always)]
    fn filled(&self, size: u32) -> (u32, u32) {
        let head_index = self.head.load(Ordering::SeqCst);
        let tail_index = self.tail.load(Ordering::SeqCst);

        let filled = tail_index.wrapping_sub(head_index);
        if filled >= size {
            (size, head_index)
        } else {
            (0, head_index)
        }
    }

    #[inline(always)]
    fn get(&self, index: u32) -> &T {
        let index = index % self.capacity;
        let buffer = unsafe { self.buffer.as_ref() };

        &buffer[index as usize]
    }

    #[inline(always)]
    fn advance_index(&mut self, offset: u32) {
        self.head.fetch_add(offset, Ordering::SeqCst);
    }

    #[inline(always)]
    fn read(&mut self, buffer: &mut [T]) -> u32 {
        let (filled, index) = self.filled(buffer.len() as u32);

        if filled > 0 {
            for offset in 0..filled {
                buffer[offset as usize] = *self.get(index + offset);
            }
            self.advance_index(filled);

            filled
        } else {
            0
        }
    }
}

impl<T: Copy> Reader<T> {
    fn new(
        buffer: NonNull<Vec<T>>,
        capacity: u32,
        head: Arc<AtomicU32>,
        tail: Arc<AtomicU32>,
    ) -> Self {
        Self {
            buffer,
            capacity,
            head,
            tail,
        }
    }
}

#[derive(Debug)]
pub enum RingBufferError {
    Initialize,
}
