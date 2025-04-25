use crate::errors::ServerError;
use rmcp::{
    schemars::schema::{Schema, SchemaObject, SingleOrVec},
    serde_json,
};
use std::collections::HashMap;

pub fn str_to_custom_scalar_map(
    string_custom_scalar_file: &str,
) -> Result<HashMap<String, SchemaObject>, ServerError> {
    // Parse the string into an initial map of serde_json::Values
    let parsed_custom_scalar_file: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(string_custom_scalar_file).map_err(ServerError::CustomScalarConfig)?;

    // Validate each of the values in the map and coerce into schemars::schema::SchemaObject
    let custom_scalar_map = parsed_custom_scalar_file
        .into_iter()
        .map(|(key, value)| {
            let value_string = value.to_string();
            // The only way I could find to do this was to reparse it.
            let schema: SchemaObject = serde_json::from_str(value_string.as_str())
                .map_err(ServerError::CustomScalarConfig)?;

            if has_invalid_schema(&Schema::Object(schema.clone())) {
                Err(ServerError::CustomScalarJsonSchema(value))
            } else {
                Ok((key, schema))
            }
        })
        .collect::<Result<_, ServerError>>()?;
    Ok::<_, ServerError>(custom_scalar_map)
}

// Unknown keys will be put into "extensions" in the schema object, check for those and consider those invalid
fn has_invalid_schema(schema: &Schema) -> bool {
    match schema {
        Schema::Object(schema_object) => {
            !schema_object.extensions.is_empty()
                || schema_object
                    .object
                    .as_ref()
                    .is_some_and(|object| object.properties.values().any(has_invalid_schema))
                || schema_object.array.as_ref().is_some_and(|object| {
                    object.items.as_ref().is_some_and(|items| match items {
                        SingleOrVec::Single(item) => has_invalid_schema(item),
                        SingleOrVec::Vec(items) => items.iter().any(has_invalid_schema),
                    })
                })
        }
        Schema::Bool(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};

    use rmcp::schemars::schema::{
        InstanceType, ObjectValidation, Schema, SchemaObject, SingleOrVec,
    };

    use crate::json_schema_helpers::str_to_custom_scalar_map;

    #[test]
    fn empty_file() {
        let result = str_to_custom_scalar_map("").err().unwrap();

        insta::assert_debug_snapshot!(result, @r###"
            CustomScalarConfig(
                Error("EOF while parsing a value", line: 1, column: 0),
            )
        "###)
    }

    #[test]
    fn only_spaces() {
        let result = str_to_custom_scalar_map("    ").err().unwrap();

        insta::assert_debug_snapshot!(result, @r###"
            CustomScalarConfig(
                Error("EOF while parsing a value", line: 1, column: 4),
            )
        "###)
    }

    #[test]
    fn invalid_json() {
        let result = str_to_custom_scalar_map("Hello: }").err().unwrap();

        insta::assert_debug_snapshot!(result, @r###"
            CustomScalarConfig(
                Error("expected value", line: 1, column: 1),
            )
        "###)
    }

    #[test]
    fn invalid_simple_schema() {
        let result = str_to_custom_scalar_map(
            r###"{
                "custom": {
                    "test": true
                }
            }"###,
        )
        .err()
        .unwrap();

        insta::assert_debug_snapshot!(result, @r###"
            CustomScalarJsonSchema(
                Object {
                    "test": Bool(true),
                },
            )
        "###)
    }

    #[test]
    fn invalid_complex_schema() {
        let result = str_to_custom_scalar_map(
            r###"{
                "custom": {
                    "type": "object",
                    "properties": {
                        "test": {
                            "test": true
                        }
                    }
                }
            }"###,
        )
        .err()
        .unwrap();

        insta::assert_debug_snapshot!(result, @r###"
            CustomScalarJsonSchema(
                Object {
                    "properties": Object {
                        "test": Object {
                            "test": Bool(true),
                        },
                    },
                    "type": String("object"),
                },
            )
        "###)
    }

    #[test]
    fn valid_schema() {
        let result = str_to_custom_scalar_map(
            r###"
        {
            "simple": {
                "type": "string"
            },
            "complex": {
                "type": "object",
                "properties": { "name": { "type": "string" } }
            }
        }
        "###,
        )
        .unwrap();

        let expected_data = HashMap::from_iter([
            (
                "simple".to_string(),
                SchemaObject {
                    instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::String))),
                    ..Default::default()
                },
            ),
            (
                "complex".to_string(),
                SchemaObject {
                    instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Object))),
                    object: Some(Box::new(ObjectValidation {
                        properties: BTreeMap::from_iter([(
                            "name".to_string(),
                            Schema::Object(SchemaObject {
                                instance_type: Some(SingleOrVec::Single(Box::new(
                                    InstanceType::String,
                                ))),
                                ..Default::default()
                            }),
                        )]),
                        ..Default::default()
                    })),
                    ..Default::default()
                },
            ),
        ]);
        assert_eq!(result, expected_data);
    }
}
