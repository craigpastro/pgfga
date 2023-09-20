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
            .read_schemas(Some(schema_id))?
            .pop()
            .ok_or_else(|| PgFgaError::UnknownSchemaId(schema_id))?
            .try_into()?;

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

        if self.is_relation(resource_namespace, action) {
            // If the action is a relation we can attempt a direct check.
            if self
                .storage
                .read_tuple(
                    self.schema_id,
                    resource_namespace,
                    resource_id,
                    action,
                    subject_namespace,
                    subject_id,
                    subject_action,
                )?
                .is_some()
            {
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
                );

                // We want to ignore errors here.
                if let Ok(true) = result {
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
        resource_namespace: &str,
        resource_id: &str,
        subject_namespace: &str,
        subject_id: &str,
        subject_action: &str,
        rewrite: &Rewrite,
        depth: i64,
    ) -> Result<bool, PgFgaError> {
        if depth == MAX_DEPTH {
            return Err(PgFgaError::MaxDepth);
        }

        match rewrite {
            Rewrite::ComputedUserset(computed_userset) => self.check_cu(
                resource_namespace,
                resource_id,
                subject_namespace,
                subject_id,
                subject_action,
                computed_userset,
                depth,
            ),
            Rewrite::TupleToUserset(tupleset, computed_userset) => self.check_ttu(
                resource_namespace,
                resource_id,
                subject_namespace,
                subject_id,
                subject_action,
                tupleset,
                computed_userset,
                depth,
            ),
            Rewrite::Union(rewrites) => self.check_union(
                resource_namespace,
                resource_id,
                subject_namespace,
                subject_id,
                subject_action,
                rewrites,
                depth,
            ),
            Rewrite::Intersection(rewrites) => self.check_intersection(
                resource_namespace,
                resource_id,
                subject_namespace,
                subject_id,
                subject_action,
                rewrites,
                depth,
            ),
            Rewrite::Exclusion(minuend, subtrahend) => self.check_exclusion(
                resource_namespace,
                resource_id,
                subject_namespace,
                subject_id,
                subject_action,
                minuend,
                subtrahend,
                depth,
            ),
        }
    }

    fn check_cu(
        &self,
        resource_namespace: &str,
        resource_id: &str,
        subject_namespace: &str,
        subject_id: &str,
        subject_action: &str,
        computed_userset: &str,
        depth: i64,
    ) -> Result<bool, PgFgaError> {
        self.check_with_depth(
            resource_namespace,
            resource_id,
            computed_userset,
            subject_namespace,
            subject_id,
            subject_action,
            depth,
        )
    }

    fn check_ttu(
        &self,
        resource_namespace: &str,
        resource_id: &str,
        subject_namespace: &str,
        subject_id: &str,
        subject_action: &str,
        tupleset: &str,
        computed_userset: &str,
        depth: i64,
    ) -> Result<bool, PgFgaError> {
        let tuples = self.storage.read_tuples(
            self.schema_id,
            resource_namespace,
            resource_id,
            tupleset,
            "",
            "",
            "",
        )?;

        for tuple in tuples {
            let new_resource_namespace = tuple.subject_namespace.as_str();
            let new_resource_id = tuple.subject_id.as_str();
            let new_resource_action = tuple.subject_action.as_str();

            // If computed_userset is not actually a relation or permission
            // in new_resource_namespace then there is nothing to do so
            // skip it.
            if !self.is_relation(new_resource_namespace, computed_userset)
                && !self.is_permission(new_resource_namespace, tupleset)
            {
                continue;
            }

            // If the subject has a nonempty relation (new_resource_action)
            // which does not match the computed_userset then skip it.
            if !new_resource_action.is_empty() && new_resource_action != computed_userset {
                continue;
            }

            let result = self.check_with_depth(
                new_resource_namespace,
                new_resource_id,
                computed_userset,
                subject_namespace,
                subject_id,
                subject_action,
                depth + 1,
            );

            if let Ok(true) = result {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn check_union(
        &self,
        resource_namespace: &str,
        resource_id: &str,
        subject_namespace: &str,
        subject_id: &str,
        subject_action: &str,
        rewrites: &Vec<Rewrite>,
        depth: i64,
    ) -> Result<bool, PgFgaError> {
        for rewrite in rewrites {
            let result = self.check_rewrite(
                resource_namespace,
                resource_id,
                subject_namespace,
                subject_id,
                subject_action,
                rewrite,
                depth + 1,
            );

            if let Ok(true) = result {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn check_intersection(
        &self,
        resource_namespace: &str,
        resource_id: &str,
        subject_namespace: &str,
        subject_id: &str,
        subject_action: &str,
        rewrites: &Vec<Rewrite>,
        depth: i64,
    ) -> Result<bool, PgFgaError> {
        for rewrite in rewrites {
            let result = self.check_rewrite(
                resource_namespace,
                resource_id,
                subject_namespace,
                subject_id,
                subject_action,
                rewrite,
                depth + 1,
            );

            if let Ok(false) = result {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn check_exclusion(
        &self,
        resource_namespace: &str,
        resource_id: &str,
        subject_namespace: &str,
        subject_id: &str,
        subject_action: &str,
        minuend: &Rewrite,
        subtrahend: &Rewrite,
        depth: i64,
    ) -> Result<bool, PgFgaError> {
        let minuend_result = self.check_rewrite(
            resource_namespace,
            resource_id,
            subject_namespace,
            subject_id,
            subject_action,
            minuend,
            depth + 1,
        );

        if let Ok(false) = minuend_result {
            return Ok(false);
        }

        let subtrahend_result = self.check_rewrite(
            resource_namespace,
            resource_id,
            subject_namespace,
            subject_id,
            subject_action,
            subtrahend,
            depth + 1,
        );

        if let Ok(true) = subtrahend_result {
            return Ok(false);
        }

        // minuend_result = true && subtrahend_result = false
        Ok(true)
    }

    fn is_relation(&self, namespace: &str, action: &str) -> bool {
        self.schema
            .namespaces
            .get(namespace)
            .and_then(|ns| ns.relations.get(action))
            .is_some()
    }

    fn is_permission(&self, namespace: &str, action: &str) -> bool {
        self.schema
            .namespaces
            .get(namespace)
            .and_then(|ns| ns.permissions.get(action))
            .is_some()
    }
}
