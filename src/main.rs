#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;

mod auth;
mod models;
mod repositories;
mod schema;

use models::*;
use repositories::*;
use auth::BasicAuth;
use rocket::http::Status;
use rocket::fairing::AdHoc;
use rocket::response::status;
use rocket_contrib::json::Json;
use rocket_contrib::json::JsonValue;

embed_migrations!();

#[database("sqlite_path")]
struct DbConn(diesel::SqliteConnection);

#[get("/rustaceans")]
async fn get_rustaceans(_auth: BasicAuth, conn: DbConn) -> Result<JsonValue, status::Custom<JsonValue>> {
    conn.run(|c| {
        RustaceanRepository::load_all(c)
            .map(|rustaceans| json!(rustaceans))
            .map_err(|e| status::Custom(Status::InternalServerError, json!(e.to_string())))
    }).await
}
#[get("/rustaceans/<id>")]
async fn view_rustacean(id: i32, _auth: BasicAuth, conn: DbConn) -> Result<JsonValue, status::Custom<JsonValue>> {
    conn.run(move |c| {
        RustaceanRepository::find(c, id)
            .map(|rustaceans| json!(rustaceans))
            .map_err(|e| status::Custom(Status::InternalServerError, json!(e.to_string())))
    }).await
}
#[post("/rustaceans", format = "json", data="<new_rustacean>")]
async fn create_rustacean(_auth: BasicAuth, conn: DbConn, new_rustacean: Json<NewRustacean>) -> Result<JsonValue, status::Custom<JsonValue>> {
    conn.run(|c| {
        RustaceanRepository::create(c, new_rustacean.into_inner())
            .map(|rustaceans| json!(rustaceans))
            .map_err(|e| status::Custom(Status::InternalServerError, json!(e.to_string())))
    }).await
}
#[put("/rustaceans/<_id>", format = "json", data="<rustacean>")]
async fn update_rustacean(_id: i32, _auth: BasicAuth, conn: DbConn, rustacean: Json<Rustacean>) -> Result<JsonValue, status::Custom<JsonValue>> {
    conn.run(move |c| {
        RustaceanRepository::save(c, rustacean.into_inner())
            .map(|rustaceans| json!(rustaceans))
            .map_err(|e| status::Custom(Status::InternalServerError, json!(e.to_string())))
    }).await
}
#[delete("/rustaceans/<id>")]
async fn delete_rustacean(id: i32, _auth: BasicAuth, conn: DbConn) -> Result<status::NoContent, status::Custom<JsonValue>> {
    conn.run(move |c| {
        RustaceanRepository::delete(c, id)
            .map(|_| status::NoContent)
            .map_err(|e| status::Custom(Status::InternalServerError, json!(e.to_string())))
    }).await
}

#[catch(404)]
fn not_found() -> JsonValue {
    json!("Not found!")
}

async fn run_db_migrations(rocket: rocket::Rocket) -> Result<rocket::Rocket, rocket::Rocket> {
    DbConn::get_one(&rocket).await
        .expect("failed to retrieve database connection")
        .run(|c| match embedded_migrations::run(c) {
            Ok(()) => Ok(rocket),
            Err(e) => {
                println!("Failed to run database migrations: {:?}", e);
                Err(rocket)
            }
        }).await
}

#[rocket::main]
async fn main() {
    let _ = rocket::ignite()
        .mount("/", routes![
            get_rustaceans,
            create_rustacean,
            view_rustacean,
            update_rustacean,
            delete_rustacean
        ])
        .register(catchers![
            not_found
        ])
        .attach(DbConn::fairing())
        .attach(AdHoc::on_attach("Database Migrations", run_db_migrations))
        .launch()
        .await;
}

