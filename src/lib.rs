use pgrx::prelude::*;
use pgrx::spi;

pgrx::pg_module_magic!();

pub mod schema;

extension_sql!(
    r#"
    CREATE SCHEMA rebac;

    CREATE TABLE rebac.schema (
        rowid BIGINT GENERATED ALWAYS AS IDENTITY,
        id UUID PRIMARY KEY DEFAULT gen_random_uuid() ,
        schema TEXT NOT NULL, -- JSON???
        created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
    );

    CREATE TABLE rebac.tuple (
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
    name = "initialize_pg_rebac"
);

#[pg_extern]
fn pg_rebac_create_schema(schema: &str) -> Result<Option<pgrx::Uuid>, spi::Error> {
    Spi::get_one_with_args(
        "INSERT INTO rebac.schema (schema) VALUES ($1) RETURNING id",
        vec![(PgBuiltInOids::TEXTOID.oid(), schema.into_datum())],
    )
}

#[pg_extern]
fn pg_rebac_create_tuple(
    schema_id: pgrx::Uuid,
    resource_namespace: &str,
    resource_id: &str,
    relation: &str,
    subject_namespace: &str,
    subject_id: &str,
    subject_action: default!(&str, "''"),
) -> Result<(), spi::Error> {
    Spi::run_with_args(
        "
        INSERT INTO rebac.tuple (
            schema_id,
            resource_namespace,
            resource_id,
            relation,
            subject_namespace,
            subject_id,
            subject_action
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING rowid
        ",
        Some(vec![
            (PgBuiltInOids::UUIDOID.oid(), schema_id.into_datum()),
            (
                PgBuiltInOids::VARCHAROID.oid(),
                resource_namespace.into_datum(),
            ),
            (PgBuiltInOids::VARCHAROID.oid(), resource_id.into_datum()),
            (PgBuiltInOids::VARCHAROID.oid(), relation.into_datum()),
            (
                PgBuiltInOids::VARCHAROID.oid(),
                subject_namespace.into_datum(),
            ),
            (PgBuiltInOids::VARCHAROID.oid(), subject_id.into_datum()),
            (PgBuiltInOids::VARCHAROID.oid(), subject_action.into_datum()),
        ]),
    )
}

#[pg_extern]
fn pg_rebac_delete_tuple(
    schema_id: pgrx::Uuid,
    resource_namespace: &str,
    resource_id: &str,
    relation: &str,
    subject_namespace: &str,
    subject_id: &str,
    subject_action: default!(&str, "''"),
) -> Result<(), spi::Error> {
    Spi::run_with_args(
        "
        DELETE FROM rebac.tuple
        WHERE schema_id = $1
            AND resource_namespace = $2
            AND resource_id = $3
            AND relation = $4
            AND subject_namespace = $5
            AND subject_id = $6
            AND subject_action = $7
        ",
        Some(vec![
            (PgBuiltInOids::UUIDOID.oid(), schema_id.into_datum()),
            (
                PgBuiltInOids::VARCHAROID.oid(),
                resource_namespace.into_datum(),
            ),
            (PgBuiltInOids::VARCHAROID.oid(), resource_id.into_datum()),
            (PgBuiltInOids::VARCHAROID.oid(), relation.into_datum()),
            (
                PgBuiltInOids::VARCHAROID.oid(),
                subject_namespace.into_datum(),
            ),
            (PgBuiltInOids::VARCHAROID.oid(), subject_id.into_datum()),
            (PgBuiltInOids::VARCHAROID.oid(), subject_action.into_datum()),
        ]),
    )
}

#[pg_extern]
fn hello_pg_rebac() -> &'static str {
    "Hello, pg_rebac"
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    #[pg_test]
    fn test_hello_pg_rebac() {
        assert_eq!("Hello, pg_rebac", crate::hello_pg_rebac());
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
