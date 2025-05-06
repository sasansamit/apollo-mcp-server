//! Tree shaking for GraphQL schemas

use crate::errors::ServerError;
use apollo_compiler::ast::{
    Definition, DirectiveList, Document, Field, FieldDefinition, FragmentDefinition, NamedType,
    ObjectTypeDefinition, OperationDefinition, OperationType, SchemaDefinition, SchemaExtension,
    Selection, Type, UnionTypeDefinition,
};
use apollo_compiler::schema::ExtendedType;
use apollo_compiler::validation::Valid;
use apollo_compiler::{Name, Node, Schema};
use std::collections::HashMap;

struct RootOperationNames {
    query: String,
    mutation: String,
    subscription: String,
}

impl RootOperationNames {
    fn new(schema: &Schema) -> Self {
        Self {
            query: schema
                .root_operation(OperationType::Query)
                .map(|r| r.to_string())
                .unwrap_or("query".to_string()),
            mutation: schema
                .root_operation(OperationType::Mutation)
                .map(|r| r.to_string())
                .unwrap_or("mutation".to_string()),
            subscription: schema
                .root_operation(OperationType::Subscription)
                .map(|r| r.to_string())
                .unwrap_or("subscription".to_string()),
        }
    }
    fn name_for_operation_type(&self, operation_type: OperationType) -> &str {
        match operation_type {
            OperationType::Query => &self.query,
            OperationType::Mutation => &self.mutation,
            OperationType::Subscription => &self.subscription,
        }
    }
}
/// Tree shaker for GraphQL schemas
pub struct SchemaTreeShaker<'document> {
    schema: &'document Schema,
    document: &'document Document,
    named_type_nodes: HashMap<String, TreeNode>,
    directive_nodes: HashMap<String, TreeNode>,
    operation_types: Vec<OperationType>,
    operation_type_names: RootOperationNames,
    named_fragments: HashMap<String, Node<FragmentDefinition>>,
}

#[derive(Clone, Default)]
struct TreeNode {
    referenced_type_names: Vec<String>,
    referected_directive_names: Vec<String>,
    retain: bool,
    filtered_field: Option<Vec<String>>,
}

