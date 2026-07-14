use crate::backend::checkpoint::checkpoint_line::CheckpointEdit;
use crate::backend::mostly_one_vec::MostlyOneVec;

#[derive(Debug)]
pub(crate) struct Checkpoint {
    pub inner: MostlyOneVec<SingleEdit>,
}

#[derive(Debug, Clone)]
pub(crate) struct SingleEdit {
    pub edit: CheckpointEdit,
}
