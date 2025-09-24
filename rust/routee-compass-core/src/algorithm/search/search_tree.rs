use super::EdgeTraversal;
use crate::model::network::VertexId;
use crate::{algorithm::search::Direction, model::label::Label};
use std::collections::{HashMap, HashSet};

/// A node in the search tree containing parent/child relationships and traversal data
#[derive(Debug, Clone)]
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
    }
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
            parent: parent,
            children: HashSet::new(),
            direction,
        }
    }

    pub fn label(&self) -> &Label {
        match self {
            SearchTreeNode::Root { label, ..} => label,
            SearchTreeNode::Branch { label, ..} => label,
        }
    }

    pub fn vertex_id(&self) -> VertexId {
        match self {
            SearchTreeNode::Root { label, ..} => label.vertex_id(),
            SearchTreeNode::Branch { label, ..} => label.vertex_id(),
        }
    }

    pub fn parent_label(&self) -> Option<&Label> {
        match self {
            SearchTreeNode::Root { ..} => None,
            SearchTreeNode::Branch { parent, ..} => Some(parent),
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
            SearchTreeNode::Root {  children, ..} => children.insert(child_label),
            SearchTreeNode::Branch { children, .. } => children.insert(child_label),
        };
    }

    pub fn remove_child(&mut self, child_label: &Label) {
        match self {
            SearchTreeNode::Root {  children, ..} => children.remove(child_label),
            SearchTreeNode::Branch { children, .. } => children.remove(child_label),
        };
    }
}

/// A search tree that supports efficient lookups and bi-directional parent/child traversal
/// Designed for route planning algorithms that need both indexing and backtracking capabilities
pub struct SearchTree {
    /// Fast lookup by label
    nodes: HashMap<Label, SearchTreeNode>,
    /// The root node (None if empty tree)
    root: Option<Label>,
    /// Tree orientation for bi-directional search support
    direction: Direction,
}

impl SearchTree {
    /// Create a new empty search tree with the specified orientation
    pub fn new(direction: Direction) -> Self {
        Self {
            nodes: HashMap::new(),
            root: None,
            direction,
        }
    }

    /// Create a new search tree with the given root node.
    pub fn with_root(root_label: Label, orientation: Direction) -> Self {
        let mut tree = Self::new(orientation);
        tree.set_root(root_label);
        tree
    }

    /// Set the root node of the tree
    pub fn set_root(&mut self, root_label: Label) {
        let root_node = SearchTreeNode::new_root(root_label.clone(), self.direction);
        self.nodes.insert(root_label.clone(), root_node);
        self.root = Some(root_label);
    }

    /// Insert a node with a parent relationship
    pub fn insert(
        &mut self,
        label: Label,
        edge_traversal: EdgeTraversal,
        parent_label: Label,
    ) -> Result<(), SearchTreeError> {
        // Verify parent exists
        // If parent doesn't exist but tree is empty, make parent the root
        if !self.nodes.contains_key(&parent_label) {
            if self.is_empty() {
                self.set_root(parent_label.clone());
            } else {
                return Err(SearchTreeError::ParentNotFound(parent_label));
            }
        }

        // Create the new node
        let new_node = SearchTreeNode::new_child(
            label.clone(),
            edge_traversal,
            parent_label.clone(),
            self.direction,
        );

        // Add child relationship to parent
        if let Some(parent_node) = self.nodes.get_mut(&parent_label) {
            parent_node.add_child(label.clone());
        }

        // Insert the new node
        self.nodes.insert(label, new_node);

        Ok(())
    }

    /// Get a node by its label
    pub fn get(&self, label: &Label) -> Option<&SearchTreeNode> {
        self.nodes.get(label)
    }

    /// Get a mutable reference to a node by its label
    pub fn get_mut(&mut self, label: &Label) -> Option<&mut SearchTreeNode> {
        self.nodes.get_mut(label)
    }

    /// Get the root label
    pub fn root(&self) -> Option<&Label> {
        self.root.as_ref()
    }

