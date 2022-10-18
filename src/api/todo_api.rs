use actix_web::{
    post, get, put, delete,
    web::{Data, Json, Path},
    HttpResponse,
};

use crate::repository::{surrealdb_repo::SurrealDBRepo};
use crate::model::todo_model::{Todo, TodoBMC, TodoPatch};

#[post("/todos")]
pub async fn create_todo(db: Data<SurrealDBRepo>, new_todo: Json<Todo>) -> HttpResponse {
    let data = Todo {
        id: None,
        title: new_todo.title.to_owned(),
        body: new_todo.body.to_owned(),
    };
    
    let todo_detail = TodoBMC::create(db, "todo", data ).await;

    match todo_detail {
         Ok(todo) => HttpResponse::Ok().json(todo),
         Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[get("/todos/{id}")]
pub async fn get_todo(db: Data<SurrealDBRepo>, path: Path<String>) -> HttpResponse {
    let id = path.into_inner();
    
    if id.is_empty() {
        return HttpResponse::BadRequest().body("invalid ID");
    }
    
    let todo_detail = TodoBMC::get(db, &id).await;
    
    match todo_detail {
        Ok(todo) => HttpResponse::Ok().json(todo),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[put("/todos/{id}")]
pub async fn update_todo(
    db: Data<SurrealDBRepo>,
    path: Path<String>,
    todo_patch: Json<TodoPatch>,
) -> HttpResponse {
    let id = path.into_inner();
    
    if id.is_empty() {
        return HttpResponse::BadRequest().body("invalid ID");
    };

    let data = TodoPatch {
        title: todo_patch.title.to_owned(),
        body: todo_patch.body.to_owned(),
    };
    
    let update_result = TodoBMC::update(db, &id, data).await;
    
    match update_result {
        Ok(todo) => HttpResponse::Ok().json(todo),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
#[delete("/todos/{id}")]
pub async fn delete_todo(db: Data<SurrealDBRepo>, path: Path<String>) -> HttpResponse {
    let id = path.into_inner();
    
    if id.is_empty() {
        return HttpResponse::BadRequest().body("invalid ID");
    };
    
    let result = TodoBMC::delete(db, &id).await;
    
    match result {
        Ok(todo) => HttpResponse::Ok().json(todo),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[get("/todos")]
pub async fn get_todos(db: Data<SurrealDBRepo>) -> HttpResponse {
    let result = TodoBMC::get_all(db).await;
    
    match result {
        Ok(todos) => HttpResponse::Ok().json(todos),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
   }
}