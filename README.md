# Quantum Circuit Framework in Rust

## Overview

This project is a **Rust-based reimplementation of a quantum computing framework**, inspired by existing tools like Qiskit. It was built from scratch with the goal of gaining a deeper understanding of how quantum software stacks actually work under the hood.

Rather than simply using existing libraries, this project explores each component in detail: circuit construction, transpilation, simulation, API interaction, and result visualization.

---

## Features

### Quantum Circuit Construction
- Programmatic construction of quantum circuits
- Support for a **subset of quantum gates**
- Designed to be modular and extensible

---

### IBM Quantum API Integration
- Connect to IBM Quantum services
- Retrieve submitted jobs
- List available backends
- Execute circuits on real quantum hardware via API

---

### Transpilation
- Transpile circuits using the **coupling map** of a selected backend
- Adapt circuits to hardware constraints
- Basic mapping and transformation logic

---

### OpenQASM Parsing (Partial)
- Decode OpenQASM files
- **Limited implementation** (only a subset of the language is supported)
- Designed as a proof of concept rather than full compliance

---

### Simulation Engine
- State vector simulation
- Provides:
  - Exact probability amplitudes
  - Measurement results via configurable **shots**
- Useful for testing circuits locally before running on hardware

---

### Result Visualization
Supports multiple output formats:
- Terminal display (quick debugging)
- JSON export (for further processing)
- HTML rendering (for visual exploration)

---

## Project Motivation

The main objective of this project was:

> To deeply understand the internal structure and responsibilities of a quantum computing framework.

This includes:
- How circuits are represented and manipulated
- How transpilation adapts circuits to hardware constraints
- How simulation works using state vectors
- How real quantum hardware is accessed via APIs

---

## Limitations

- Not all quantum gates are implemented  
- OpenQASM support is partial  
- Transpiler is simplified compared to production frameworks  
- Simulation is basic and not optimized for large circuits  

This project is primarily **educational and experimental**.

---

## Documentation

Documentation is available and was **partially generated using AI tools** to improve clarity and coverage.

It includes:
- Explanations of the architecture
- Module breakdowns
- Usage examples

---

## Getting Started

### Clone the repository
You can clone my repo and try it. Here is an exemple workflow :
```rust

use rusty_quantum::api::service::Service;
use rusty_quantum::circuit::quantum_circuit::QuantumCircuit;
use std::error::Error;
use rusty_quantum::api::job_builder::{JobRequest, SamplerJobBuilder, SamplerPub, JobOptionsBuilder};
use rusty_quantum::simulator::parser::parse_qasm_full;
use rusty_quantum::visualizer::{export_json, export_html, print_histogram, JobMetadata};
use rusty_quantum::simulator::simulator::simulate_statevector;

// Bell state circuit example
#[tokio::main]
async fn main()-> Result<(), Box<dyn Error>> {


	// Create a quantum circuit with 2 qubits and 2 classical bits
	let mut circuit = QuantumCircuit::new(2, 2);
	circuit.h(0)?;
	circuit.cx(0, 1)?;
	circuit.measure([0,1].to_vec(), [0,1].to_vec())?;
		
	// Set the service with your IBM Quantum API key
	let token = "YOUR-IBM-QUANTUM-API-KEY";
	let mut ibm_service = Service::builder()
	.token(token.to_string())
	.build().await?;

	// List available backends and select the first one (you could use any other selection criteria)
	let backend = ibm_service.get_backends().await?.list()[0].clone();
	ibm_service.use_backend(backend);


	// Transpile the circuit to the selected backend's native gates and topology
	let transpile_result = ibm_service.transpile_circuit(&circuit, &ibm_service.backend_name()).await?;

	// Build a job request to run the transpiled circuit on the backend's sampler
	let shots = 100;
	let job = JobRequest::Sampler(
		SamplerJobBuilder::new(ibm_service.backend_name())
			.add_pub(SamplerPub::new(&transpile_result.qasm).shots(shots))
			.options(
				JobOptionsBuilder::new()
					.dynamical_decoupling(true)
					.build()
			)
	);

	// Submit the job and wait for it to complete. Here we are using a helper method that combines submission and polling until completion, with a polling interval of 5 seconds.
	let (response, id) = ibm_service.run_and_collect(job, 5).await?;
	let counts = response.to_counts()?;
	println!("Job completed with counts: {:?}", counts);

	// Get the job metadata (like execution time, backend info, etc.) for visualization and export
	let job_from_ibm = ibm_service.get_specific_job(&id).await?;
	let meta = JobMetadata::from_job(&job_from_ibm, shots); // Note that here shots should not be given as a parameter and is one of the improvement that could be added

	// Print a histogram of the results, and export the results to JSON and HTML files for further analysis and sharing.
	print_histogram(&counts, &meta, "Bell State", 40);
	export_json(&counts, &meta, "results/bell.json")?;
	export_html(&counts, &meta, "Bell State — ibm_fez", "results/bell.html")?;
	


	// Now try using the local simulator to parse the transpiled QASM and simulate the statevector. This allows us to compare the exact statevector probabilities with the sampled counts from the IBM backend.
	let decode = parse_qasm_full(&transpile_result.qasm)?;
	let results = simulate_statevector(&decode.instructions, decode.n_qubits)?;
	
	// Here we print the exact probabilities from the statevector simulation, and also a sample of 100 shots from the statevector to compare with the IBM backend results.
	println!("Statevector exact probabilities : {:?}", results.probabilities(1e-10));
	println!("Statevector sample over {}: {:?}", shots, results.sample(10));

	// Here we should see that the local simulator gives us only the |00> and |11> states with approximately equal probabilities, while the IBM backend's sampled counts should also show a similar distribution (with some noise due to the finite number of shots and hardware imperfections).

	Ok(())
}
```

If you have any questions, feedback, or ideas, feel free to reach out!

- GitHub: https://github.com/OnyxBrumeSky
- Email: mkhoury@student.42.fr
