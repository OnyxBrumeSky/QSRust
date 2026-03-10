
use num::complex::Complex32;
use qc::circuit::quantum_circuit::QuantumCircuit;
use qc::api::generator::ToQasm;
use qc::circuit::unitary_gate::UnitaryGate;
use std::error::Error;

fn main()-> Result<(), Box<dyn Error>> {

	let mut circuit = QuantumCircuit::new(3, 3);

	circuit.h(0)?;
	circuit.cx(0, 1)?;
	circuit.rz(1.57, 0)?;
	circuit.ry(3.14, 1)?;
	circuit.swap(0, 1)?;
	circuit.x(0)?;
	circuit.y(1)?;
	circuit.z(0)?;
	let sqrt2_inv = 1.0f32 / 2.0f32.sqrt();
	let hadamard = [
            Complex32::new(sqrt2_inv, 0.0),
            Complex32::new(sqrt2_inv, 0.0),
            Complex32::new(sqrt2_inv, 0.0),
            Complex32::new(-sqrt2_inv, 0.0),
        ];
	let u_hadamard = UnitaryGate::new(&hadamard).unwrap();
	print!("{}", u_hadamard);
	circuit.append(&u_hadamard, None, Some(String::from("CustomGate")))?;

	circuit.measure([0,1].to_vec(), [0,1].to_vec())?;
	
	print!("Quantum Circuit:\n{}", circuit);
	let asm = circuit.to_qasm();
	print!("{}", asm);
	Ok(())
}