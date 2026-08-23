use crate::backend::checkpoint::checkpoint_line::CheckpointEdit;
use crate::backend::mostly_one_vec::MostlyOneVec;

#[derive()]
pub(crate) struct Checkpoint {
    pub inner: MostlyOneVec<SingleEdit>,
}

#[derive(Clone)]
pub(crate) struct SingleEdit {
    pub edit: CheckpointEdit,
}
