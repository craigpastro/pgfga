use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Schema {
    namespaces: Vec<Namespace>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Namespace {
    name: String,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    relations: Vec<Relation>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    permissions: Vec<Permission>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Relation {
    name: String,
    type_restrictions: Vec<TypeRestriction>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Permission {
    name: String,
    rewrite: Rewrite,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]

enum TypeRestriction {
    Namespace(String),

    NamespaceAction(String, String),
}

#[derive(Debug, Deserialize, Serialize)]
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
    fn it_works() {
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
        println!(">>> {}", serialized);
    }
}
