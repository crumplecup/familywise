use crate::prelude::*;
use api_lib::prelude::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use shared::prelude::*;
use tower::ServiceExt;
use tracing::{info, trace};

#[tokio::test]
async fn local_user_lifecycle() {
    if let Ok(()) = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init()
    {};
    let settings = DatabaseSettings::from_env().unwrap();
    settings.delete_db().await;
    settings.create_db().await;
    settings.migrate_db().await;
    {
        let db_pool = settings.get_connection_pool();
        let app_state = AppState::new(db_pool);
        let app = app_state.app();

        // Create user
        let user = new_user();
        trace!(
            "Sending local request to create user {}.",
            &user.username_ref()
        );
        trace!("{:#?}", serde_json::json!(&user));

        let request = Request::builder()
            .method("POST")
            .header("Content-Type", "application/json")
            .uri("/api/users")
            .body(Body::from(serde_json::json!(&user).to_string()))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        trace!("{:#?}", &response);
        assert!(&response.status().is_success());
        assert_eq!(&response.status(), &StatusCode::CREATED);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: User = serde_json::from_slice(&body).unwrap();
        let (username, _) = prune_str(body.username_ref()).unwrap();
        let (password, _) = prune_str(body.password_hash_ref()).unwrap();
        assert_eq!(user.username_ref(), username);
        assert_eq!(user.password_hash_ref(), password);
        let id: uuid::Uuid = body.id();
        info!("Create user successful.");

        // Get users
        let request = Request::builder()
            .method("GET")
            .header("Content-Type", "application/json")
            .uri("/api/users")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        trace!("{:#?}", &response);
        assert!(&response.status().is_success());
        assert_eq!(&response.status(), &StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Vec<User> = serde_json::from_slice(&body).unwrap();
        trace!("Body: {:#?}", &body);
        if !body.is_empty() {
            let usr = body[0].clone();
            let (username, _) = prune_str(usr.username_ref()).unwrap();
            let (password, _) = prune_str(usr.password_hash_ref()).unwrap();
            assert_eq!(user.username_ref(), username);
            assert_eq!(user.password_hash_ref(), password);
        }
        info!("Get users successful.");

        // Get user
        info!(
            "Sending local request to get user {}.",
            &user.username_ref()
        );
        let url = format!("/api/users/{}", &id);
        trace!("Url is {}", &url);
        let request = Request::builder()
            .method("GET")
            .header("Content-Type", "application/json")
            .uri(&url)
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        trace!("{:#?}", &response);
        assert!(&response.status().is_success());
        assert_eq!(&response.status(), &StatusCode::OK);
        // info!("Response: {:#?}", response.into_body());

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: User = serde_json::from_slice(&body).unwrap();
        let (username, _) = prune_str(body.username_ref()).unwrap();
        let (password, _) = prune_str(body.password_hash_ref()).unwrap();
        assert_eq!(user.username_ref(), username);
        assert_eq!(user.password_hash_ref(), password);
        info!("Get user successful.");

        // Update user
        let mut user = new_user();
        {
            user.set_id(id.clone());
        }
        info!(
            "Sending local request to update user {}.",
            &user.username_ref()
        );
        trace!("{:#?}", serde_json::json!(&user));

        let request = Request::builder()
            .method("PUT")
            .header("Content-Type", "application/json")
            .uri(&url)
            .body(Body::from(serde_json::json!(&user).to_string()))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        trace!("{:#?}", &response);
        assert!(&response.status().is_success());
        assert_eq!(&response.status(), &StatusCode::OK);

        // Calling get by id to compare database value for id
        // to new user values.
        let request = Request::builder()
            .method("GET")
            .header("Content-Type", "application/json")
            .uri(&url)
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: User = serde_json::from_slice(&body).unwrap();
        let (username, _) = prune_str(body.username_ref()).unwrap();
        let (password, _) = prune_str(body.password_hash_ref()).unwrap();
        assert_eq!(user.username_ref(), username);
        assert_eq!(user.password_hash_ref(), password);
        info!("Update user successful.");

        // Delete user
        info!(
            "Sending local request to update user {}.",
            &user.username_ref()
        );

        let request = Request::builder()
            .method("DELETE")
            .header("Content-Type", "application/json")
            .uri(&url)
            .body(Body::from(serde_json::json!(&user).to_string()))
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        info!("{:#?}", &response);
        assert!(&response.status().is_success());
        assert_eq!(&response.status(), &StatusCode::OK);
    }

    settings.delete_db().await;
}
