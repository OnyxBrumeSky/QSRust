
use qc::circuit::quantum_circuit::QuantumCircuit;
use qc::circuit::unitary_gate::UnitaryGate;
use num::{complex::Complex32};
use std::error::Error;

fn main()-> Result<(), Box<dyn Error>> {

	let mut qc : QuantumCircuit = QuantumCircuit::new(3,3);
	let _ = qc.h(&[0]);
	let _ = qc.h(&[0,2]);
	let _ = qc.h(&[0,1,2]);
	let _ = qc.h(&[0,1,2]);
	let _ = qc.cx(1, 2);

	let binding = vec![0,1];
 	let _ = qc.h(&binding);
	let sqrt2_inv = 1.0f32 / 2.0f32.sqrt();
	let hadamard = [
            Complex32::new(sqrt2_inv, 0.0),
            Complex32::new(sqrt2_inv, 0.0),
            Complex32::new(sqrt2_inv, 0.0),
            Complex32::new(-sqrt2_inv, 0.0),
        ];
	let u_hadamard = UnitaryGate::new(&hadamard).unwrap();
	

	qc.append(&u_hadamard, 0, "label")?;

	let mut qc2 = QuantumCircuit::new(3,3);;

	qc2.append(&qc, 0, "qc 1")?;

	//qc.u(&hadamard);
	//qc.h(&[0, 1, 3, 5, 2]);
	print!("{}", qc2);
	Ok(())
}