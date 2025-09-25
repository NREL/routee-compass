use std::collections::HashSet;

use allocative::Allocative;
use serde::Serialize;

use crate::{
    algorithm::search::{Direction, EdgeTraversal},
    model::{label::Label, network::VertexId},
};

/// A node in the search tree containing parent/child relationships and traversal data
#[derive(Debug, Clone, Allocative, Serialize)]
pub enum SearchTreeNode {
    Root {
        /// The label for this node
        label: Label,
        /// Children node labels
        children: HashSet<Label>,
        /// Tree orientation this node belongs to
        direction: Direction,
    },
    Branch {
        /// The label for this node
        label: Label,
        /// The edge traversal that led to this node (None for root)
        incoming_edge: EdgeTraversal,
        /// Parent node label (None for root)
        parent: Label,
        /// Children node labels
        children: HashSet<Label>,
        /// Tree orientation this node belongs to
        direction: Direction,
    },
}

impl SearchTreeNode {
    pub fn new_root(label: Label, orientation: Direction) -> Self {
        Self::Root {
            label: label.clone(),
            children: HashSet::new(),
            direction: orientation,
        }
    }

    pub fn new_child(
        label: Label,
        edge_traversal: EdgeTraversal,
        parent: Label,
        direction: Direction,
    ) -> Self {
        Self::Branch {
            label: label.clone(),
            incoming_edge: edge_traversal,
            parent,
            children: HashSet::new(),
            direction,
        }
    }

    pub fn label(&self) -> &Label {
        match self {
            SearchTreeNode::Root { label, .. } => label,
            SearchTreeNode::Branch { label, .. } => label,
        }
    }

    pub fn vertex_id(&self) -> &VertexId {
        match self {
            SearchTreeNode::Root { label, .. } => label.vertex_id(),
            SearchTreeNode::Branch { label, .. } => label.vertex_id(),
        }
    }

    pub fn parent_label(&self) -> Option<&Label> {
        match self {
            SearchTreeNode::Root { .. } => None,
            SearchTreeNode::Branch { parent, .. } => Some(parent),
        }
    }

    pub fn children(&self) -> &HashSet<Label> {
        match self {
            SearchTreeNode::Root { children, .. } => children,
            SearchTreeNode::Branch { children, .. } => children,
        }
    }

    pub fn incoming_edge(&self) -> Option<&EdgeTraversal> {
        match self {
            SearchTreeNode::Root { .. } => None,
            SearchTreeNode::Branch { incoming_edge, .. } => Some(incoming_edge),
        }
    }

    pub fn is_root(&self) -> bool {
        match self {
            SearchTreeNode::Root { .. } => true,
            SearchTreeNode::Branch { .. } => false,
        }
    }

    pub fn add_child(&mut self, child_label: Label) {
        match self {
            SearchTreeNode::Root { children, .. } => children.insert(child_label),
            SearchTreeNode::Branch { children, .. } => children.insert(child_label),
        };
    }

    pub fn remove_child(&mut self, child_label: &Label) {
        match self {
            SearchTreeNode::Root { children, .. } => children.remove(child_label),
            SearchTreeNode::Branch { children, .. } => children.remove(child_label),
        };
    }
}
