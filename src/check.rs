use crate::error::PgFgaError;
use crate::schema::Schema;
use crate::storage::Storage;

pub const MAX_DEPTH: u64 = 25;

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
        depth: u64,
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

        // If the action is a relation we can attempt a direct check.
        if is_relation {
            if let Some(_) = self.storage.read_tuple(
                self.schema_id,
                resource_namespace,
                resource_id,
                action,
                subject_namespace,
                subject_id,
                subject_action,
            )? {
                return Ok(true);
            } else {
                return Ok(false);
            }
        }

        // self.schema.namespaces.get(resource_namespace).map(|ns| ns.)

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
        _depth: u64,
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
        _depth: u64,
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
        _depth: u64,
    ) -> Result<bool, PgFgaError> {
        Ok(false)
    }
}
