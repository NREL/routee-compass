use crate::model::traversal::state::state_variable::StateVar;

pub enum UpdateOperation {
    Replace,
    Add,
    Multiply,
}

impl UpdateOperation {
    pub fn perform_operation(&self, prev: &StateVar, next: &StateVar) -> StateVar {
        match self {
            UpdateOperation::Replace => *next,
            UpdateOperation::Add => *prev + *next,
            UpdateOperation::Multiply => StateVar(prev.0 * next.0),
        }
    }
}
