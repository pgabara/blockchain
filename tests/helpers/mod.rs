pub mod client;
pub mod server;

pub async fn repeat_until<T, R, F, P>(future: F, predicate: P, max_times: u8) -> T
where
    F: Fn() -> R + Sized,
    R: Future<Output = T>,
    P: Fn(&T) -> bool,
{
    let mut max_times = max_times;
    loop {
        let result = future().await;

        if predicate(&result) || max_times == 0 {
            return result;
        }

        max_times -= 1;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
