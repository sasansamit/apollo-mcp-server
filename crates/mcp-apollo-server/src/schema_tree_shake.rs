//! Tree shaking for GraphQL schema types

use crate::errors::ServerError;
use apollo_compiler::Schema;
use apollo_compiler::ast::{
    Definition, Document, OperationType, SchemaDefinition, SchemaExtension,
};
use apollo_compiler::validation::Valid;
use std::collections::HashMap;

/// Tree shaker for GraphQL schemas
pub struct SchemaTreeShaker<'document> {
    document: &'document Document,
    visited_named_types: HashMap<String, VisitedNode>,
    visted_directives: HashMap<String, VisitedNode>,
    operation_types: Vec<OperationType>,
    operation_type_names: HashMap<OperationType, String>,
}

#[derive(Clone, Default)]
struct VisitedNode {
    referenced_type_names: Vec<String>,
    referected_directive_names: Vec<String>,
    retain: bool,
}

fn visit(
    is_directive: bool,
    type_name: &str,
    visited_named_types: &mut HashMap<String, VisitedNode>,
    visited_directives: &mut HashMap<String, VisitedNode>,
) {
    if let Some((type_names, directive_names)) = if let Some(visited_node) = if is_directive {
        visited_directives.get_mut(type_name)
    } else {
        visited_named_types.get_mut(type_name)
    } {
        visited_node.retain = true;
        Some((
            visited_node.referenced_type_names.clone(),
            visited_node.referected_directive_names.clone(),
        ))
    } else {
        None
    } {
        type_names
            .iter()
            .for_each(|t| visit(false, t, visited_named_types, visited_directives));
        directive_names
            .iter()
            .for_each(|t| visit(true, t, visited_named_types, visited_directives));
    }
}

