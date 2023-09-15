use crate::schema::Schema;
use pgrx::spi;
use pgrx::spi::SpiClient;

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
    ) -> Result<bool, spi::Error> {
        self.check_cu(
            resource_namespace,
            resource_id,
            action,
            subject_namespace,
            subject_id,
            subject_action,
        )
    }

    fn check_cu(
        &self,
        resource_namespace: &str,
        resource_id: &str,
        action: &str,
        subject_namespace: &str,
        subject_id: &str,
        subject_action: &str,
    ) -> Result<bool, spi::Error> {
        Ok(false)
    }

    fn check_ttu(
        &self,
        resource_namespace: &str,
        resource_id: &str,
        action: &str,
        subject_namespace: &str,
        subject_id: &str,
        subject_action: &str,
    ) -> Result<bool, spi::Error> {
        Ok(false)
    }

    fn check_union(
        &self,
        resource_namespace: &str,
        resource_id: &str,
        action: &str,
        subject_namespace: &str,
        subject_id: &str,
        subject_action: &str,
    ) -> Result<bool, spi::Error> {
        Ok(false)
    }
}
