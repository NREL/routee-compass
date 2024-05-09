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
