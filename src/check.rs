use crate::error::PgFgaError;
use crate::schema::{Rewrite, Schema};
use crate::storage::Storage;

pub const MAX_DEPTH: i64 = 25;

pub struct Checker<'a> {
    storage: Storage<'a>,
    schema_id: pgrx::Uuid,
    schema: Schema,
}

impl<'a> Checker<'a> {
    pub fn new(storage: Storage<'a>, schema_id: pgrx::Uuid) -> Result<Self, PgFgaError> {
        let schema = storage
            .read_schema(schema_id)?
            .expect("schema corresponding to the schema_id does not exist")
            .get_schema()?;

        Ok(Checker {
            storage,
            schema_id,
            schema,
        })
    }

    pub fn check(
        &self,
        resource_namespace: &str,
        resource_id: &str,
        action: &str,
        subject_namespace: &str,
        subject_id: &str,
        subject_action: &str,
    ) -> Result<bool, PgFgaError> {
        self.check_with_depth(
            resource_namespace,
            resource_id,
            action,
            subject_namespace,
            subject_id,
            subject_action,
            0,
        )
    }

    fn check_with_depth(
        &self,
        resource_namespace: &str,
        resource_id: &str,
        action: &str,
        subject_namespace: &str,
        subject_id: &str,
        subject_action: &str,
        depth: i64,
    ) -> Result<bool, PgFgaError> {
        if depth == MAX_DEPTH {
            return Err(PgFgaError::MaxDepth);
        }

        let is_relation = self
            .schema
            .namespaces
            .get(resource_namespace)
            .and_then(|ns| ns.relations.get(action))
            .is_some();

        if is_relation {
            // If the action is a relation we can attempt a direct check.
            if let Some(_) = self.storage.read_tuple(
                self.schema_id,
                resource_namespace,
                resource_id,
                action,
                subject_namespace,
                subject_id,
                subject_action,
            )? {
                // We found it.
                return Ok(true);
            }

            // Now let's take a look at subject sets.
            let tuples = self.storage.read_subjectset_tuples(
                self.schema_id,
                resource_namespace,
                resource_id,
                action,
            )?;

            for tuple in tuples {
                let result = self.check_with_depth(
                    &tuple.subject_namespace,
                    &tuple.subject_id,
                    &tuple.subject_action,
                    &tuple.subject_namespace,
                    subject_id,
                    subject_action,
                    depth + 1,
                )?;
                if result {
                    return Ok(true);
                }
            }
        }

        // Now let's take a look at permissions
        let rw = self
            .schema
            .namespaces
            .get(resource_namespace)
            .and_then(|ns| ns.permissions.get(action));

        if let Some(rewrite) = rw {
            return self.check_rewrite(
                resource_namespace,
                resource_id,
                subject_namespace,
                subject_id,
                subject_action,
                rewrite,
                depth,
            );
        }

        Ok(false)
    }

    fn check_rewrite(
        &self,
        _resource_namespace: &str,
        _resource_id: &str,
        _subject_namespace: &str,
        _subject_id: &str,
        _subject_action: &str,
        _rewrite: &Rewrite,
        _depth: i64,
    ) -> Result<bool, PgFgaError> {
        Ok(true)
    }

    fn check_cu(
        &self,
        _resource_namespace: &str,
        _resource_id: &str,
        _action: &str,
        _subject_namespace: &str,
        _subject_id: &str,
        _subject_action: &str,
        _depth: i64,
    ) -> Result<bool, PgFgaError> {
        Ok(false)
    }

    fn check_ttu(
        &self,
        _resource_namespace: &str,
        _resource_id: &str,
        _action: &str,
        _subject_namespace: &str,
        _subject_id: &str,
        _subject_action: &str,
        _depth: i64,
    ) -> Result<bool, PgFgaError> {
        Ok(false)
    }

    fn check_union(
        &self,
        _resource_namespace: &str,
        _resource_id: &str,
        _action: &str,
        _subject_namespace: &str,
        _subject_id: &str,
        _subject_action: &str,
        _depth: i64,
    ) -> Result<bool, PgFgaError> {
        Ok(false)
    }
}
