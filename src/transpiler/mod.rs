//! Module de transpilation des circuits quantiques.
//!
//! Ce module convertit un [`QuantumCircuit`] en un circuit physique
//! compatible avec un backend IBM Quantum, en quatre étapes successives :
//!
//! 1. [`gate_decomposer`] — décompose toutes les portes en `U3 + CX`
//! 2. [`placement`] — assigne les qubits virtuels aux qubits physiques connectés
//! 3. [`router`] — insère des SWAPs pour corriger les connexions manquantes
//! 4. [`transpiler`] — orchestre le pipeline et génère le QASM final
//!
//! Le [`coupling_map`] fournit la topologie du backend utilisée par le placement et le router.
//!
//! [`QuantumCircuit`]: crate::circuit::quantum_circuit::QuantumCircuit

/// Topologie de connectivité du backend et détection de la porte native à 2 qubits.
pub mod coupling_map;

/// Décomposition des portes en portes universelles `U3 + CX`.
pub mod gate_decomposer;

/// Routing par insertion de SWAPs pour corriger les connexions manquantes.
pub mod router;

/// Placement des qubits virtuels sur les qubits physiques du coupling map.
pub mod placement;

/// Pipeline de transpilation complet : décomposition → placement → routing → QASM.
pub mod transpiler;