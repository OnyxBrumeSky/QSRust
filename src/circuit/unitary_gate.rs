use num::{complex::Complex32};
use nalgebra::DMatrix;
use colored::*;
use std::{fmt};
use crate::instruction::to_gate::ToGate;
use crate::instruction::i_struct::IStruct;

#[derive(Clone)]
pub struct UnitaryGate {
	qubits : usize,
	matrix :  DMatrix<Complex32>,
}


impl UnitaryGate {

	pub fn new(matrice : &[Complex32]) -> Result<Self, ColoredString>{
		
		let len = matrice.len();
		let n = (len as f64).sqrt().round() as usize;
		if n * n != len || !n.is_power_of_two() {
			return Err("Error: Tried to apply a non unitary quantique matrice".red());
		}
		let mat = DMatrix::from_row_slice(n, n, &matrice);
		let product = &mat.adjoint() * &mat;
		let identity = DMatrix::<Complex32>::identity(n, n);
		let tol = 1e-6f32;
    	let max_diff = product.iter()
        	.zip(identity.iter())
       	 	.map(|(a, b)| (*a - *b).norm())
       	 	.fold(0.0, f32::max);
		if max_diff > tol
		{
			return Err("Error: tried to apply U gate that is not unitary.\n".red());
		}
		let qubits = n.ilog2() as usize;
		Ok(UnitaryGate { qubits: (qubits), matrix : (mat) })
	}	
	
	fn to_u_gate(&self, position :Vec<usize>, label : String) -> IStruct {
		IStruct::GATE{position, instruction: vec![IStruct::U{matrix: self.matrix.clone(), target: Vec::new()}], label: label}
	}


}


impl fmt::Display for UnitaryGate{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}",self.matrix)?;
		write!(f, "Number of qbits needed : {}",self.qubits)?;
		Ok(())
	}	

}	

impl ToGate for UnitaryGate {
	fn get_size(&self)->usize {
		self.qubits
	}

	fn to_gate(&self, position :Vec<usize>, label : Option<String>) -> IStruct {
		let label = label.unwrap_or(String::from("Gate"));
		self.to_u_gate(position, label)
	}

}

