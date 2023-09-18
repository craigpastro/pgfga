# Postgres Fine-Grained Authorization (pgfga)

## What is this?

This is an experimental Postgres extension for doing fine-grained authorization
(fga), written with [pgrx](https://github.com/pgcentralfoundation/pgrx).

This is a WIP. There is no documentation. There are no tests. There are no
validations. There are plans to add these things, and a bunch more. See the
"roadmap" below. Please help out if you are interested!

## Usage

```bash
pgfga=# CREATE EXTENSION pgfga;

pgfga=# SELECT * FROM pgfga.create_schema('{"namespaces":{"document":{"relations":{"viewer":[{"namespace":"user"}]},"permissions":{"can_view":{"union":[{"computedUserset":"viewer"},{"tupleToUserset":["parent","can_view"]}]}}},"user":{"relations":{},"permissions":{}}}}');
         create_schema          
--------------------------------------
 31c1cf4f-f1de-42fb-8e24-9f407805dadf


pgfga=# SELECT pgfga.create_tuple('31c1cf4f-f1de-42fb-8e24-9f407805dadf', 'document', '1', 'viewer', 'user', 'anya', '');
 create_tuple 
--------------
 
(1 row)

pgfga=# SELECT * FROM pgfga.check('31c1cf4f-f1de-42fb-8e24-9f407805dadf', 'document', '1', 'viewer', 'user', 'anya', '');
 check 
-------
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

- `pgfga.create_schema`
- `pgfga.read_schema`
- `pgfga.read_schemas`
- `pgfga.create_tuple`
- `pgfga.read_tuples`
- `pgfga.delete_tuple`
- `pgfga.check`

### pgfga.read_tuples

```sql
pgfga.read_tuples(
    schema_id::UUID,
    resource_namespace::VARCHAR(128),
    resource_id::VARCHAR(128),
    relation::VARCHAR(128),
    subject_namespace::VARCHAR(128),
    subject_id::VARCHAR(128),
    subject_action::VARCHAR(128) DEFAULT '',
)
```

`pgfga.read_tuples` acts a filter. Empty strings will match everything.

#### Examples

1. Read all tuples within a given `schema_id`:

   ```sql
   SELECT * FROM pgfga.read_tuples(schema_id, '', '', '', '', '');
   ```

## Roadmap

- Tests
- Documentation
- Clean up code
- Add the proper indices
- Client library to make this easier to use
- Add intersection and exclusion to the schema
- Writing tuples:
  - Only allow tuples to be written to particular schemas
  - Validate those tuples against the schema before persisting
- Create many tuples function
- Delete many tuples function
- Function signatures are out of control. Do I need more structs or type
  aliases?
- ?
