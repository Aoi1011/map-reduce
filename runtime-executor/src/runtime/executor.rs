use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::{self, Thread},
};

use crate::future::{Future, PollState};

type Task = Box<dyn Future<Output = String>>;

thread_local! {
    static CURRENT_EXEC: ExecutorCore = ExecutorCore::default();
}

#[derive(Default)]
struct ExecutorCore {
    /// This will hold all the top-level futures associated with the executor on this thread and
    /// allow us to give each an id property to identify them
    tasks: RefCell<HashMap<usize, Task>>,

    /// Stores the IDs of tasks that should be polled by the executor
    ready_queue: Arc<Mutex<Vec<usize>>>,

    /// This is the counter that gives out the next available I
    next_id: Cell<usize>,
}

pub fn spawn<F>(future: F)
where
    F: Future<Output = String> + 'static,
{
    CURRENT_EXEC.with(|e| {
        let id = e.next_id.get();
        e.tasks.borrow_mut().insert(id, Box::new(future));
        e.ready_queue.lock().map(|mut q| q.push(id)).unwrap();
        e.next_id.set(id + 1);
    })
}

pub struct Executor;

impl Executor {
    pub fn new() -> Self {
        Self {}
    }

    /// Takes a lock on `read_queue` and pops off an ID that's ready from the back of
    /// Vec.
    fn pop_ready(&self) -> Option<usize> {
        CURRENT_EXEC.with(|q| q.ready_queue.lock().map(|mut q| q.pop()).unwrap())
    }

    /// Takes the ID of a top-level future as an argument, removes the future from
    /// the tasks collection, and returns it (if the task is found)
    fn get_future(&self, id: usize) -> Option<Task> {
        CURRENT_EXEC.with(|q| q.tasks.borrow_mut().remove(&id))
    }

    /// Creates a new Waker instance.
    fn get_waker(&self, id: usize) -> Waker {
        Waker {
            id,
            thread: thread::current(),
            ready_queue: CURRENT_EXEC.with(|q| q.ready_queue.clone()),
        }
    }

    /// Takes and id property and a Task property and inserts them into our tasks collection.
    fn insert_task(&self, id: usize, task: Task) {
        CURRENT_EXEC.with(|q| q.tasks.borrow_mut().insert(id, task));
    }

    /// Simple returns a count of how many tasks we have in the queue.
    fn task_count(&self) -> usize {
        CURRENT_EXEC.with(|q| q.tasks.borrow().len())
    }

    /// The entry point our Executor.
    pub fn block_on<F>(&mut self, future: F)
    where
        F: Future<Output = String> + 'static,
    {
        spawn(future);
        loop {
            while let Some(id) = self.pop_ready() {
                let mut future = match self.get_future(id) {
                    Some(f) => f,
                    None => continue,
                };
                let waker = self.get_waker(id);
                match future.poll(&waker) {
                    PollState::NotReady => self.insert_task(id, future),
                    PollState::Ready(_) => continue,
                }
            }

            let task_count = self.task_count();
            let name = thread::current().name().unwrap_or_default().to_string();
            if task_count > 0 {
                println!("{name}: {task_count} pending tasks. Sleep until notified.");
                thread::park();
            } else {
                println!("{name}: All tasks are finished");
                break;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Waker {
    /// A handle to the Thread object we mentioned earlier
    thread: Thread,

    /// An usize that identifies which task this Waker is associated with
    id: usize,

    /// This is a reference that can be shared between threads to a Vec<usize>, where usize
    /// represents the ID of a task that's in the ready queue.
    ready_queue: Arc<Mutex<Vec<usize>>>,
}

impl Waker {
    pub fn wake(&self) {
        self.ready_queue
            .lock()
            .map(|mut q| q.push(self.id))
            .unwrap();

        self.thread.unpark()
    }
}
