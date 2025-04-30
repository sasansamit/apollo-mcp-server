//! Tree shaking for GraphQL schema types

use crate::errors::ServerError;
use crate::operations::MutationMode;
use crate::sanitize::Sanitize;
use apollo_compiler::ast::{Definition, FragmentDefinition, OperationType, Selection};
use apollo_compiler::collections::IndexMap;
use apollo_compiler::schema::{ExtendedType, ObjectType};
use apollo_compiler::validation::Valid;
use apollo_compiler::{Name, Node, Schema};
use derive_new::new;

/// Tree shaker for GraphQL schema types
#[derive(new)]
pub struct TreeShaker<'schema> {
    schema: &'schema Schema,
    fragments: &'schema [&'schema Node<FragmentDefinition>],
    #[new(default)]
    types: IndexMap<Name, Vec<Selection>>,
}

impl<'schema> TreeShaker<'schema> {
    /// Retain the given type and anything reachable from it, filtering for the selection set.
    /// If the same type would be retained with multiple selection sets, a union of the selection
    /// sets is retained.
    pub fn retain(
        &mut self,
        type_name: Name,
        extended_type: &'schema ExtendedType,
        selection_set: &'schema [Selection],
    ) {
        let mut stack = vec![];
        self.types.insert(type_name.clone(), selection_set.to_vec());
        stack.push((type_name, extended_type.clone(), selection_set.to_vec()));
        while let Some((type_name, extended_type, selection_set)) = stack.pop() {
            self.inner_retain(type_name, extended_type, selection_set, &mut stack);
        }
    }

