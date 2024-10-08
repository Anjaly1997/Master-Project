#![cfg_attr(not(test), no_std)]

use core::sync::atomic::{AtomicUsize, Ordering};

// Constants for the ring buffer
const SIZE: usize = 256;
const MSGS: usize = 10;

#[derive(Debug)]
enum QueueError {
    FullBuffer,
    EmptyBuffer,
}

#[derive(Debug)]
struct Message([u8; SIZE]);

struct QueueingPort {
    buffer: [u8; SIZE * MSGS],
    write_index: AtomicUsize,
    read_index: AtomicUsize,
    message_count: AtomicUsize,
}

impl QueueingPort {
    fn new() -> QueueingPort {
        QueueingPort {
            buffer: [0; SIZE * MSGS],
            write_index: AtomicUsize::new(0),
            read_index: AtomicUsize::new(0),
            message_count: AtomicUsize::new(0),
        }
    }

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

    fn dequeue(&mut self) -> Result<Message, QueueError> {
        if self.message_count.load(Ordering::SeqCst) == 0 {
            return Err(QueueError::EmptyBuffer);
        }

        let read_index = self.read_index.load(Ordering::SeqCst);
        let start = read_index * SIZE;
        let mut msg_array = [0u8; SIZE];
        for i in 0..SIZE {
            msg_array[i] = self.buffer[start + i];
            self.buffer[start + i] = 0;
        }

        self.read_index.store((read_index + 1) % MSGS, Ordering::SeqCst);
        self.message_count.fetch_sub(1, Ordering::SeqCst);
        Ok(Message(msg_array))
    }
}

// Define a panic handler for no_std
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enqueue_dequeue_test() {
        let mut qp = QueueingPort::new();
        let message = Message([1; SIZE]);
        
        assert!(qp.enqueue(message).is_ok(), "Enqueue should be successful");
        let result = qp.dequeue();
        
        assert!(result.is_ok(), "Dequeue should return a message");
        assert_eq!(result.unwrap().0, [1; SIZE], "Dequeued message should match the enqueued message");
    }

    #[test]
    fn full_queue_test() {
        let mut qp = QueueingPort::new();
        let message = Message([1; SIZE]);
        
        // Fill the queue
        for _ in 0..MSGS {
            assert!(qp.enqueue(message).is_ok());
        }

        // The next enqueue should return a FullBuffer error
        assert!(qp.enqueue(message).is_err());
    }

    #[test]
    fn empty_queue_test() {
        let mut qp = QueueingPort::new();

        // Try to dequeue from an empty queue
        assert!(qp.dequeue().is_err(), "Dequeue should fail with empty buffer");
    }
}
