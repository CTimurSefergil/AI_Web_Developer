use actix_cors::Cors;

use actix_web::dev::WebService;
use actix_web::test::ok_service;
use actix_web::{http::header, web, App, HttpServer, Responder, HttpResponse};

use serde::{Deserialize, Serialize};

use reqwest::Client as HttpClient;

use async_trait::async_trait;

use std::sync::Mutex;
use std::collections::HashMap;
use std::{fs, io};
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Task {
    id: u64,
    name: String,
    complete: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    id: u64,
    username: String,
    password: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Database {
    // hashmap's are easy to convert to json files
    tasks: HashMap<u64, Task>, 
    users: HashMap<u64, User>
}
impl Database {

    fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            users: HashMap::new()
        }
    }

    // CRUD DATA

    fn insert_task(&mut self, task: Task) {
        self.tasks.insert(task.id, task);
    }

    fn remove_task(&mut self, id: u64) {
        self.tasks.remove(&id);
    }

    fn get_task(&self, id: u64) -> Option<&Task> {
        self.tasks.get(&id)
    }

    fn get_all_tasks(&self) -> Vec<&Task> {
        self.tasks.values().collect()
    }

    fn update_task(&mut self, task: Task) {
        self.tasks.insert(task.id, task);
    }

    // USER DATA RELATED FUNCTIONS

    fn insert_user(&mut self, user: User) {
        self.users.insert(user.id, user);
    }

    fn get_user_by_name(&self, username: &str) -> Option<&User> {
        self.users.values().find(|u| u.username == username)
    }

    // DATABASE SAVING

    fn save_to_file(&self) -> std::io::Result<()> {
        let data = serde_json::to_string(&self)?;
        let mut file = fs::File::create("database.json")?;
        file.write_all(data.as_bytes())?;
        Ok(())
    }

    fn load_from_file() -> std::io::Result<Self> {
        let file = fs::read_to_string("database.json")?;
        let db: Database = serde_json::from_str(&file)?;
        Ok(db)
    }

}

struct AppState {
    db: Mutex<Database>
}

async fn create_task(app_state: web::Data<AppState>, task: web::Json<Task>) -> impl Responder {
    let mut db = app_state.db.lock().unwrap();
    // into inner converts json type to needed type
    db.insert_task(task.into_inner()); // İNCELE
    let _ = db.save_to_file(); // İNCELE
    HttpResponse::Ok().finish()
}

async fn update_tasks(app_state: web::Data<AppState>, task: web::Json<Task>) -> impl Responder {
    let mut db = app_state.db.lock().unwrap();
    // into inner converts json type to needed type
    db.update_task(task.into_inner()); // İNCELE
    let _ = db.save_to_file(); // İNCELE
    HttpResponse::Ok().finish()
}

async fn get_task(app_state: web::Data<AppState>, id: web::Path<u64>) -> impl Responder {
    let db = app_state.db.lock().unwrap();
    match db.get_task(id.into_inner()) {
        Some(task) => HttpResponse::Ok().json(task),
        None => HttpResponse::NotFound().finish()
    }
}

async fn get_all_tasks(app_state: web::Data<AppState>) -> impl Responder {
    let db = app_state.db.lock().unwrap();
    let tasks = db.get_all_tasks();
    HttpResponse::Ok().json(tasks)
}

async fn delete_task(app_state: web::Data<AppState>, id: web::Path<u64>) -> impl Responder {
    let mut db = app_state.db.lock().unwrap();
    db.remove_task(id.into_inner());
    let _ = db.save_to_file();
    HttpResponse::Ok().finish()
}

async fn register(app_state: web::Data<AppState>, user: web::Json<User>) -> impl Responder {
    let mut db = app_state.db.lock().unwrap();
    // into inner converts json type to needed type
    db.insert_user(user.into_inner()); // İNCELE
    let _ = db.save_to_file(); // İNCELE
    HttpResponse::Ok().finish()
}

async fn login(app_state: web::Data<AppState>, user: web::Json<User>) -> impl Responder {
    let db = app_state.db.lock().unwrap();
    match db.get_user_by_name(&user.username) {
        Some(stored_user) if stored_user.password == user.password => HttpResponse::Ok().body("Logged in!"),
        _ => HttpResponse::BadRequest().body("Invalid password or username")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db: Database = match Database::load_from_file() {
        Ok(db) => db,
        Err(_) => Database::new()
    };

    let data = web::Data::new(AppState {
        db: Mutex::new(db)
    });

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::permissive()
                    .allowed_origin_fn(|origin, _req_head| {
                        origin.as_bytes().starts_with(b"http://localhost") || origin == "null"
                    })
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600),
            )
            .app_data(data.clone())
            // ROUTE IS WHAT IS GOING TO RUN
            .route("/task", web::post().to(create_task))
            .route("/task", web::get().to(get_all_tasks))
            .route("/task", web::put().to(update_tasks))
            .route("/task/{id}", web::delete().to(delete_task))
            .route("/task/{id}", web::get().to(get_task))
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))

            
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

