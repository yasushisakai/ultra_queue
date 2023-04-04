use crate::models::{
    hash, AliasRequest, ServiceState, Task, TaskRequest, TaskResponse, VoteRequest,
};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use std::{collections::HashMap, sync::Arc};
use tower_http::cors::CorsLayer;

async fn list_tasks(State(state): State<Arc<ServiceState>>) -> Json<Vec<(String, Task)>> {
    let data = state.db.lock().unwrap();

    let mut list: Vec<(String, Task)> = data
        .iter()
        .map(|(id, task)| (id.clone(), task.clone()))
        .collect();

    // sort by status
    list.sort_by(|a, b| b.1.added.cmp(&a.1.added));
    Json(list)
}

async fn submit_task(
    State(state): State<Arc<ServiceState>>,
    Json(request): Json<TaskRequest>,
) -> Json<TaskResponse> {
    let input = request.input.clone();
    let id = hash(&input);

    let task = Task::new(&input);

    {
        let mut db = state.db.lock().unwrap();
        match db.get(&id) {
            Some(task) => {
                let response = TaskResponse::from_id_and_task(&id, &task);
                return Json(response);
            }
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

async fn dump(State(state): State<Arc<ServiceState>>) -> Json<HashMap<String, Task>> {
    let db = state.db.lock().unwrap();
    Json(db.clone())
}

async fn apply(State(state): State<Arc<ServiceState>>, Json(newdb): Json<HashMap<String, Task>>) {
    let mut db = state.db.lock().unwrap();
    *db = newdb;
}

async fn check_task(
    State(state): State<Arc<ServiceState>>,
    Path(id): Path<String>,
) -> Json<Vec<TaskResponse>> {
    let db = state.db.lock().unwrap();
    let tasks = db
        .iter()
        .filter(|(tid, _)| tid.starts_with(&id))
        .map(|(k, v)| TaskResponse::from_id_and_task(k, v))
        .collect();
    Json(tasks)
}

async fn check_votes(
    State(state): State<Arc<ServiceState>>,
    Path(id): Path<String>,
) -> Json<Vec<String>> {
    let votes = state.votes.lock().unwrap();

    match votes.get(&id) {
        Some(vts) => Json(vts.iter().cloned().collect()),
        None => Json(Vec::new()),
    }
}

async fn vote(
    State(state): State<Arc<ServiceState>>,
    Json(request): Json<VoteRequest>,
) -> Json<TaskResponse> {
    let mut votes = state.votes.lock().unwrap();
    let city_votes = votes.entry(request.city_id.clone()).or_default();
    let mut tasks = state.db.lock().unwrap();
    let task = tasks.get_mut(&request.city_id).unwrap();

    if request.is_increment {
        if !city_votes.iter().any(|id| id == &request.user_id) {
            city_votes.insert(request.user_id);
            task.increment();
        }
        let response = TaskResponse::from_id_and_task(&request.city_id, task);
        Json(response)
    } else {
        if city_votes.iter().any(|id| id == &request.user_id) {
            city_votes.remove(&request.user_id);
            task.decrement();
        }
        let response = TaskResponse::from_id_and_task(&request.city_id, task);
        Json(response)
    }
}

async fn alias(
    State(state): State<Arc<ServiceState>>,
    Json(request): Json<AliasRequest>,
) -> Json<bool> {
    let mut aliases = state.alias.lock().unwrap();
    aliases.insert(request.user_id, request.alias);
    Json(true)
}

async fn get_alias(State(state): State<Arc<ServiceState>>) -> Json<HashMap<String, String>> {
    let alias = state.alias.lock().unwrap();
    Json(alias.clone())
}

pub fn routes(state: Arc<ServiceState>) -> Router {
    let routes = Router::new()
        .route("/list", get(list_tasks))
        .route("/submit", post(submit_task))
        .route("/check/:id", get(check_task))
        .route("/votes/:id", get(check_votes))
        .route("/vote", post(vote))
        .route("/dump", get(dump))
        .route("/apply", post(apply))
        .route("/alias", get(get_alias).post(alias))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let app = Router::new().nest("/api", routes);
    app
}
