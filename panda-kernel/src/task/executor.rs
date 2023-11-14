use core::task::{Context, Poll, Waker};

use alloc::{collections::BTreeMap, sync::Arc};
use crossbeam_queue::ArrayQueue;

use super::{id::TaskId, task_waker::TaskWaker, Task};

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn is_queue_empty(&self) -> bool {
        self.task_queue.is_empty()
    }

    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id();

        if self.tasks.insert(task_id, task).is_some() {
            panic!("task with id {:?} already exists", task_id);
        }
        self.task_queue.push(task_id).unwrap();
    }

    pub fn step(&mut self) {
        let Self {
            task_queue,
            tasks,
            waker_cache,
        } = self;

        while let Some(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // task was already completed
            };

            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }
}
