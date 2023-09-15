use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Schema {
    namespaces: Vec<Namespace>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Namespace {
    name: String,
    relations: Vec<Relation>,
    permissions: Vec<Permission>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct Relation {
    name: String,
    type_restrictions: Vec<TypeRestriction>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Permission {
    name: String,
    rewrite: Rewrite,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]

enum TypeRestriction {
    Namespace(String),
    NamespaceAction(String, String),
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
enum Rewrite {
    ComputedUserset(String),
    TupleToUserset(String, String),
    Union(Vec<Rewrite>),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ser_then_der_works() {
        let schema = Schema {
            namespaces: vec![
                Namespace {
                    name: "user".to_string(),
                    relations: vec![],
                    permissions: vec![],
                },
                Namespace {
                    name: "document".to_string(),
                    relations: vec![Relation {
                        name: "viewer".to_string(),
                        type_restrictions: vec![TypeRestriction::Namespace("user".to_string())],
                    }],
                    permissions: vec![Permission {
                        name: "can_view".to_string(),
                        rewrite: Rewrite::Union(vec![
                            Rewrite::ComputedUserset("viewer".to_string()),
                            Rewrite::TupleToUserset("parent".to_string(), "can_view".to_string()),
                        ]),
                    }],
                },
            ],
        };

        let serialized = serde_json::to_string(&schema).unwrap();
        let deserialized: Schema = serde_json::from_str(&serialized).unwrap();

        assert_eq!(schema, deserialized);
    }
}
