use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Schema {
    pub namespaces: HashMap<String, Namespace>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Namespace {
    #[serde(default)]
    pub relations: HashMap<String, Vec<TypeRestriction>>,

    #[serde(default)]
    pub permissions: HashMap<String, Rewrite>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TypeRestriction {
    Namespace(String),
    NamespaceAction(String, String),
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Rewrite {
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
            namespaces: HashMap::from([
                (
                    "user".to_string(),
                    Namespace {
                        relations: HashMap::new(),
                        permissions: HashMap::new(),
                    },
                ),
                (
                    "document".to_string(),
                    Namespace {
                        relations: HashMap::from([(
                            "viewer".to_string(),
                            vec![TypeRestriction::Namespace("user".to_string())],
                        )]),
                        permissions: HashMap::from([(
                            "can_view".to_string(),
                            Rewrite::Union(vec![
                                Rewrite::ComputedUserset("viewer".to_string()),
                                Rewrite::TupleToUserset(
                                    "parent".to_string(),
                                    "can_view".to_string(),
                                ),
                            ]),
                        )]),
                    },
                ),
            ]),
        };

        let serialized = serde_json::to_string(&schema).unwrap();
        let deserialized: Schema = serde_json::from_str(&serialized).unwrap();

        assert_eq!(schema, deserialized);
    }
}
