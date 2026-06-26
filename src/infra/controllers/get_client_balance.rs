pub async fn get_client_balance(
    state: SharedState,
    path: web::Path<u64>,
) -> Result<HttpResponse, AppError> {
    let client_id = path.into_inner();
    let guard = state.read().await;
    let balance = guard.get_balance(client_id)?;
    Ok(HttpResponse::Ok().json(json!({
        "client_id": client_id,
        "balance": balance
    })))
}