use crate::error::PgFgaError;
use crate::schema::Schema;
use pgrx::prelude::*;
use pgrx::spi::SpiClient;

pub struct Storage<'a> {
    client: SpiClient<'a>,
}

pub struct TupleRow {
    rowid: i64,
    schema_id: pgrx::Uuid,
    resource_namespace: String,
    resource_id: String,
    relation: String,
    subject_namespace: String,
    subject_id: String,
    subject_action: String,
}

// TODO: probably don't need this.
impl TupleRow {
    // fn new(
    //     rowid: i64,
    //     schema_id: pgrx::Uuid,
    //     resource_namespace: &str,
    //     resource_id: &str,
    //     relation: &str,
    //     subject_namespace: &str,
    //     subject_id: &str,
    //     subject_action: &str,
    // ) -> Self {
    //     Self {
    //         rowid: rowid,
    //         schema_id,
    //         resource_namespace: resource_namespace.to_string(),
    //         resource_id: resource_id.to_string(),
    //         relation: relation.to_string(),
    //         subject_namespace: subject_namespace.to_string(),
    //         subject_id: subject_id.to_string(),
    //         subject_action: subject_action.to_string(),
    //     }
    // }

    pub fn into_tuple(
        self,
    ) -> (
        i64,
        pgrx::Uuid,
        String,
        String,
        String,
        String,
        String,
        String,
    ) {
        (
            self.rowid,
            self.schema_id,
            self.resource_namespace,
            self.resource_id,
            self.relation,
            self.subject_namespace,
            self.subject_id,
            self.subject_action,
        )
    }
}

pub struct SchemaRow {
    rowid: i64,
    id: pgrx::Uuid,
    schema: pgrx::Json,
    created_at: pgrx::TimestampWithTimeZone,
}

impl SchemaRow {
    pub fn into_tuple(self) -> (i64, pgrx::Uuid, pgrx::Json, pgrx::TimestampWithTimeZone) {
        (self.rowid, self.id, self.schema, self.created_at)
    }

    pub fn get_schema(self) -> Result<Schema, PgFgaError> {
        Ok(serde_json::from_value(self.schema.0)?)
    }
}

impl<'a> Storage<'a> {
    pub fn new(client: SpiClient<'a>) -> Self {
        Self { client }
    }

    pub fn read_schema(&self, id: pgrx::Uuid) -> Result<Option<SchemaRow>, PgFgaError> {
        let tup_table = self.client.select(
            "SELECT * FROM fga.schema WHERE id = $1",
            Some(1),
            Some(vec![(PgBuiltInOids::UUIDOID.oid(), id.into_datum())]),
        )?;

        let mut results = Vec::new();
        for row in tup_table {
            let schema_row = SchemaRow {
                rowid: row["rowid"].value()?.expect("no rowid"),
                id: row["id"].value::<pgrx::Uuid>()?.expect("no id"),
                schema: row["schema"].value()?.expect("no schema"),
                created_at: row["created_at"].value()?.expect("no created_at"),
            };

            results.push(schema_row)
        }

        Ok(if !results.is_empty() {
            results.pop()
        } else {
            None
        })
    }

    pub fn read_tuple(
        &self,
        schema_id: pgrx::Uuid,
        resource_namespace: &str,
        resource_id: &str,
        relation: &str,
        subject_namespace: &str,
        subject_id: &str,
        subject_action: &str,
    ) -> Result<Option<TupleRow>, PgFgaError> {
        let query = "
        SELECT * FROM fga.tuple
        WHERE schema_id = $1
            AND resource_namespace = $2
            AND resource_id = $3
            AND relation = $4
            AND subject_namespace = $5
            AND subject_id = $6
            AND subject_action = $7
        ";

        let args = vec![
            (PgBuiltInOids::TEXTOID.oid(), schema_id.into_datum()),
            (
                PgBuiltInOids::TEXTOID.oid(),
                resource_namespace.into_datum(),
            ),
            (PgBuiltInOids::TEXTOID.oid(), resource_id.into_datum()),
            (PgBuiltInOids::TEXTOID.oid(), relation.into_datum()),
            (PgBuiltInOids::TEXTOID.oid(), subject_namespace.into_datum()),
            (PgBuiltInOids::TEXTOID.oid(), subject_id.into_datum()),
            (PgBuiltInOids::TEXTOID.oid(), subject_action.into_datum()),
        ];

        // TODO: do this with maps and such.
        let tup_table = self.client.select(query, Some(1), Some(args))?;

        let mut results = Vec::new();
        for row in tup_table {
            let tuple_row = TupleRow {
                rowid: row["rowid"].value()?.expect("no rowid"),
                schema_id: row["schema_id"]
                    .value::<pgrx::Uuid>()?
                    .expect("no schema_id"),
                resource_namespace: row["resource_namespace"]
                    .value::<String>()?
                    .expect("no resource_namespace"),
                resource_id: row["resource_id"]
                    .value::<String>()?
                    .expect("no resource_id"),
                relation: row["relation"].value::<String>()?.expect("no relation"),
                subject_namespace: row["subject_namespace"]
                    .value::<String>()?
                    .expect("no subject_namespace"),
                subject_id: row["subject_id"].value::<String>()?.expect("no subject_id"),
                subject_action: row["subject_action"]
                    .value::<String>()?
                    .expect("no subject_action"),
            };

            results.push(tuple_row)
        }

        Ok(if !results.is_empty() {
            results.pop()
        } else {
            None
        })
    }

