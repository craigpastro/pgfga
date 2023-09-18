use crate::error::PgFgaError;
use crate::schema::Schema;
use pgrx::prelude::*;
use pgrx::spi::{SpiClient, SpiHeapTupleData};

pub struct Storage<'a> {
    client: SpiClient<'a>,
}

pub struct TupleRow {
    pub rowid: i64,
    pub schema_id: pgrx::Uuid,
    pub resource_namespace: String,
    pub resource_id: String,
    pub relation: String,
    pub subject_namespace: String,
    pub subject_id: String,
    pub subject_action: String,
}

impl From<TupleRow>
    for (
        i64,
        pgrx::Uuid,
        String,
        String,
        String,
        String,
        String,
        String,
    )
{
    fn from(row: TupleRow) -> Self {
        (
            row.rowid,
            row.schema_id,
            row.resource_namespace,
            row.resource_id,
            row.relation,
            row.subject_namespace,
            row.subject_id,
            row.subject_action,
        )
    }
}

impl<'a> TryFrom<SpiHeapTupleData<'a>> for TupleRow {
    type Error = spi::Error;

    fn try_from(row: SpiHeapTupleData<'a>) -> Result<Self, Self::Error> {
        Ok(TupleRow {
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
        })
    }
}

pub struct SchemaRow {
    pub rowid: i64,
    pub id: pgrx::Uuid,
    pub schema: pgrx::Json,
    pub created_at: pgrx::TimestampWithTimeZone,
}

impl From<SchemaRow> for (i64, pgrx::Uuid, pgrx::Json, pgrx::TimestampWithTimeZone) {
    fn from(row: SchemaRow) -> Self {
        (row.rowid, row.id, row.schema, row.created_at)
    }
}

impl TryFrom<SchemaRow> for Schema {
    type Error = PgFgaError;

    fn try_from(row: SchemaRow) -> Result<Self, Self::Error> {
        Ok(serde_json::from_value(row.schema.0)?)
    }
}

impl<'a> Storage<'a> {
    pub fn new(client: SpiClient<'a>) -> Self {
        Self { client }
    }

    pub fn create_schema(&mut self, schema: pgrx::Json) -> Result<Option<pgrx::Uuid>, PgFgaError> {
        let result = self
            .client
            .update(
                "INSERT INTO pgfga.schema (schema) VALUES ($1) RETURNING id",
                None,
                Some(vec![(PgBuiltInOids::JSONOID.oid(), schema.into_datum())]),
            )?
            .first()
            .get_one::<pgrx::Uuid>()?;

        Ok(result)
    }

    pub fn read_schemas(&self, id: Option<pgrx::Uuid>) -> Result<Vec<SchemaRow>, PgFgaError> {
        let mut query = "SELECT * FROM pgfga.schema".to_string();
        let mut args = vec![];

        if let Some(schema_id) = id {
            query.push_str(" WHERE id = $1");
            args.push((PgBuiltInOids::UUIDOID.oid(), schema_id.into_datum()));
        }

        let results = self
            .client
            .select(&query, None, Some(args))?
            .map(|row| {
                Ok(SchemaRow {
                    rowid: row["rowid"].value()?.expect("no rowid"),
                    id: row["id"].value::<pgrx::Uuid>()?.expect("no id"),
                    schema: row["schema"].value()?.expect("no schema"),
                    created_at: row["created_at"].value()?.expect("no created_at"),
                })
            })
            .collect::<Result<Vec<_>, spi::Error>>()?;

        Ok(results)
    }

    pub fn create_tuple(
        &mut self,
        schema_id: pgrx::Uuid,
        resource_namespace: &str,
        resource_id: &str,
        relation: &str,
        subject_namespace: &str,
        subject_id: &str,
        subject_action: &str,
    ) -> Result<(), PgFgaError> {
        let query = "
        INSERT INTO pgfga.tuple (
            schema_id,
            resource_namespace,
            resource_id,
            relation,
            subject_namespace,
            subject_id,
            subject_action
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT DO NOTHING
        RETURNING rowid
        ";

        let args = vec![
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
        ];

        self.client.update(query, Some(1), Some(args))?;

        Ok(())
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
        SELECT * FROM pgfga.tuple
        WHERE schema_id = $1
            AND resource_namespace = $2
            AND resource_id = $3
            AND relation = $4
            AND subject_namespace = $5
            AND subject_id = $6
            AND subject_action = $7
        ";

        let args = vec![
            (PgBuiltInOids::UUIDOID.oid(), schema_id.into_datum()),
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

        let mut results = self
            .client
            .select(query, None, Some(args))?
            .map(|row| TupleRow::try_from(row))
            .collect::<Result<Vec<_>, spi::Error>>()?;

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
        let mut query = "SELECT * FROM pgfga.tuple WHERE schema_id = $1".to_string();
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

        let results = self
            .client
            .select(&query, None, Some(args))?
            .map(|row| TupleRow::try_from(row))
            .collect::<Result<Vec<_>, spi::Error>>()?;

        Ok(results)
    }

    pub fn read_subjectset_tuples(
        &self,
        schema_id: pgrx::Uuid,
        resource_namespace: &str,
        resource_id: &str,
        relation: &str,
    ) -> Result<Vec<TupleRow>, PgFgaError> {
        let mut query = "SELECT * FROM pgfga.tuple WHERE schema_id = $1".to_string();
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

        let results = self
            .client
            .select(&query, None, Some(args))?
            .map(|row| TupleRow::try_from(row))
            .collect::<Result<Vec<_>, spi::Error>>()?;

        Ok(results)
    }

    pub fn delete_tuple(
        &mut self,
        schema_id: pgrx::Uuid,
        resource_namespace: &str,
        resource_id: &str,
        relation: &str,
        subject_namespace: &str,
        subject_id: &str,
        subject_action: &str,
    ) -> Result<(), PgFgaError> {
        let query = "
        DELETE FROM pgfga.tuple
        WHERE schema_id = $1
            AND resource_namespace = $2
            AND resource_id = $3
            AND relation = $4
            AND subject_namespace = $5
            AND subject_id = $6
            AND subject_action = $7
        ";

        let args = vec![
            (PgBuiltInOids::UUIDOID.oid(), schema_id.into_datum()),
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

        self.client.update(query, Some(1), Some(args))?;

        Ok(())
    }
}
