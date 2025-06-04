use sqlx::{
    types::chrono::{DateTime, Utc},
    FromRow, Pool, Postgres,
};

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct EpochTestRow {
    keys_ordered_by_epoch_id: Option<i32>,
    keys_ordered_by_epoch_row_num: Option<i64>,
    keys_ordered_by_epoch_transaction_timestamp: Option<DateTime<Utc>>,
    keys_ordered_by_epoch_epoch: Option<i32>,
    keys_ordered_by_epoch_pk_json: Option<serde_json::Value>,
    keys_ordered_by_transaction_timestamp_id: Option<i32>,
    keys_ordered_by_transaction_timestamp_row_num: Option<i64>,
    keys_ordered_by_transaction_timestamp_transaction_timestamp: Option<DateTime<Utc>>,
    keys_ordered_by_transaction_timestamp_epoch: Option<i32>,
    keys_ordered_by_transaction_timestamp_pk_json: Option<serde_json::Value>,
}

pub async fn ordering_by_transaction_timestamp_and_epoch_should_be_the_same(
    pool: &Pool<Postgres>,
) -> std::vec::Vec<EpochTestRow> {
    sqlx::query_file_as!(EpochTestRow, "src/check_epochs_are_in_serial_order.sql")
        .fetch_all(&pool.clone())
        .await
        .expect("Query results")
}
