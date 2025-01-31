use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use spsc::ring_buffer::{BufferReader, BufferWriter};

const BUFFER_SIZE: usize = 4096;

fn ring_buffer_1(v1: usize, v2: usize) {
    let (mut writer, mut reader) = spsc::ring_buffer::RingBuffer::<u64>::new(BUFFER_SIZE).unwrap();

    let reader = std::thread::spawn(move || {
        let mut read_buffer: Vec<u64> = (0..v1 as u64).map(|_| 0).collect();
        let mut index = 0;

        while index != v1 {
            let n = reader.read(&mut read_buffer[index..index + v2]);
            index += n as usize;
        }

        read_buffer
    });

    let writer = std::thread::spawn(move || {
        let write_buffer: Vec<u64> = (0..v1 as u64).map(|value| value).collect();
        let mut index = 0;

        while index != v1 {
            let n = writer.write(&write_buffer[index..index + v2]);
            index += n as usize;
        }

        write_buffer
    });

    let write_buffer = writer.join().unwrap();
    let read_buffer = reader.join().unwrap();
    assert!(write_buffer == read_buffer);
}

fn ring_buffer_2(v1: usize, v2: usize) {
    let writer = spsc::vecdeque::RingBuffer::<u64>::new(BUFFER_SIZE);
    let reader = writer.clone();

    let reader = std::thread::spawn(move || {
        let mut read_buffer: Vec<u64> = (0..v1 as u64).map(|_| 0).collect();
        let mut index = 0;

        while index != v1 {
            let n = reader.read(&mut read_buffer[index..index + v2]);
            index += n;
        }

        read_buffer
    });

    let writer = std::thread::spawn(move || {
        let write_buffer: Vec<u64> = (0..v1 as u64).map(|value| value).collect();
        let mut index = 0;

        while index != v1 {
            let n = writer.write(&write_buffer[index..index + v2]);
            index += n;
        }

        write_buffer
    });

    let write_buffer = writer.join().unwrap();
    let read_buffer = reader.join().unwrap();
    assert!(write_buffer == read_buffer);
}

fn benchmark_ring_buffer_1(c: &mut Criterion) {
    c.bench_function("Ring Buffer 1", |b| {
        b.iter(|| ring_buffer_1(black_box(100_000), black_box(1)))
    });
}

fn benchmark_ring_buffer_2(c: &mut Criterion) {
    c.bench_function("Ring Buffer 2", |b| {
        b.iter(|| ring_buffer_2(black_box(100_000), black_box(1)))
    });
}

criterion_group!(benchmark, benchmark_ring_buffer_1, benchmark_ring_buffer_2);
criterion_main!(benchmark);