impl<'document> SchemaTreeShaker<'document> {
    pub fn new(document: &'document Document) -> Self {
        let mut schema_defs = Vec::default();
        let mut schema_exts = Vec::default();
        let mut visited_named_types: HashMap<String, VisitedNode> = HashMap::default();
        let mut visted_directives: HashMap<String, VisitedNode> = HashMap::default();

        document.definitions.iter().for_each(|def| match def {
            Definition::ObjectTypeDefinition(object_def) => {
                let visited_node = visited_named_types
                    .entry(object_def.name.to_string())
                    .or_default();

                object_def.fields.iter().for_each(|field| {
                    visited_node
                        .referenced_type_names
                        .push(field.ty.inner_named_type().to_string());
                });
                object_def.directives.iter().for_each(|directive| {
                    visited_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
                object_def
                    .implements_interfaces
                    .iter()
                    .for_each(|interface| {
                        visited_node
                            .referenced_type_names
                            .push(interface.to_string());
                    });
            }
            Definition::ObjectTypeExtension(object_def) => {
                let visited_node = visited_named_types
                    .entry(object_def.name.to_string())
                    .or_default();
                object_def.fields.iter().for_each(|field| {
                    visited_node
                        .referenced_type_names
                        .push(field.ty.inner_named_type().to_string());
                });
                object_def.directives.iter().for_each(|directive| {
                    visited_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
                object_def
                    .implements_interfaces
                    .iter()
                    .for_each(|interface| {
                        visited_node
                            .referenced_type_names
                            .push(interface.to_string());
                    });
            }
            Definition::DirectiveDefinition(directive_def) => {
                let visited_node = visted_directives
                    .entry(directive_def.name.to_string())
                    .or_default();
                directive_def.arguments.iter().for_each(|arg| {
                    visited_node
                        .referenced_type_names
                        .push(arg.ty.inner_named_type().to_string());
                });
            }
            Definition::InputObjectTypeDefinition(input_def) => {
                let visited_node = visited_named_types
                    .entry(input_def.name.to_string())
                    .or_default();
                input_def.fields.iter().for_each(|field| {
                    visited_node
                        .referenced_type_names
                        .push(field.ty.inner_named_type().to_string());
                });
                input_def.directives.iter().for_each(|directive| {
                    visited_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
            }
            Definition::InputObjectTypeExtension(input_def) => {
                let visited_node = visited_named_types
                    .entry(input_def.name.to_string())
                    .or_default();
                input_def.fields.iter().for_each(|field| {
                    visited_node
                        .referenced_type_names
                        .push(field.ty.inner_named_type().to_string());
                });
                input_def.directives.iter().for_each(|directive| {
                    visited_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
            }
            Definition::EnumTypeDefinition(enum_def) => {
                let visited_node = visited_named_types
                    .entry(enum_def.name.to_string())
                    .or_default();
                enum_def.directives.iter().for_each(|directive| {
                    visited_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
            }
            Definition::EnumTypeExtension(enum_def) => {
                let visited_node = visited_named_types
                    .entry(enum_def.name.to_string())
                    .or_default();
                enum_def.directives.iter().for_each(|directive| {
                    visited_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
            }
            Definition::ScalarTypeDefinition(scalar_def) => {
                let visited_node = visited_named_types
                    .entry(scalar_def.name.to_string())
                    .or_default();
                scalar_def.directives.iter().for_each(|directive| {
                    visited_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
            }
            Definition::ScalarTypeExtension(scalar_def) => {
                let visited_node = visited_named_types
                    .entry(scalar_def.name.to_string())
                    .or_default();
                scalar_def.directives.iter().for_each(|directive| {
                    visited_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
            }
            Definition::UnionTypeDefinition(union_def) => {
                let visited_node = visited_named_types
                    .entry(union_def.name.to_string())
                    .or_default();
                union_def.directives.iter().for_each(|directive| {
                    visited_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
                union_def.members.iter().for_each(|member| {
                    visited_node.referenced_type_names.push(member.to_string());
                });
            }
            Definition::UnionTypeExtension(union_def) => {
                let visited_node = visited_named_types
                    .entry(union_def.name.to_string())
                    .or_default();
                union_def.directives.iter().for_each(|directive| {
                    visited_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
                union_def.members.iter().for_each(|member| {
                    visited_node.referenced_type_names.push(member.to_string());
                });
            }
            Definition::InterfaceTypeDefinition(interface_def) => {
                let visited_node = visited_named_types
                    .entry(interface_def.name.to_string())
                    .or_default();
                interface_def.fields.iter().for_each(|field| {
                    visited_node
                        .referenced_type_names
                        .push(field.ty.inner_named_type().to_string());
                });
                interface_def.directives.iter().for_each(|directive| {
                    visited_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
                interface_def
                    .implements_interfaces
                    .iter()
                    .for_each(|interface| {
                        visited_node
                            .referenced_type_names
                            .push(interface.to_string());
                    });
            }
            Definition::InterfaceTypeExtension(interface_def) => {
                let visited_node = visited_named_types
                    .entry(interface_def.name.to_string())
                    .or_default();
                interface_def.fields.iter().for_each(|field| {
                    visited_node
                        .referenced_type_names
                        .push(field.ty.inner_named_type().to_string());
                });
                interface_def.directives.iter().for_each(|directive| {
                    visited_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
                interface_def
                    .implements_interfaces
                    .iter()
                    .for_each(|interface| {
                        visited_node
                            .referenced_type_names
                            .push(interface.to_string());
                    });
            }
            Definition::SchemaDefinition(schema_def) => schema_defs.push(schema_def),
            Definition::SchemaExtension(schema_ext) => schema_exts.push(schema_ext),
            Definition::OperationDefinition(_) => {} // Error?
            Definition::FragmentDefinition(_) => {}  // Error?
        });

        Self {
            document,
            visited_named_types,
            visted_directives,
            operation_types: Vec::default(),
            operation_type_names: schema_defs
                .iter()
                .flat_map(|def| def.root_operations.iter())
                .chain(
                    schema_exts
                        .iter()
                        .flat_map(|def| def.root_operations.iter()),
                )
                .map(|node| (node.0, node.1.to_string()))
                .collect(),
        }
    }

    pub fn retain_operation_type(&mut self, operation_type: OperationType) {
        self.operation_types.push(operation_type);
        let operation_type_name =
            self.operation_type_names
                .entry(operation_type)
                .or_insert(match operation_type {
                    OperationType::Query => "Query".to_string(),
                    OperationType::Mutation => "Mutation".to_string(),
                    OperationType::Subscription => "Subscription".to_string(),
                });

        visit(
            false,
            operation_type_name,
            &mut self.visited_named_types,
            &mut self.visted_directives,
        );
    }

    /// Return the set of types retained after tree shaking.
    pub fn shaken(&mut self) -> Result<Valid<Schema>, ServerError> {
        let filtered_definitions = self
            .document
            .definitions
            .clone()
            .into_iter()
            .filter_map(|def| match def.clone() {
                Definition::ObjectTypeDefinition(object_def) => self
                    .visited_named_types
                    .get(object_def.name.as_str())
                    .and_then(|visited| visited.retain.then_some(def)),
                Definition::ObjectTypeExtension(object_def) => self
                    .visited_named_types
                    .get(object_def.name.as_str())
                    .and_then(|visited| visited.retain.then_some(def)),
                Definition::DirectiveDefinition(directive_def) => self
                    .visted_directives
                    .get(directive_def.name.as_str())
                    .and_then(|visited| visited.retain.then_some(def)),
                Definition::InputObjectTypeDefinition(input_def) => self
                    .visited_named_types
                    .get(input_def.name.as_str())
                    .and_then(|visited| visited.retain.then_some(def)),
                Definition::InputObjectTypeExtension(input_def) => self
                    .visited_named_types
                    .get(input_def.name.as_str())
                    .and_then(|visited| visited.retain.then_some(def)),
                Definition::EnumTypeDefinition(enum_def) => self
                    .visited_named_types
                    .get(enum_def.name.as_str())
                    .and_then(|visited| visited.retain.then_some(def)),
                Definition::EnumTypeExtension(enum_def) => self
                    .visited_named_types
                    .get(enum_def.name.as_str())
                    .and_then(|visited| visited.retain.then_some(def)),
                Definition::ScalarTypeDefinition(scalar_def) => self
                    .visited_named_types
                    .get(scalar_def.name.as_str())
                    .and_then(|visited| visited.retain.then_some(def)),
                Definition::ScalarTypeExtension(scalar_def) => self
                    .visited_named_types
                    .get(scalar_def.name.as_str())
                    .and_then(|visited| visited.retain.then_some(def)),
                Definition::UnionTypeDefinition(union_def) => self
                    .visited_named_types
                    .get(union_def.name.as_str())
                    .and_then(|visited| visited.retain.then_some(def)),
                Definition::UnionTypeExtension(union_def) => self
                    .visited_named_types
                    .get(union_def.name.as_str())
                    .and_then(|visited| visited.retain.then_some(def)),
                Definition::InterfaceTypeDefinition(interface_def) => self
                    .visited_named_types
                    .get(interface_def.name.as_str())
                    .and_then(|visited| visited.retain.then_some(def)),
                Definition::InterfaceTypeExtension(interface_def) => self
                    .visited_named_types
                    .get(interface_def.name.as_str())
                    .and_then(|visited| visited.retain.then_some(def)),
                Definition::SchemaDefinition(schema_def) => {
                    let filtered_root_operations = schema_def
                        .root_operations
                        .clone()
                        .into_iter()
                        .filter(|root_operation| self.operation_types.contains(&root_operation.0))
                        .collect();

                    let new_schema_def = SchemaDefinition {
                        root_operations: filtered_root_operations,
                        description: schema_def.description.clone(),
                        directives: schema_def.directives.clone(),
                    };
                    Some(Definition::SchemaDefinition(apollo_compiler::Node::new(
                        new_schema_def,
                    )))
                }
                Definition::SchemaExtension(schema_ext) => {
                    let filtered_root_operations = schema_ext
                        .root_operations
                        .clone()
                        .into_iter()
                        .filter(|root_operation| self.operation_types.contains(&root_operation.0))
                        .collect();

                    let new_schema_def = SchemaExtension {
                        root_operations: filtered_root_operations,
                        directives: schema_ext.directives.clone(),
                    };
                    Some(Definition::SchemaExtension(apollo_compiler::Node::new(
                        new_schema_def,
                    )))
                }
                Definition::OperationDefinition(_) => None,
                Definition::FragmentDefinition(_) => None,
            })
            .collect();

        let mut document = Document::new();
        document.definitions = filtered_definitions;
        document
            .to_schema_validate()
            .map_err(|e| ServerError::GraphQLSchema(Box::new(e)))
    }
}

#[cfg(test)]
mod test {
    use apollo_compiler::{ast::OperationType, parser::Parser};

    use crate::schema_tree_shake::SchemaTreeShaker;

    #[test]
    fn should_remove_type_mutation_mode_none() {
        let source_text = r#"
            type Query { id: String }
            type Mutation { id: String }
            type Subscription { id: String }
        "#;
        let document = Parser::new()
            .parse_ast(source_text, "schema.graphql")
            .unwrap();
        let mut shaker = SchemaTreeShaker::new(&document);
        shaker.retain_operation_type(OperationType::Query);
        assert_eq!(
            shaker.shaken().unwrap().to_string(),
            "type Query {\n  id: String\n}\n"
        );
    }

    #[test]
    fn should_remove_type_mutation_mode_all() {
        let source_text = r#"
            type Query { id: String }
            type Mutation { id: String }
            type Subscription { id: String }
        "#;
        let document = Parser::new()
            .parse_ast(source_text, "schema.graphql")
            .unwrap();
        let mut shaker = SchemaTreeShaker::new(&document);
        shaker.retain_operation_type(OperationType::Query);
        shaker.retain_operation_type(OperationType::Mutation);
        assert_eq!(
            shaker.shaken().unwrap().to_string(),
            "type Query {\n  id: String\n}\n\ntype Mutation {\n  id: String\n}\n"
        );
    }

    #[test]
    fn should_remove_custom_names_mutation_mode_none() {
        let source_text = r#"
            schema {
              query: CustomQuery,
              mutation: CustomMutation,
              subscription: CustomSubscription
            }
            type CustomQuery { id: String }
            type CustomMutation { id: String }
            type CustomSubscription { id: String }
        "#;
        let document = Parser::new()
            .parse_ast(source_text, "schema.graphql")
            .unwrap();
        let mut shaker = SchemaTreeShaker::new(&document);
        shaker.retain_operation_type(OperationType::Query);
        assert_eq!(
            shaker.shaken().unwrap().to_string(),
            "schema {\n  query: CustomQuery\n}\n\ntype CustomQuery {\n  id: String\n}\n"
        );
    }

    #[test]
    fn should_remove_custom_names_mutation_mode_all() {
        let source_text = r#"
            schema {
              query: CustomQuery,
              mutation: CustomMutation,
              subscription: CustomSubscription
            }
            type CustomQuery { id: String }
            type CustomMutation { id: String }
            type CustomSubscription { id: String }
        "#;
        let document = Parser::new()
            .parse_ast(source_text, "schema.graphql")
            .unwrap();
        let mut shaker = SchemaTreeShaker::new(&document);
        shaker.retain_operation_type(OperationType::Query);
        shaker.retain_operation_type(OperationType::Mutation);
        assert_eq!(
            shaker.shaken().unwrap().to_string(),
            "schema {\n  query: CustomQuery\n  mutation: CustomMutation\n}\n\ntype CustomQuery {\n  id: String\n}\n\ntype CustomMutation {\n  id: String\n}\n"
        );
    }

    #[test]
    fn should_remove_orphan_types() {
        let source_text = r#"
            type Query { id: UsedInQuery }
            type Mutation { id: UsedInMutation }
            type Subscription { id: UsedInSubscription }
            scalar UsedInQuery
            type UsedInMutation { id: String }
            enum UsedInSubscription { VALUE }
        "#;
        let document = Parser::new()
            .parse_ast(source_text, "schema.graphql")
            .unwrap();
        let mut shaker = SchemaTreeShaker::new(&document);
        shaker.retain_operation_type(OperationType::Query);
        shaker.retain_operation_type(OperationType::Mutation);
        assert_eq!(
            shaker.shaken().unwrap().to_string(),
            "type Query {\n  id: UsedInQuery\n}\n\ntype Mutation {\n  id: UsedInMutation\n}\n\nscalar UsedInQuery\n\ntype UsedInMutation {\n  id: String\n}\n"
        );
    }
}
