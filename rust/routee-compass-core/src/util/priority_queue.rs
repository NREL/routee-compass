use std::{
    collections::hash_map::RandomState,
    hash::Hash,
    ops::{Deref, DerefMut},
};

use allocative::Allocative;
use priority_queue::PriorityQueue;

pub struct InternalPriorityQueue<I: Hash + Eq, P: Ord, S = RandomState>(pub PriorityQueue<I, P, S>);

impl<H: Hash + Eq, I: Ord, S> Deref for InternalPriorityQueue<H, I, S> {
    type Target = PriorityQueue<H, I, S>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<H: Hash + Eq, I: Ord, S> DerefMut for InternalPriorityQueue<H, I, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<H: Hash + Eq + Allocative, I: Ord + Allocative, S> Allocative
    for InternalPriorityQueue<H, I, S>
{
    fn visit<'a, 'b: 'a>(&self, visitor: &'a mut allocative::Visitor<'b>) {
        let _visitor = visitor.enter_self_sized::<Self>();
    }
}

impl<I: Hash + Eq, P: Ord> Default for InternalPriorityQueue<I, P, RandomState> {
    fn default() -> InternalPriorityQueue<I, P, RandomState> {
        InternalPriorityQueue(PriorityQueue::new())
    }
}

mod tests {
    use crate::{
        model::{
            road_network::vertex_id::VertexId,
            unit::{cost::ReverseCost, Cost},
        },
        util::priority_queue::InternalPriorityQueue,
    };

    #[test]
    fn test_priority_queue() {
        let mut costs: InternalPriorityQueue<VertexId, ReverseCost> =
            InternalPriorityQueue::default();

        costs.push(VertexId(0), Cost::from(0.0003).into());
        costs.push(VertexId(1), Cost::from(0.0001).into());
        costs.push(VertexId(2), Cost::from(0.0002).into());

        let (vertex_id, _cost) = costs.pop().unwrap();

        assert_eq!(vertex_id, VertexId(1));

        costs.push_increase(VertexId(0), Cost::from(0.00001).into());

        let (vertex_id, _cost) = costs.pop().unwrap();

        assert_eq!(vertex_id, VertexId(0));

        let (vertex_id, _cost) = costs.pop().unwrap();

        assert_eq!(vertex_id, VertexId(2));
    }
}