    fn inner_retain(
        &mut self,
        type_name: Name,
        extended_type: ExtendedType,
        selection_set: Vec<Selection>,
        stack: &mut Vec<(Name, ExtendedType, Vec<Selection>)>,
    ) {
        if let Some(existing) = self.types.get_mut(&type_name) {
            existing.extend(selection_set.clone());
        } else {
            self.types.insert(type_name, selection_set.clone());
        }
        match extended_type {
            ExtendedType::Object(ty) => {
                for (name, field) in ty.fields.iter() {
                    if let Some(sub_selection) = self.selected(name, &selection_set) {
                        let field_type_name = field.ty.inner_named_type().clone();
                        if let Some(field_type) =
                            self.schema.types.get(field.ty.inner_named_type().as_str())
                        {
                            stack.push((
                                field_type_name,
                                field_type.clone(),
                                sub_selection.clone(),
                            ));
                        }
                    }
                }
            }
            ExtendedType::Union(union) => {
                for member_type_name in union.members.iter() {
                    if let Some(member_type) = self.schema.types.get(member_type_name.name.as_str())
                    {
                        stack.push((
                            member_type_name.name.clone(),
                            member_type.clone(),
                            selection_set.to_vec(),
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    /// Determine if a field is selected by a selection set, and if so, return the sub-selection.
    fn selected(&self, field_name: &Name, selection: &[Selection]) -> Option<Vec<Selection>> {
        selection.iter().find_map(|selection| match selection {
            Selection::Field(field) => {
                if field.name.to_string() == field_name.to_string() {
                    Some(field.selection_set.clone())
                } else {
                    None
                }
            }
            Selection::InlineFragment(fragment) => {
                self.selected(field_name, &fragment.selection_set)
            }
            Selection::FragmentSpread(fragment) => self
                .fragments
                .iter()
                .find(|f| f.name.as_str() == fragment.fragment_name.as_str())
                .and_then(|f| self.selected(field_name, &f.selection_set)),
        })
    }

    /// Return the set of types retained after tree shaking.
    pub fn shaken(&self) -> impl Iterator<Item = ExtendedType> {
        self.types.iter().filter_map(|(name, selection_set)| {
            if let Some(extended_type) = self.schema.types.get(name.as_str()) {
                if extended_type.is_built_in() {
                    None
                } else {
                    match extended_type {
                        ExtendedType::Object(object_type) => {
                            let mut fields = IndexMap::default();
                            for (name, field) in object_type.fields.iter() {
                                if self.selected(name, selection_set).is_some() {
                                    fields.insert(
                                        name.clone(),
                                        field.as_ref().clone().sanitize().into(),
                                    );
                                }
                            }
                            Some(ExtendedType::from(ObjectType {
                                fields,
                                ..object_type.as_ref().clone().sanitize()
                            }))
                        }
                        ExtendedType::Scalar(ty) => {
                            Some(ExtendedType::from(ty.as_ref().clone().sanitize()))
                        }
                        ExtendedType::Enum(ty) => {
                            Some(ExtendedType::from(ty.as_ref().clone().sanitize()))
                        }
                        _ => None,
                    }
                }
            } else {
                None
            }
        })
    }
}

// TODO: this should be moved into the tree shaker, or a tree shaker variant.
pub fn remove_root_types(
    schema: &Valid<Schema>,
    mut document: apollo_compiler::ast::Document,
    mutation_mode: &MutationMode,
) -> Result<Valid<Schema>, ServerError> {
    let subscription_type_name = schema.root_operation(OperationType::Subscription);
    let mutation_type_name = schema.root_operation(OperationType::Mutation);

    let defs = document
        .definitions
        .into_iter()
        .filter_map(|def| match def.clone() {
            Definition::ObjectTypeDefinition(object_def) => {
                if subscription_type_name
                    .map(|subscription_type_name| object_def.name == *subscription_type_name)
                    .unwrap_or_default()
                    || mutation_type_name
                        .map(|mutation_type_name| {
                            object_def.name == *mutation_type_name
                                && *mutation_mode != MutationMode::All
                        })
                        .unwrap_or_default()
                {
                    return None;
                }
                Some(def)
            }
            Definition::ObjectTypeExtension(object_def) => {
                if let Some(subscription_type_name) = subscription_type_name {
                    if object_def.name.to_string() == subscription_type_name.to_string() {
                        return None;
                    }
                } else if let Some(mutation_type_name) = mutation_type_name {
                    if *mutation_mode != MutationMode::All
                        && object_def.name.to_string() == mutation_type_name.to_string()
                    {
                        return None;
                    }
                }
                Some(def)
            }
            Definition::SchemaDefinition(schema_definition) => {
                let modified_schema_definition = apollo_compiler::ast::SchemaDefinition {
                    description: schema_definition.description.clone(),
                    directives: schema_definition.directives.clone(),
                    root_operations: schema_definition
                        .root_operations
                        .clone()
                        .into_iter()
                        .filter(|node| {
                            node.0 != OperationType::Subscription
                                && (*mutation_mode == MutationMode::All
                                    || node.0 != OperationType::Mutation)
                        })
                        .collect(),
                };

                Some(Definition::SchemaDefinition(Node::new(
                    modified_schema_definition,
                )))
            }
            _ => Some(def),
        })
        .collect();

    document.definitions = defs;

    document
        .to_schema_validate()
        .map_err(|e| ServerError::GraphQLSchema(Box::new(e)))
}

#[cfg(test)]
mod test {
    use apollo_compiler::parser::Parser;

    use crate::{operations::MutationMode, tree_shake::remove_root_types};

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
        assert_eq!(
            remove_root_types(&schema, document, &MutationMode::None)
                .unwrap()
                .to_string(),
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
        assert_eq!(
            remove_root_types(&schema, document, &MutationMode::All)
                .unwrap()
                .to_string(),
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
        assert_eq!(
            remove_root_types(&schema, document, &MutationMode::None)
                .unwrap()
                .to_string(),
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
        assert_eq!(
            remove_root_types(&schema, document, &MutationMode::All)
                .unwrap()
                .to_string(),
            "schema {\n  query: CustomQuery\n  mutation: CustomMutation\n}\n\ntype CustomQuery {\n  id: String\n}\n\ntype CustomMutation {\n  id: String\n}\n"
        );
    }

    // TODO: it would be nice if we did remove the types only used in a removed root type.
    #[test]
    fn shouldnt_remove_other_types() {
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
        assert_eq!(
            remove_root_types(&schema, document, &MutationMode::None)
                .unwrap()
                .to_string(),
            "type Query {\n  id: UsedInQuery\n}\n\nscalar UsedInQuery\n\ntype UsedInMutation {\n  id: String\n}\n\nenum UsedInSubscription {\n  VALUE\n}\n"
        );
    }
}
