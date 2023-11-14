mod executor;
mod id;
mod task;
mod task_waker;

use core::future::Future;

use alloc::vec::Vec;
use conquer_once::spin::OnceCell;
use spin::Mutex;

use self::executor::Executor;
pub use self::{id::TaskId, task::Task};

static mut EXECUTOR: OnceCell<Mutex<Executor>> = OnceCell::uninit();

// keeps a list of tasks that were started in the middle of a step
static mut NEW_TASKS: OnceCell<Mutex<Vec<Task>>> = OnceCell::uninit();

pub fn init() {
    unsafe {
        EXECUTOR.init_once(|| Mutex::new(Executor::new()));
        NEW_TASKS.init_once(|| Mutex::new(Vec::new()));
    }
}

pub fn start(task: impl Future<Output = ()> + 'static) {
    let executor = unsafe { EXECUTOR.get().expect("executor not initialized") };
    let task = Task::new(task);

    match executor.try_lock() {
        Some(mut executor) => {
            executor.spawn(task);
        },
        
        None => {
            let mut new_tasks = unsafe { NEW_TASKS.get().expect("New tasks list not initialized").lock() };
            new_tasks.push(task);
        }        
    }
}

pub fn step() {
    let mut executor = unsafe { EXECUTOR.get().expect("executor not initialized").lock() };
    executor.step();

    // if any tasks were started during the stepping, then spawn them now
    let mut new_tasks = unsafe { NEW_TASKS.get().expect("New tasks list not initialized").lock() };
    while let Some(task) = new_tasks.pop() {
        executor.spawn(task);
    }
}

pub fn is_queue_empty() -> bool {
    let executor = unsafe { EXECUTOR.get().expect("executor not initialized").lock() };
    executor.is_queue_empty()
}
