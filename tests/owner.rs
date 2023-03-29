use proximity_service::{ApiPayload, CreateOwner};
use std::collections::HashMap;

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
        .unwrap();

    assert!(response.status().is_success());

    let json_response = response.json::<HashMap<String, i32>>().await.unwrap();

    assert!(json_response.contains_key("id"));
}
