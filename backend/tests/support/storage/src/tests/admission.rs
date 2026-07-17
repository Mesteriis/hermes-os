use hermes_storage_migrations::{MigrationAdmissionErrorV1, admit_owner_local_additive_sql};

#[test]
fn admits_owner_local_create_table() {
    let result = admit_owner_local_additive_sql(
        "notes",
        "CREATE TABLE hermes_data.notes_entries (entry_id uuid);",
    );
    assert_eq!(result, Ok(()));
}

#[test]
fn admits_owner_local_add_column() {
    let result = admit_owner_local_additive_sql(
        "notes",
        "ALTER TABLE hermes_data.notes_entries ADD COLUMN title text;",
    );
    assert_eq!(result, Ok(()));
}

#[test]
fn rejects_non_migration_statement() {
    let result = admit_owner_local_additive_sql("notes", "SELECT 1;");
    assert_eq!(result, Err(MigrationAdmissionErrorV1::Forbidden));
}

#[test]
fn rejects_wrong_schema_and_non_additive_alteration() {
    let wrong_schema = admit_owner_local_additive_sql(
        "notes",
        "CREATE TABLE public.notes_entries (entry_id uuid);",
    );
    let drop_column = admit_owner_local_additive_sql(
        "notes",
        "ALTER TABLE hermes_data.notes_entries DROP COLUMN title;",
    );
    assert_eq!(wrong_schema, Err(MigrationAdmissionErrorV1::Forbidden));
    assert_eq!(drop_column, Err(MigrationAdmissionErrorV1::Forbidden));
}
