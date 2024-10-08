#![no_std]

use core::sync::atomic::{AtomicUsize, Ordering};
use core::cell::UnsafeCell;

// Constants for the size of each message and the maximum number of messages.
const SIZE: usize = 256;
const MSGS: usize = 10;

// Static buffer representing shared memory in `no_std` environment.
static mut SHARED_BUFFER: [u8; SIZE * MSGS] = [0; SIZE * MSGS];

// Ring buffer queue using atomic operations for thread-safe access.
struct QueueingPort {
    buffer: &'static mut [u8; SIZE * MSGS],   // Shared memory buffer.
    write_index: AtomicUsize,                // Index for writing the next message.
    read_index: AtomicUsize,                 // Index for reading the next message.
    message_count: AtomicUsize,              // Keeps track of the number of messages in the queue.
}

#[derive(Debug)]
enum QueueError {
    FullBuffer,   // Returned when trying to enqueue into a full queue.
    EmptyBuffer,  // Returned when trying to dequeue from an empty queue.
}

#[derive(Debug)]
struct Message([u8; SIZE]); 

impl QueueingPort {
    // Initializes a new queue with shared memory.
    fn new() -> Self {
        QueueingPort {
            buffer: unsafe { &mut SHARED_BUFFER },  // Accesses shared memory buffer.
            write_index: AtomicUsize::new(0),
            read_index: AtomicUsize::new(0),
            message_count: AtomicUsize::new(0),
        }
    }

    // Enqueues a message into the buffer.
    fn enqueue(&mut self, message: Message) -> Result<(), QueueError> {
        if self.message_count.load(Ordering::SeqCst) >= MSGS {
            return Err(QueueError::FullBuffer);
        }

        let write_index = self.write_index.load(Ordering::SeqCst);
        let start = write_index * SIZE;
        for i in 0..SIZE {
            self.buffer[start + i] = message.0[i];
        }

        // Updates write index and increments the message count.
        self.write_index.store((write_index + 1) % MSGS, Ordering::SeqCst);
        self.message_count.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    // Dequeues a message from the buffer.
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

        // Updates read index and decrements the message count.
        self.read_index.store((read_index + 1) % MSGS, Ordering::SeqCst);
        self.message_count.fetch_sub(1, Ordering::SeqCst);
        Ok(Message(msg_array))
    }
}

// Simulated shared memory between "threads" in `no_std` using UnsafeCell.
struct Shared<T> {
    inner: UnsafeCell<T>,
}

unsafe impl<T> Sync for Shared<T> {}

// Global queue shared between two simulated "threads".
static QUEUE: Shared<QueueingPort> = Shared {
    inner: UnsafeCell::new(QueueingPort::new()),
};

// Simulated writer thread.
fn write_thread() {
    let queue = unsafe { &mut *QUEUE.inner.get() };

    for i in 0..5 {
        let message = Message([i as u8; SIZE]);
        queue.enqueue(message).ok();  // Ignore errors for simplicity.
    }
}

// Simulated reader thread.
fn read_thread() {
    let queue = unsafe { &mut *QUEUE.inner.get() };

    for _ in 0..5 {
        queue.dequeue().ok();  // Ignore errors for simplicity.
    }
}

// Panic handler for `no_std` environments.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// Main function simulates both writer and reader accessing the shared queue.
fn main() {
    write_thread();  // Simulate writing messages.
    read_thread();   // Simulate reading messages.
}
