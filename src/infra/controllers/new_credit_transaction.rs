pub async fn new_credit_transaction(
    state: SharedState,
    payload: web::Json<CreditTransaction>,
) -> Result<HttpResponse, AppError> {
    let mut guard = state.write().await;
    let new_balance = guard.credit(payload.client_id, payload.credit_amount)?;
    log::info!(
        "Crédito aplicado: client_id={} amount={} balance={}",
        payload.client_id, payload.credit_amount, new_balance
    );
    Ok(HttpResponse::Ok().json(json!({
        "client_id": payload.client_id,
        "amount": payload.credit_amount,
        "new_balance": new_balance,
        "type": "credit"
    })))
}