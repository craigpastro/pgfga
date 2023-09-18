# Postgres Fine-Grained Authorization (pgfga)

## What is this?

This is an experimental Postgres extension for doing fine-grained authorization
(fga), written with [pgrx](https://github.com/pgcentralfoundation/pgrx).

FGA here means Relationship Based Access Control (ReBAC) based off the
[Zanzibar paper](https://zanzibar.tech/), and is similar to what
[Nungwi](https://github.com/craigpastro/nungwi),
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

## Extension functions

The `pgfga` extension is comprised of the following functions:

- `pgfga.create_schema`
- `pgfga.read_schema`
- `pgfga.read_schemas`
- `pgfga.create_tuple`
- `pgfga.read_tuples`
- `pgfga.delete_tuple`
- `pgfga.check`

See below for descriptions and examples.

### `pgfga.create_schema`

```sql
SELECT * FROM pgfga.create_schema(schema::json);
            create_schema             
--------------------------------------
 e8f8971e-12d7-40a6-b45c-b39d621fd34f
(1 row)
```

The JSON notation for the schema is based on the `Schema` struct found in
[./src/schema.rs](./src/schema.rs). It is most closely related to the schemas of
[SpiceDB](https://github.com/authzed/spicedb). In the future it would be nice to
write a DSL for schemas and a parser so that we don't have to specify the schema
using JSON. It would also be very nice if all the people who have wrote ReBAC
implementations could decide on a single DSL.

### `pgfga.read_schema`

```sql
SELECT * FROM pgfga.read_schema(id::uuid);
 rowid |                  id                  |       schema                       |          created_at           
-------+--------------------------------------+------------------------------------+-------------------------------
     1 | 35777d4d-3b66-47e5-907b-191b682b92c4 | {"namespaces":{"document":{...}}}" | 2023-09-17 09:40:00.897494-07
(1 row)
```

Read the schema corresponding to the given id.

### `pgfga.read_schemas`

```sql
SELECT * FROM pgfga.read_schemas();
 rowid |                  id                  |       schema                       |          created_at           
-------+--------------------------------------+------------------------------------+-------------------------------
     1 | 35777d4d-3b66-47e5-907b-191b682b92c4 | {"namespaces":{"document":{...}}}" | 2023-09-17 09:40:00.897494-07
     2 | 3fd56696-29c5-47d7-8d6d-5b95405b9169 | {"namespaces":{"folder":{...}}}"   | 2023-09-17 09:45:11.504819-07
(1 row)
```

Read all the schemas.

### `pgfga.create_tuple`

```sql
SELECT pgfga.create_tuple(
    schema_id::uuid,
    resource_namespace::varchar(128),
    resource_id::varchar(128),
    relation::varchar(128),
    subject_namespace::varchar(128),
    subject_id::varchar(128),
    subject_action::varchar(128) default '',
);
 create_tuple 
--------------
            1
(1 row)
```

Create a tuple. It returns the number of tuples created.

### `pgfga.read_tuples`

```sql
SELECT * FROM pgfga.read_tuples(
    schema_id::uuid,
    resource_namespace::varchar(128),
    resource_id::varchar(128),
    relation::varchar(128),
    subject_namespace::varchar(128),
    subject_id::varchar(128),
    subject_action::varchar(128) default '',
);
 rowid | schema_id | resource_namespace | resource_id | relation | subject_namespace | subject_id | subject_action 
-------+-----------+--------------------+-------------+----------+-------------------+------------+----------------
...
(n rows)
```

`pgfga.read_tuples` acts a filter. Empty strings will match everything. This
function will return all tuples that match the filter.

#### Examples

1. Read all tuples within a given `schema_id`:

   ```sql
   SELECT * FROM pgfga.read_tuples(schema_id, '', '', '', '', '');
   ```

### `pgfga.delete_tuple`

```sql
SELECT pgfga.delete_tuple(
    schema_id::uuid,
    resource_namespace::varchar(128),
    resource_id::varchar(128),
    relation::varchar(128),
    subject_namespace::varchar(128),
    subject_id::varchar(128),
    subject_action::varchar(128) default '',
);
 delete_tuple 
--------------
            1
(1 row)
```

Delete the given tuple. It returns the number of tuples deleted.

### `pgfga.check`

```sql
SELECT pgfga.check(
    schema_id::uuid,
    resource_namespace::varchar(128),
    resource_id::varchar(128),
    relation::varchar(128),
    subject_namespace::varchar(128),
    subject_id::varchar(128),
    subject_action::varchar(128) default ''
);
 check 
-------
 t
(1 row)
```

Check if the `subject` has the `relation` with the `resource`.

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
