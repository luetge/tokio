#![cfg(feature = "full")]

use tokio::{
    task::JoinSet,
    time::{Duration, Instant},
};

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

#[tokio::test(start_paused = true)]
async fn test_fast_forward_nanoseconds_works() {
    for _ in 0..1000 {
        let mut set = JoinSet::new();

        let (txstart, rxstart) = tokio::sync::broadcast::channel(1);
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);

        let mut rxstart0 = rxstart.resubscribe();
        let tx0 = tx.clone();
        println!("prepare");
        set.spawn(async move {
            // Make sure this one registers second
            rxstart0.recv().await.unwrap();
            sleep(Duration::from_nanos(2)).await;
            println!("woke up 0");
            tx0.send(0).await.unwrap();
        });

        let mut rxstart1 = rxstart.resubscribe();
        let tx1 = tx.clone();
        set.spawn(async move {
            rxstart1.recv().await.unwrap();
            sleep(Duration::from_nanos(1)).await;
            println!("woke up 1");
            tx1.send(1).await.unwrap();
        });

        println!("trigger");
        txstart.send(()).unwrap();
        println!("waiting");
        set.join_next().await.unwrap().unwrap();
        println!("readout");

        assert_eq!(rx.recv().await.unwrap(), 1);
        assert_eq!(rx.recv().await.unwrap(), 0);
    }
}