    /// Get the parent of a node
    pub fn get_parent(&self, label: &Label) -> Option<&SearchTreeNode> {
        let node = self.get(label)?;
        let parent_label = node.parent_label()?;
        self.get(parent_label)
    }

    /// Get all children of a node
    pub fn get_children(&self, label: &Label) -> Vec<&SearchTreeNode> {
        match self.get(label) {
            None => vec![],
            Some(node) => {
                node.children()
                    .iter()
                    .filter_map(|child_label| self.get(child_label))
                    .collect()
            },
        }
    }

    /// Get all child labels of a node
    pub fn get_child_labels(&self, label: &Label) -> Vec<Label> {
        if let Some(node) = self.get(label) {
            node.children().iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Check if the tree contains a node with the given label
    pub fn contains(&self, label: &Label) -> bool {
        self.nodes.contains_key(label)
    }

    /// Get the number of nodes in the tree
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get the tree orientation
    pub fn direction(&self) -> Direction {
        self.direction
    }

    /// Reconstruct a path from root to the given target label
    /// This is the primary backtracking method for route reconstruction
    pub fn reconstruct_path(
        &self,
        target_label: &Label,
    ) -> Result<Vec<EdgeTraversal>, SearchTreeError> {
        let mut path = Vec::new();
        let mut current_label = target_label;

        // Walk up from target to root
        loop {
            let current_node = self
                .get(current_label)
                .ok_or_else(|| SearchTreeError::LabelNotFound(current_label.clone()))?;

            // If this is the root, we're done, otherwise traverse path
            match current_node {
                SearchTreeNode::Root { .. } => break,
                SearchTreeNode::Branch { incoming_edge, parent, .. } => {
                    path.push(incoming_edge.clone());
                    current_label = parent;
                },
            }
        }

        // For forward search, reverse the path to go from root to target
        // For reverse search, keep the path as-is (it's already from target to source)
        match self.direction {
            Direction::Forward => {
                path.reverse();
                Ok(path)
            }
            Direction::Reverse => Ok(path),
        }
    }

    /// Backtrack from a leaf vertex to construct a path using the tree's inherent direction
    ///
    /// # Arguments
    /// * `leaf_vertex` - The vertex ID to backtrack from
    ///
    /// # Returns
    /// A path of EdgeTraversals from root to leaf (forward) or leaf to root (reverse)
    pub fn backtrack(&self, leaf_vertex: VertexId) -> Result<Vec<EdgeTraversal>, SearchTreeError> {
        // Find the label for this vertex - there might be multiple labels for the same vertex
        // in state-dependent searches, so we need to find the right one
        let target_label = self
            .find_label_for_vertex(leaf_vertex)
            .ok_or(SearchTreeError::VertexNotFound(leaf_vertex))?;

        self.reconstruct_path(target_label)
    }

    /// Find a label for the given vertex ID
    /// In case of multiple labels for the same vertex (state-dependent search),
    /// returns the first one found. For more precise control, use reconstruct_path directly.
    fn find_label_for_vertex(&self, vertex: VertexId) -> Option<&Label> {
        self.nodes.keys().find(|label| label.vertex_id() == vertex)
    }

    /// Get all labels in the tree
    pub fn labels(&self) -> impl Iterator<Item = &Label> {
        self.nodes.keys()
    }

    /// Get all nodes in the tree
    pub fn nodes(&self) -> impl Iterator<Item = &SearchTreeNode> {
        self.nodes.values()
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum SearchTreeError {
    #[error("parent not found for label {0}")]
    ParentNotFound(Label),
    #[error("Label not found in tree: {0}")]
    LabelNotFound(Label),
    #[error("Node is missing parent reference: {0}")]
    MissingParent(Label),
    #[error("Invalid branch structure: {0}")]
    InvalidBranchStructure(String),
    #[error("Vertex not found in tree: {0}")]
    VertexNotFound(VertexId),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        network::{EdgeId, EdgeListId, VertexId},
        unit::Cost,
    };

    fn create_test_edge_traversal(edge_id: usize, cost: f64) -> EdgeTraversal {
        EdgeTraversal {
            edge_id: EdgeId(edge_id),
            edge_list_id: EdgeListId(0),
            access_cost: Cost::new(0.0),
            traversal_cost: Cost::new(cost),
            result_state: vec![],
        }
    }

    fn create_test_label(vertex_id: usize) -> Label {
        Label::Vertex(VertexId(vertex_id))
    }

    #[test]
    fn test_new_empty_tree() {
        let tree = SearchTree::new(Direction::Forward);
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.direction(), Direction::Forward);
        assert!(tree.root().is_none());
    }

    #[test]
    fn test_tree_with_root() {
        let root_label = create_test_label(0);
        let tree = SearchTree::with_root(root_label.clone(), Direction::Forward);

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 1);
        assert_eq!(tree.root(), Some(&root_label));
        assert!(tree.contains(&root_label));

        let root_node = tree.get(&root_label).unwrap();
        assert!(root_node.is_root());
        assert_eq!(root_node.vertex_id(), VertexId(0));
        assert!(root_node.children().is_empty());
    }

    #[test]
    fn test_insert_child_nodes() {
        let root_label = create_test_label(0);
        let mut tree = SearchTree::with_root(root_label.clone(), Direction::Forward);

        // Insert first child
        let child1_label = create_test_label(1);
        let child1_traversal = create_test_edge_traversal(1, 10.0);
        tree.insert(
            child1_label.clone(),
            child1_traversal.clone(),
            root_label.clone(),
        )
        .unwrap();

        // Insert second child
        let child2_label = create_test_label(2);
        let child2_traversal = create_test_edge_traversal(2, 15.0);
        tree.insert(
            child2_label.clone(),
            child2_traversal.clone(),
            root_label.clone(),
        )
        .unwrap();

        assert_eq!(tree.len(), 3);

        // Verify root has two children
        let children = tree.get_children(&root_label);
        assert_eq!(children.len(), 2);

        let child_labels = tree.get_child_labels(&root_label);
        assert!(child_labels.contains(&child1_label));
        assert!(child_labels.contains(&child2_label));

        // Verify child nodes
        let child1_node = tree.get(&child1_label).unwrap();
        assert!(!child1_node.is_root());
        assert_eq!(child1_node.parent_label(), Some(&root_label));
        assert_eq!(
            child1_node.incoming_edge().unwrap().edge_id,
            EdgeId(1)
        );

        let child2_node = tree.get(&child2_label).unwrap();
        assert!(!child2_node.is_root());
        assert_eq!(child2_node.parent_label(), Some(&root_label));
        assert_eq!(
            child2_node.incoming_edge().unwrap().edge_id,
            EdgeId(2)
        );
    }

    #[test]
    fn test_insert_with_nonexistent_parent() {
        let root_label = create_test_label(0);
        let mut tree = SearchTree::with_root(root_label, Direction::Forward);

        let child_label = create_test_label(1);
        let child_traversal = create_test_edge_traversal(1, 10.0);
        let nonexistent_parent = create_test_label(99);

        let result = tree.insert(child_label, child_traversal, nonexistent_parent.clone());
        assert!(matches!(result, Err(SearchTreeError::ParentNotFound(_))));
    }

    #[test]
    fn test_get_parent() {
        let root_label = create_test_label(0);
        let mut tree = SearchTree::with_root(root_label.clone(), Direction::Forward);

        let child_label = create_test_label(1);
        let child_traversal = create_test_edge_traversal(1, 10.0);
        tree.insert(child_label.clone(), child_traversal, root_label.clone())
            .unwrap();

        // Root has no parent
        assert!(tree.get_parent(&root_label).is_none());

        // Child has root as parent
        let parent = tree.get_parent(&child_label).unwrap();
        assert_eq!(parent.label(), &root_label);
    }

    #[test]
    fn test_reconstruct_path_forward_orientation() {
        let root_label = create_test_label(0);
        let mut tree = SearchTree::with_root(root_label.clone(), Direction::Forward);

        // Build a linear path: 0 -> 1 -> 2 -> 3
        let child1_label = create_test_label(1);
        let child1_traversal = create_test_edge_traversal(1, 10.0);
        tree.insert(
            child1_label.clone(),
            child1_traversal.clone(),
            root_label.clone(),
        )
        .unwrap();

        let child2_label = create_test_label(2);
        let child2_traversal = create_test_edge_traversal(2, 15.0);
        tree.insert(
            child2_label.clone(),
            child2_traversal.clone(),
            child1_label.clone(),
        )
        .unwrap();

        let child3_label = create_test_label(3);
        let child3_traversal = create_test_edge_traversal(3, 20.0);
        tree.insert(
            child3_label.clone(),
            child3_traversal.clone(),
            child2_label.clone(),
        )
        .unwrap();

        // Reconstruct path to child3
        let path = tree.reconstruct_path(&child3_label).unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0].edge_id, EdgeId(1)); // root -> 1
        assert_eq!(path[1].edge_id, EdgeId(2)); // 1 -> 2
        assert_eq!(path[2].edge_id, EdgeId(3)); // 2 -> 3
    }

    #[test]
    fn test_reconstruct_path_reverse_orientation() {
        let root_label = create_test_label(0);
        let mut tree = SearchTree::with_root(root_label.clone(), Direction::Reverse);

        // Build a linear path: 0 -> 1 -> 2 -> 3
        let child1_label = create_test_label(1);
        let child1_traversal = create_test_edge_traversal(1, 10.0);
        tree.insert(
            child1_label.clone(),
            child1_traversal.clone(),
            root_label.clone(),
        )
        .unwrap();

        let child2_label = create_test_label(2);
        let child2_traversal = create_test_edge_traversal(2, 15.0);
        tree.insert(
            child2_label.clone(),
            child2_traversal.clone(),
            child1_label.clone(),
        )
        .unwrap();

        let child3_label = create_test_label(3);
        let child3_traversal = create_test_edge_traversal(3, 20.0);
        tree.insert(
            child3_label.clone(),
            child3_traversal.clone(),
            child2_label.clone(),
        )
        .unwrap();

        // Reconstruct path to child3 (reverse orientation keeps natural order)
        let path = tree.reconstruct_path(&child3_label).unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0].edge_id, EdgeId(3)); // 3 -> 2
        assert_eq!(path[1].edge_id, EdgeId(2)); // 2 -> 1
        assert_eq!(path[2].edge_id, EdgeId(1)); // 1 -> root
    }

    #[test]
    fn test_reconstruct_path_nonexistent_label() {
        let root_label = create_test_label(0);
        let tree = SearchTree::with_root(root_label, Direction::Forward);

        let nonexistent_label = create_test_label(99);
        let result = tree.reconstruct_path(&nonexistent_label);
        assert!(matches!(result, Err(SearchTreeError::LabelNotFound(_))));
    }

    #[test]
    fn test_iterators() {
        let root_label = create_test_label(0);
        let mut tree = SearchTree::with_root(root_label.clone(), Direction::Forward);

        let child1_label = create_test_label(1);
        let child1_traversal = create_test_edge_traversal(1, 10.0);
        tree.insert(child1_label.clone(), child1_traversal, root_label.clone())
            .unwrap();

        let child2_label = create_test_label(2);
        let child2_traversal = create_test_edge_traversal(2, 15.0);
        tree.insert(child2_label.clone(), child2_traversal, root_label.clone())
            .unwrap();

        // Test labels iterator
        let labels: HashSet<_> = tree.labels().cloned().collect();
        assert_eq!(labels.len(), 3);
        assert!(labels.contains(&root_label));
        assert!(labels.contains(&child1_label));
        assert!(labels.contains(&child2_label));

        // Test nodes iterator
        let node_count = tree.nodes().count();
        assert_eq!(node_count, 3);

        let vertex_ids: HashSet<_> = tree.nodes().map(|n| n.vertex_id()).collect();
        assert_eq!(vertex_ids.len(), 3);
        assert!(vertex_ids.contains(&VertexId(0)));
        assert!(vertex_ids.contains(&VertexId(1)));
        assert!(vertex_ids.contains(&VertexId(2)));
    }

    #[test]
    fn test_backtrack_forward_tree() {
        let root_label = create_test_label(0);
        let mut tree = SearchTree::with_root(root_label.clone(), Direction::Forward);

        // Build a linear path: 0 -> 1 -> 2 -> 3
        let child1_label = create_test_label(1);
        let child1_traversal = create_test_edge_traversal(1, 10.0);
        tree.insert(
            child1_label.clone(),
            child1_traversal.clone(),
            root_label.clone(),
        )
        .unwrap();

        let child2_label = create_test_label(2);
        let child2_traversal = create_test_edge_traversal(2, 15.0);
        tree.insert(
            child2_label.clone(),
            child2_traversal.clone(),
            child1_label.clone(),
        )
        .unwrap();

        let child3_label = create_test_label(3);
        let child3_traversal = create_test_edge_traversal(3, 20.0);
        tree.insert(
            child3_label.clone(),
            child3_traversal.clone(),
            child2_label.clone(),
        )
        .unwrap();

        // Backtrack from vertex 3 using tree's inherent direction (Forward)
        let path = tree.backtrack(VertexId(3)).unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0].edge_id, EdgeId(1)); // root -> 1
        assert_eq!(path[1].edge_id, EdgeId(2)); // 1 -> 2
        assert_eq!(path[2].edge_id, EdgeId(3)); // 2 -> 3
    }

    #[test]
    fn test_backtrack_reverse_tree() {
        let root_label = create_test_label(0);
        let mut tree = SearchTree::with_root(root_label.clone(), Direction::Reverse);

        // Build a linear path: 0 -> 1 -> 2 -> 3
        let child1_label = create_test_label(1);
        let child1_traversal = create_test_edge_traversal(1, 10.0);
        tree.insert(
            child1_label.clone(),
            child1_traversal.clone(),
            root_label.clone(),
        )
        .unwrap();

        let child2_label = create_test_label(2);
        let child2_traversal = create_test_edge_traversal(2, 15.0);
        tree.insert(
            child2_label.clone(),
            child2_traversal.clone(),
            child1_label.clone(),
        )
        .unwrap();

        let child3_label = create_test_label(3);
        let child3_traversal = create_test_edge_traversal(3, 20.0);
        tree.insert(
            child3_label.clone(),
            child3_traversal.clone(),
            child2_label.clone(),
        )
        .unwrap();

        // Backtrack from vertex 3 using tree's inherent direction (Reverse)
        let path = tree.backtrack(VertexId(3)).unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0].edge_id, EdgeId(3)); // 3 -> 2
        assert_eq!(path[1].edge_id, EdgeId(2)); // 2 -> 1
        assert_eq!(path[2].edge_id, EdgeId(1)); // 1 -> root
    }

    #[test]
    fn test_backtrack_nonexistent_vertex() {
        let root_label = create_test_label(0);
        let tree = SearchTree::with_root(root_label, Direction::Forward);

        let result = tree.backtrack(VertexId(99));
        assert!(matches!(
            result,
            Err(SearchTreeError::VertexNotFound(VertexId(99)))
        ));
    }

    #[test]
    fn test_backtrack_root_vertex() {
        let root_label = create_test_label(0);
        let tree = SearchTree::with_root(root_label.clone(), Direction::Forward);

        // Backtracking from root should return empty path
        let path = tree.backtrack(VertexId(0)).unwrap();
        assert_eq!(path.len(), 0);
    }

    #[test]
    fn test_find_label_for_vertex() {
        let root_label = create_test_label(0);
        let mut tree = SearchTree::with_root(root_label.clone(), Direction::Forward);

        let child1_label = create_test_label(1);
        let child1_traversal = create_test_edge_traversal(1, 10.0);
        tree.insert(child1_label.clone(), child1_traversal, root_label.clone())
            .unwrap();

        // Test finding existing vertex
        let found_label = tree.find_label_for_vertex(VertexId(1));
        assert_eq!(found_label, Some(&child1_label));

        // Test finding non-existent vertex
        let not_found = tree.find_label_for_vertex(VertexId(99));
        assert_eq!(not_found, None);
    }

    #[test]
    fn test_auto_root_creation() {
        let mut tree = SearchTree::new(Direction::Forward);
        assert!(tree.is_empty());
        assert!(tree.root().is_none());

        // Insert first node - parent should become root automatically
        let parent_label = create_test_label(0);
        let child_label = create_test_label(1);
        let edge_traversal = create_test_edge_traversal(1, 10.0);

        tree.insert(
            child_label.clone(),
            edge_traversal.clone(),
            parent_label.clone(),
        )
        .unwrap();

        // Verify root was created automatically
        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 2); // root + child
        assert_eq!(tree.root(), Some(&parent_label));

        // Verify structure
        let root_node = tree.get(&parent_label).unwrap();
        assert!(root_node.is_root());
        assert_eq!(root_node.children().len(), 1);
        assert!(root_node.children().contains(&child_label));

        let child_node = tree.get(&child_label).unwrap();
        assert!(!child_node.is_root());
        assert_eq!(child_node.parent_label(), Some(&parent_label));
        assert_eq!(
            child_node.incoming_edge().unwrap().edge_id,
            EdgeId(1)
        );
    }

    #[test]
    fn test_auto_root_creation_chain() {
        let mut tree = SearchTree::new(Direction::Forward);

        // Build a chain: 0 -> 1 -> 2 -> 3 by only calling insert
        let label0 = create_test_label(0);
        let label1 = create_test_label(1);
        let label2 = create_test_label(2);
        let label3 = create_test_label(3);

        // First insert creates root automatically
        tree.insert(
            label1.clone(),
            create_test_edge_traversal(1, 10.0),
            label0.clone(),
        )
        .unwrap();

        // Subsequent inserts work normally
        tree.insert(
            label2.clone(),
            create_test_edge_traversal(2, 15.0),
            label1.clone(),
        )
        .unwrap();
        tree.insert(
            label3.clone(),
            create_test_edge_traversal(3, 20.0),
            label2.clone(),
        )
        .unwrap();

        // Verify final structure
        assert_eq!(tree.len(), 4);
        assert_eq!(tree.root(), Some(&label0));

        // Verify backtracking works
        let path = tree.backtrack(VertexId(3)).unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0].edge_id, EdgeId(1)); // 0 -> 1
        assert_eq!(path[1].edge_id, EdgeId(2)); // 1 -> 2
        assert_eq!(path[2].edge_id, EdgeId(3)); // 2 -> 3
    }

    #[test]
    fn test_insert_without_auto_root_when_parent_exists() {
        let mut tree = SearchTree::new(Direction::Forward);
        let root_label = create_test_label(0);

        // Manually create root first
        tree.set_root(root_label.clone());

        // Insert should work normally without creating a new root
        let child_label = create_test_label(1);
        let edge_traversal = create_test_edge_traversal(1, 10.0);
        tree.insert(child_label.clone(), edge_traversal, root_label.clone())
            .unwrap();

        // Root should still be the same
        assert_eq!(tree.len(), 2);
        assert_eq!(tree.root(), Some(&root_label));

        // Trying to insert with non-existent parent should still fail
        let orphan_label = create_test_label(99);
        let nonexistent_parent = create_test_label(999);
        let result = tree.insert(
            orphan_label,
            create_test_edge_traversal(99, 5.0),
            nonexistent_parent.clone(),
        );
        assert!(matches!(result, Err(SearchTreeError::ParentNotFound(_))));
    }
}
