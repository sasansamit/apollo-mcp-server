use std::collections::HashMap;

use apollo_compiler::{
    Schema as GraphqlSchema,
    ast::{Definition, Type},
    parser::Parser,
};
use rmcp::schemars::schema::{
    ArrayValidation, InstanceType, ObjectValidation, RootSchema, Schema, SchemaObject, SingleOrVec,
    SubschemaValidation,
};

pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub schema: RootSchema,
}

/**
 * TODO
 * - support for input objects
 * - support for custom scalars
 * - error handling
 */
pub fn operation_to_json_schema(
    uri: &str,
    source_text: &str,
    graphql_schema: &GraphqlSchema,
    custom_scalar_map: Option<&HashMap<String, SchemaObject>>,
) -> ToolDefinition {
    let document = Parser::new()
        .parse_ast(source_text, uri)
        .expect("failed to parse operation");
    let operation_defs = document.definitions.iter().filter_map(|def| match def {
        Definition::OperationDefinition(operation_def) => Some(operation_def),
        Definition::FragmentDefinition(_) => None,
        _ => panic!("Schema definitions were passed in, only operations and fragments are allowed"),
    });

    let operation_count = operation_defs.clone().count();
    assert!(
        operation_count <= 1,
        "too many operations in document: {operation_count}"
    );

    let mut obj = ObjectValidation::default();

    let operation = operation_defs
        .clone()
        .nth(0)
        .expect("no operations in document");

    operation.variables.iter().for_each(|variable| {
        let variable_name = variable.name.to_string();
        let schema = type_to_schema(variable.ty.as_ref(), graphql_schema, custom_scalar_map);
        obj.properties.insert(variable_name.clone(), schema);
        if variable.ty.is_non_null() {
            obj.required.insert(variable_name);
        }
    });

    ToolDefinition {
        name: operation
            .name
            .clone()
            .expect("Operations require names")
            .to_string(),
        description: "".to_string(),
        schema: RootSchema {
            schema: SchemaObject {
                instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Object))),
                object: Some(Box::new(obj)),
                ..Default::default()
            },
            ..Default::default()
        },
    }
}

