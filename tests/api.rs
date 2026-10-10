use gamejolt_rs::{GameJoltClient, HashAlgorithm, Signer};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn authentication_builds_signed_request() {
    let server = MockServer::start().await;
    let unsigned_url = format!(
        "{}/api/game/v1_2/users/auth/?username=player&user_token=token&game_id=123&format=json",
        server.uri()
    );
    let signature = Signer::new("private-key", HashAlgorithm::Sha1)
        .sign_url(&unsigned_url)
        .unwrap();
    Mock::given(method("GET"))
        .and(path("/api/game/v1_2/users/auth/"))
        .and(query_param("game_id", "123"))
        .and(query_param("username", "player"))
        .and(query_param("user_token", "token"))
        .and(query_param("format", "json"))
        .and(query_param("signature", &signature))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "response": {
                "success": "true",
                "message": "User authenticated"
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = GameJoltClient::builder()
        .game_id("123")
        .private_key("private-key")
        .base_url(format!("{}/api/game/v1_2", server.uri()))
        .build()
        .unwrap();

    client
        .users()
        .authenticate("player", "token")
        .await
        .unwrap();
}

#[tokio::test]
async fn failed_api_response_is_an_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/game/v1_2/users/auth/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "response": {
                "success": "false",
                "message": "Invalid user token",
                "code": 11
            }
        })))
        .mount(&server)
        .await;

    let client = GameJoltClient::builder()
        .game_id("123")
        .private_key("private-key")
        .base_url(format!("{}/api/game/v1_2", server.uri()))
        .build()
        .unwrap();

    let error = client
        .users()
        .authenticate("player", "bad-token")
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        gamejolt_rs::GameJoltError::ApiError { code: 11, .. }
    ));
}

#[tokio::test]
async fn malformed_success_field_is_rejected() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/game/v1_2/users/auth/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "response": {"message": "missing status"}
        })))
        .mount(&server)
        .await;

    let client = GameJoltClient::builder()
        .game_id("123")
        .private_key("private-key")
        .base_url(format!("{}/api/game/v1_2", server.uri()))
        .build()
        .unwrap();

    let error = client
        .users()
        .authenticate("player", "token")
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        gamejolt_rs::GameJoltError::SerializationError(_)
    ));
}

#[tokio::test]
async fn debug_output_redacts_private_key() {
    let key = "secret-that-must-not-leak";
    let builder = GameJoltClient::builder().game_id("123").private_key(key);
    assert!(!format!("{builder:?}").contains(key));
    let client = builder.build().unwrap();
    assert!(!format!("{client:?}").contains(key));
}

#[cfg(all(feature = "macros", feature = "datastore"))]
#[derive(serde::Serialize, serde::Deserialize, gamejolt_rs::GameJoltDatastore)]
struct DerivedSave {
    stage: u32,
}

#[cfg(all(feature = "macros", feature = "datastore"))]
#[test]
fn datastore_derive_expands_in_downstream_code() {
    assert_eq!(
        <DerivedSave as gamejolt_rs::datastore::GameJoltDatastore>::datastore_type_name(),
        "DerivedSave"
    );
    let _ = DerivedSave { stage: 1 }.stage;
}

#[cfg(all(feature = "macros", feature = "scores"))]
#[derive(gamejolt_rs::GameJoltScore)]
struct DerivedScore;

#[cfg(all(feature = "macros", feature = "scores"))]
#[test]
fn score_derive_expands_in_downstream_code() {
    assert_eq!(
        <DerivedScore as gamejolt_rs::scores::GameJoltScore>::score_type_name(),
        "DerivedScore"
    );
}
