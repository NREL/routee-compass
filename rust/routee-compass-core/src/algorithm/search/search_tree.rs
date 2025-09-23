use super::EdgeTraversal;
use crate::model::network::VertexId;
use crate::{algorithm::search::Direction, model::label::Label};
use std::collections::{HashMap, HashSet};

/// A node in the search tree containing parent/child relationships and traversal data
#[derive(Debug, Clone)]
pub struct SearchTreeNode {
    /// The label for this node
    pub label: Label,
    /// The edge traversal that led to this node (None for root)
    pub edge_traversal: Option<EdgeTraversal>,
    /// Parent node label (None for root)
    pub parent: Option<Label>,
    /// Children node labels
    pub children: HashSet<Label>,
    /// Tree orientation this node belongs to
    pub direction: Direction,
}

impl SearchTreeNode {
    pub fn new_root(label: Label, orientation: Direction) -> Self {
        Self {
            label: label.clone(),
            edge_traversal: None,
            parent: None,
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
        Self {
            label: label.clone(),
            edge_traversal: Some(edge_traversal),
            parent: Some(parent),
            children: HashSet::new(),
            direction,
        }
    }

    pub fn vertex_id(&self) -> VertexId {
        self.label.vertex_id()
    }

    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    pub fn add_child(&mut self, child_label: Label) {
        self.children.insert(child_label);
    }

    pub fn remove_child(&mut self, child_label: &Label) {
        self.children.remove(child_label);
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
        self.get(label)?
            .parent
            .as_ref()
            .and_then(|parent_label| self.get(parent_label))
    }

    /// Get all children of a node
    pub fn get_children(&self, label: &Label) -> Vec<&SearchTreeNode> {
        if let Some(node) = self.get(label) {
            node.children
                .iter()
                .filter_map(|child_label| self.get(child_label))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all child labels of a node
    pub fn get_child_labels(&self, label: &Label) -> Vec<Label> {
        if let Some(node) = self.get(label) {
            node.children.iter().cloned().collect()
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

            // If this is the root, we're done
            if current_node.is_root() {
                break;
            }

            // Add the edge traversal that led to this node
            if let Some(ref edge_traversal) = current_node.edge_traversal {
                path.push(edge_traversal.clone());
            }

            // Move to parent
            current_label = current_node
                .parent
                .as_ref()
                .ok_or_else(|| SearchTreeError::MissingParent(current_label.clone()))?;
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

    /// Backtrack from a leaf vertex to construct a path with explicit direction override
    ///
    /// # Arguments
    /// * `leaf_vertex` - The vertex ID to backtrack from
    /// * `direction` - Direction to use for path construction (overrides tree's direction)
    ///
    /// # Returns
    /// A path of EdgeTraversals oriented according to the specified direction
    pub fn backtrack_with_direction(
        &self,
        leaf_vertex: VertexId,
        direction: Direction,
    ) -> Result<Vec<EdgeTraversal>, SearchTreeError> {
        let target_label = self
            .find_label_for_vertex(leaf_vertex)
            .ok_or(SearchTreeError::VertexNotFound(leaf_vertex))?;

        let mut path = Vec::new();
        let mut current_label = target_label;

        // Walk up from target to root
        loop {
            let current_node = self
                .get(current_label)
                .ok_or_else(|| SearchTreeError::LabelNotFound(current_label.clone()))?;

            // If this is the root, we're done
            if current_node.is_root() {
                break;
            }

            // Add the edge traversal that led to this node
            if let Some(ref edge_traversal) = current_node.edge_traversal {
                path.push(edge_traversal.clone());
            }

            // Move to parent
            current_label = current_node
                .parent
                .as_ref()
                .ok_or_else(|| SearchTreeError::MissingParent(current_label.clone()))?;
        }

        // Apply direction-specific ordering
        match direction {
            Direction::Forward => {
                path.reverse();
                Ok(path)
            }
            Direction::Reverse => Ok(path),
        }
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

    /// Convert to a HashMap<Label, SearchTreeBranch> for compatibility with existing code
    pub fn to_search_tree_branches(&self) -> HashMap<Label, super::SearchTreeBranch> {
        self.nodes
            .iter()
            .filter_map(|(label, node)| {
                if let Some(ref edge_traversal) = node.edge_traversal {
                    let terminal_label = node.parent.as_ref()?.clone();
                    Some((
                        label.clone(),
                        super::SearchTreeBranch {
                            terminal_label,
                            edge_traversal: edge_traversal.clone(),
                        },
                    ))
                } else {
                    // Skip root nodes as they don't have edge traversals
                    None
                }
            })
            .collect()
    }

    /// Create from a HashMap<Label, SearchTreeBranch> for compatibility
    pub fn from_search_tree_branches(
        branches: HashMap<Label, super::SearchTreeBranch>,
        direction: Direction,
    ) -> Result<Self, SearchTreeError> {
        let mut tree = Self::new(direction);

        if branches.is_empty() {
            return Ok(tree);
        }

        // First pass: find the root (node that appears as terminal_label but not as a key)
        // The terminal_label points to the parent node
        let mut potential_roots: HashSet<Label> = HashSet::new();
        let child_labels: HashSet<Label> = branches.keys().cloned().collect();

        // Collect all terminal_labels (these are parents)
        for branch in branches.values() {
            potential_roots.insert(branch.terminal_label.clone());
        }

        // Root is a potential root that is not also a child
        let mut roots: Vec<_> = potential_roots.difference(&child_labels).cloned().collect();

        if roots.len() != 1 {
            return Err(SearchTreeError::InvalidBranchStructure(format!(
                "Expected exactly one root, found {}. Potential roots: {:?}, Child labels: {:?}",
                roots.len(),
                potential_roots,
                child_labels
            )));
        }

        let root_label = roots.pop().unwrap();
        tree.set_root(root_label.clone());

        // Second pass: build the tree by inserting nodes whose parents already exist
        let mut pending = branches;
        while !pending.is_empty() {
            let mut progress = false;
            let mut to_remove = Vec::new();

            for (label, branch) in &pending {
                if tree.contains(&branch.terminal_label) {
                    tree.insert(
                        label.clone(),
                        branch.edge_traversal.clone(),
                        branch.terminal_label.clone(),
                    )?;
                    to_remove.push(label.clone());
                    progress = true;
                }
            }

            if !progress {
                return Err(SearchTreeError::InvalidBranchStructure(format!(
                    "Circular dependencies or missing parents in branch structure. Remaining: {:?}",
                    pending.keys().collect::<Vec<_>>()
                )));
            }

            for label in to_remove {
                pending.remove(&label);
            }
        }

        Ok(tree)
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
        assert!(root_node.children.is_empty());
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
        assert_eq!(child1_node.parent, Some(root_label.clone()));
        assert_eq!(
            child1_node.edge_traversal.as_ref().unwrap().edge_id,
            EdgeId(1)
        );

        let child2_node = tree.get(&child2_label).unwrap();
        assert!(!child2_node.is_root());
        assert_eq!(child2_node.parent, Some(root_label.clone()));
        assert_eq!(
            child2_node.edge_traversal.as_ref().unwrap().edge_id,
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
        assert_eq!(parent.label, root_label);
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
    fn test_to_search_tree_branches() {
        let root_label = create_test_label(0);
        let mut tree = SearchTree::with_root(root_label.clone(), Direction::Forward);

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

        let branches = tree.to_search_tree_branches();
        assert_eq!(branches.len(), 2); // Root is excluded (no edge traversal)

        let branch1 = branches.get(&child1_label).unwrap();
        assert_eq!(branch1.terminal_label, root_label);
        assert_eq!(branch1.edge_traversal.edge_id, EdgeId(1));

        let branch2 = branches.get(&child2_label).unwrap();
        assert_eq!(branch2.terminal_label, child1_label);
        assert_eq!(branch2.edge_traversal.edge_id, EdgeId(2));
    }

    #[test]
    fn test_from_search_tree_branches() {
        use super::super::SearchTreeBranch;
        use std::collections::HashMap;

        // Create branches representing: root(0) -> 1 -> 2
        let mut branches = HashMap::new();

        let root_label = create_test_label(0);
        let child1_label = create_test_label(1);
        let child2_label = create_test_label(2);

        branches.insert(
            child1_label.clone(),
            SearchTreeBranch {
                terminal_label: root_label.clone(),
                edge_traversal: create_test_edge_traversal(1, 10.0),
            },
        );

        branches.insert(
            child2_label.clone(),
            SearchTreeBranch {
                terminal_label: child1_label.clone(),
                edge_traversal: create_test_edge_traversal(2, 15.0),
            },
        );

        let tree = SearchTree::from_search_tree_branches(branches, Direction::Forward).unwrap();

        assert_eq!(tree.len(), 3);
        assert_eq!(tree.root(), Some(&root_label));

        // Verify structure
        let children = tree.get_child_labels(&root_label);
        assert_eq!(children.len(), 1);
        assert!(children.contains(&child1_label));

        let grandchildren = tree.get_child_labels(&child1_label);
        assert_eq!(grandchildren.len(), 1);
        assert!(grandchildren.contains(&child2_label));

        // Verify path reconstruction
        let path = tree.reconstruct_path(&child2_label).unwrap();
        assert_eq!(path.len(), 2);
        assert_eq!(path[0].edge_id, EdgeId(1));
        assert_eq!(path[1].edge_id, EdgeId(2));
    }

    #[test]
    fn test_from_search_tree_branches_invalid_structure() {
        use super::super::SearchTreeBranch;
        use std::collections::HashMap;

        // Create invalid structure with circular reference
        let mut branches = HashMap::new();

        let label1 = create_test_label(1);
        let label2 = create_test_label(2);

        branches.insert(
            label1.clone(),
            SearchTreeBranch {
                terminal_label: label2.clone(),
                edge_traversal: create_test_edge_traversal(1, 10.0),
            },
        );

        branches.insert(
            label2.clone(),
            SearchTreeBranch {
                terminal_label: label1.clone(),
                edge_traversal: create_test_edge_traversal(2, 15.0),
            },
        );

        let result = SearchTree::from_search_tree_branches(branches, Direction::Forward);
        assert!(matches!(
            result,
            Err(SearchTreeError::InvalidBranchStructure(_))
        ));
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
    fn test_backtrack_with_direction_override() {
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

        // Backtrack from vertex 3 with explicit Forward direction (same as tree's direction)
        let forward_path = tree
            .backtrack_with_direction(VertexId(3), Direction::Forward)
            .unwrap();
        assert_eq!(forward_path.len(), 3);
        assert_eq!(forward_path[0].edge_id, EdgeId(1)); // root -> 1
        assert_eq!(forward_path[1].edge_id, EdgeId(2)); // 1 -> 2
        assert_eq!(forward_path[2].edge_id, EdgeId(3)); // 2 -> 3

        // Backtrack from vertex 3 with explicit Reverse direction (override tree's direction)
        let reverse_path = tree
            .backtrack_with_direction(VertexId(3), Direction::Reverse)
            .unwrap();
        assert_eq!(reverse_path.len(), 3);
        assert_eq!(reverse_path[0].edge_id, EdgeId(3)); // 3 -> 2
        assert_eq!(reverse_path[1].edge_id, EdgeId(2)); // 2 -> 1
        assert_eq!(reverse_path[2].edge_id, EdgeId(1)); // 1 -> root
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
        assert_eq!(root_node.children.len(), 1);
        assert!(root_node.children.contains(&child_label));

        let child_node = tree.get(&child_label).unwrap();
        assert!(!child_node.is_root());
        assert_eq!(child_node.parent, Some(parent_label));
        assert_eq!(
            child_node.edge_traversal.as_ref().unwrap().edge_id,
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
