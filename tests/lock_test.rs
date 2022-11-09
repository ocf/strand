use kube::Client;
use strand::lock::Lock;

fn get_lock() -> Lock {
    Lock {
        name: "test".into(),
        namespace: "default".into(),
    }
}

// Rust tries to run tests in parallel, but this doesn't work for
// the lock test because... obviously... so I put them all in one
// function!

#[tokio::test]
async fn lock_test() {
    let lock = get_lock();
    let client: Client = Client::try_default().await.unwrap();

    let _ = lock.force_release(client.clone()).await;

    lock.acquire(client.clone(), "alice").await.unwrap();
    lock.release(client.clone(), "alice").await.unwrap();

    // Someone tries to acquire an already held lock...

    lock.acquire(client.clone(), "alice").await.unwrap();
    let res = lock.acquire(client.clone(), "bob").await;
    assert!(res.is_err());
    lock.release(client.clone(), "alice").await.unwrap();

    // Freeing when doesn't exist...

    lock.acquire(client.clone(), "alice").await.unwrap();
    lock.release(client.clone(), "alice").await.unwrap();
    let res = lock.release(client.clone(), "alice").await;
    assert!(res.is_err());
    
    // Freeing when someone else holds the lock...

    lock.acquire(client.clone(), "alice").await.unwrap();
    let res = lock.release(client.clone(), "bob").await;
    assert!(res.is_err());
}
