use apollo_compiler::{
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
pub fn operation_to_json_schema(uri: &str, source_text: &str) -> ToolDefinition {
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

    operation
        .variables
        .iter()
        .for_each(|variable| {
            let variable_name = variable.name.to_string();
            let schema = type_to_schema(variable.ty.as_ref());
            obj.properties.insert(variable_name.clone(), schema);
            if variable.ty.is_non_null() {
                obj.required.insert(variable_name);
            }
        });

        ToolDefinition {
            name: operation.name.clone().expect("Operations require names").to_string(),
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

fn type_to_schema(variable_type: &Type) -> Schema {
    match variable_type {
        Type::NonNullNamed(named) | Type::Named(named) => Schema::Object(SchemaObject {
            instance_type: Some(SingleOrVec::Single(match named.as_str() {
                "String" | "ID" => Box::new(InstanceType::String),
                "Int" | "Float" => Box::new(InstanceType::Number),
                "Boolean" => Box::new(InstanceType::Boolean),
                _ => panic!("Only build in scalars are currently supported"),
            })),
            ..Default::default()
        }),
        Type::NonNullList(list_type) | Type::List(list_type) => Schema::Object(SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Array))),
            subschemas: if list_type.is_non_null() {
                None
            } else {
                Some(Box::new(SubschemaValidation {
                    one_of: Some(vec![
                        type_to_schema(list_type),
                        Schema::Object(SchemaObject {
                            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Null))),
                            ..Default::default()
                        }),
                    ]),
                    ..Default::default()
                }))
            },
            array: if list_type.is_non_null() {
                Some(Box::new(ArrayValidation {
                    items: Some(SingleOrVec::Single(Box::new(type_to_schema(list_type)))),
                    ..Default::default()
                }))
            } else {
                None
            },
            ..Default::default()
        }),
    }
}

#[cfg(test)]
mod tests {
    use crate::operation_to_json_schema::{operation_to_json_schema, ToolDefinition};
    use rmcp::serde_json::json;

    #[test]
    fn no_variables() {
        let ToolDefinition { name: _name, description: _desciption, schema } = operation_to_json_schema("operation.graphql", "query { id }");
        assert_eq!(json!(schema), json!({"type": "object"}))
    }

    #[test]
    fn nullable_named_type() {
        let ToolDefinition { name: _name, description: _desciption, schema } = operation_to_json_schema("operation.graphql", "query($id: ID) { id }");
        assert_eq!(
            json!(schema),
            json!({
                "type": "object",
                "properties": { "id": {"type": "string"} }
            })
        )
    }

    #[test]
    fn non_nullable_named_type() {
        let ToolDefinition { name: _name, description: _desciption, schema } = operation_to_json_schema("operation.graphql", "query($id: ID!) { id }");
        assert_eq!(
            json!(schema),
            json!({
                "type": "object",
                "properties": { "id": {"type": "string"} },
                "required": ["id"]
            })
        )
    }

    #[test]
    fn non_nullable_list_of_nullable_named_type() {
        let ToolDefinition { name: _name, description: _desciption, schema } = operation_to_json_schema("operation.graphql", "query($id: [ID]!) { id }");
        assert_eq!(
            json!(schema),
            json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "array",
                        "oneOf": [{"type": "string"}, {"type": "null"}]
                    }
                },
                "required": ["id"]
            })
        )
    }

    #[test]
    fn non_nullable_list_of_non_nullable_named_type() {
        let ToolDefinition { name: _name, description: _desciption, schema } =
            operation_to_json_schema("operation.graphql", "query($id: [ID!]!) { id }");
        assert_eq!(
            json!(schema),
            json!({
                "type": "object",
                "properties": { "id": {"type": "array", "items": { "type": "string" }} },
                "required": ["id"]
            })
        )
    }

    #[test]
    fn nullable_list_of_nullable_named_type() {
        let ToolDefinition { name: _name, description: _desciption, schema } = operation_to_json_schema("operation.graphql", "query($id: [ID]) { id }");
        assert_eq!(
            json!(schema),
            json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "array",
                        "oneOf": [{"type": "string"}, {"type": "null"}]
                    }
                },
            })
        )
    }

    #[test]
    fn nullable_list_of_non_nullable_named_type() {
        let ToolDefinition { name: _name, description: _desciption, schema } = operation_to_json_schema("operation.graphql", "query($id: [ID!]) { id }");
        assert_eq!(
            json!(schema),
            json!({
                "type": "object",
                "properties": { "id": {"type": "array", "items": { "type": "string" }} },
            })
        )
    }

    #[test]
    fn nullable_list_of_nullable_lists_of_nullable_named_types() {
        let ToolDefinition { name: _name, description: _desciption, schema } =
            operation_to_json_schema("operation.graphql", "query($id: [[ID]]) { id }");
        assert_eq!(
            json!(schema),
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
            })
        )
    }

    #[test]
    #[should_panic]
    fn multiple_operations_should_panic() {
        operation_to_json_schema("operation.graphql", "query { id } query { id }");
    }

    #[test]
    #[should_panic]
    fn no_operations_should_panic() {
        operation_to_json_schema("operation.graphql", "fragment Test on Query { id }");
    }

    #[test]
    #[should_panic]
    fn custom_scalar_should_panic() {
        operation_to_json_schema("operation.graphql", "query($id: CustomId) { id }");
    }

    #[test]
    #[should_panic]
    fn schema_should_panic() {
        operation_to_json_schema("operation.graphql", "type Query { id: String }");
    }
}