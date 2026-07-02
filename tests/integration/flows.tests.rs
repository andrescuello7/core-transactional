use actix_web::{test, web, App};
use core-transactional::infra::http::server::register_routes;
use core-transactional::infra::storage::alloc_memory::{Alloc, Command};
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

async fn create_client(
    app: &impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
    doc: &str,
    name: &str,
) -> u64 {
    let req = test::TestRequest::post()
        .uri("/new_client")
        .set_json(json!({
            "client_name": name,
            "birth_date": "1990-01-01",
            "document_number": doc,
            "country": "AR"
        }))
        .to_request();
    let resp: Value = test::read_body_json(test::call_service(app, req).await).await;
    resp["client_id"].as_u64().unwrap()
}


#[actix_web::test]
async fn flow_create_credit_debit_and_check_balance() {
    let app = build_app().await;
    let id = create_client(&app, "FLOW-001", "Roberto Sosa").await;

    let req = test::TestRequest::post()
        .uri("/new_credit_transaction")
        .set_json(json!({ "client_id": id, "credit_amount": "1000.00" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let req = test::TestRequest::post()
        .uri("/new_debit_transaction")
        .set_json(json!({ "client_id": id, "debit_amount": "300.00" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let req = test::TestRequest::get()
        .uri(&format!("/client_balance/{}", id))
        .to_request();
    let body: Value = test::read_body_json(test::call_service(&app, req).await).await;

    let balance_str = body["balance"].as_str().unwrap_or("0");
    let balance: f64 = balance_str.parse().unwrap();
    assert!((balance - 700.0).abs() < 0.001);
}


#[actix_web::test]
async fn flow_balance_resets_to_zero_after_store() {
    let app = build_app().await;
    let id = create_client(&app, "FLOW-002", "Carmen Vidal").await;

    let req = test::TestRequest::post()
        .uri("/new_credit_transaction")
        .set_json(json!({ "client_id": id, "credit_amount": "500.00" }))
        .to_request();
    test::call_service(&app, req).await;

    let req = test::TestRequest::post()
        .uri("/store_balances")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let req = test::TestRequest::get()
        .uri(&format!("/client_balance/{}", id))
        .to_request();
    let body: Value = test::read_body_json(test::call_service(&app, req).await).await;
    let balance_str = body["balance"].as_str().unwrap_or("0");
    let balance: f64 = balance_str.parse().unwrap();
    assert_eq!(balance, 0.0);
}

#[actix_web::test]
async fn flow_multiple_clients_independent_balances() {
    let app = build_app().await;
    let id_a = create_client(&app, "FLOW-003A", "Mario Fernández").await;
    let id_b = create_client(&app, "FLOW-003B", "Elena Castro").await;

    let req = test::TestRequest::post()
        .uri("/new_credit_transaction")
        .set_json(json!({ "client_id": id_a, "credit_amount": "200.00" }))
        .to_request();
    test::call_service(&app, req).await;

    let req = test::TestRequest::post()
        .uri("/new_credit_transaction")
        .set_json(json!({ "client_id": id_b, "credit_amount": "800.00" }))
        .to_request();
    test::call_service(&app, req).await;

    let req = test::TestRequest::post()
        .uri("/new_debit_transaction")
        .set_json(json!({ "client_id": id_b, "debit_amount": "300.00" }))
        .to_request();
    test::call_service(&app, req).await;

    let req = test::TestRequest::get()
        .uri(&format!("/client_balance/{}", id_a))
        .to_request();
    let body: Value = test::read_body_json(test::call_service(&app, req).await).await;
    let balance_a: f64 = body["balance"].as_str().unwrap_or("0").parse().unwrap();
    assert!((balance_a - 200.0).abs() < 0.001, "balance de A debe ser 200, fue {}", balance_a);

    let req = test::TestRequest::get()
        .uri(&format!("/client_balance/{}", id_b))
        .to_request();
    let body: Value = test::read_body_json(test::call_service(&app, req).await).await;
    let balance_b: f64 = body["balance"].as_str().unwrap_or("0").parse().unwrap();
    assert!((balance_b - 500.0).abs() < 0.001, "balance de B debe ser 500, fue {}", balance_b);
}

#[actix_web::test]
async fn flow_can_transact_after_store() {
    let app = build_app().await;
    let id = create_client(&app, "FLOW-004", "Julio Ramos").await;

    let req = test::TestRequest::post()
        .uri("/new_credit_transaction")
        .set_json(json!({ "client_id": id, "credit_amount": "400.00" }))
        .to_request();
    test::call_service(&app, req).await;

    let req = test::TestRequest::post()
        .uri("/store_balances")
        .to_request();
    test::call_service(&app, req).await;

    let req = test::TestRequest::post()
        .uri("/new_credit_transaction")
        .set_json(json!({ "client_id": id, "credit_amount": "150.00" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let req = test::TestRequest::get()
        .uri(&format!("/client_balance/{}", id))
        .to_request();
    let body: Value = test::read_body_json(test::call_service(&app, req).await).await;
    let balance: f64 = body["balance"].as_str().unwrap_or("0").parse().unwrap();
    assert!((balance - 150.0).abs() < 0.001);
}
