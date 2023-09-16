# Postgres Fine-Grained Authorization (pgfga)

## What is this?

This is an experimental Postgres extension for doing fine-grained authorization
(fga), written with [pgrx](https://github.com/pgcentralfoundation/pgrx).

This is a WIP. There is no documentation. There are no tests. There are no
validations. (I plan on adding some.) I don't know if it works. I am not very
good with Rust, nor Postgres.

## Usage

```bash
pgfga=# CREATE EXTENSION pgfga;

pgfga=# SELECT * FROM pgfga_create_schema('{"namespaces":{"document":{"relations":{"viewer":[{"namespace":"user"}]},"permissions":{"can_view":{"union":[{"computedUserset":"viewer"},{"tupleToUserset":["parent","can_view"]}]}}},"user":{"relations":{},"permissions":{}}}}');
         pgfga_create_schema          
--------------------------------------
 31c1cf4f-f1de-42fb-8e24-9f407805dadf


pgfga=# SELECT pgfga_create_tuple('31c1cf4f-f1de-42fb-8e24-9f407805dadf', 'document', '1', 'viewer', 'user', 'anya', '');
 pgfga_create_tuple 
--------------------

(1 row)

pgfga=# SELECT * FROM pgfga_check('31c1cf4f-f1de-42fb-8e24-9f407805dadf', 'document', '1', 'viewer', 'user', 'anya', '');
 pgfga_check 
-------------
 t
(1 row)
```

## Installation

Requires [pgrx](https://github.com/pgcentralfoundation/pgrx). If you have pgrx
installed then

```
cargo pgrx init
```

and

```
cargo pgrx run
```

will drop you into a psql prompt:

```
psql (15.3)
Type "help" for help.

pgfga=# CREATE EXTENSION pgfga;
CREATE EXTENSION
```

## Available functions

See [./src/lib.rs](./src/lib.rs) for type signatures.

- `pgfga_create_schema`
- `pgfga_read_schema`
- `pgfga_create_tuple`
- `pgfga_read_tuples`
- `pgfga_delete_tuple`
- `pgfga_check`

## TODOs

- Tests
- Documentation
- Clean up code
- Add the proper indices
- Client library to make this easier to use
- Add intersection and exclusion to the schema
- Read all schemas function
- Create many tuples function
- Delete many tuples function
- Function signatures are out of control. Do I need more structs?
- ?
