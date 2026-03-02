
use qc::circuit::quantum_circuit::QuantumCircuit;
use num::{complex::Complex32, integer::Roots};
fn main(){

	let mut qc : QuantumCircuit = QuantumCircuit::new(3,3);
	qc.h(&[0]);
	qc.h(&[0,2]);
	qc.h(&[0,1,2]);
	qc.h(&[0,1,2]);
	qc.cx(1, 2);
	let binding = vec![0,1];
 	qc.h(&binding);
	let sqrt2_inv = 1.0f32 / 2.0f32.sqrt();
	let hadamard = [
            Complex32::new(sqrt2_inv, 0.0),
            Complex32::new(sqrt2_inv, 0.0),
            Complex32::new(sqrt2_inv, 0.0),
            Complex32::new(-sqrt2_inv, 0.0),
        ];
	
	//qc.u(&hadamard);
	//qc.h(&[0, 1, 3, 5, 2]);
	print!("{}", qc)
}