impl<'document> SchemaTreeShaker<'document> {
    pub fn new(document: &'document Document, schema: &'document Schema) -> Self {
        let mut named_type_nodes: HashMap<String, TreeNode> = HashMap::default();
        let mut directive_nodes: HashMap<String, TreeNode> = HashMap::default();

        document.definitions.iter().for_each(|def| match def {
            Definition::ObjectTypeDefinition(object_def) => {
                let tree_node = named_type_nodes
                    .entry(object_def.name.to_string())
                    .or_default();

                object_def.fields.iter().for_each(|field| {
                    tree_node
                        .referenced_type_names
                        .push(field.ty.inner_named_type().to_string());
                });
                object_def.directives.iter().for_each(|directive| {
                    tree_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
                object_def
                    .implements_interfaces
                    .iter()
                    .for_each(|interface| {
                        tree_node.referenced_type_names.push(interface.to_string());
                    });
            }
            Definition::ObjectTypeExtension(object_def) => {
                let tree_node = named_type_nodes
                    .entry(object_def.name.to_string())
                    .or_default();
                object_def.fields.iter().for_each(|field| {
                    tree_node
                        .referenced_type_names
                        .push(field.ty.inner_named_type().to_string());
                });
                object_def.directives.iter().for_each(|directive| {
                    tree_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
                object_def
                    .implements_interfaces
                    .iter()
                    .for_each(|interface| {
                        tree_node.referenced_type_names.push(interface.to_string());
                    });
            }
            Definition::DirectiveDefinition(directive_def) => {
                let tree_node = directive_nodes
                    .entry(directive_def.name.to_string())
                    .or_default();
                directive_def.arguments.iter().for_each(|arg| {
                    tree_node
                        .referenced_type_names
                        .push(arg.ty.inner_named_type().to_string());
                });
            }
            Definition::InputObjectTypeDefinition(input_def) => {
                let tree_node = named_type_nodes
                    .entry(input_def.name.to_string())
                    .or_default();
                input_def.fields.iter().for_each(|field| {
                    tree_node
                        .referenced_type_names
                        .push(field.ty.inner_named_type().to_string());
                });
                input_def.directives.iter().for_each(|directive| {
                    tree_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
            }
            Definition::InputObjectTypeExtension(input_def) => {
                let tree_node = named_type_nodes
                    .entry(input_def.name.to_string())
                    .or_default();
                input_def.fields.iter().for_each(|field| {
                    tree_node
                        .referenced_type_names
                        .push(field.ty.inner_named_type().to_string());
                });
                input_def.directives.iter().for_each(|directive| {
                    tree_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
            }
            Definition::EnumTypeDefinition(enum_def) => {
                let tree_node = named_type_nodes
                    .entry(enum_def.name.to_string())
                    .or_default();
                enum_def.directives.iter().for_each(|directive| {
                    tree_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
            }
            Definition::EnumTypeExtension(enum_def) => {
                let tree_node = named_type_nodes
                    .entry(enum_def.name.to_string())
                    .or_default();
                enum_def.directives.iter().for_each(|directive| {
                    tree_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
            }
            Definition::ScalarTypeDefinition(scalar_def) => {
                let tree_node = named_type_nodes
                    .entry(scalar_def.name.to_string())
                    .or_default();
                scalar_def.directives.iter().for_each(|directive| {
                    tree_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
            }
            Definition::ScalarTypeExtension(scalar_def) => {
                let tree_node = named_type_nodes
                    .entry(scalar_def.name.to_string())
                    .or_default();
                scalar_def.directives.iter().for_each(|directive| {
                    tree_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
            }
            Definition::UnionTypeDefinition(union_def) => {
                let tree_node = named_type_nodes
                    .entry(union_def.name.to_string())
                    .or_default();
                union_def.directives.iter().for_each(|directive| {
                    tree_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
                union_def.members.iter().for_each(|member| {
                    tree_node.referenced_type_names.push(member.to_string());
                });
            }
            Definition::UnionTypeExtension(union_def) => {
                let tree_node = named_type_nodes
                    .entry(union_def.name.to_string())
                    .or_default();
                union_def.directives.iter().for_each(|directive| {
                    tree_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
                union_def.members.iter().for_each(|member| {
                    tree_node.referenced_type_names.push(member.to_string());
                });
            }
            Definition::InterfaceTypeDefinition(interface_def) => {
                let tree_node = named_type_nodes
                    .entry(interface_def.name.to_string())
                    .or_default();
                interface_def.fields.iter().for_each(|field| {
                    tree_node
                        .referenced_type_names
                        .push(field.ty.inner_named_type().to_string());
                });
                interface_def.directives.iter().for_each(|directive| {
                    tree_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
                interface_def
                    .implements_interfaces
                    .iter()
                    .for_each(|interface| {
                        tree_node.referenced_type_names.push(interface.to_string());
                    });
            }
            Definition::InterfaceTypeExtension(interface_def) => {
                let tree_node = named_type_nodes
                    .entry(interface_def.name.to_string())
                    .or_default();
                interface_def.fields.iter().for_each(|field| {
                    tree_node
                        .referenced_type_names
                        .push(field.ty.inner_named_type().to_string());
                });
                interface_def.directives.iter().for_each(|directive| {
                    tree_node
                        .referected_directive_names
                        .push(directive.name.to_string())
                });
                interface_def
                    .implements_interfaces
                    .iter()
                    .for_each(|interface| {
                        tree_node.referenced_type_names.push(interface.to_string());
                    });
            }
            Definition::SchemaDefinition(_) => {}
            Definition::SchemaExtension(_) => {}
            Definition::OperationDefinition(_) => {} // Error?
            Definition::FragmentDefinition(_) => {}  // Error?
        });

        Self {
            document,
            schema,
            named_type_nodes,
            directive_nodes,
            operation_types: Vec::default(),
            named_fragments: HashMap::default(),
            operation_type_names: RootOperationNames::new(schema),
        }
    }

    pub fn retain_operation_type(
        &mut self,
        operation_type: OperationType,
        selection_set: Option<&Vec<Selection>>,
    ) {
        self.operation_types.push(operation_type);
        let operation_type_name = self
            .operation_type_names
            .name_for_operation_type(operation_type);

        if let Some(operation_type_extended_type) = self.schema.types.get(operation_type_name) {
            retain_type(
                operation_type_extended_type,
                selection_set,
                &mut self.named_type_nodes,
                &mut self.directive_nodes,
                &self.named_fragments,
                self.schema,
            );
        } else {
            tracing::error!("root operation type {} not found in schema", operation_type);
        }
    }

    pub fn retain_operation(&mut self, operation: &OperationDefinition, document: &Document) {
        self.named_fragments = document
            .definitions
            .iter()
            .filter_map(|def| match def {
                Definition::FragmentDefinition(fragment_def) => {
                    Some((fragment_def.name.to_string(), fragment_def.clone()))
                }
                _ => None,
            })
            .collect();
        self.retain_operation_type(operation.operation_type, Some(&operation.selection_set))
    }

    /// Return the set of types retained after tree shaking.
    pub fn shaken(&mut self) -> Result<Valid<Schema>, ServerError> {
        let mut document = Document::new();
        document.definitions = self
            .document
            .definitions
            .iter()
            .filter_map(|def| match def {
                Definition::DirectiveDefinition(directive_def) => self
                    .directive_nodes
                    .get(directive_def.name.as_str())
                    .and_then(|n| n.retain.then_some(def.clone())),
                Definition::SchemaDefinition(schema_def) => {
                    let filtered_root_operations = schema_def
                        .root_operations
                        .clone()
                        .into_iter()
                        .filter(|root_operation| {
                            root_operation.0 == OperationType::Query
                                || self.operation_types.contains(&root_operation.0)
                        })
                        .collect();

                    Some(Definition::SchemaDefinition(apollo_compiler::Node::new(
                        SchemaDefinition {
                            root_operations: filtered_root_operations,
                            description: schema_def.description.clone(),
                            directives: schema_def.directives.clone(),
                        },
                    )))
                }
                Definition::SchemaExtension(schema_ext) => {
                    let filtered_root_operations = schema_ext
                        .root_operations
                        .clone()
                        .into_iter()
                        .filter(|root_operation| {
                            root_operation.0 == OperationType::Query
                                || self.operation_types.contains(&root_operation.0)
                        })
                        .collect();

                    Some(Definition::SchemaExtension(apollo_compiler::Node::new(
                        SchemaExtension {
                            root_operations: filtered_root_operations,
                            directives: schema_ext.directives.clone(),
                        },
                    )))
                }
                Definition::OperationDefinition(_) => {
                    tracing::warn!("operation definition found in schema");
                    None
                }
                Definition::FragmentDefinition(_) => {
                    tracing::warn!("fragment definition found in schema");
                    None
                }
                // TODO: extensions, interfaces
                Definition::ObjectTypeDefinition(object_def) => {
                    self.named_type_nodes
                        .get(object_def.name.as_str())
                        .and_then(|tree_node| {
                            if let Some(fitlered_fields) = &tree_node.filtered_field {
                                tree_node.retain.then_some(Definition::ObjectTypeDefinition(
                                    Node::new(ObjectTypeDefinition {
                                        description: object_def.description.clone(),
                                        directives: object_def.directives.clone(),
                                        name: object_def.name.clone(),
                                        implements_interfaces: object_def
                                            .implements_interfaces
                                            .clone(),
                                        fields: object_def
                                            .fields
                                            .clone()
                                            .into_iter()
                                            .filter(|field| {
                                                fitlered_fields.contains(&field.name.to_string())
                                            })
                                            .collect(),
                                    }),
                                ))
                            } else if tree_node.retain {
                                Some(def.clone())
                            } else if let Some(root_op_name) =
                                self.schema.root_operation(OperationType::Query)
                            {
                                if *root_op_name == object_def.name {
                                    // All schemas need a query root operation to be valid, so we add a stub one here if it's not retained
                                    Some(Definition::ObjectTypeDefinition(Node::new(
                                        ObjectTypeDefinition {
                                            description: None,
                                            directives: DirectiveList::default(),
                                            fields: vec![Node::new(FieldDefinition {
                                                arguments: Vec::default(),
                                                description: None,
                                                directives: DirectiveList::default(),
                                                name: Name::new_unchecked("stub"),
                                                ty: Type::Named(NamedType::new_unchecked("String")),
                                            })],
                                            implements_interfaces: Vec::default(),
                                            name: object_def.name.clone(),
                                        },
                                    )))
                                } else {
                                    None
                                }
                            } else {
                                tracing::error!("object type {} not found", object_def.name);
                                None
                            }
                        })
                }
                Definition::UnionTypeDefinition(union_def) => self
                    .named_type_nodes
                    .get(union_def.name.as_str())
                    .is_some_and(|n| n.retain)
                    .then(|| {
                        Definition::UnionTypeDefinition(Node::new(UnionTypeDefinition {
                            description: union_def.description.clone(),
                            directives: union_def.directives.clone(),
                            name: union_def.name.clone(),
                            members: union_def
                                .members
                                .clone()
                                .into_iter()
                                .filter(|member| {
                                    if let Some(member_tree_node) =
                                        self.named_type_nodes.get(member.as_str())
                                    {
                                        member_tree_node.retain
                                    } else {
                                        tracing::error!("union member {} not found", member);
                                        false
                                    }
                                })
                                .collect(),
                        }))
                    }),
                _ => def
                    .name()
                    .map(|name| {
                        if let Some(tree_node) = self.named_type_nodes.get(name.as_str()) {
                            tree_node.retain
                        } else {
                            tracing::error!("node {} not found", name);
                            false
                        }
                    })
                    .and_then(|retained| retained.then_some(def.clone())),
            })
            .collect();

        document
            .to_schema_validate()
            .map_err(|e| ServerError::GraphQLSchema(Box::new(e)))
    }
}

fn selection_set_to_fields(
    selection_set: &Selection,
    named_fragments: &HashMap<String, Node<FragmentDefinition>>,
) -> Vec<Node<Field>> {
    match selection_set {
        Selection::Field(field) => vec![field.clone()],
        Selection::FragmentSpread(fragment) => named_fragments
            .get(fragment.fragment_name.as_str())
            .map(|f| {
                f.selection_set
                    .iter()
                    .flat_map(|s| selection_set_to_fields(s, named_fragments))
                    .collect()
            })
            .unwrap_or_default(),
        Selection::InlineFragment(fragment) => fragment
            .selection_set
            .iter()
            .flat_map(|s| selection_set_to_fields(s, named_fragments))
            .collect(),
    }
}

fn retain_type(
    extended_type: &ExtendedType,
    selection_set: Option<&Vec<Selection>>,
    named_type_nodes: &mut HashMap<String, TreeNode>,
    directive_nodes: &mut HashMap<String, TreeNode>,
    named_fragments: &HashMap<String, Node<FragmentDefinition>>,
    schema: &Schema,
) {
    let type_name = extended_type.name().as_str();
    if let Some((referenced_type_names, referected_directive_names, selected_fields)) =
        named_type_nodes.get_mut(type_name).map(|n| {
            n.retain = true;

            let selected_fields = if let Some(selection_set) = selection_set {
                let selected_fields = selection_set
                    .iter()
                    .flat_map(|s| selection_set_to_fields(s, named_fragments))
                    .collect::<Vec<_>>();

                let additional_fields = selected_fields
                    .iter()
                    .map(|f| f.name.to_string())
                    .collect::<Vec<_>>();

                n.filtered_field = Some(
                    [
                        n.filtered_field.clone().unwrap_or_default(),
                        additional_fields,
                    ]
                    .concat(),
                );
                Some(selected_fields)
            } else {
                None
            };

            (
                n.referenced_type_names.clone(),
                n.referected_directive_names.clone(),
                selected_fields,
            )
        })
    {
        if let Some(selected_fields) = selected_fields {
            selected_fields.iter().for_each(|field| {
                match extended_type {
                    ExtendedType::Union(union_def) => union_def.members.iter().for_each(|member| {
                        if let Some(member_type) = schema.types.get(member.as_str()) {
                            let memeber_selection_set = selection_set
                                .map(|selection_set| {
                                    selection_set
                                        .clone()
                                        .into_iter()
                                        .filter(|selection| match selection {
                                            Selection::Field(_) => true,
                                            Selection::FragmentSpread(fragment) => {
                                                if let Some(fragment_def) = named_fragments
                                                    .get(fragment.fragment_name.as_str())
                                                {
                                                    fragment_def.type_condition == member.as_str()
                                                } else {
                                                    tracing::error!(
                                                        "fragment {} not found",
                                                        fragment.fragment_name
                                                    );
                                                    false
                                                }
                                            }
                                            Selection::InlineFragment(fragment) => fragment
                                                .type_condition
                                                .clone()
                                                .is_none_or(|type_condition| {
                                                    type_condition.as_str() == member.as_str()
                                                }),
                                        })
                                        .collect::<Vec<Selection>>()
                                })
                                .and_then(|s| if s.is_empty() { None } else { Some(s) });

                            if selection_set.is_none() || memeber_selection_set.is_some() {
                                retain_type(
                                    member_type,
                                    memeber_selection_set.as_ref(),
                                    named_type_nodes,
                                    directive_nodes,
                                    named_fragments,
                                    schema,
                                );
                            }
                        } else {
                            tracing::error!("union member {} not found", member);
                        }
                    }),
                    _ => {
                        let field_type = match extended_type {
                            ExtendedType::Object(def) => Some(&def.fields),
                            ExtendedType::Interface(def) => Some(&def.fields),
                            _ => None,
                        }
                        .and_then(|type_def_fields| type_def_fields.get(field.name.as_str()));
                        if let Some(field_type) = field_type {
                            let field_type_name = field_type.ty.inner_named_type();
                            if let Some(field_type_def) = schema.types.get(field_type_name) {
                                retain_type(
                                    field_type_def,
                                    Some(&field.selection_set),
                                    named_type_nodes,
                                    directive_nodes,
                                    named_fragments,
                                    schema,
                                );
                            } else {
                                tracing::error!("field type {} not found", field_type_name);
                            }

                            field_type.arguments.iter().for_each(|arg| {
                                let arg_type_name = arg.ty.inner_named_type();
                                if let Some(arg_type) = schema.types.get(arg_type_name) {
                                    retain_type(
                                        arg_type,
                                        None,
                                        named_type_nodes,
                                        directive_nodes,
                                        named_fragments,
                                        schema,
                                    );
                                } else {
                                    tracing::error!(
                                        "field argument type {} not found",
                                        arg_type_name
                                    );
                                }
                            });
                        } else {
                            tracing::error!("field {} not found", field.name);
                        }
                    }
                }

                field.directives.iter().for_each(|directive| {
                    retain_directive(
                        directive.name.as_str(),
                        named_type_nodes,
                        directive_nodes,
                        named_fragments,
                        schema,
                    );
                })
            });
        } else {
            referenced_type_names.iter().for_each(|t| {
                if let Some(referenced_type_def) = schema.types.get(t.as_str()) {
                    retain_type(
                        referenced_type_def,
                        None,
                        named_type_nodes,
                        directive_nodes,
                        named_fragments,
                        schema,
                    )
                } else {
                    tracing::error!("referenced type {} not found", t);
                }
            });
            referected_directive_names.iter().for_each(|t| {
                retain_directive(
                    t,
                    named_type_nodes,
                    directive_nodes,
                    named_fragments,
                    schema,
                )
            });
        }
    }
}

fn retain_directive(
    directive_name: &str,
    named_type_nodes: &mut HashMap<String, TreeNode>,
    directive_nodes: &mut HashMap<String, TreeNode>,
    named_fragments: &HashMap<String, Node<FragmentDefinition>>,
    schema: &Schema,
) {
    // let type_name = type_def.name().as_str();
    if let Some(referenced_type_names) = directive_nodes.get_mut(directive_name).map(|n| {
        n.retain = true;
        n.referenced_type_names.clone()
    }) {
        referenced_type_names.iter().for_each(|t| {
            if let Some(arg_type) = schema.types.get(t.as_str()) {
                retain_type(
                    arg_type,
                    None,
                    named_type_nodes,
                    directive_nodes,
                    named_fragments,
                    schema,
                )
            } else {
                tracing::error!("referenced type {} not found", t);
            }
        });
    }
}

#[cfg(test)]
mod test {

    use apollo_compiler::{ast::OperationType, parser::Parser};

    use crate::{
        operations::{MutationMode, operation_defs},
        schema_tree_shake::SchemaTreeShaker,
    };

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
        let schema = document.to_schema_validate().unwrap();
        let mut shaker = SchemaTreeShaker::new(&document, &schema);
        shaker.retain_operation_type(OperationType::Query, None);
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
        let schema = document.to_schema_validate().unwrap();
        let mut shaker = SchemaTreeShaker::new(&document, &schema);
        shaker.retain_operation_type(OperationType::Query, None);
        shaker.retain_operation_type(OperationType::Mutation, None);
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
        let schema = document.to_schema_validate().unwrap();
        let mut shaker = SchemaTreeShaker::new(&document, &schema);
        shaker.retain_operation_type(OperationType::Query, None);
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
        let schema = document.to_schema_validate().unwrap();
        let mut shaker = SchemaTreeShaker::new(&document, &schema);
        shaker.retain_operation_type(OperationType::Query, None);
        shaker.retain_operation_type(OperationType::Mutation, None);
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
        let schema = document.to_schema_validate().unwrap();
        let mut shaker = SchemaTreeShaker::new(&document, &schema);
        shaker.retain_operation_type(OperationType::Query, None);
        shaker.retain_operation_type(OperationType::Mutation, None);
        assert_eq!(
            shaker.shaken().unwrap().to_string(),
            "type Query {\n  id: UsedInQuery\n}\n\ntype Mutation {\n  id: UsedInMutation\n}\n\nscalar UsedInQuery\n\ntype UsedInMutation {\n  id: String\n}\n"
        );
    }

    #[test]
    fn should_work_with_selection_set() {
        let source_text = r#"
            type Query { id: UsedInQuery unused: UsedInQueryButUnusedField }
            type Mutation { id: UsedInMutation }
            type Subscription { id: UsedInSubscription }
            scalar UsedInQuery
            type UsedInQueryButUnusedField { id: String, unused: String }
            type UsedInMutation { id: String }
            enum UsedInSubscription { VALUE }
        "#;
        let document = Parser::new()
            .parse_ast(source_text, "schema.graphql")
            .unwrap();
        let schema = document.to_schema_validate().unwrap();
        let mut shaker = SchemaTreeShaker::new(&document, &schema);
        let (operation_document, operation_def, _comments) =
            operation_defs("query TestQuery { id }", false, MutationMode::None).unwrap();
        shaker.retain_operation(&operation_def, &operation_document);
        assert_eq!(
            shaker.shaken().unwrap().to_string(),
            "type Query {\n  id: UsedInQuery\n}\n\nscalar UsedInQuery\n"
        );
    }
}
