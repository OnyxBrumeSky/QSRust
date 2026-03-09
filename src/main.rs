
use qc::circuit::quantum_circuit::QuantumCircuit;

use std::error::Error;

fn main()-> Result<(), Box<dyn Error>> {

	let mut qc : QuantumCircuit = QuantumCircuit::new(3,3);
	qc.h(0)?;
	qc.h(1)?;
	qc.h(2)?;
	qc.cx(1, 2)?;
	
	print!("Original circuit : \n{}\n", qc);

	let qc2 = qc.remap(vec![2, 0, 1])?;

	print!("Remapped circuit : \n{}\n", qc2);
	
	Ok(())
}