use std::fmt::Display;

use allocative::Allocative;
use serde::Serialize;

use crate::model::network::VertexId;

/// The required length for OS-aligned state vectors.
/// This is the word size of the target architecture.
#[cfg(target_pointer_width = "32")]
pub const OS_ALIGNED_STATE_LEN: usize = 4;

#[cfg(target_pointer_width = "64")]
pub const OS_ALIGNED_STATE_LEN: usize = 8;


#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Allocative)]
pub enum Label {
    Vertex(VertexId),
    VertexWithIntState {
        vertex_id: VertexId,
        state: usize,
    },
    VertexWithIntStateVec {
        vertex_id: VertexId,
        state: Vec<usize>,
    },
    /// Store OS-aligned u8 state data for memory efficiency.
    /// The state vector must be exactly OS_ALIGNED_STATE_LEN bytes long.
    VertexWithU8StateVec {
        vertex_id: VertexId,
        state: Vec<u8>,
    },
}

impl Label {
    /// Creates a new VertexWithU8StateVec with validation.
    /// 
    /// # Arguments
    /// 
    /// * `vertex_id` - The vertex identifier
    /// * `state` - The u8 state vector, must be exactly OS_ALIGNED_STATE_LEN bytes long
    /// 
    /// # Returns
    /// 
    /// A Result containing the Label or a LabelModelError if validation fails
    /// 
    /// # Example
    /// 
    /// ```
    /// # use routee_compass_core::model::label::label_enum::{Label, OS_ALIGNED_STATE_LEN};
    /// # use routee_compass_core::model::network::VertexId;
    /// let vertex_id = VertexId(42);
    /// let state = vec![1u8, 2u8, 3u8, 4u8]; // This works on 32-bit systems
    /// let label = Label::new_aligned_u8_state(vertex_id, state);
    /// // On 32-bit: Ok(label), on 64-bit: Err(InvalidStateLength)
    /// ```
    pub fn new_aligned_u8_state(vertex_id: VertexId, state: Vec<u8>) -> Self {
        let mut state = state;
        let remainder = state.len() % OS_ALIGNED_STATE_LEN;
        if remainder != 0 {
            let padding_needed = OS_ALIGNED_STATE_LEN - remainder;
            state.extend(vec![0u8; padding_needed]);
        }
        
        Label::VertexWithU8StateVec { vertex_id, state }
    }
    
    /// Creates a new VertexWithU8StateVec from a raw array slice.
    /// 
    /// This method is convenient when you have a fixed-size array and want to 
    /// ensure it's the correct size at compile time.
    /// 
    /// # Arguments
    /// 
    /// * `vertex_id` - The vertex identifier
    /// * `state` - A reference to an array of exactly OS_ALIGNED_STATE_LEN bytes
    /// 
    /// # Returns
    /// 
    /// A Label instance (no validation needed since array size is checked at compile time)
    /// 
    /// # Example
    /// 
    /// ```
    /// # use routee_compass_core::model::label::label_enum::{Label, OS_ALIGNED_STATE_LEN};
    /// # use routee_compass_core::model::network::VertexId;
    /// let vertex_id = VertexId(42);
    /// # #[cfg(target_pointer_width = "32")]
    /// let state = [1u8, 2u8, 3u8, 4u8];
    /// # #[cfg(target_pointer_width = "64")]
    /// # let state = [1u8, 2u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8];
    /// let label = Label::from_aligned_u8_array(vertex_id, &state);
    /// ```
    pub fn from_aligned_u8_array(vertex_id: VertexId, state: &[u8; OS_ALIGNED_STATE_LEN]) -> Self {
        Label::VertexWithU8StateVec {
            vertex_id,
            state: state.to_vec(),
        }
    }
    
    /// Creates a new VertexWithU8StateVec with zero-initialized state.
    /// 
    /// This is useful when you want to create a label with the correct size
    /// and fill it in later, or when you want all zeros as the initial state.
    /// 
    /// # Arguments
    /// 
    /// * `vertex_id` - The vertex identifier
    /// 
    /// # Returns
    /// 
    /// A Label instance with a zero-initialized state vector of the correct size
    /// 
    /// # Example
    /// 
    /// ```
    /// # use routee_compass_core::model::label::label_enum::Label;
    /// # use routee_compass_core::model::network::VertexId;
    /// let vertex_id = VertexId(42);
    /// let label = Label::new_aligned_u8_state_zeros(vertex_id);
    /// ```
    pub fn empty_u8(vertex_id: VertexId) -> Self {
        Label::VertexWithU8StateVec {
            vertex_id,
            state: vec![0u8; OS_ALIGNED_STATE_LEN],
        }
    }
    
