use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;

struct Queue<T> {
    data: Mutex<VecDeque<T>>,
}

impl<T> Queue<T> {
    fn new() -> Self {
        Self {
            data: Mutex::new(VecDeque::new()),
        }
    }

    fn enqueue(&self, item: T) {
        let mut data = self.data.lock().unwrap();
        data.push_back(item);
    }

    fn dequeue(&self) -> Option<T> {
        let mut data = self.data.lock().unwrap();
        data.pop_front()
    }
}
