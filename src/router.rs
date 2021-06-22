pub struct Router<T> {
    inbound_queue: Queue<T>,
    outbound_queue: Queue<T>
}

impl <T> Router<T> {
    pub fn new() -> Self {
        Router {
            inbound_queue: Queue::new(),
            outbound_queue: Queue::new(),
        }
    }

    pub fn send_inbound(&mut self, message: T) {
        self.inbound_queue.push(message)
    }

    pub fn send_outbound(&mut self, message: T) {
        self.outbound_queue.push(message)
    }

    pub fn consume_all_inbound(&mut self) -> Vec<T> {
        self.inbound_queue.remove_all()
    }

    pub fn consume_all_outbound(&mut self) -> Vec<T> {
        self.outbound_queue.remove_all()
    }

    pub fn forward_inbound(&mut self, other: &mut Self) {
        self.consume_all_outbound()
            .into_iter()
            .for_each(|message| other.send_inbound(message))
    }

    pub fn forward_outbound(&mut self, other: &mut Self) {
        self.consume_all_outbound()
            .into_iter()
            .for_each(|message| other.send_outbound(message))
    }
}

pub struct Queue<T> {
    queue: Vec<T>
}

impl <T> Queue<T> {
    pub fn new() -> Self {
        Queue {
            queue: Vec::new()
        }
    }

    pub fn remove_all(&mut self) -> Vec<T> {
        self.queue.drain(..)
            .collect()
    }

    pub fn push(&mut self, sent: T) {
        self.queue.push(sent)
    }
}
