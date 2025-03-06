#![no_std]

extern crate alloc;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::cell::UnsafeCell;
use alloc::sync::Arc;
use spin::Mutex;
use core::thread;
use core::time::Duration;
use shared_memory::{Shmem, ShmemConf};


const SIZE: usize = 256;
const MSGS: usize = 10;

// Thread-safe ring buffer queue using atomic operations
struct QueueingPort {
    buffer: &'static mut [u8; SIZE * MSGS], 
    write_index: AtomicUsize,              
    read_index: AtomicUsize,               
    message_count: AtomicUsize,             
}

#[derive(Debug)]
enum QueueError {
    FullBuffer,   // Returned when trying to enqueue into a full queue
    EmptyBuffer,  // Returned when trying to dequeue from an empty queue
}

#[derive(Debug, Clone, Copy)]
struct Message([u8; SIZE]);

impl QueueingPort {
    // Initializes a new queue in shared memory
    fn new() -> Self {
        let shmem = ShmemConf::new().size(SIZE * MSGS).create().unwrap();
        let buffer = unsafe { &mut *(shmem.as_ptr() as *mut [u8; SIZE * MSGS]) };

        QueueingPort {
            buffer, 
            write_index: AtomicUsize::new(0),
            read_index: AtomicUsize::new(0),
            message_count: AtomicUsize::new(0),
        }
    }

    // Enqueues a message into the buffer
    fn enqueue(&mut self, message: Message) -> Result<(), QueueError> {
        if self.message_count.load(Ordering::SeqCst) >= MSGS {
            return Err(QueueError::FullBuffer);
        }

        let write_index = self.write_index.load(Ordering::SeqCst);
        let start = write_index * SIZE;
        for i in 0..SIZE {
            self.buffer[start + i] = message.0[i];
        }

        self.write_index.store((write_index + 1) % MSGS, Ordering::SeqCst);
        self.message_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    // Dequeues a message from the buffer
    fn dequeue(&mut self) -> Result<Message, QueueError> {
        if self.message_count.load(Ordering::SeqCst) == 0 {
            return Err(QueueError::EmptyBuffer);
        }

        let read_index = self.read_index.load(Ordering::SeqCst);
        let start = read_index * SIZE;
        let mut msg_array = [0u8; SIZE];
        for i in 0..SIZE {
            msg_array[i] = self.buffer[start + i];
        }

        self.read_index.store((read_index + 1) % MSGS, Ordering::SeqCst);
        self.message_count.fetch_sub(1, Ordering::SeqCst);
        Ok(Message(msg_array))
    }
}

// Thread-safe queue using `spin::Mutex`
static QUEUE: Mutex<QueueingPort> = Mutex::new(QueueingPort::new());

// Multi-Threading 

// Writer thread function
fn writer(queue: Arc<Mutex<QueueingPort>>) {
    for i in 0..10 {
        let message = Message([i as u8; SIZE]);
        let mut queue = queue.lock();
        if queue.enqueue(message).is_err() {
            println!("Queue full, skipping message");
        }
        thread::sleep(Duration::from_millis(100));
    }
}

// Reader thread function
fn reader(queue: Arc<Mutex<QueueingPort>>) {
    for _ in 0..10 {
        let mut queue = queue.lock();
        if let Ok(msg) = queue.dequeue() {
            assert_eq!(msg.0[0], 0); 
        }
        thread::sleep(Duration::from_millis(100));
    }
}

// Panic handler for `no_std`
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

//  Unit Tests 
#[cfg(test)]
mod tests {
    use super::*;
    use alloc::sync::Arc;
    use spin::Mutex;
    use core::thread;
    use core::sync::atomic::AtomicBool;
    use core::time::Duration;

    #[test]
    fn test_concurrent_read_write() {
        let queue = Arc::new(Mutex::new(QueueingPort::new()));
        let writer_queue = Arc::clone(&queue);
        let reader_queue = Arc::clone(&queue);

        let writer_thread = thread::spawn(move || {
            for i in 0..10 {
                let message = Message([i as u8; SIZE]);
                let mut queue = writer_queue.lock();
                queue.enqueue(message).ok(); 
                std::thread::sleep(Duration::from_millis(100));
            }
        });

        let reader_thread = thread::spawn(move || {
            for _ in 0..10 {
                let mut queue = reader_queue.lock();
                if let Ok(msg) = queue.dequeue() {
                    assert_eq!(msg.0[0], 0); 
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        });

        writer_thread.join().unwrap();
        reader_thread.join().unwrap();
    }
}
