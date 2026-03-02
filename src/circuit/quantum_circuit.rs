use std::{array, fmt::{self, write}, vec};
use crate::instruction::i_struct::IStruct;

const INSTRUCTION_ADDED : &str  = "Instruction succesfully added";

pub struct QuantumCircuit
{
	qubits : usize,
	clbits : usize,
	instructions : Vec<IStruct>,

}


impl QuantumCircuit {

	pub fn new(q_bits : usize, cl_bits : usize) -> Self{
		QuantumCircuit { qubits: (q_bits), clbits: (cl_bits), instructions: (Vec::new()) }
	}

	pub fn get_q_bits(&self) -> usize {
		self.qubits
	}

	pub fn get_cl_bits(&self) -> usize {
		self.clbits
	}

	pub fn h(&mut self, elements : &[usize]) -> Option<&str>{
		if elements.len() > self.qubits
		{
			panic!("Error: tried to apply H gate to non existent qbits.\n")
		} else {
			self.instructions.push(IStruct::H(elements));
			Some(INSTRUCTION_ADDED)
		}
	}
}

impl fmt for QuantumCircuit{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "")
	}
}
