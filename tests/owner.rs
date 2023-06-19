use proximity_service::{
    ApiPayload, CreateOwner, CreateOwnerResponse, Owners, UpdateCredentials, UpdateProfile,
};

mod utils;

#[tokio::test]
async fn test_get_owner() {
    let (address, db) = utils::spawn_app().await;

    let owner_id = utils::test_setup(&db).await;

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/owner/{}", &address, owner_id))
        .send()
        .await
        .unwrap();

    let actual = response.status();

    let expected = reqwest::StatusCode::OK;

    assert_eq!(actual, expected);

    let owner_response: Owners = response.json().await.unwrap();

    assert_eq!(owner_response.id, owner_id);

    utils::test_teardown(owner_id, &db).await.unwrap();
}

#[tokio::test]
async fn test_post_owner() {
    let (address, db) = utils::spawn_app().await;

    let new_owner = ApiPayload {
        payload: CreateOwner {
            name: String::from("David Hayer"),
            email: String::from("solidsnake@sonsofliberty.test"),
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

    let json_response: CreateOwnerResponse = response.json().await.unwrap();

    assert!(json_response.id > 0);

    utils::test_teardown(json_response.id, &db).await.unwrap();
}

#[tokio::test]
async fn test_delete_owner() {
    let (address, db) = utils::spawn_app().await;

    let owner_id = utils::test_setup(&db).await;

    let client = reqwest::Client::new();

    let response = client
        .delete(&format!("{}/owner/{}", &address, &owner_id))
        .send()
        .await
        .unwrap();

    let actual = response.status();

    let expected = reqwest::StatusCode::NO_CONTENT;

    assert_eq!(actual, expected)
}

#[tokio::test]
async fn test_update_profile_information() {
    let (address, db) = utils::spawn_app().await;

    let owner_id = utils::test_setup(&db).await;

    let client = reqwest::Client::new();

    let patch = ApiPayload {
        payload: UpdateProfile {
            name: String::from("Jane"),
            owner_id,
        },
    };

    let response = client
        .patch(format!("{}/owner/{}/profile", &address, &owner_id))
        .json(&patch)
        .send()
        .await
        .unwrap();

    let actual = response.status();

    let expected = reqwest::StatusCode::NO_CONTENT;

    assert_eq!(actual, expected)
}

#[tokio::test]
async fn test_update_credentials() {
    let (address, db) = utils::spawn_app().await;

    let owner_id = utils::test_setup(&db).await;

    let client = reqwest::Client::new();

    let patch = ApiPayload {
        payload: UpdateCredentials {
            email: String::from("gray_fox@thepatriots.com"),
            password: String::from("lalilulelo"),
            owner_id,
        },
    };

    let request = client
        .patch(format!("{}/owner/{}/credentials", &address, owner_id))
        .json(&patch)
        .send()
        .await
        .unwrap();

    let actual = request.status();

    let expected = reqwest::StatusCode::NO_CONTENT;

    assert_eq!(actual, expected)
}
