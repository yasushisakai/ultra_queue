mod endpoints;
mod models;
mod process;

use process::process;

use std::sync::Arc;
use std::time::Duration;

use tokio::join;
use tokio::time::sleep as asleep;

use axum::Server;
use endpoints::routes;
use models::{ServiceState, Status};

#[tokio::main]
async fn main() {
    let state = Arc::new(ServiceState::default());

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
                Some((id, input)) => {
                    {
                        let mut db = state_clone.db.lock().unwrap();
                        let mut task = db.get_mut(&id).unwrap();
                        task.status = Status::Processing;
                    }

                    let id_clone = id.clone();

                    let success = tokio::task::spawn_blocking(move || process(id_clone, input))
                        .await
                        .unwrap();

                    {
                        let mut db = state_clone.db.lock().unwrap();
                        let mut task = db.get_mut(&id).unwrap();
                        task.status = if success { Status::Done } else { Status::Error };
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
