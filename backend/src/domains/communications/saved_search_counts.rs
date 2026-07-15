use std::collections::HashMap;

use sqlx::postgres::PgPool;
use sqlx::{Postgres, QueryBuilder, Row};

use crate::domains::communications::messages::models::MessageSearchQuery;
use crate::domains::communications::messages::query_parser::parse_communication_message_search_query;
use crate::domains::communications::messages::search::append_message_search_filter;
use crate::domains::communications::saved_searches::{
    CommunicationSavedSearchError, SavedSearchRecord,
};

pub(crate) async fn count_messages_for_saved_search<'e, E>(
    executor: E,
    record: &SavedSearchRecord,
) -> Result<i64, CommunicationSavedSearchError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let mut builder = QueryBuilder::<Postgres>::new(
        "SELECT count(*)::BIGINT AS message_count FROM communication_messages m WHERE 1 = 1",
    );
    append_saved_search_filters(&mut builder, record);
    let row = builder.build().fetch_one(executor).await?;
    Ok(row.try_get::<i64, _>("message_count")?)
}

pub(crate) async fn load_message_counts_for_saved_searches(
    pool: &PgPool,
    records: &[SavedSearchRecord],
) -> Result<HashMap<String, i64>, CommunicationSavedSearchError> {
    if records.is_empty() {
        return Ok(HashMap::new());
    }

    let mut builder = QueryBuilder::<Postgres>::new("");
    for (index, record) in records.iter().enumerate() {
        if index > 0 {
            builder.push(" UNION ALL ");
        }
        builder.push("SELECT ");
        builder.push_bind(record.saved_search_id.clone());
        builder.push(
            " AS saved_search_id, count(*)::BIGINT AS message_count FROM communication_messages m WHERE 1 = 1",
        );
        append_saved_search_filters(&mut builder, record);
    }
    let rows = builder.build().fetch_all(pool).await?;

    let mut counts = HashMap::new();
    for row in rows {
        counts.insert(
            row.try_get::<String, _>("saved_search_id")?,
            row.try_get::<i64, _>("message_count")?,
        );
    }

    Ok(counts)
}

fn append_saved_search_filters<'args>(
    builder: &mut QueryBuilder<'args, Postgres>,
    record: &SavedSearchRecord,
) {
    if let Some(account_id) = record.account_id.as_deref() {
        builder.push(" AND m.account_id = ");
        builder.push_bind(account_id.to_owned());
    }
    if let Some(workflow_state) = record.workflow_state {
        builder.push(" AND m.workflow_state = ");
        builder.push_bind(workflow_state.as_str().to_owned());
    }
    if let Some(channel_kind) = record.channel_kind.as_deref() {
        builder.push(" AND m.channel_kind = ");
        builder.push_bind(channel_kind.to_owned());
    }
    if let Some(local_state) = record.local_state.persisted_filter() {
        builder.push(" AND m.local_state = ");
        builder.push_bind(local_state.to_owned());
    }
    append_message_search_filter(builder, "m", &parsed_search_query(record));
}

fn parsed_search_query(record: &SavedSearchRecord) -> MessageSearchQuery {
    parse_communication_message_search_query(Some(&record.query))
}
