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
fn create_schema(schema: pgrx::Json) -> Result<Option<pgrx::Uuid>, PgFgaError> {
    Spi::connect(|client| Storage::new(client).create_schema(schema))
}

#[pg_extern]
fn read_schema(
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
        Spi::connect(|client| Storage::new(client).read_schemas(Some(id)))?
            .into_iter()
            .map(|row| row.into())
            .collect();

    Ok(TableIterator::new(results))
}

#[pg_extern]
fn read_schemas() -> Result<
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
        Spi::connect(|client| Storage::new(client).read_schemas(None))?
            .into_iter()
            .map(|row| row.into())
            .collect();

    Ok(TableIterator::new(results))
}

#[pg_extern]
fn create_tuple(
    schema_id: pgrx::Uuid,
    resource_namespace: &str,
    resource_id: &str,
    relation: &str,
    subject_namespace: &str,
    subject_id: &str,
    subject_action: default!(&str, "''"),
) -> Result<i64, PgFgaError> {
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
fn read_tuples(
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
    .map(|row| row.into())
    .collect();

    Ok(TableIterator::new(result))
}

#[pg_extern]
fn delete_tuple(
    schema_id: pgrx::Uuid,
    resource_namespace: &str,
    resource_id: &str,
    relation: &str,
    subject_namespace: &str,
    subject_id: &str,
    subject_action: default!(&str, "''"),
) -> Result<i64, PgFgaError> {
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
fn check(
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

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;
    use serde_json::json;
    use uuid;

    #[pg_test]
    fn test_cannot_create_invalid_schema() {
        let err = create_schema(pgrx::Json(json!({"foo":"bar"}))).unwrap_err();
        assert!(matches!(err, PgFgaError::SerdeError(_)));
    }

    #[pg_test]
    fn test_create_and_read_schema() {
        let schema = json!({"namespaces":{"user":{"relations": {}, "permissions": {}}}});
        let id = create_schema(pgrx::Json(schema.clone())).unwrap().unwrap();

        let mut iter = read_schema(id).unwrap();
        let (_, got_id, got_schema, _) = iter.next().unwrap();

        assert_eq!(got_id, id);
        assert_eq!(got_schema.0, schema);
        assert!(iter.next().is_none());
    }

    #[pg_test]
    fn test_create_and_read_schemas() {
        let schema = json!({"namespaces":{"user":{"relations": {}, "permissions": {}}}});
        let id1 = create_schema(pgrx::Json(schema.clone())).unwrap().unwrap();
        let id2 = create_schema(pgrx::Json(schema.clone())).unwrap().unwrap();

        let mut iter = read_schemas().unwrap();
        let (_, got_id1, got_schema1, _) = iter.next().unwrap();

        assert_eq!(got_id1, id1);
        assert_eq!(got_schema1.0, schema);

        let (_, got_id2, got_schema2, _) = iter.next().unwrap();

        assert_eq!(got_id2, id2);
        assert_eq!(got_schema2.0, schema);

        assert!(iter.next().is_none());
    }

    #[pg_test]
    fn test_cannot_create_tuple_on_nonexistant_schema() {
        let schema_id = pgrx::Uuid::from_bytes(uuid::Uuid::new_v4().into_bytes());

        let err =
            create_tuple(schema_id, "document", "1", "viewer", "user", "anya", "").unwrap_err();
        assert!(matches!(err, PgFgaError::UnknownSchemaId(_)));
    }

    #[pg_test]
    fn test_create_duplicate_tuple_returns_zero() {
        let id = create_schema(pgrx::Json(
            json!({"namespaces":{"document":{"relations": {}, "permissions": {}}}}),
        ))
        .unwrap()
        .unwrap();

        let first = create_tuple(id, "document", "1", "viewer", "user", "anya", "").unwrap();
        assert_eq!(first, 1);

        let second = create_tuple(id, "document", "1", "viewer", "user", "anya", "").unwrap();
        assert_eq!(second, 0);
    }

    #[pg_test]
    fn test_read_tuples_works() {
        let id = create_schema(pgrx::Json(
            json!({"namespaces":{"document":{"relations": {}, "permissions": {}}}}),
        ))
        .unwrap()
        .unwrap();

        let tup1 = ("document", "1", "parent", "folder", "x", "");
        let tup2 = ("folder", "x", "viewer", "user", "anya", "");

        create_tuple(id, tup1.0, tup1.1, tup1.2, tup1.3, tup1.4, tup1.5).unwrap();
        create_tuple(id, tup2.0, tup2.1, tup2.2, tup2.3, tup2.4, tup2.5).unwrap();

        // Read all the tuples.
        let mut iter = read_tuples(id, "", "", "", "", "", "").unwrap();
        let (_, _, got_rns, got_rid, got_rel, got_sns, got_sid, got_sact) = iter.next().unwrap();

        assert_eq!(
            (
                got_rns.as_str(),
                got_rid.as_str(),
                got_rel.as_str(),
                got_sns.as_str(),
                got_sid.as_str(),
                got_sact.as_str()
            ),
            tup1
        );

        let (_, _, got_rns, got_rid, got_rel, got_sns, got_sid, got_sact) = iter.next().unwrap();

        assert_eq!(
            (
                got_rns.as_str(),
                got_rid.as_str(),
                got_rel.as_str(),
                got_sns.as_str(),
                got_sid.as_str(),
                got_sact.as_str()
            ),
            tup2
        );

        assert!(iter.next().is_none());

        // Filter on relation.
        let mut iter = read_tuples(id, "", "", "parent", "", "", "").unwrap();
        let (_, _, got_rns, got_rid, got_rel, got_sns, got_sid, got_sact) = iter.next().unwrap();

        assert_eq!(
            (
                got_rns.as_str(),
                got_rid.as_str(),
                got_rel.as_str(),
                got_sns.as_str(),
                got_sid.as_str(),
                got_sact.as_str()
            ),
            tup1
        );

        assert!(iter.next().is_none());

        // Filter on resource_namespace.
        let mut iter = read_tuples(id, "folder", "", "", "", "", "").unwrap();
        let (_, _, rns1, rid1, rel1, sns1, sid1, sact1) = iter.next().unwrap();

        assert_eq!(
            (
                rns1.as_str(),
                rid1.as_str(),
                rel1.as_str(),
                sns1.as_str(),
                sid1.as_str(),
                sact1.as_str()
            ),
            tup2
        );

        assert!(iter.next().is_none());
    }

    #[pg_test]
    fn test_delete_tuple_works() {
        let id = create_schema(pgrx::Json(
            json!({"namespaces":{"document":{"relations": {}, "permissions": {}}}}),
        ))
        .unwrap()
        .unwrap();

        create_tuple(id, "document", "1", "viewer", "user", "anya", "").unwrap();

        let first = delete_tuple(id, "document", "1", "viewer", "user", "anya", "").unwrap();
        assert_eq!(first, 1);

        // Second delete should return 0 if we try to delete the same tuple again.
        let second = delete_tuple(id, "document", "1", "viewer", "user", "anya", "").unwrap();
        assert_eq!(second, 0);

        // And let's just verify there are no tuples.
        let mut iter = read_tuples(id, "", "", "", "", "", "").unwrap();
        assert!(iter.next().is_none())
    }

    //
    // Check tests
    //

    #[pg_test]
    fn test_direct_check() {
        let schema = pgrx::Json(json!(
            {
                "namespaces": {
                    "user": {},
                    "document": {
                        "relations": {
                            "viewer": [
                                {
                                    "namespace": "user",
                                }
                            ]
                        },
                    },
                },
            }
        ));

        let id = create_schema(schema).unwrap().unwrap();

        create_tuple(id, "document", "1", "viewer", "user", "anya", "").unwrap();

        let should_be_true = check(id, "document", "1", "viewer", "user", "anya", "").unwrap();
        assert!(should_be_true);

        let should_be_false1 = check(id, "document", "2", "viewer", "user", "anya", "").unwrap();
        assert!(!should_be_false1);

        let should_be_false2 = check(id, "document", "2", "viewer", "user", "beatrix", "").unwrap();
        assert!(!should_be_false2);
    }

    #[pg_test]
    fn test_computed_userset() {
        let schema = pgrx::Json(json!(
            {
                "namespaces": {
                    "user": {},
                    "document": {
                        "relations": {
                            "viewer": [
                                {
                                    "namespace": "user",
                                }
                            ]
                        },
                        "permissions": {
                            "can_view": {
                                "computedUserset": "viewer"
                            }
                        }
                    },
                },
            }
        ));

        let id = create_schema(schema).unwrap().unwrap();

        create_tuple(id, "document", "1", "viewer", "user", "anya", "").unwrap();

        let should_be_true = check(id, "document", "1", "can_view", "user", "anya", "").unwrap();
        assert!(should_be_true);

        let should_be_false1 = check(id, "document", "2", "can_view", "user", "anya", "").unwrap();
        assert!(!should_be_false1);

        let should_be_false2 =
            check(id, "document", "2", "can_view", "user", "beatrix", "").unwrap();
        assert!(!should_be_false2);
    }

    #[pg_test]
    fn test_tuple_to_userset() {
        let schema = pgrx::Json(json!(
            {
                "namespaces": {
                    "user": {},
                    "folder": {
                        "relations": {
                            "viewer": [
                                {
                                    "namespace": "user",
                                }
                            ]
                        },
                        "permissions": {
                            "can_view": {
                                "computedUserset": "viewer"
                            }
                        }
                    },
                    "document": {
                        "relations": {
                            "parent": [
                                {
                                    "namespace": "folder",
                                }
                            ]
                        },
                        "permissions": {
                            "can_view": {
                                "tupleToUserset": ["parent", "can_view"]
                            }
                        }
                    },
                },
            }
        ));

        let id = create_schema(schema).unwrap().unwrap();

        create_tuple(id, "folder", "x", "viewer", "user", "anya", "").unwrap();
        create_tuple(id, "document", "1", "parent", "folder", "x", "").unwrap();

        let should_be_true1 = check(id, "folder", "x", "can_view", "user", "anya", "").unwrap();
        assert!(should_be_true1);

        let should_be_true2 = check(id, "document", "1", "can_view", "user", "anya", "").unwrap();
        assert!(should_be_true2);

        let should_be_false1 = check(id, "document", "2", "can_view", "user", "anya", "").unwrap();
        assert!(!should_be_false1);

        let should_be_false2 =
            check(id, "document", "2", "can_view", "user", "beatrix", "").unwrap();
        assert!(!should_be_false2);
    }

    #[pg_test]
    fn test_union() {
        let schema = pgrx::Json(json!(
            {
                "namespaces": {
                    "user": {},
                    "document": {
                        "relations": {
                            "viewer": [
                                {
                                    "namespace": "user",
                                }
                            ],
                            "editor": [
                                {
                                    "namespace": "user",
                                }
                            ]
                        },
                        "permissions": {
                            "can_view": {
                                "union": [
                                    {
                                        "computedUserset": "viewer",
                                    },
                                    {
                                        "computedUserset": "editor",
                                    },
                                ]
                            }
                        }
                    },
                },
            }
        ));

        let id = create_schema(schema).unwrap().unwrap();

        create_tuple(id, "document", "1", "viewer", "user", "anya", "").unwrap();
        create_tuple(id, "document", "2", "editor", "user", "beatrix", "").unwrap();

        let should_be_true1 = check(id, "document", "1", "can_view", "user", "anya", "").unwrap();
        assert!(should_be_true1);

        let should_be_false1 =
            check(id, "document", "1", "can_view", "user", "beatrix", "").unwrap();
        assert!(!should_be_false1);

        let should_be_true2 =
            check(id, "document", "2", "can_view", "user", "beatrix", "").unwrap();
        assert!(should_be_true2);

        let should_be_false2 = check(id, "document", "2", "can_view", "user", "anya", "").unwrap();
        assert!(!should_be_false2);
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
