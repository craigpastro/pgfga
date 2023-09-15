use crate::error::PgFgaError;
use crate::schema::Schema;
use pgrx::spi::SpiClient;

pub const MAX_DEPTH: u64 = 25;

pub struct Checker<'a> {
    client: SpiClient<'a>,
    schema: Schema,
}

impl<'a> Checker<'a> {
    pub fn new(client: SpiClient<'_>, schema: Schema) -> Checker<'_> {
        Checker { client, schema }
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
        _resource_namespace: &str,
        _resource_id: &str,
        _action: &str,
        _subject_namespace: &str,
        _subject_id: &str,
        _subject_action: &str,
        depth: u64,
    ) -> Result<bool, PgFgaError> {
        if depth == MAX_DEPTH {
            return Err(PgFgaError::MaxDepth);
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
