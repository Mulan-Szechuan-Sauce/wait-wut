use std::{any::Any, collections::VecDeque, future::Future, pin::Pin, sync::{Arc, Mutex}, task::{Context, Poll, RawWaker, RawWakerVTable, Waker}, thread::{self, Thread}, time::Instant};
use futures::{task::ArcWake, FutureExt};

use crate::wait_handle::WaitHandle;

pub struct Task {
    future: Pin<Box<dyn Future<Output = Box<dyn Any>> + Send>>,
    output_type: std::any::TypeId,
}
// type Task = Pin<Box<dyn Future<Output = ()> + Send>>;

#[derive(Default)]
pub struct WaitWutRuntime {
    ready_to_poll_queue: VecDeque<Task>,
    sleepy_queue: VecDeque<Task>,
}

impl WaitWutRuntime {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn spawn<F, T>(&mut self, future: F) -> WaitHandle<T>
    where
        T: 'static,
        F: Future<Output = T> + Send + 'static,
    {
        let task = Task {
            future: Box::pin(future.map(|x| Box::new(x) as Box<dyn Any>)),
            output_type: std::any::TypeId::of::<T>(),
        };
        self.ready_to_poll_queue.push_back(task);
        WaitHandle { result: None }
    }

    pub fn run(&mut self) {
        let waker = futures::task::waker(CurrentThreadWaker::new());
        let mut cx = std::task::Context::from_waker(&waker);

        loop {
            self.run_once(&mut cx);
        }
    }

    fn run_once(&mut self, cx: &mut Context) -> ParkStatus {
        // check if anything in read_to_poll can be finished instead of the main future
        let mut park_status = ParkStatus::ShouldPark;
        while let Some(mut t) = self.ready_to_poll_queue.pop_front() {
            match t.future.as_mut().poll(cx) {
                std::task::Poll::Ready(_) => {
                    park_status = ParkStatus::Continue;
                },
                std::task::Poll::Pending => {
                    self.sleepy_queue.push_back(t);
                },
            }
        }

        park_status
    }

    pub fn block_on<F: Future>(&mut self, future: F) -> F::Output {
        let waker = futures::task::waker(CurrentThreadWaker::new());
        let mut cx = std::task::Context::from_waker(&waker);

        let mut pin_fut = Box::pin(future);
        let mut n = 1;
        loop {
            println!("Call {n}");
            n += 1;

            if let Poll::Ready(val) = pin_fut.as_mut().poll(&mut cx) {
                return val;
            }

            if self.run_once(&mut cx) == ParkStatus::ShouldPark {
                thread::park();
            }
        }
    }

    pub fn danger_queue_drain(&mut self) {
        let waker = futures::task::waker(CurrentThreadWaker::new());
        let mut cx = std::task::Context::from_waker(&waker);

        while !self.ready_to_poll_queue.is_empty() && !self.sleepy_queue.is_empty() {
            if self.run_once(&mut cx) == ParkStatus::ShouldPark {
                thread::park();
            }
        }
    }
}

struct CurrentThreadWaker {
    thread: Thread,
}

impl CurrentThreadWaker {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            thread: thread::current(),
        })
    }
}

impl ArcWake for CurrentThreadWaker {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // 1. Move the associated future from sleepy queue into ready_to_poll_queue
        // 2. unpark a thread worker if there are any parked
        arc_self.thread.unpark();
    }
}

#[derive(Eq, PartialEq)]
enum ParkStatus {
    ShouldPark,
    Continue,
}
