# pg_fga

An experimental Postgres extension for doing fine-grained access (fga) within
Postgres.

```
select * from pg_fga_create_schema('{"namespaces":{"document":{"relations":{"viewer":[{"namespace":"user"}]},"permissions":{"can_view":{"union":[{"computedUserset":"viewer"},{"tupleToUserset":["parent","can_view"]}]}}},"user":{"relations":{},"permissions":{}}}}');

select * from pg_fga_read_schema('cf64b948-440c-485b-9bd6-a7bd7435dea2');

select pg_fga_create_tuple('cf64b948-440c-485b-9bd6-a7bd7435dea2', 'foo', 'document', '1', 'viewer', 'user', 'anya');

select * from pg_fga_read_tuples('cf64b948-440c-485b-9bd6-a7bd7435dea2', '', '', '', '', '', '');

select pg_fga_delete_tuple('cf64b948-440c-485b-9bd6-a7bd7435dea2', 'foo', 'document', '1', 'viewer', 'user', 'anya');

select * from pg_fga_check('cf64b948-440c-485b-9bd6-a7bd7435dea2', '', '', '', '', '', '');
```

## TODOs

- read all schemas
- create many tuples
- delete many tuples
- check
