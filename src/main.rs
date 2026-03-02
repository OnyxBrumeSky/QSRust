
use qc::circuit::quantum_circuit::QuantumCircuit;

fn main(){

	let mut qc : QuantumCircuit = QuantumCircuit::new(3,3);
	qc.h(&[0]);
	qc.h(&[0,2]);
	qc.h(&[0,1,2]);
	qc.h(&[0,1,2]);
	//qc.h(&[0, 1, 3, 5, 2]);
	print!("{}", qc)
}