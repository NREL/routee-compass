use std::collections::{HashMap, HashSet};
use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd)]
pub struct Restriction {
    pub weight_limit_lbs: u32,
    pub height_limit_feet: u8,
    pub width_limit_feet: u8,
    pub length_limit_feet: u8,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Node {
    pub id: u32,
}

impl Node {
    pub fn new(id: u32) -> Self {
        Node { id }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd)]
pub struct Link {
    pub id: u32,
    pub start_node: Node,
    pub end_node: Node,
    pub road_class: u8,
    pub speed: u8,
    pub distance: u32,
    pub grade: i16,
    pub restriction: Option<Restriction>,
}

impl Link {
    pub fn new(
        id: u32,
        start_node: Node,
        end_node: Node,
        road_class: u8,
        speed: u8,
        distance: u32,
        grade: i16,
        restriction: Option<Restriction>,
    ) -> Self {
        Link {
            id,
            start_node,
            end_node,
            road_class,
            speed,
            distance,
            grade,
            restriction,
        }
    }
}

pub struct Graph {
    adjacency_list: HashMap<Node, HashSet<Link>>,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            adjacency_list: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.adjacency_list.entry(node).or_insert_with(HashSet::new);
    }

    pub fn add_edge(&mut self, link: Link) {
        self.adjacency_list
            .entry(link.start_node)
            .or_insert_with(HashSet::new)
            .insert(link);
    }

    pub fn neighbors(&self, node: &Node) -> Option<&HashSet<Link>> {
        self.adjacency_list.get(node)
    }
}
