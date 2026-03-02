use std::fmt;


pub enum IStruct<'a> {
	H(&'a[usize],),
	X(&'a[usize],),
	CX(&'a[usize],),
	Y(&'a[usize],),
	Z(&'a[usize],),
}

impl <'a>fmt::Display for IStruct<'a>{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			IStruct::H(qbits) =>{
				write!(f, "H gate is applied to the qbits {:?}", qbits)
			}
			IStruct::X(qbits) =>{
				write!(f, "X gate is applied to the qbits {:?}", qbits)
			}
			IStruct::CX(qbits) =>{
				write!(f, "CX gate is applied to the qbits {:?}", qbits)
			}
			_ => {write!(f, "Display trait to gate is not implemented")}
		}
	}
}