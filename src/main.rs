use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use std::thread::sleep;
use tokio::time::sleep as asleep;

#[derive(Debug)]
struct Task {
    input: String,
    status: Status,
}

#[derive(Debug)]
enum Status {
    Qued,
    Processing,
    Done,
}

type Que = Arc<Mutex<VecDeque<(String, String)>>>;
type Database = Arc<Mutex<HashMap<String, Task>>>;

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

    let que_clone = Arc::clone(&que);
    let db_clone = Arc::clone(&db);
    // creates tasks
    tokio::task::spawn(async move {
        for i in 0..3 {
            {
                let mut que = que_clone.lock().unwrap();
                let mut db = db_clone.lock().unwrap();
                let id = format!("{}", i);
                let task = Task {
                    input: id.clone(),
                    status: Status::Qued,
                };
                db.insert(id.clone(), task);
                println!("added task: {}", id);

                que.push_back((id.clone(), id));
            }
            asleep(Duration::from_secs(1)).await;
        }

        asleep(Duration::from_secs(8)).await;

        println!("--report #1--");
        {
            let db = db_clone.lock().unwrap();
            for t in db.iter() {
                println!("{:?}", t);
            }
        }

        for i in 3..6 {
            {
                let mut que = que_clone.lock().unwrap();
                let mut db = db_clone.lock().unwrap();
                let id = format!("{}", i);
                let task = Task {
                    input: id.clone(),
                    status: Status::Qued,
                };
                db.insert(id.clone(), task);
                println!("added task: {}", id);

                que.push_back((id.clone(), id));
            }
            asleep(Duration::from_secs(1)).await;
        }

        asleep(Duration::from_secs(8)).await;
        println!("--report #2--");
        {
            let db = db_clone.lock().unwrap();
            for t in db.iter() {
                println!("{:?}", t);
            }
        }

        for i in 6..9 {
            {
                let mut que = que_clone.lock().unwrap();
                let mut db = db_clone.lock().unwrap();
                let id = format!("{}", i);
                let task = Task {
                    input: id.clone(),
                    status: Status::Qued,
                };
                db.insert(id.clone(), task);
                println!("added task: {}", id);

                que.push_back((id.clone(), id));
            }
            asleep(Duration::from_secs(1)).await;
        }
    })
    .await
    .unwrap();

    compute.await.unwrap();
}
