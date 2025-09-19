use std::fmt::Display;

use allocative::Allocative;
use serde::Serialize;

use crate::model::{label::label_model_error::LabelModelError, network::VertexId};

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
    /// Store u8 state data. more efficient memory layout for smaller
    /// numbers or categorical data with 256 or fewer categories.
    /// 
    /// For memory alignment, the Vec<u8> will be extended to the 
    /// nearest integer multiple of OS_ALIGNED_STATE_LEN that covers
    /// the provided state values.
    /// 
    /// In order to ensure reading the state value produces a slice
    /// of the same length as the Vec<u8> used to construct this 
    /// Label, we also store a state_len: u8 value. This limits to
    /// state sizes up to 256 elements. This is guaranteed when using
    /// the get_u8_state method for retrieval.
    VertexWithU8StateVec {
        vertex_id: VertexId,
        state: Vec<u8>,
        state_len: u8
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
    pub fn new_u8_state(vertex_id: VertexId, state: &[u8]) -> Result<Self, LabelModelError> {
        let mut label_state = state.to_vec();
        let state_len: u8 = state.len().try_into().map_err(|_| LabelModelError::BadLabelVecSize(state.len(), u8::MAX as usize))?;
        let remainder = label_state.len() % OS_ALIGNED_STATE_LEN;
        if remainder != 0 {
            let padding_needed = OS_ALIGNED_STATE_LEN - remainder;
            label_state.extend(vec![0u8; padding_needed]);
        }
        
        Ok(Label::VertexWithU8StateVec { vertex_id, state_len, state: label_state })
    }
    
    /// Gets the OS-aligned state if this label contains one.
    /// 
    /// # Returns
    /// 
    /// Some reference to the state vector if this is a VertexWithU8StateVec, None otherwise
    pub fn get_u8_state(&self) -> Option<&[u8]> {
        match self {
            Label::VertexWithU8StateVec { state, state_len, .. } => {
                let len: usize = (*state_len).into();
                Some(&state[0..len])
            },
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
            Label::VertexWithU8StateVec { vertex_id, state_len, state } => {
                write!(f, "VertexWithU8StateVec({vertex_id}, {state_len}, {state:?})")
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
        
        let label = Label::new_u8_state(vertex_id, &state).expect("test failed");

        assert_eq!(label.vertex_id(), vertex_id);
        assert_eq!(label.get_u8_state(), Some(state.as_slice()));
    }

    #[test]
    fn test_aligned_u8_state_none_for_other_variants() {
        let vertex_id = VertexId(42);
        
        let vertex_label = Label::Vertex(vertex_id);
        assert_eq!(vertex_label.get_u8_state(), None);
        
        let int_state_label = Label::VertexWithIntState { vertex_id, state: 123 };
        assert_eq!(int_state_label.get_u8_state(), None);
        
        // Test that VertexWithU8StateVec does return the state
        let valid_state = vec![1, 2, 3];
        let u8_vec_label = Label::new_u8_state(vertex_id, &valid_state).expect("test failed");
        let mut expected = [0; super::OS_ALIGNED_STATE_LEN].to_vec();
        expected[0] = 1;
        expected[1] = 2;
        expected[2] = 3;
        assert_eq!(u8_vec_label.get_u8_state(), Some(valid_state.as_slice()));
    }

    #[test]
    fn test_aligned_label_display() {
        let vertex_id = VertexId(42);
        let state = vec![1u8; OS_ALIGNED_STATE_LEN];
        let label = Label::new_u8_state(vertex_id, &state).expect("test failed");
        
        let display_string = format!("{}", label);
        assert!(display_string.contains("VertexWithU8StateVec"));
        assert!(display_string.contains("42"));
    }
}
