# pg_rebac

An experimental Postgres extension for doing ReBAC within Postgres.

```
select * from pg_rebac_create_schema('{"namespaces":[{"name": "user", "relations":[], "permissions":[]},{ "name": "document", "relations":[{ "name": "viewer", "typeRestrictions":[{ "namespace": "user"}]}], "permissions":[{ "name": "can_view", "rewrite":{ "union":[{ "computedUserset": "viewer"},{"tupleToUserset":["parent","can_view"]}]}}]}]}');

select * from pg_rebac_read_schema('cf64b948-440c-485b-9bd6-a7bd7435dea2');

select pg_rebac_create_tuple('cf64b948-440c-485b-9bd6-a7bd7435dea2', 'foo', 'document', '1', 'viewer', 'user', 'anya');

select * from pg_rebac_read_tuples('cf64b948-440c-485b-9bd6-a7bd7435dea2', '', '', '', '', '', '');

select pg_rebac_delete_tuple('cf64b948-440c-485b-9bd6-a7bd7435dea2', 'foo', 'document', '1', 'viewer', 'user', 'anya');

select * from pg_rebac_check('cf64b948-440c-485b-9bd6-a7bd7435dea2', '', '', '', '', '', '');
```

## TODOs

- read all schemas
- create many tuples
- delete many tuples
- check
