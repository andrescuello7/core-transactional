pub async fn store_balances(state: SharedState) -> Result<HttpResponse, AppError> {
    // Hold the write lock for the entire operation to guarantee atomicity:
    // the file snapshot and the balance reset are a single critical section.
    let mut guard = state.write().await;

    let date = Local::now().format("%d%m%Y").to_string();
    let counter = next_file_counter();
    let filename = format!("{}_{}.DAT", date, counter);

    let content: String = guard
        .balances_snapshot()
        .into_iter()
        .map(|(id, balance)| format!("{} {}", id, balance))
        .collect::<Vec<_>>()
        .join("\n");

    tokio::fs::write(&filename, &content)
        .await
        .map_err(|e| AppError::PersistenceError(e.to_string()))?;

    guard.reset_balances();

    log::info!("Balances persistidos en '{}'", filename);
    Ok(HttpResponse::Ok().json(json!({ "file": filename })))
}