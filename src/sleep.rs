use std::{future::Future, pin::Pin, sync::Mutex, thread, time::Instant};

pub struct Sleep {
    duration: std::time::Duration,
    start: std::time::Instant,
    thread: Mutex<Option<thread::JoinHandle<()>>>,
}

impl Sleep {
    pub fn new(duration: std::time::Duration) -> Self {
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

