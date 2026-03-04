use std::fmt;
use colored::Colorize;
use nalgebra::DMatrix;
use num::complex::Complex32;

#[derive(Clone)]
pub enum IStruct<'a> {
	H(&'a[usize]),
	X(&'a[usize]),
	CX(usize, usize),
	Y(&'a[usize]),
	Z(&'a[usize]),
	U(&'a DMatrix<Complex32>),
	GATE(usize, Vec<IStruct<'a>>, &'a str),
	ANY()
}

impl <'a>fmt::Display for IStruct<'a>{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			IStruct::H(qbits) =>{
				write!(f, "H gate is applied to the qbit(s) {:?}", qbits)
			}
			IStruct::X(qbits) =>{
				write!(f, "X gate is applied to the qbit(s) {:?}", qbits)
			}
			IStruct::CX(control, target) =>{
				write!(f, "CX gate is applied to the control qbit {:?} and target qbit {:?}", control, target)
			}
			IStruct::Y(qbits) =>{
				write!(f, "X gate is applied to the qbit(s) {:?}", qbits)
			}
			IStruct::Z(qbits) =>{
				write!(f, "X gate is applied to the qbit(s) {:?}", qbits)
			}
			IStruct::U(mat) =>{
				write!(f, "U gate : {}", mat)
			}
			IStruct::GATE(position, instruction, label) =>{
				writeln!(f, "Gate {} is applied to the circuit at position {}", label.blue(), position)?;
				for elements in instruction {
					writeln!(f, "{}", elements)?;
				}
				Ok(())
			}
			_ => {write!(f, "Display trait to gate is not implemented")}
		}
	}
}