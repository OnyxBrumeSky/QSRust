use std::fmt;
use crate::instruction::i_struct::IStruct;
use crate::instruction::to_gate::ToGate;
use colored::*;


const INSTRUCTION_ADDED : &str  = "Instruction succesfully added";

#[derive(Clone)]
pub struct QuantumCircuit
{
	qubits : usize,
	clbits : usize,
	instructions : Vec<IStruct>,

}


impl  QuantumCircuit {

	pub fn new(q_bits : usize, cl_bits : usize) -> Self{
		QuantumCircuit { qubits: (q_bits), clbits: (cl_bits), instructions: (Vec::new()) }
	}

	pub fn get_q_bits(&self) -> usize {
		self.qubits
	}

	pub fn get_cl_bits(&self) -> usize {
		self.clbits
	}

	pub fn h(&mut self, e : usize) -> Result<&str, ColoredString> {
		if e >= self.qubits
		{
			Err("tried to apply H gate to non existent qbits.\n".red())
		} else {
			self.instructions.push(IStruct::H(e));
			Ok(INSTRUCTION_ADDED)
		}
	}

	pub fn x(&mut self, e : usize) -> Result<&str, ColoredString> {
		if e >= self.qubits
		{
			Err("tried to apply X gate to non existent qbits.\n".red())
		} else {
			self.instructions.push(IStruct::X(e));
			Ok(INSTRUCTION_ADDED)
		}
	}

	pub fn cx(&mut self, control : usize, target : usize) -> Result<&str, ColoredString>{
		if control >= self.qubits || target >= self.qubits
		{
			Err("tried to apply cx gate to non existent qbits.\n".red())
		} else if control == target {
			Err("tried to apply cx gate to same qbits.\n".red())
		} 
		else {
			self.instructions.push(IStruct::CX{control, target});
			Ok(INSTRUCTION_ADDED)
		}
	}

	pub fn y(&mut self, e : usize) -> Result<&str, ColoredString>{
		if e >= self.qubits
		{
			Err("tried to apply Y gate to non existent qbits.\n".red())
		} else {
			self.instructions.push(IStruct::Y(e));
			Ok(INSTRUCTION_ADDED)
		}
	}

	pub fn z(&mut self, e : usize) ->  Result<&str, ColoredString>{
		if e >= self.qubits
		{
			Err("tried to apply Z gate to non existent qbits.\n".red())
		} else {
			self.instructions.push(IStruct::Z(e));
			Ok(INSTRUCTION_ADDED)
		}
	}

	pub fn append<T: ToGate>(&mut self, gate : &T, position : Option<Vec<usize>>, label : Option<String>) -> Result<&Self, ColoredString> {
		let position = position.unwrap_or(vec![0]);
		if position.iter().any(|&x| x >= self.qubits) || gate.get_size() > self.qubits || position.len() > gate.get_size(){
			return Err("tried to apply a gate that doesn't fit the circuit.\n".red());
		}
		self.instructions.push(gate.to_gate(position, label));
		Ok(self)
	}

	pub fn measure(&mut self, q_bits : Vec<usize>, cl_bits : Vec<usize>) -> Result<&str, ColoredString> {
		if q_bits.iter().any(|&x| x >= self.qubits) || cl_bits.iter().any(|&x| x >= self.clbits)
		{
			Err("tried to apply measure to non existent qbits or clbits.\n".red())
		} else if q_bits.len() != cl_bits.len() {
			Err("tried to apply measure with different number of qbits and clbits.\n".red())
		}
		else {
			self.instructions.push(IStruct::MEASURE(q_bits, cl_bits));
			Ok(INSTRUCTION_ADDED)
		}
	}

	pub fn remap(&mut self, mapping : Vec<usize>) -> Result<QuantumCircuit, ColoredString> {
		if mapping.len() != self.qubits || mapping.iter().any(|&x| x >= self.qubits) || mapping.iter().any(|&x| x >= self.qubits) {
			return Err("tried to remap with invalid mapping.\n".red());
		}
		let mut new_instructions = Vec::new();
		for instruction in &self.instructions {
			match instruction {
				IStruct::H(qbits) => new_instructions.push(IStruct::H(mapping[*qbits])),
				IStruct::X(qbits) => new_instructions.push(IStruct::X(mapping[*qbits])),
				IStruct::Y(qbits) => new_instructions.push(IStruct::Y(mapping[*qbits])),
				IStruct::Z(qbits) => new_instructions.push(IStruct::Z(mapping[*qbits])),
				IStruct::CX{control, target} => new_instructions.push(IStruct::CX{control: mapping[*control], target: mapping[*target]}),
				IStruct::U{matrix, target} => new_instructions.push(IStruct::U{matrix: matrix.clone(), target: target.iter().map(|&x| mapping[x]).collect()}),
				IStruct::MEASURE(q_bits, cl_bits) => new_instructions.push(IStruct::MEASURE(q_bits.iter().map(|&x| mapping[x]).collect(), cl_bits.clone())),
				IStruct::GATE{position, instruction, label} => 
				new_instructions.push(IStruct::GATE{position: position.clone(), instruction: instruction.clone(), label: label.to_string()}),
				_ => new_instructions.push(instruction.clone())
			}
		}
		Ok(QuantumCircuit { qubits: self.qubits, clbits: self.clbits, instructions: Vec::from(new_instructions) })
	}





}



impl fmt::Display for QuantumCircuit{
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

impl ToGate for QuantumCircuit {
	fn to_gate(&self, position : Vec<usize>, label: Option<String>) -> IStruct {
		let label = label.unwrap_or(String::from("Gate"));
		IStruct::GATE{position, instruction: self.instructions.clone(), label: label}
	}

	fn get_size(&self)->usize {
		self.qubits
	}
}


