use crate::backend::little_string::LittleString;
use crate::backend::mostly_one_vec::MostlyOneVec;

#[derive(Debug, Clone)]
pub(crate) struct CheckpointEdit {
    pub start_line: usize,
    pub start_col: usize,
    pub removed_data: MostlyOneVec<LittleString>,
    pub added_data: MostlyOneVec<LittleString>,
}
