use std::{future::Future, pin::Pin};

pub struct WaitHandle<T> {
    pub result: Option<T>,
}

impl<T> Future for WaitHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        todo!()
        // std::task::Poll::Ready(self.result)
    }
}
