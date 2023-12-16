use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{
        AtomicBool,
        Ordering::{self, SeqCst},
    },
    task::{Context, Poll},
};

use futures_util::task::AtomicWaker;

pub struct AsyncFlag {
    waker: AtomicWaker,
    set: AtomicBool,
}

impl AsyncFlag {
    pub fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
            set: AtomicBool::new(false),
        }
    }

    pub fn signal(&mut self) {
        self.set.store(true, Ordering::Relaxed);
        self.waker.wake();
    }
}

impl Future for AsyncFlag {
    type Output = ();

    fn poll(self: Pin<&mut Self>, lw: &mut Context<'_>) -> Poll<()> {
        // Register **before** checking `set` to avoid a race condition
        // that would result in lost notifications.
        self.waker.register(lw.waker());

        if self.set.load(SeqCst) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
