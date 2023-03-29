use std::sync::Arc;
use axum::{Router, routing::{get, post}, extract::{State, Path}, Json};
use crate::models::{Task, ServiceState, TaskRequest, Status, hash, TaskResponse};

async fn list_tasks(
    State(state): State<Arc<ServiceState>>
    ) -> Json<Vec<(String, Status)>> {
    let data = state.db.lock().unwrap();

    let mut list: Vec<(String, Status)> = data
        .iter()
        .map(|(id, task)|(id.clone(), task.status.clone())).collect();

    // sort by status 
    list.sort_by(|a,b|a.1.cmp(&b.1));
    Json(list)
}

async fn submit_task(
    State(state): State<Arc<ServiceState>>,
    Json(request): Json<TaskRequest>
    ) -> Json<TaskResponse> {

    let input = request.input.clone();
    let id = hash(&input);

    let task = Task {
        input: input.clone(),
        status: Status::Queued
    };

    {
        let mut db = state.db.lock().unwrap();
        match db.get(&id) {
            Some(task) => {
                let response = TaskResponse::from_id_and_task(&id, &task);
                return Json(response)
            },
            None => {
                db.insert(id.clone(), task.clone());
            }
        }
    }

   // add to the que 
    {
        let mut que = state.que.lock().unwrap();
        que.push_back((id.clone(), input));
    }
    let response = TaskResponse::from_id_and_task(&id, &task);
    Json(response)
}

async fn check_task(
    State(state): State<Arc<ServiceState>>,
    Path(id): Path<String>
    ) -> Json<Vec<TaskResponse>> {
    let db = state.db.lock().unwrap();
    let tasks = db.iter()
        .filter(|(tid,_)|tid.starts_with(&id))
        .map(|(k,v)|TaskResponse::from_id_and_task(k, v))
        .collect();
    Json(tasks)
}

pub fn routes(state: Arc<ServiceState>) -> Router {

    let routes = Router::new()
        .route("/list", get(list_tasks))
        .route("/submit", post(submit_task))
        .route("/check/:id", get(check_task))
        .with_state(state);

    let app = Router::new().nest("/api", routes);

    app
}
