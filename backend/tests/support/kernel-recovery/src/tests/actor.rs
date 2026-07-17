use std::sync::{Arc, Barrier};
use std::time::Duration;

use rusqlite::Connection;

use crate::StoreError;
use crate::control_store_handle::{CONTROL_STORE_QUEUE_CAPACITY, ControlStoreHandle};

#[test]
fn rejects_work_when_the_bounded_queue_is_full() {
    let handle = ControlStoreHandle::spawn(Connection::open_in_memory().unwrap()).unwrap();
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let actor_entered = Arc::clone(&entered);
    let actor_release = Arc::clone(&release);
    handle
        .sender
        .try_send(Box::new(move |_| {
            actor_entered.wait();
            actor_release.wait();
        }))
        .unwrap();
    entered.wait();
    for _ in 0..CONTROL_STORE_QUEUE_CAPACITY {
        handle.sender.try_send(Box::new(|_| {})).unwrap();
    }
    assert!(matches!(
        handle.call(|_| Ok(())),
        Err(StoreError::QueueFull)
    ));
    release.wait();
}

#[test]
fn reports_the_standard_operation_deadline() {
    let handle = ControlStoreHandle::spawn(Connection::open_in_memory().unwrap()).unwrap();
    let result = handle.call(|_| {
        std::thread::sleep(Duration::from_millis(2_100));
        Ok(())
    });
    assert!(matches!(result, Err(StoreError::DeadlineExceeded)));
}

#[test]
fn reports_actor_termination() {
    let handle = ControlStoreHandle::spawn(Connection::open_in_memory().unwrap()).unwrap();
    let _: Result<(), StoreError> = handle.call(|_| panic!("test actor termination"));
    assert!(matches!(
        handle.call(|_| Ok(())),
        Err(StoreError::ActorStopped)
    ));
}

#[test]
fn serializes_concurrent_writers() {
    let handle = ControlStoreHandle::spawn(Connection::open_in_memory().unwrap()).unwrap();
    handle
        .call(|connection| {
            connection.execute("CREATE TABLE value (id INTEGER PRIMARY KEY)", [])?;
            Ok(())
        })
        .unwrap();
    let writers: Vec<_> = (0..16)
        .map(|id| {
            let writer = handle.clone();
            std::thread::spawn(move || {
                writer.call(move |connection| {
                    connection.execute("INSERT INTO value (id) VALUES (?1)", [id])?;
                    Ok(())
                })
            })
        })
        .collect();
    for writer in writers {
        writer.join().unwrap().unwrap();
    }
    let count: i64 = handle
        .call(|connection| {
            connection
                .query_row("SELECT count(*) FROM value", [], |row| row.get(0))
                .map_err(StoreError::from)
        })
        .unwrap();
    assert_eq!(count, 16);
}
