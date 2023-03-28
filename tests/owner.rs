use proximity_service::{ApiPayload, CreateOwner};

mod utils;

#[tokio::test]
async fn test_post_owner() {
    let address = utils::spawn_app().await;

    let new_owner = ApiPayload {
        payload: CreateOwner {
            name: String::from("Me"),
            email: String::from("solidsnake@sonsofliberty.om"),
            password: String::from("lalilulelo"),
        },
    };

    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/owner", &address))
        .json(&new_owner)
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
}
