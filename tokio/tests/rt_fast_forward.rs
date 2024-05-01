#![cfg(feature = "full")]

use tokio::{time::{Duration, Instant}, task::JoinSet};

async fn sleep(interval: Duration) {
    println!("scheduled sleep: {:?}", interval);
    tokio::time::sleep(interval).await;
}

#[tokio::test(start_paused = true)]
async fn test_fast_forward_works() {
    let base_instant = Instant::now();
    let mut set = JoinSet::new();
    set.spawn(async move {
            // Make sure this one registers second
            sleep(Duration::from_secs(10)).await;
            // Then wait very long
            sleep(Duration::from_secs(70_000_000)).await;
            println!("[{:?}] this breaks it", base_instant.elapsed());
        });

    set.spawn(async move {
            // Spawn a task that runs every now and then
            let mut prev = Instant::now();
            let interval = Duration::from_secs(10_000_000);
            loop {
                sleep(interval).await;
                println!("[{:?}] tock!", base_instant.elapsed());
                let now = Instant::now();
                assert!(now - prev < interval * 2);
                prev = now;
            }
        });

    set.join_next().await.unwrap().unwrap()
}