    // TODO: the return type of this function should be an interator?
    pub fn read_tuples(
        &self,
        schema_id: pgrx::Uuid,
        resource_namespace: &str,
        resource_id: &str,
        relation: &str,
        subject_namespace: &str,
        subject_id: &str,
        subject_action: &str,
    ) -> Result<Vec<TupleRow>, PgFgaError> {
        let mut query = "SELECT * FROM fga.tuple WHERE schema_id = $1".to_string();
        let mut args = vec![(PgBuiltInOids::UUIDOID.oid(), schema_id.into_datum())];

        if !resource_namespace.is_empty() {
            args.push((
                PgBuiltInOids::TEXTOID.oid(),
                resource_namespace.into_datum(),
            ));
            query.push_str(&format!(" AND resource_namespace = ${}", args.len()));
        }

        if !resource_id.is_empty() {
            args.push((PgBuiltInOids::TEXTOID.oid(), resource_id.into_datum()));
            query.push_str(&format!(" AND resource_id = ${}", args.len()));
        }

        if !relation.is_empty() {
            args.push((PgBuiltInOids::TEXTOID.oid(), relation.into_datum()));
            query.push_str(&format!(" AND relation = ${}", args.len()));
        }

        if !subject_namespace.is_empty() {
            args.push((PgBuiltInOids::TEXTOID.oid(), subject_namespace.into_datum()));
            query.push_str(&format!(" AND subject_namespace = ${}", args.len()));
        }

        if !subject_id.is_empty() {
            args.push((PgBuiltInOids::TEXTOID.oid(), subject_id.into_datum()));
            query.push_str(&format!(" AND subject_id = ${}", args.len()));
        }

        if !subject_action.is_empty() {
            args.push((PgBuiltInOids::TEXTOID.oid(), subject_action.into_datum()));
            query.push_str(&format!(" AND subject_action = ${}", args.len()));
        }

        let tup_table = self.client.select(&query, None, Some(args))?;

        let mut results = Vec::with_capacity(tup_table.len());
        for row in tup_table {
            let tuple_row = TupleRow {
                rowid: row["rowid"].value()?.expect("no rowid"),
                schema_id: row["schema_id"]
                    .value::<pgrx::Uuid>()?
                    .expect("no schema_id"),
                resource_namespace: row["resource_namespace"]
                    .value::<String>()?
                    .expect("no resource_namespace"),
                resource_id: row["resource_id"]
                    .value::<String>()?
                    .expect("no resource_id"),
                relation: row["relation"].value::<String>()?.expect("no relation"),
                subject_namespace: row["subject_namespace"]
                    .value::<String>()?
                    .expect("no subject_namespace"),
                subject_id: row["subject_id"].value::<String>()?.expect("no subject_id"),
                subject_action: row["subject_action"]
                    .value::<String>()?
                    .expect("no subject_action"),
            };

            results.push(tuple_row)
        }

        Ok(results)
    }

    pub fn read_subjectset_tuples(
        &self,
        schema_id: pgrx::Uuid,
        resource_namespace: &str,
        resource_id: &str,
        relation: &str,
    ) -> Result<Vec<TupleRow>, PgFgaError> {
        let mut query = "SELECT * FROM fga.tuple WHERE schema_id = $1".to_string();
        let mut args = vec![(PgBuiltInOids::UUIDOID.oid(), schema_id.into_datum())];

        if !resource_namespace.is_empty() {
            args.push((
                PgBuiltInOids::TEXTOID.oid(),
                resource_namespace.into_datum(),
            ));
            query.push_str(&format!(" AND resource_namespace = ${}", args.len()));
        }

        if !resource_id.is_empty() {
            args.push((PgBuiltInOids::TEXTOID.oid(), resource_id.into_datum()));
            query.push_str(&format!(" AND resource_id = ${}", args.len()));
        }

        if !relation.is_empty() {
            args.push((PgBuiltInOids::TEXTOID.oid(), relation.into_datum()));
            query.push_str(&format!(" AND relation = ${}", args.len()));
        }

        let tup_table = self.client.select(&query, None, Some(args))?;

        let mut results = Vec::with_capacity(tup_table.len());
        for row in tup_table {
            let tuple_row = TupleRow {
                rowid: row["rowid"].value()?.expect("no rowid"),
                schema_id: row["schema_id"]
                    .value::<pgrx::Uuid>()?
                    .expect("no schema_id"),
                resource_namespace: row["resource_namespace"]
                    .value::<String>()?
                    .expect("no resource_namespace"),
                resource_id: row["resource_id"]
                    .value::<String>()?
                    .expect("no resource_id"),
                relation: row["relation"].value::<String>()?.expect("no relation"),
                subject_namespace: row["subject_namespace"]
                    .value::<String>()?
                    .expect("no subject_namespace"),
                subject_id: row["subject_id"].value::<String>()?.expect("no subject_id"),
                subject_action: row["subject_action"]
                    .value::<String>()?
                    .expect("no subject_action"),
            };

            results.push(tuple_row)
        }

        Ok(results)
    }
}
