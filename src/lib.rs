use check::Checker;
use error::PgFgaError;
use pgrx::prelude::*;
use storage::Storage;

pgrx::pg_module_magic!();

pub mod check;
pub mod error;
pub mod schema;
pub mod storage;

extension_sql!(
    r#"
    CREATE SCHEMA pgfga;

    CREATE TABLE pgfga.schema (
        rowid BIGINT GENERATED ALWAYS AS IDENTITY,
        id UUID PRIMARY KEY DEFAULT gen_random_uuid() ,
        schema JSON NOT NULL,
        created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
    );

    CREATE TABLE pgfga.tuple (
        rowid BIGINT GENERATED ALWAYS AS IDENTITY,
        schema_id UUID NOT NULL,
        resource_namespace VARCHAR(128) NOT NULL,
        resource_id VARCHAR(128) NOT NULL,
        relation VARCHAR(128) NOT NULL,
        subject_namespace VARCHAR(128) NOT NULL,
        subject_id VARCHAR(128) NOT NULL,
        subject_action VARCHAR(128) DEFAULT ''::TEXT NOT NULL,
        PRIMARY KEY (schema_id, resource_namespace, resource_id, relation, subject_namespace, subject_id, subject_action)
    );

    -- TODO: add indices!

    "#,
    name = "initialize_pgfga"
);

#[pg_extern]
fn pgfga_create_schema(schema: pgrx::Json) -> Result<Option<pgrx::Uuid>, PgFgaError> {
    Spi::connect(|client| Storage::new(client).create_schema(schema))
}

#[pg_extern]
fn pgfga_read_schema(
    id: pgrx::Uuid,
) -> Result<
    TableIterator<
        'static,
        (
            name!(rowid, i64),
            name!(id, pgrx::Uuid),
            name!(schema, pgrx::Json),
            name!(created_at, pgrx::TimestampWithTimeZone),
        ),
    >,
    PgFgaError,
> {
    let results: Vec<(i64, pgrx::Uuid, pgrx::Json, pgrx::TimestampWithTimeZone)> =
        Spi::connect(|client| Storage::new(client).read_schema(id))?
            .into_iter()
            .map(|row| row.into_tuple())
            .collect();

    Ok(TableIterator::new(results))
}

#[pg_extern]
fn pgfga_create_tuple(
    schema_id: pgrx::Uuid,
    resource_namespace: &str,
    resource_id: &str,
    relation: &str,
    subject_namespace: &str,
    subject_id: &str,
    subject_action: default!(&str, "''"),
) -> Result<(), PgFgaError> {
    Spi::connect(|client| {
        Storage::new(client).create_tuple(
            schema_id,
            resource_namespace,
            resource_id,
            relation,
            subject_namespace,
            subject_id,
            subject_action,
        )
    })
}

#[pg_extern]
fn pgfga_read_tuples(
    schema_id: pgrx::Uuid,
    resource_namespace: &str,
    resource_id: &str,
    relation: &str,
    subject_namespace: &str,
    subject_id: &str,
    subject_action: default!(&str, "''"),
) -> Result<
    TableIterator<
        'static,
        (
            name!(rowid, i64),
            name!(schema_id, pgrx::Uuid),
            name!(resource_namespace, String),
            name!(resource_id, String),
            name!(relation, String),
            name!(subject_namespace, String),
            name!(subject_id, String),
            name!(subject_action, String),
        ),
    >,
    PgFgaError,
> {
    let result: Vec<(
        i64,
        pgrx::Uuid,
        String,
        String,
        String,
        String,
        String,
        String,
    )> = Spi::connect(|client| {
        Storage::new(client).read_tuples(
            schema_id,
            resource_namespace,
            resource_id,
            relation,
            subject_namespace,
            subject_id,
            subject_action,
        )
    })?
    .into_iter()
    .map(|row| row.into_tuple())
    .collect();

    Ok(TableIterator::new(result))
}

#[pg_extern]
fn pgfga_delete_tuple(
    schema_id: pgrx::Uuid,
    resource_namespace: &str,
    resource_id: &str,
    relation: &str,
    subject_namespace: &str,
    subject_id: &str,
    subject_action: default!(&str, "''"),
) -> Result<(), PgFgaError> {
    Spi::connect(|client| {
        Storage::new(client).delete_tuple(
            schema_id,
            resource_namespace,
            resource_id,
            relation,
            subject_namespace,
            subject_id,
            subject_action,
        )
    })
}

#[pg_extern]
fn pgfga_check(
    schema_id: pgrx::Uuid,
    resource_namespace: &str,
    resource_id: &str,
    action: &str,
    subject_namespace: &str,
    subject_id: &str,
    subject_action: default!(&str, "''"),
) -> Result<bool, PgFgaError> {
    Spi::connect(|client| {
        Checker::new(Storage::new(client), schema_id)?.check(
            resource_namespace,
            resource_id,
            action,
            subject_namespace,
            subject_id,
            subject_action,
        )
    })
}

// TODO: delete when I actually write some tests.
#[pg_extern]
fn hello_pgfga() -> &'static str {
    "Hello, pgfga"
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    #[pg_test]
    fn test_hello_pgfga() {
        assert_eq!("Hello, pgfga", crate::hello_pgfga());
    }
}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
