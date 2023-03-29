mod endpoints;
mod models;

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use tokio::join;
use tokio::time::sleep as asleep;

use axum::Server;
use endpoints::routes;
use models::{Database, Que, ServiceState, Status};

// syncronous 'heavy' process
fn process(id: String) {
    sleep(Duration::from_secs(5));
    println!("  <process> ended: {}", id);
}

#[tokio::main]
async fn main() {
    let que: Que = Mutex::new(VecDeque::new());
    let db: Database = Mutex::new(HashMap::new());
    let state: Arc<ServiceState> = Arc::new(ServiceState { que, db });

    let state_clone = Arc::clone(&state);
    // processes tasks
    let compute = tokio::task::spawn(async move {
        // wait for the other 'spawn' created tasks midway
        asleep(Duration::from_secs(2)).await;

        loop {
            let new_task = {
                let mut que = state_clone.que.lock().unwrap();
                que.pop_front()
            };

            match new_task {
                Some((id, _input)) => {
                    {
                        let mut db = state_clone.db.lock().unwrap();
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
                        let mut db = state_clone.db.lock().unwrap();
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

    // listens for tasks
    let server =
        Server::bind(&"0.0.0.0:3000".parse().unwrap()).serve(routes(state).into_make_service());

    let (_, _) = join!(server, compute);
}
