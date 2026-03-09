/// Module contenant les définitions des structures nécessaires pour représenter
/// les instructions quantiques, telles que les portes, les mesures, et autres opérations.
pub mod i_struct;

/// Module fournissant des traits et des implémentations pour convertir des structures
/// ou des objets en portes quantiques, standardisant leur manipulation dans un circuit.
pub mod to_gate;

/// Module définissant la logique quantique, incluant les opérations et transformations
/// appliquées aux qubits dans un circuit quantique.
pub mod q_logic;