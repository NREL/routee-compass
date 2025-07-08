use crate::model::{
    label::{label_enum::Label, label_model_error::LabelModelError},
    network::VertexId,
    state::{StateModel, StateVariable},
};

/// Creates labels for vertices in a search algorithm that can be used to distinguish
/// between different states at the same vertex.
///
/// Labels are used in search algorithms to determine if two search states at the same
/// vertex are equivalent or if they represent different states that should be explored
/// separately. This is particularly important for algorithms that need to track
/// state-dependent information at vertices.
///
/// See the [`super::default`] module for implementations bundled with RouteE Compass:
///   - [VertexLabelModel]: creates simple vertex-based labels without state information
///
/// [VertexLabelModel]: super::default::vertex_label_model::VertexLabelModel
pub trait LabelModel: Send + Sync {
    /// Creates a label from the current search state at a given vertex.
    ///
    /// This method is called during search to generate a label that uniquely identifies
    /// the state at a vertex. The label is used to determine if two search states at
    /// the same vertex are equivalent and can be merged, or if they represent different
    /// states that should be explored separately.
    ///
    /// # Arguments
    ///
    /// * `vertex_id` - the vertex identifier for which to create a label
    /// * `state` - the current state vector at this vertex
    /// * `state_model` - provides access to the state vector components
    ///
    /// # Returns
    ///
    /// A [Label] that uniquely identifies this state at the vertex, or an error
    ///
    /// # Examples
    ///
    /// A simple vertex label model might return:
    /// ```ignore
    /// Ok(Label::Vertex(vertex_id))
    /// ```
    ///
    /// A state-dependent label model might return:
    /// ```ignore
    /// let battery_soc = state_model.get_energy(state, "trip_soc")?;
    /// Ok(Label::VertexWithIntState {
    ///     vertex_id,
    ///     state: vec![battery_soc.value() as u64],
    /// })
    /// ```
    fn label_from_state(
        &self,
        vertex_id: VertexId,
        state: &[StateVariable],
        state_model: &StateModel,
    ) -> Result<Label, LabelModelError>;
}
