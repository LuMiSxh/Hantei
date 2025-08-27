use super::definition::FlowDefinition;
use crate::error::RecipeConversionError;

/// A trait for custom data models that can be converted into a Hantei `FlowDefinition`.
///
/// This is the primary extension point for making Hantei format-agnostic. By implementing
/// this trait on your own configuration structs, you provide a translation layer that
/// allows the Hantei compiler to process your custom recipe format.
///
/// # Example
///
/// ```rust,no_run
/// use hantei::prelude::*;
/// use hantei::error::RecipeConversionError;
///
/// // 1. Define your custom structs for parsing your format.
/// struct MyCustomNode { id: String, operation: String }
/// struct MyCustomRecipe { nodes: Vec<MyCustomNode> }
///
/// // 2. Implement `IntoFlow` for your top-level struct.
/// impl IntoFlow for MyCustomRecipe {
///     fn into_flow(self) -> Result<FlowDefinition, RecipeConversionError> {
///         let mut hantei_nodes = Vec::new();
///         for node in self.nodes {
///             // Your logic to convert `MyCustomNode` into `FlowNodeDefinition`
///             let hantei_node = FlowNodeDefinition {
///                 id: node.id,
///                 operation_type: node.operation, // Map the operation name
///                 // ... fill in other fields ...
/// #                input_type: None,
/// #                literal_values: None,
/// #                data_fields: None,
///             };
///             hantei_nodes.push(hantei_node);
///         }
///
///         Ok(FlowDefinition {
///             nodes: hantei_nodes,
///             edges: vec![], // Convert your edges here as well
///         })
///     }
/// }
/// ```
pub trait IntoFlow {
    /// Consumes the object and converts it into a Hantei-compatible logic flow.
    fn into_flow(self) -> Result<FlowDefinition, RecipeConversionError>;
}
