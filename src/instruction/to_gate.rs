use std::usize;

use crate::instruction::i_struct::IStruct;

pub trait ToGate {
    fn get_size(&self)->usize;
    fn to_gate(&self, position :Vec<usize>, label : Option<String>) -> IStruct;
}

