pub async fn new_client(
    state: SharedState,
    payload: web::Json<NewClientPayload>,
) -> Result<HttpResponse, AppError> {
    let mut guard = state.write().await;
    let client = guard.add_client(
        payload.document_number.clone(),
        payload.client_name.clone(),
        payload.birth_date,
        payload.country.clone(),
    )?;
    log::info!("Nuevo cliente creado: id={} doc={}", client.client_id, client.document_number);
    Ok(HttpResponse::Created().json(client))
}