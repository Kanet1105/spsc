const BUFFER_SIZE: usize = 4096;

fn main() {
    // test_ring_buffer_1(100_000, 100);
    test_ring_buffer_2(100_000, 100);
}

#[allow(unused)]
fn test_ring_buffer_1(v1: usize, v2: usize) {
    use spsc::ring_buffer::{BufferReader, BufferWriter};

    let (mut writer, mut reader) = spsc::ring_buffer::RingBuffer::<u64>::new(BUFFER_SIZE).unwrap();

    std::thread::spawn(move || {
        let mut read_buffer: Vec<u64> = (0..v1 as u64).map(|_| 0).collect();
        let mut index = 0;

        while index != v1 {
            let n = reader.read(&mut read_buffer[index..index + v2]);

            if n > 0 {
                println!("{:?}", &read_buffer[index..index + v2]);
            }

            index += n as usize;
        }

        read_buffer
    });

    let write_buffer: Vec<u64> = (0..v1 as u64).map(|value| value).collect();
    let mut write_index = 0;
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        let n = writer.write(&write_buffer[write_index..write_index + v2]);
        write_index += n as usize;
    }
}

fn test_ring_buffer_2(v1: usize, v2: usize) {
    let writer = spsc::vecdeque::RingBuffer::<u64>::new(BUFFER_SIZE);
    let reader = writer.clone();

    std::thread::spawn(move || {
        let mut read_buffer: Vec<u64> = (0..v1 as u64).map(|_| 0).collect();
        let mut index = 0;

        while index != v1 {
            let n = reader.read(&mut read_buffer[index..index + v2]);

            if n > 0 {
                println!("{:?}", &read_buffer[index..index + v2]);
            }

            index += n;
        }

        read_buffer
    });

    let write_buffer: Vec<u64> = (0..v1 as u64).map(|value| value).collect();
    let mut write_index = 0;
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        let n = writer.write(&write_buffer[write_index..write_index + v2]);
        write_index += n;
    }
}
