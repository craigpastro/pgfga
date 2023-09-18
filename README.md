# Postgres Fine-Grained Authorization (pgfga)

## What is this?

This is an experimental Postgres extension for doing fine-grained authorization
(fga), written with [pgrx](https://github.com/pgcentralfoundation/pgrx).

FGA here means Relationship Based Access Control (ReBAC) based off the
[Zanzibar paper](https://zanzibar.tech/), and is similar to what
[OpenFGA](https://github.com/openfga/openfga),
[Permify](https://github.com/Permify/permify).
[SpiceDB](https://github.com/authzed/spicedb),
[Warrent](https://github.com/warrant-dev/warrant), and others have done.

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

### `pgfga.create_schema`

```sql
pgfga.create_schema(schema::json)::uuid
```

The JSON notation for the schema is based on the `Schema` struct found in
[./src/schema.rs](./src/schema.rs). It is most closely related to the schemas of
[SpiceDB](https://github.com/authzed/spicedb). In the future it would be nice to
write a DSL for schemas and a parser so that we don't have to specify the schema
using JSON. It would also be very nice if all the people who have wrote ReBAC
implementations could decide on a single DSL.

### `pgfga.read_tuples`

```sql
pgfga.read_tuples(
    schema_id::uuid,
    resource_namespace::varchar(128),
    resource_id::varchar(128),
    relation::varchar(128),
    subject_namespace::varchar(128),
    subject_id::varchar(128),
    subject_action::varchar(128) default '',
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
