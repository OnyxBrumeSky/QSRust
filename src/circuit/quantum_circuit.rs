use std::fmt;
use crate::instruction::i_struct::IStruct;
use colored::*;

const INSTRUCTION_ADDED : &str  = "Instruction succesfully added";

pub struct QuantumCircuit<'a>
{
	qubits : usize,
	clbits : usize,
	instructions : Vec<IStruct<'a>>,

}


impl <'a> QuantumCircuit<'a> {

	pub fn new(q_bits : usize, cl_bits : usize) -> Self{
		QuantumCircuit { qubits: (q_bits), clbits: (cl_bits), instructions: (Vec::new()) }
	}

	pub fn get_q_bits(&self) -> usize {
		self.qubits
	}

	pub fn get_cl_bits(&self) -> usize {
		self.clbits
	}

	pub fn h(&mut self, elements : &'a[usize]) -> Option<&str>{
		if elements.iter().any(|&x| x >= self.qubits)
		{
			panic!("{}","Error: tried to apply H gate to non existent qbits.\n".red())
		} else {
			self.instructions.push(IStruct::H(elements));
			Some(INSTRUCTION_ADDED)
		}
	}

	pub fn x(&mut self, elements : &'a[usize]) -> Option<&str>{
		if elements.iter().any(|&x| x >= self.qubits)
		{
			panic!("{}","Error: tried to apply X gate to non existent qbits.\n".red())
		} else {
			self.instructions.push(IStruct::X(elements));
			Some(INSTRUCTION_ADDED)
		}
	}

	pub fn cx(&mut self, elements : &'a[usize]) -> Option<&str>{
		if elements.iter().any(|&x| x >= self.qubits)
		{
			panic!("{}","Error: tried to apply cx gate to non existent qbits.\n".red())
		} else {
			self.instructions.push(IStruct::H(elements));
			Some(INSTRUCTION_ADDED)
		}
	}


}



impl <'a>fmt::Display for QuantumCircuit<'a>{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		writeln!(f, "{}","Quantum circuit :".cyan())?;
		writeln!(f, "nb of quantum bits {}, nb of classical bits {}.", self.qubits.to_string().yellow(), self.clbits.to_string().yellow())?;
		writeln!(f, "{}", "Liste of instructions:".bright_green())?;
		for elements in &self.instructions {
			writeln!(f, "{}", elements.to_string().green())?
		}
		Ok(())
	}
}