fn schema_factory(
    instance_type: InstanceType,
    object_validation: Option<ObjectValidation>,
    array_validation: Option<ArrayValidation>,
    subschema_validation: Option<SubschemaValidation>,
) -> Schema {
    Schema::Object(SchemaObject {
        instance_type: Some(SingleOrVec::Single(Box::new(instance_type))),
        object: object_validation.map(|validation| Box::new(validation)),
        array: array_validation.map(|validation| Box::new(validation)),
        subschemas: subschema_validation.map(|validation| Box::new(validation)),
        ..Default::default()
    })
}
fn type_to_schema(
    variable_type: &Type,
    graphql_schema: &GraphqlSchema,
    custom_scalar_map: Option<&HashMap<String, SchemaObject>>,
) -> Schema {
    match variable_type {
        Type::NonNullNamed(named) | Type::Named(named) => match named.as_str() {
            "String" | "ID" => schema_factory(InstanceType::String, None, None, None),
            "Int" | "Float" => schema_factory(InstanceType::Number, None, None, None),
            "Boolean" => schema_factory(InstanceType::Boolean, None, None, None),
            _ => {
                if let Some(input_type) = graphql_schema.get_input_object(named) {
                    let mut obj = ObjectValidation::default();

                    input_type.fields.iter().for_each(|(name, field)| {
                        obj.properties.insert(
                            name.to_string(),
                            type_to_schema(field.ty.as_ref(), graphql_schema, custom_scalar_map),
                        );

                        if field.is_required() {
                            obj.required.insert(name.to_string());
                        }
                    });

                    schema_factory(InstanceType::Object, Some(obj), None, None)
                } else if graphql_schema.get_scalar(named).is_some() {
                    if let Some(custom_scalar_map) = custom_scalar_map {
                        if let Some(custom_scalar_schema_object) =
                            custom_scalar_map.get(named.as_str())
                        {
                            Schema::Object(custom_scalar_schema_object.clone())
                        } else {
                            panic!("custom scalar missing from custom_scalar_map")
                        }
                    } else {
                        panic!(
                            "custom scalars aren't currently supported without a custom_scalar_map"
                        )
                    }
                } else {
                    // TODO: Should this be an "any" type or an error?
                    panic!("Type not found in schema! {named}")
                }
            }
        },
        Type::NonNullList(list_type) | Type::List(list_type) => {
            let inner_type_schema = type_to_schema(list_type, graphql_schema, custom_scalar_map);
            schema_factory(
                InstanceType::Array,
                None,
                list_type.is_non_null().then(|| ArrayValidation {
                    items: Some(SingleOrVec::Single(Box::new(inner_type_schema.clone()))),
                    ..Default::default()
                }),
                (!list_type.is_non_null()).then(|| SubschemaValidation {
                    one_of: Some(vec![
                        inner_type_schema,
                        Schema::Object(SchemaObject {
                            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Null))),
                            ..Default::default()
                        }),
                    ]),
                    ..Default::default()
                }),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::operation_to_json_schema::{ToolDefinition, operation_to_json_schema};
    use apollo_compiler::parser::Parser;
    use rmcp::{
        schemars::schema::{InstanceType, SchemaObject, SingleOrVec},
        serde_json::{self, json},
    };

    fn expect_json_schema(
        source_text: &str,
        expected_json: serde_json::Value,
        custom_scalar_map: Option<&HashMap<String, SchemaObject>>,
    ) {
        let mut parser = Parser::new();
        let document = parser
            .parse_ast(
                "
                    type Query { id: String }
                    scalar RealCustomScalar
                    input RealInputObject {
                        optional: String
                        required: String!
                    }
                ",
                "operation.graphql",
            )
            .expect("failed to parse operation");
        let grpahql_schema = document.to_schema().unwrap();

        let ToolDefinition {
            name: _name,
            description: _desciption,
            schema,
        } = operation_to_json_schema(
            "operation.graphql",
            source_text,
            &grpahql_schema,
            custom_scalar_map,
        );
        assert_eq!(json!(schema), expected_json)
    }

    #[test]
    fn no_variables() {
        expect_json_schema("query QueryName { id }", json!({"type": "object"}), None)
    }

    #[test]
    fn nullable_named_type() {
        expect_json_schema(
            "query QueryName($id: ID) { id }",
            json!({
                "type": "object",
                "properties": { "id": {"type": "string"} }
            }),
            None,
        )
    }

    #[test]
    fn non_nullable_named_type() {
        expect_json_schema(
            "query QueryName($id: ID!) { id }",
            json!({
                "type": "object",
                "properties": { "id": {"type": "string"} },
                "required": ["id"]
            }),
            None,
        )
    }

    #[test]
    fn non_nullable_list_of_nullable_named_type() {
        expect_json_schema(
            "query QueryName($id: [ID]!) { id }",
            json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "array",
                        "oneOf": [{"type": "string"}, {"type": "null"}]
                    }
                },
                "required": ["id"]
            }),
            None,
        )
    }

    #[test]
    fn non_nullable_list_of_non_nullable_named_type() {
        expect_json_schema(
            "query QueryName($id: [ID!]!) { id }",
            json!({
                "type": "object",
                "properties": { "id": {"type": "array", "items": { "type": "string" }} },
                "required": ["id"]
            }),
            None,
        )
    }

    #[test]
    fn nullable_list_of_nullable_named_type() {
        expect_json_schema(
            "query QueryName($id: [ID]) { id }",
            json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "array",
                        "oneOf": [{"type": "string"}, {"type": "null"}]
                    }
                },
            }),
            None,
        )
    }

    #[test]
    fn nullable_list_of_non_nullable_named_type() {
        expect_json_schema(
            "query QueryName($id: [ID!]) { id }",
            json!({
                "type": "object",
                "properties": { "id": {"type": "array", "items": { "type": "string" }} },
            }),
            None,
        )
    }

    #[test]
    fn nullable_list_of_nullable_lists_of_nullable_named_types() {
        expect_json_schema(
            "query QueryName($id: [[ID]]) { id }",
            json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "array",
                        "oneOf": [
                            {"type": "array", "oneOf": [{"type": "string"}, {"type": "null"}]},
                            {"type": "null"}
                        ]
                    }
                },
            }),
            None,
        )
    }

    #[test]
    fn nullable_input_object() {
        expect_json_schema(
            "query QueryName($id: RealInputObject) { id }",
            json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "object",
                        "properties": {
                            "optional": { "type": "string" },
                            "required": { "type": "string" }
                        },
                        "required": ["required"]
                    }
                },
            }),
            None,
        )
    }

    #[test]
    #[should_panic(expected = "too many operations in document: 2")]
    fn multiple_operations_should_panic() {
        expect_json_schema(
            "query QueryName { id } query QueryName { id }",
            json!({}),
            None,
        )
    }

    #[test]
    #[should_panic(expected = "Operations require names")]
    fn unnamed_operations_should_panic() {
        expect_json_schema("query { id }", json!({}), None)
    }

    #[test]
    #[should_panic(expected = "no operations in document")]
    fn no_operations_should_panic() {
        expect_json_schema("fragment Test on Query { id }", json!({}), None)
    }

    #[test]
    #[should_panic(
        expected = "Schema definitions were passed in, only operations and fragments are allowed"
    )]
    fn schema_should_panic() {
        expect_json_schema("type Query { id: String }", json!({}), None)
    }

    #[test]
    #[should_panic(expected = "Type not found in schema! FakeType")]
    fn unknown_type_should_panic() {
        expect_json_schema("query QueryName($id: FakeType) { id }", json!({}), None)
    }

    #[test]
    #[should_panic(expected = "custom scalars aren't currently supported")]
    fn custom_scalar_without_map_should_panic() {
        expect_json_schema(
            "query QueryName($id: RealCustomScalar) { id }",
            json!({}),
            None,
        )
    }

    #[test]
    #[should_panic(expected = "custom scalar missing from custom_scalar_map")]
    fn custom_scalar_with_map_but_not_found_should_panic() {
        expect_json_schema(
            "query QueryName($id: RealCustomScalar) { id }",
            json!({}),
            Some(&HashMap::new()),
        )
    }

    #[test]
    fn custom_scalar_with_map() {
        let mut custom_scalar_map = HashMap::new();
        custom_scalar_map.insert(
            "RealCustomScalar".to_string(),
            SchemaObject {
                instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::String))),
                ..Default::default()
            },
        );
        expect_json_schema(
            "query QueryName($id: RealCustomScalar) { id }",
            json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string"
                    }
                },
            }),
            Some(&custom_scalar_map),
        )
    }
}
