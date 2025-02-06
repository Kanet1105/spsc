use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use spsc::ring_buffer::{BufferReader, BufferWriter};

const BUFFER_SIZE: u32 = 4096;

fn ring_buffer_1(
    v1: usize,
    v2: usize,
    mut writer: spsc::ring_buffer::Writer<u64>,
    mut reader: spsc::ring_buffer::Reader<u64>,
) {
    let reader_thread = std::thread::spawn(move || {
        let mut read_buffer: Vec<u64> = (0..v1 as u64).map(|_| 0).collect();
        let mut index = 0;

        while index != v1 {
            let n = reader.read(&mut read_buffer[index..index + v2]);
            index += n as usize;
        }

        read_buffer
    });

    let writer_thread = std::thread::spawn(move || {
        let write_buffer: Vec<u64> = (0..v1 as u64).map(|value| value).collect();
        let mut index = 0;

        while index != v1 {
            let n = writer.write(&write_buffer[index..index + v2]);
            index += n as usize;
        }

        write_buffer
    });

    let write_buffer = writer_thread.join().unwrap();
    let read_buffer = reader_thread.join().unwrap();
    assert!(write_buffer == read_buffer);
}

fn ring_buffer_2(
    v1: usize,
    v2: usize,
    reader: spsc::vecdeque::RingBuffer<u64>,
    writer: spsc::vecdeque::RingBuffer<u64>,
) {
    let reader_thread = std::thread::spawn(move || {
        let mut read_buffer: Vec<u64> = (0..v1 as u64).map(|_| 0).collect();
        let mut index = 0;

        while index != v1 {
            let n = reader.read(&mut read_buffer[index..index + v2]);
            index += n;
        }

        read_buffer
    });

    let writer_thread = std::thread::spawn(move || {
        let write_buffer: Vec<u64> = (0..v1 as u64).map(|value| value).collect();
        let mut index = 0;

        while index != v1 {
            let n = writer.write(&write_buffer[index..index + v2]);
            index += n;
        }

        write_buffer
    });

    let write_buffer = writer_thread.join().unwrap();
    let read_buffer = reader_thread.join().unwrap();
    assert!(write_buffer == read_buffer);
}

fn benchmark_ring_buffer_1(c: &mut Criterion) {
    let (writer, reader) = spsc::ring_buffer::ring_buffer::<u64>(BUFFER_SIZE).unwrap();

    c.bench_function("Ring Buffer 1", |b| {
        b.iter(|| {
            ring_buffer_1(
                black_box(100_000),
                black_box(1),
                writer.clone(),
                reader.clone(),
            )
        })
    });
}

fn benchmark_ring_buffer_2(c: &mut Criterion) {
    let ring_buffer = spsc::vecdeque::RingBuffer::<u64>::new(BUFFER_SIZE as usize);

    c.bench_function("Ring Buffer 2", |b| {
        b.iter(|| {
            ring_buffer_2(
                black_box(100_000),
                black_box(1),
                ring_buffer.clone(),
                ring_buffer.clone(),
            )
        })
    });
}

criterion_group!(benchmark, benchmark_ring_buffer_1, benchmark_ring_buffer_2);
criterion_main!(benchmark);
