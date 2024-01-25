// Constants for the ring buffer
const SIZE: usize = 256; 
const MSGS: usize = 10; 

#[derive(Debug)]
enum QueueError {
    FullBuffer,
}

//message of fixed size
#[derive(Debug)]
struct Message([u8; SIZE]);

// queueing port
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

    // Adds a message to the queue
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

    // Removes a message from the queue
    fn dequeue(&mut self) -> Option<Message> {
        if self.message_count == 0 {
            return None;
        }
        
        let start = self.read_index * SIZE;
        let mut msg_array = [0u8; SIZE];
        for i in 0..SIZE {
            msg_array[i] = self.buffer[start + i];
            // Clear the buffer spot
            self.buffer[start + i] = 0;
        }
        let message = Message(msg_array);

        self.read_index = (self.read_index + 1) % MSGS;
        self.message_count -= 1;
        Some(message)
    }
}

fn main() {
    let mut port = QueueingPort::new();
    let msg = Message([1; SIZE]); 
    match port.enqueue(msg) {
        Ok(()) => println!("Message enqueued successfully"),
        Err(QueueError::FullBuffer) => println!("Queue is full"),
    }

    match port.dequeue() {
        Some(message) => println!("Message dequeued successfully: {:?}", message),
        None => println!("Queue is empty"),
    }
}
