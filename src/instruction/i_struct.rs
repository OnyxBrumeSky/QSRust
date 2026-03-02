use std::fmt;

use num::complex::Complex32;


pub enum IStruct<'a> {
	H(&'a[usize]),
	X(&'a[usize]),
	CX(usize, usize),
	Y(&'a[usize]),
	Z(&'a[usize]),
	U(&'a[Complex32]),
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
				write!(f, "U gate is applied to the qbit(s) {:?}", mat)
			}
			_ => {write!(f, "Display trait to gate is not implemented")}
		}
	}
}