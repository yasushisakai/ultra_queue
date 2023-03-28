mod endpoints;
mod models;

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use std::thread::sleep;
use axum::routing::get;
use axum::{Server, Router};
use models::{Que, Database, Status};
use tokio::join;
use tokio::time::sleep as asleep;

use crate::models::Task;

// syncronous 'heavy' process
fn process(id: String) {
    println!("  <process> started: {}", id);
    sleep(Duration::from_secs(5));
    println!("  <process> ended: {}", id);
}

#[tokio::main]
async fn main() {
    let que: Que = Arc::new(Mutex::new(VecDeque::new()));
    let db: Database = Arc::new(Mutex::new(HashMap::new()));

    let que_clone = Arc::clone(&que);
    let db_clone = Arc::clone(&db);
    // processes tasks
    let compute = tokio::task::spawn(async move {
        // wait for the other 'spawn' created tasks midway
        asleep(Duration::from_secs(2)).await;

        loop {
            let new_task = {
                let mut que = que_clone.lock().unwrap();
                que.pop_front()
            };

            match new_task {
                Some((id, _input)) => {
                    {
                        let mut db = db_clone.lock().unwrap();
                        let mut task = db.get_mut(&id).unwrap();
                        task.status = Status::Processing;
                    }

                    let id_clone = id.clone();
                    tokio::task::spawn_blocking(move || {
                        process(id_clone);
                    })
                    .await
                    .unwrap();
                    {
                        let mut db = db_clone.lock().unwrap();
                        let mut task = db.get_mut(&id).unwrap();
                        task.status = Status::Done;
                    }
                }
                None => {
                    asleep(Duration::from_secs(1)).await;
                }
            }
        }
    });

    // let que_clone = Arc::clone(&que);
    // let db_clone = Arc::clone(&db);
    
    let app = Router::new().route("/", get(|| async {
        println!("got hi");
        "hi"
    }));

    // listens for tasks
    let server = Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service());

    let (_,_) = join!(server, compute);
}
