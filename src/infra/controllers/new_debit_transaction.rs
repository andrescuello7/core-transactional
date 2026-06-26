pub async fn new_debit_transaction(
    state: SharedState,
    payload: web::Json<DebitTransaction>,
) -> Result<HttpResponse, AppError> {
    let mut guard = state.write().await;
    let new_balance = guard.debit(payload.client_id, payload.debit_amount)?;
    log::info!(
        "Débito aplicado: client_id={} amount={} balance={}",
        payload.client_id, payload.debit_amount, new_balance
    );
    Ok(HttpResponse::Ok().json(json!({
        "client_id": payload.client_id,
        "amount": payload.debit_amount,
        "new_balance": new_balance,
        "type": "debit"
    })))
}