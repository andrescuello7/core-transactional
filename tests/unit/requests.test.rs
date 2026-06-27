use actix_web::{test, web, App};
use prex::infra::http::server::register_routes;
use prex::infra::storage::alloc_memory::{Alloc, Command};
use serde_json::{json, Value};
use tokio::sync::mpsc;

async fn build_app() -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    let (tx, rx) = mpsc::channel::<Command>(32);
    tokio::spawn(async move { Alloc::new().run(rx).await });

    test::init_service(
        App::new()
            .app_data(web::Data::new(tx))
            .configure(register_routes),
    )
    .await
}

#[actix_web::test]
async fn post_new_client_returns_201_and_client_id() {
    let app = build_app().await;

    let req = test::TestRequest::post()
        .uri("/new_client")
        .set_json(json!({
            "client_name": "Ana García",
            "birth_date": "1988-03-21",
            "document_number": "20388001",
            "country": "AR"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let body: Value = test::read_body_json(resp).await;
    assert!(body["client_id"].as_u64().is_some());
}

#[actix_web::test]
async fn post_new_client_duplicate_document_returns_400() {
    let app = build_app().await;

    let payload = json!({
        "client_name": "Pedro López",
        "birth_date": "1975-11-05",
        "document_number": "DUP-99999",
        "country": "AR"
    });

    test::call_service(&app, test::TestRequest::post().uri("/new_client").set_json(&payload).to_request()).await;

    let req = test::TestRequest::post()
        .uri("/new_client")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "DUPLICATED_DOCUMENT");
}

#[actix_web::test]
async fn get_client_balance_returns_200_with_client_data() {
    let app = build_app().await;

    let create_req = test::TestRequest::post()
        .uri("/new_client")
        .set_json(json!({
            "client_name": "Marta Ruiz",
            "birth_date": "1995-07-14",
            "document_number": "GB-10001",
            "country": "UY"
        }))
        .to_request();
    let create_resp: Value = test::read_body_json(test::call_service(&app, create_req).await).await;
    let client_id = create_resp["client_id"].as_u64().unwrap();

    let req = test::TestRequest::get()
        .uri(&format!("/client_balance/{}", client_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["client_id"], client_id);
    assert_eq!(body["document_number"], "GB-10001");
}

#[actix_web::test]
async fn get_client_balance_unknown_id_returns_404() {
    let app = build_app().await;

    let req = test::TestRequest::get()
        .uri("/client_balance/99999")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "CLIENT_NOT_FOUND");
}

#[actix_web::test]
async fn post_credit_returns_200_with_new_balance() {
    let app = build_app().await;

    let create_req = test::TestRequest::post()
        .uri("/new_client")
        .set_json(json!({
            "client_name": "Luis Torres",
            "birth_date": "1990-01-01",
            "document_number": "CR-20001",
            "country": "AR"
        }))
        .to_request();
    let create_resp: Value = test::read_body_json(test::call_service(&app, create_req).await).await;
    let client_id = create_resp["client_id"].as_u64().unwrap();

    let req = test::TestRequest::post()
        .uri("/new_credit_transaction")
        .set_json(json!({
            "client_id": client_id,
            "credit_amount": "500.75"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["client_id"], client_id);
    assert_eq!(body["type"], "credit");
    assert!(body["new_balance"].as_str().is_some() || body["new_balance"].is_number());
}

#[actix_web::test]
async fn post_credit_unknown_client_returns_404() {
    let app = build_app().await;

    let req = test::TestRequest::post()
        .uri("/new_credit_transaction")
        .set_json(json!({
            "client_id": 99999,
            "credit_amount": "100.00"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "CLIENT_NOT_FOUND");
}

#[actix_web::test]
async fn post_credit_with_negative_amount_returns_400() {
    let app = build_app().await;

    let create_req = test::TestRequest::post()
        .uri("/new_client")
        .set_json(json!({
            "client_name": "Neg Test",
            "birth_date": "1990-01-01",
            "document_number": "CR-NEG01",
            "country": "AR"
        }))
        .to_request();
    let create_resp: Value = test::read_body_json(test::call_service(&app, create_req).await).await;
    let client_id = create_resp["client_id"].as_u64().unwrap();

    let req = test::TestRequest::post()
        .uri("/new_credit_transaction")
        .set_json(json!({
            "client_id": client_id,
            "credit_amount": "-50.00"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "NEGATIVE_AMOUNT");
}

#[actix_web::test]
async fn post_debit_returns_200_with_new_balance() {
    let app = build_app().await;

    let create_req = test::TestRequest::post()
        .uri("/new_client")
        .set_json(json!({
            "client_name": "Sofía Méndez",
            "birth_date": "1985-09-30",
            "document_number": "DB-30001",
            "country": "AR"
        }))
        .to_request();
    let create_resp: Value = test::read_body_json(test::call_service(&app, create_req).await).await;
    let client_id = create_resp["client_id"].as_u64().unwrap();

    let credit_req = test::TestRequest::post()
        .uri("/new_credit_transaction")
        .set_json(json!({ "client_id": client_id, "credit_amount": "1000.00" }))
        .to_request();
    test::call_service(&app, credit_req).await;

    let req = test::TestRequest::post()
        .uri("/new_debit_transaction")
        .set_json(json!({
            "client_id": client_id,
            "debit_amount": "350.00"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["client_id"], client_id);
    assert_eq!(body["type"], "debit");
}

#[actix_web::test]
async fn post_debit_insufficient_funds_returns_400() {
    let app = build_app().await;

    let create_req = test::TestRequest::post()
        .uri("/new_client")
        .set_json(json!({
            "client_name": "Carlos Seco",
            "birth_date": "1992-04-12",
            "document_number": "DB-IF001",
            "country": "AR"
        }))
        .to_request();
    let create_resp: Value = test::read_body_json(test::call_service(&app, create_req).await).await;
    let client_id = create_resp["client_id"].as_u64().unwrap();

    let req = test::TestRequest::post()
        .uri("/new_debit_transaction")
        .set_json(json!({
            "client_id": client_id,
            "debit_amount": "1.00"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "INSUFFICIENT_FUNDS");
}

#[actix_web::test]
async fn post_debit_unknown_client_returns_404() {
    let app = build_app().await;

    let req = test::TestRequest::post()
        .uri("/new_debit_transaction")
        .set_json(json!({
            "client_id": 99999,
            "debit_amount": "100.00"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "CLIENT_NOT_FOUND");
}

#[actix_web::test]
async fn post_store_balances_returns_200() {
    let app = build_app().await;

    let req = test::TestRequest::post()
        .uri("/store_balances")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
}