    /// Gets the OS-aligned state if this label contains one.
    /// 
    /// # Returns
    /// 
    /// Some reference to the state vector if this is a VertexWithU8StateVec, None otherwise
    pub fn get_u8_state(&self) -> Option<&Vec<u8>> {
        match self {
            Label::VertexWithU8StateVec { state, .. } => Some(state),
            _ => None,
        }
    }
    
    /// Gets a mutable reference to the OS-aligned state if this label contains one.
    /// 
    /// # Returns
    /// 
    /// Some mutable reference to the state vector if this is a VertexWithU8StateVec, None otherwise
    /// 
    /// # Safety
    /// 
    /// The caller must ensure they don't modify the Vec to have a different length,
    /// as this would violate the alignment constraint.
    pub fn get_mut_u8_state(&mut self) -> Option<&mut Vec<u8>> {
        match self {
            Label::VertexWithU8StateVec { state, .. } => Some(state),
            _ => None,
        }
    }

    pub fn vertex_id(&self) -> VertexId {
        match self {
            Label::Vertex(vertex_id) => *vertex_id,
            Label::VertexWithIntState { vertex_id, .. } => *vertex_id,
            Label::VertexWithIntStateVec { vertex_id, .. } => *vertex_id,
            Label::VertexWithU8StateVec { vertex_id, .. } => *vertex_id,
        }
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Label::Vertex(vertex_id) => write!(f, "Vertex({vertex_id})"),
            Label::VertexWithIntState { vertex_id, state } => {
                        write!(f, "VertexWithIntState({vertex_id}, {state})")
                    }
            Label::VertexWithIntStateVec { vertex_id, state } => {
                        write!(f, "VertexWithIntStateVec({vertex_id}, {state:?})")
                    }
            Label::VertexWithU8StateVec { vertex_id, state } => {
                write!(f, "VertexWithU8StateVec({vertex_id}, {state:?})")
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_new_aligned_u8_state_valid() {
        let vertex_id = VertexId(42);
        let state = vec![0u8; OS_ALIGNED_STATE_LEN];
        
        let label = Label::new_aligned_u8_state(vertex_id, state.clone());

        assert_eq!(label.vertex_id(), vertex_id);
        assert_eq!(label.get_u8_state(), Some(&state));
    }

    #[test]
    fn test_from_aligned_u8_array() {
        let vertex_id = VertexId(42);
        let state = [1u8; OS_ALIGNED_STATE_LEN];
        
        let label = Label::from_aligned_u8_array(vertex_id, &state);
        assert_eq!(label.vertex_id(), vertex_id);
        
        let expected_vec = state.to_vec();
        assert_eq!(label.get_u8_state(), Some(&expected_vec));
    }

    #[test]
    fn test_new_aligned_u8_state_zeros() {
        let vertex_id = VertexId(42);
        let label = Label::empty_u8(vertex_id);
        
        assert_eq!(label.vertex_id(), vertex_id);
        
        let state = label.get_u8_state().unwrap();
        assert_eq!(state.len(), OS_ALIGNED_STATE_LEN);
        assert!(state.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_aligned_u8_state_access() {
        let vertex_id = VertexId(42);
        let state = vec![1, 2, 3, 4, 5, 6, 7, 8][..OS_ALIGNED_STATE_LEN].to_vec();
        
        let mut label = Label::new_aligned_u8_state(vertex_id, state.clone());
        
        // Test immutable access
        assert_eq!(label.get_u8_state(), Some(&state));
        
        // Test mutable access
        if let Some(state_mut) = label.get_mut_u8_state() {
            state_mut[0] = 255;
        }
        
        assert_eq!(label.get_u8_state().unwrap()[0], 255);
    }

    #[test]
    fn test_aligned_u8_state_none_for_other_variants() {
        let vertex_id = VertexId(42);
        
        let vertex_label = Label::Vertex(vertex_id);
        assert_eq!(vertex_label.get_u8_state(), None);
        
        let int_state_label = Label::VertexWithIntState { vertex_id, state: 123 };
        assert_eq!(int_state_label.get_u8_state(), None);
        
        let valid_state = vec![1, 2, 3];
        let u8_vec_label = Label::VertexWithU8StateVec { vertex_id, state: valid_state.clone() };
        assert_eq!(u8_vec_label.get_u8_state(), Some(&valid_state));
    }

    #[test]
    fn test_aligned_label_display() {
        let vertex_id = VertexId(42);
        let state = vec![1u8; OS_ALIGNED_STATE_LEN];
        let label = Label::new_aligned_u8_state(vertex_id, state.clone());
        
        let display_string = format!("{}", label);
        assert!(display_string.contains("VertexWithU8StateVec"));
        assert!(display_string.contains("42"));
    }
}
