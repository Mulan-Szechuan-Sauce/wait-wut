use std::{any::Any, collections::VecDeque, future::Future, pin::Pin, sync::Mutex, task::{RawWaker, RawWakerVTable, Waker}, time::Instant};

use futures::FutureExt;

struct Task {
    future: Pin<Box<dyn Future<Output = Box<dyn Any>> + Send>>,
    output_type: std::any::TypeId,
}
// type Task = Pin<Box<dyn Future<Output = ()> + Send>>;

#[derive(Default)]
struct WaitWutRuntime {
    ready_to_poll_queue: VecDeque<Task>,
    sleepy_queue: VecDeque<Task>,
}

impl WaitWutRuntime {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn spawn<F, T>(&mut self, future: F) -> WaitHandle<T>
    where
        T: 'static,
        F: Future<Output = T> + Send + 'static,
    {
        let task = Task {
            future: Box::pin(future.map(|x| Box::new(x) as Box<dyn Any>)),
            output_type: std::any::TypeId::of::<T>(),
        };
        self.ready_to_poll_queue.push_back(task);
        WaitHandle { result: todo!() }
    }

    fn block_on<F: Future>(&mut self, future: F) -> F::Output {
        let waker = futures::task::noop_waker();
        let mut cx = std::task::Context::from_waker(&waker);

        let mut pin_fut = Box::pin(future);
        let mut n = 1;
        loop {
            println!("Call {n}");
            n += 1;
            match pin_fut.as_mut().poll(&mut cx) {
                std::task::Poll::Ready(val) => return val,
                std::task::Poll::Pending => {
                    // check if anything in read_to_poll can be finished instead of the main future
                    // for t in self.ready_to_poll_queue.iter_mut() {
                    //     match t.as_mut().poll(&mut cx) {
                    //         std::task::Poll::Ready(_) => todo!(),
                    //         std::task::Poll::Pending => todo!(),
                    //     }
                    // }
                }
            }
        }
    }
}

struct WaitHandle<T> {
    result: T,
}

impl<T> Future for WaitHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        todo!()
        // std::task::Poll::Ready(self.result)
    }
}

struct Sleep {
    duration: std::time::Duration,
    start: std::time::Instant,
    thread: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl Sleep {
    fn new(duration: std::time::Duration) -> Self {
        Self {
            duration,
            start: std::time::Instant::now(),
            thread: Mutex::new(None),
        }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        if Instant::now() >= self.start + self.duration {
            std::task::Poll::Ready(())
        } else {
            let mut guard = self.thread.lock().unwrap();
            if guard.is_none() {
                let waker = cx.waker().clone();
                let delta = self.start + self.duration - Instant::now();
                *guard = Some(std::thread::spawn(move || {
                    std::thread::sleep(delta);
                    waker.wake();
                }));
            }
            std::task::Poll::Pending
        }
    }
}

async fn pee() -> u32 {
    6
}

async fn poo() -> u32 {
    let x = pee().await;
    x + 2
}

async fn deeply_nested() -> u32 {
    let (x, y, ()) = futures::join!(pee(), poo(), Sleep::new(std::time::Duration::from_millis(10)));
    x + y
}

fn some_runtime() {
    let mut rt = WaitWutRuntime::new();
    let f1 = rt.spawn(async { 55 });
    let f2 = rt.spawn(async { 77 });
    rt.block_on(async { f1.await + f2.await });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deep() {
        let mut rt = WaitWutRuntime::new();
        let async_result = rt.block_on(deeply_nested());

        assert_eq!(14, async_result);
    }
}
