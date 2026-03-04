use std::fmt;
use crate::instruction::i_struct::IStruct;
use crate::instruction::to_gate::ToGate;
use colored::*;


const INSTRUCTION_ADDED : &str  = "Instruction succesfully added";

#[derive(Clone)]
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

	pub fn h(&mut self, elements : &'a[usize]) -> Result<&str, ColoredString> {
		if elements.iter().any(|&x| x >= self.qubits)
		{
			Err("Error: tried to apply H gate to non existent qbits.\n".red())
		} else {
			self.instructions.push(IStruct::H(elements));
			Ok(INSTRUCTION_ADDED)
		}
	}

	pub fn x(&mut self, elements : &'a[usize]) -> Result<&str, ColoredString> {
		if elements.iter().any(|&x| x >= self.qubits)
		{
			Err("Error: tried to apply X gate to non existent qbits.\n".red())
		} else {
			self.instructions.push(IStruct::X(elements));
			Ok(INSTRUCTION_ADDED)
		}
	}

	pub fn cx(&mut self, control : usize, target : usize) -> Result<&str, ColoredString>{
		if control >= self.qubits || target >= self.qubits
		{
			Err("Error: tried to apply cx gate to non existent qbits.\n".red())
		} else if control == target {
			Err("Error: tried to apply cx gate to same qbits.\n".red())
		} 
		else {
			self.instructions.push(IStruct::CX(control, target));
			Ok(INSTRUCTION_ADDED)
		}
	}

	pub fn y(&mut self, elements : &'a[usize]) -> Result<&str, ColoredString>{
		if elements.iter().any(|&x| x >= self.qubits)
		{
			Err("Error: tried to apply Y gate to non existent qbits.\n".red())
		} else {
			self.instructions.push(IStruct::Y(elements));
			Ok(INSTRUCTION_ADDED)
		}
	}

	pub fn z(&mut self, elements : &'a[usize]) ->  Result<&str, ColoredString>{
		if elements.iter().any(|&x| x >= self.qubits)
		{
			Err("Error: tried to apply Z gate to non existent qbits.\n".red())
		} else {
			self.instructions.push(IStruct::X(elements));
			Ok(INSTRUCTION_ADDED)
		}
	}

	pub fn append<T: ToGate<'a>>(&mut self, gate : &'a T, position : usize, label : &'a str) -> Result<&Self, ColoredString> {

		if position >= self.qubits || gate.get_size() + position > self.qubits {
			return Err("Error: tried to apply a gate that doesn't fit the circuit.\n".red());
		}
		self.instructions.push(gate.to_gate(position, label));
		Ok(self)
	}



}



impl <'a>fmt::Display for QuantumCircuit<'a>{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		writeln!(f, "{}","Quantum circuit :".cyan())?;
		writeln!(f, "nb of quantum bits {}, nb of classical bits {}.", self.qubits.to_string().yellow(), self.clbits.to_string().yellow())?;
		writeln!(f, "{}", "Liste of instructions:".bright_green())?;
		for elements in &self.instructions {
			writeln!(f, "{}", elements.to_string())?
		}
		Ok(())
	}
}

impl <'a>ToGate<'a> for QuantumCircuit<'a> {
	fn to_gate(&'a self, position : usize, label: &'a str) -> IStruct<'a> {
		IStruct::GATE(position, self.instructions.clone(), label)
	}

	fn get_size(&self)->usize {
		self.qubits
	}
}


