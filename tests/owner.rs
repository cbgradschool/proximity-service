use proximity_service::{
    create_owner, ApiPayload, CreateOwner, CreateOwnerResponse, Owners, UpdateCredentials,
    UpdateProfile,
};

mod utils;

mod tests {
    use sqlx::PgPool;

    use super::*;

    struct TestSetup {
        owner_id: i32,
    }

    async fn setup(db: &PgPool) -> Result<TestSetup, sqlx::Error> {
        let owner = CreateOwner {
            name: String::from("Henry"),
            email: String::from("@gmail.com"),
            password: String::from("password"),
        };

        create_owner(owner, db).await.map(|record| TestSetup {
            owner_id: record.id,
        })
    }

    impl Drop for TestSetup {
        fn drop(&mut self) {}
    }

    #[sqlx::test]
    async fn test_get_owner(db: PgPool) {
        let (address, db) = utils::make_server(db).await;

        let test_setup = setup(&db).await.expect("Expected to get a record");

        let client = reqwest::Client::new();

        let response = client
            .get(&format!("{}/owner/{}", &address, test_setup.owner_id))
            .send()
            .await
            .unwrap();

        let actual = response.status();

        let expected = reqwest::StatusCode::OK;

        assert_eq!(actual, expected);

        let owner_response: Owners = response.json().await.unwrap();

        assert_eq!(owner_response.id, test_setup.owner_id);
    }

    #[sqlx::test]
    async fn test_post_owner(db: PgPool) {
        let (address, _) = utils::make_server(db).await;

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
    }

    #[sqlx::test]
    async fn test_delete_owner(db: PgPool) {
        let (address, db) = utils::make_server(db).await;

        let test_setup = setup(&db).await.expect("Expected to get a record");

        let client = reqwest::Client::new();

        let response = client
            .delete(&format!("{}/owner/{}", &address, &test_setup.owner_id))
            .send()
            .await
            .unwrap();

        let actual = response.status();

        let expected = reqwest::StatusCode::NO_CONTENT;

        assert_eq!(actual, expected)
    }

    #[sqlx::test]
    async fn test_update_profile_information(db: PgPool) {
        let (address, db) = utils::make_server(db).await;

        let test_setup = setup(&db).await.expect("Somethin");

        let client = reqwest::Client::new();

        let patch = ApiPayload {
            payload: UpdateProfile {
                name: String::from("Jane"),
                owner_id: test_setup.owner_id,
            },
        };

        let response = client
            .patch(format!(
                "{}/owner/{}/profile",
                &address, &test_setup.owner_id
            ))
            .json(&patch)
            .send()
            .await
            .unwrap();

        let actual = response.status();

        let expected = reqwest::StatusCode::NO_CONTENT;

        assert_eq!(actual, expected)
    }

    #[sqlx::test]
    async fn test_update_credentials(db: PgPool) {
        let (address, db) = utils::make_server(db).await;

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
}
