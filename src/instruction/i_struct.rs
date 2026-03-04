use std::fmt;
use colored::Colorize;
use nalgebra::DMatrix;
use num::complex::Complex32;

#[derive(Clone)]
pub enum IStruct {
	H(usize),
	X(usize),
	Y(usize),
	Z(usize),
	CX{control: usize, target: usize},
	
	U{matrix: DMatrix<Complex32>, target: Vec<usize>},
	
	MEASURE(Vec<usize>, Vec<usize>),
	
	GATE{position: Vec<usize>, instruction: Vec<IStruct>, label: String},

	ANY()
}

impl fmt::Display for IStruct{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			IStruct::H(qbits) =>{
				write!(f, "H gate is applied to the qbit(s) {:?}", qbits)
			}
			IStruct::X(qbits) =>{
				write!(f, "X gate is applied to the qbit(s) {:?}", qbits)
			}
			IStruct::CX{control, target} =>{
				write!(f, "CX gate is applied to the control qbit {:?} and target qbit {:?}", control, target)
			}
			IStruct::Y(qbits) =>{
				write!(f, "X gate is applied to the qbit(s) {:?}", qbits)
			}
			IStruct::Z(qbits) =>{
				write!(f, "X gate is applied to the qbit(s) {:?}", qbits)
			}
			IStruct::U{matrix, target} =>{
				writeln!(f, "U gate : {}", matrix)?;
				write!(f, "is applied to the qbit {:?}", target)
			}
			IStruct::GATE{position, instruction, label} =>{
				writeln!(f, "Gate {} is applied to the circuit at position {:?}", label.blue(), position)?;
				for elements in instruction {
					writeln!(f, "{}", elements)?;
				}
				Ok(())
			}
			IStruct::MEASURE(q_bits, cl_bits) => {
				write!(f, "Measure is applied to the qbits {:?} and clbits {:?}", q_bits, cl_bits)
			}
			_ => {write!(f, "Display trait to gate is not implemented")}
		}
	}
}