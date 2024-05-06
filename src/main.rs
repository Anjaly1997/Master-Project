#![cfg_attr(not(test), no_std)]

// Constants for the ring buffer
const SIZE: usize = 256;
const MSGS: usize = 10;

#[derive(Debug)]
enum QueueError {
    FullBuffer,
}

#[derive(Debug)]
struct Message([u8; SIZE]);

struct QueueingPort {
    buffer: [u8; SIZE * MSGS],
    write_index: usize,
    read_index: usize,
    message_count: usize,
}

impl QueueingPort {        
    fn new() -> QueueingPort {
        QueueingPort {
            buffer: [0; SIZE * MSGS],
            write_index: 0,
            read_index: 0,
            message_count: 0,
        }
    }

    fn enqueue(&mut self, message: Message) -> Result<(), QueueError> {
        if self.message_count >= MSGS {
            return Err(QueueError::FullBuffer);
        }
        
        let start = self.write_index * SIZE;
        for i in 0..SIZE {
            self.buffer[start + i] = message.0[i];
        }

        self.write_index = (self.write_index + 1) % MSGS;
        self.message_count += 1;
        Ok(())
    }

    fn dequeue(&mut self) -> Option<Message> {
        if self.message_count == 0 {
            return None;
        }
        
        let start = self.read_index * SIZE;
        let mut msg_array = [0u8; SIZE];
        for i in 0..SIZE {
            msg_array[i] = self.buffer[start + i];
            self.buffer[start + i] = 0;
        }

        self.read_index = (self.read_index + 1) % MSGS;
        self.message_count -= 1;
        Some(Message(msg_array))
    }
}

// Define a panic handler
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
        
        assert!(qp.enqueue(message).is_ok(), "Enqueue is successful");
        let result = qp.dequeue();
        
        assert!(result.is_some(), "Dequeue returning some message");
        assert_eq!(result.unwrap().0, [1; SIZE], "Dequeued message should match the enqueued message");
    }
}
