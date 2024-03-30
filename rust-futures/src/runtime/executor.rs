use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Wake, Waker, Context, Poll};
use std::thread::{self, Thread};

type Task = Pin<Box<dyn Future<Output = ()>>>;

thread_local! {
    static CURRENT_EXE: ExecuteCore = ExecuteCore::default()
}

#[derive(Default)]
pub struct ExecuteCore {
    tasks: RefCell<HashMap<usize, Task>>,
    ready_queue: Arc<Mutex<Vec<usize>>>,
    next_id: Cell<usize>,
}

pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    CURRENT_EXE.with(|e| {
        let id = e.next_id.get();
        e.tasks.borrow_mut().insert(id, Box::pin(future));
        e.ready_queue.lock().map(|mut q| q.push(id)).unwrap();
        e.next_id.set(id + 1);
    });
}

pub struct Executor;

impl Executor {
    pub fn new() -> Self {
        Self {}
    }

    fn pop_ready(&self) -> Option<usize> {
        CURRENT_EXE.with(|q| q.ready_queue.lock().map(|mut q| q.pop()).unwrap())
    }

    fn get_future(&self, id: usize) -> Option<Task> {
        CURRENT_EXE.with(|q| q.tasks.borrow_mut().remove(&id))
    }

    fn get_waker(&self, id: usize) -> Arc<MyWaker> {
        Arc::new(MyWaker {
            thread: thread::current(),
            id,
            ready_queue: CURRENT_EXE.with(|q| q.ready_queue.clone()),
        })
    }

    fn insert_task(&self, id: usize, task: Task) {
        CURRENT_EXE.with(|q| q.tasks.borrow_mut().insert(id, task));
    }

    fn task_count(&self) -> usize {
        CURRENT_EXE.with(|q| q.tasks.borrow().len())
    }

    pub fn block_on<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        spawn(future);

        loop {
            while let Some(id) = self.pop_ready() {
                let mut future = match self.get_future(id) {
                    Some(f) => f,
                    None => continue,
                };
                let waker: Waker = self.get_waker(id).into();
                let mut cx = Context::from_waker(&waker);

                match future.as_mut().poll(&mut cx) {
                    Poll::Ready(_) => continue,
                    Poll::Pending => self.insert_task(id, future),
                }
            }

            let task_count = self.task_count();
            let name = thread::current().name().unwrap_or_default().to_string();

            if task_count > 0 {
                println!("{name}: {task_count} pending tasks. Sleep until notified.");
                std::thread::park();
            } else {
                println!("{name}: All tasks are finished");
                break;
            }
        }
    }
}

#[derive(Clone)]
pub struct MyWaker {
    thread: Thread,
    id: usize,
    ready_queue: Arc<Mutex<Vec<usize>>>,
}

impl MyWaker {
    pub fn wake(&self) {
        self.ready_queue
            .lock()
            .map(|mut q| q.push(self.id))
            .unwrap();
        self.thread.unpark();
    }
}

impl Wake for MyWaker {
    fn wake(self: Arc<Self>) {
        self.ready_queue
            .lock()
            .map(|mut q| q.push(self.id))
            .unwrap();
        self.thread.unpark();
    }
}
