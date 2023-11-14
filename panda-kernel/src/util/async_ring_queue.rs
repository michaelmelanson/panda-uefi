use core::task::Context;
use core::{pin::Pin, task::Poll};

use crossbeam_queue::ArrayQueue;
use futures_util::{task::AtomicWaker, Future};

#[derive(Debug)]
pub enum AsyncRingQueueError {
    QueueOverflow,
}

pub struct AsyncRingQueue<T> {
    waker: AtomicWaker,
    queue: ArrayQueue<T>,
}

impl<T> AsyncRingQueue<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            waker: AtomicWaker::new(),
            queue: ArrayQueue::new(capacity),
        }
    }

    pub fn push(&self, item: T) -> Result<(), AsyncRingQueueError> {
        self.queue
            .push(item)
            .map_err(|_| AsyncRingQueueError::QueueOverflow)?;
        self.waker.wake();
        Ok(())
    }
}

impl<T> Future for &AsyncRingQueue<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(event) = self.queue.pop() {
            return Poll::Ready(event);
        }

        self.waker.register(cx.waker());

        match self.queue.pop() {
            Some(event) => {
                self.waker.take();
                Poll::Ready(event)
            }
            None => Poll::Pending,
        }
    }
}
