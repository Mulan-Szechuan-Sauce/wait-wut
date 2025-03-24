pub mod sleep;
pub mod wait_handle;
pub mod single_thread_runtime;

use single_thread_runtime::WaitWutRuntime;

use crate::sleep::Sleep;

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

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_deep() {
    //     let mut rt = WaitWutRuntime::new();
    //     let async_result = rt.block_on(deeply_nested());
    //
    //     assert_eq!(14, async_result);
    // }

    #[test]
    fn test_background() {
        let mut rt = WaitWutRuntime::new();
        // Some background task
        rt.spawn(async {
            Sleep::new(std::time::Duration::from_millis(10)).await;
            println!("Hello from the background!");
        });
        rt.block_on(async { 
            Sleep::new(std::time::Duration::from_secs(1)).await;
            println!("Hello after sleep");
        });
    }
}
