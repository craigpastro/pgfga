# Postgres Fine-Grained Authorization (pgfga)

An experimental Postgres extension for doing fine-grained access (fga) within
Postgres.

```sql
select * from pgfga_create_schema('{"namespaces":{"document":{"relations":{"viewer":[{"namespace":"user"}]},"permissions":{"can_view":{"union":[{"computedUserset":"viewer"},{"tupleToUserset":["parent","can_view"]}]}}},"user":{"relations":{},"permissions":{}}}}');

select * from pgfga_read_schema('cf64b948-440c-485b-9bd6-a7bd7435dea2');

select pgfga_create_tuple('cf64b948-440c-485b-9bd6-a7bd7435dea2', 'foo', 'document', '1', 'viewer', 'user', 'anya');

select * from pgfga_read_tuples('cf64b948-440c-485b-9bd6-a7bd7435dea2', '', '', '', '', '', '');

select pgfga_delete_tuple('cf64b948-440c-485b-9bd6-a7bd7435dea2', 'foo', 'document', '1', 'viewer', 'user', 'anya');

select * from pgfga_check('cf64b948-440c-485b-9bd6-a7bd7435dea2', '', '', '', '', '', '');
```

## TODOs

- read all schemas
- create many tuples
- delete many tuples
- check
