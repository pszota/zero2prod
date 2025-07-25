//! tests/health_check.rs
use sqlx::{PgPool};
use std::{ net::TcpListener};
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;


pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}


async fn spawn_app() -> TestApp{

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bid random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let configuration = get_configuration().expect("Failed to load configuration");
    let connection_pool = PgPool::connect(
            &configuration.database.connection_string()
          )
          .await.expect("Failed to connect to POstgres.");

    let server = run(listener,connection_pool.clone())
            .expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_pool: connection_pool
    }
}


#[tokio::test]
async fn health_check_works() {
    let address = spawn_app().await.address;

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}



#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_date(){


let app_address = spawn_app().await.address;

let app = spawn_app().await;
        
let clinet = reqwest::Client::new();

let body =  "name=le%20guin&email=ursula_le_guin%40gmail.com";
let response = clinet
            .post(&format!("{}/subscriptions",&app_address))
            .header("Content-Type","application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");
assert_eq!(200,response.status().as_u16());

let saved = sqlx::query!("Select email, name from subscriptions",)
    .fetch_one(&app.db_pool)
    .await.expect("Failed to fetch saved subscribtion");

assert_eq!(saved.email,"ursula_le_guin@gmail.com");
assert_eq!(saved.name,"le guin");
}
#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {

let app_address = spawn_app().await.address;
let clinet = reqwest::Client::new();

let test_cases = vec![
                                                        ("name=le%20guin", "missing the email"),
                                                        ("email=ursula_le_guin%40gmail.com", "missing the name"),
                                                        ("", "missing both name and email")
                                                        ];


    for (invalid_body, error_message) in test_cases {

        let response = clinet
                .post(&format!("{}/subscriptions",&app_address))
                .header("Content-Type","application/x-www-form-urlencoded")
                .body(invalid_body)
                .send()
                .await
                .expect("failed to execute request");

            assert_eq!(
                400,
                response.status().as_u16(),
                "The API did not fail with 400 Bad Request when the payload was {}.",
                    error_message

            );


    }                                                       

}
