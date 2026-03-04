use std::usize;

use crate::instruction::i_struct::IStruct;

pub trait ToGate<'a> {
    fn get_size(&self)->usize;
    fn to_gate(&'a self, position :usize, label : &'a str) -> IStruct<'a>;
}

