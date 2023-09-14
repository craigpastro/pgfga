# pg_rebac

An experimental Postgres extension for doing ReBAC within Postgres.

```
select pg_rebac_create_schema();
select pg_rebac_create_tuple(schema_id, 'foo', 'document', '1', 'viewer', 'user', 'anya');
select pg_rebac_delete_tuple(schema_id, 'foo', 'document', '1', 'viewer', 'user', 'anya');
```
