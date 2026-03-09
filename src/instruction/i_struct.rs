use std::fmt;
use colored::Colorize;
use nalgebra::DMatrix;
use num::complex::Complex32;

/// L'énumération `IStruct` représente les différentes instructions quantiques.
///
/// Chaque variante de l'énumération correspond à une opération ou instruction
/// spécifique pouvant être appliquée à un circuit quantique.
///
/// # Variantes
/// - `H(usize)` : Porte Hadamard appliquée à un qubit.
/// - `X(usize)` : Porte Pauli-X appliquée à un qubit.
/// - `Y(usize)` : Porte Pauli-Y appliquée à un qubit.
/// - `Z(usize)` : Porte Pauli-Z appliquée à un qubit.
/// - `CX { control: usize, target: usize }` : Porte CNOT appliquée entre un qubit de contrôle et un qubit cible.
/// - `U { matrix: DMatrix<Complex32>, target: Vec<usize> }` : Porte unitaire définie par une matrice appliquée à des qubits.
/// - `MEASURE(Vec<usize>, Vec<usize>)` : Mesure des qubits dans des bits classiques.
/// - `GATE { position: Vec<usize>, instruction: Vec<Box<IStruct>>, label: String }` : Instruction personnalisée appliquée à un circuit.
/// - `CONTROLLED { controls: Vec<usize>, target: usize, gate: Box<IStruct> }` : Porte contrôlée appliquée à un qubit cible.
/// - `SWAP { qbit1: usize, qbit2: usize }` : Échange de deux qubits.
/// - `RZ { angle: f32, target: usize }` : Rotation autour de l'axe Z avec un angle donné.
/// - `RX { angle: f32, target: usize }` : Rotation autour de l'axe X avec un angle donné.
/// - `RY { angle: f32, target: usize }` : Rotation autour de l'axe Y avec un angle donné.
/// - `ANY()` : Instruction générique ou non spécifiée.

#[derive(Clone)]
pub enum IStruct {
    H(usize),
    X(usize),
    Y(usize),
    Z(usize),
    CX { control: usize, target: usize },
    U { matrix: DMatrix<Complex32>, target: Vec<usize> },
    MEASURE(Vec<usize>, Vec<usize>),
    GATE { position: Vec<usize>, instruction: Vec<Box<IStruct>>, label: String },
    CONTROLLED { controls: Vec<usize>, target: usize, gate: Box<IStruct> },
    SWAP { qbit1: usize, qbit2: usize },
    RZ { angle: f32, target: usize },
    RX { angle: f32, target: usize },
    RY { angle: f32, target: usize },
    ANY(),
}

/// Implémentation du trait `fmt::Display` pour l'énumération `IStruct`.
///
/// Cette implémentation permet de convertir chaque variante de `IStruct`
/// en une chaîne de caractères lisible, utile pour le débogage ou l'affichage.
impl fmt::Display for IStruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IStruct::H(qbits) => {
                write!(f, "H gate is applied to the qbit(s) {:?}", qbits)
            }
            IStruct::X(qbits) => {
                write!(f, "X gate is applied to the qbit(s) {:?}", qbits)
            }
            IStruct::CX { control, target } => {
                write!(f, "CX gate is applied to the control qbit {:?} and target qbit {:?}", control, target)
            }
            IStruct::Y(qbits) => {
                write!(f, "Y gate is applied to the qbit(s) {:?}", qbits)
            }
            IStruct::Z(qbits) => {
                write!(f, "Z gate is applied to the qbit(s) {:?}", qbits)
            }
            IStruct::U { matrix, target } => {
                writeln!(f, "U gate : {}", matrix)?;
                write!(f, "is applied to the qbit {:?}", target)
            }
            IStruct::GATE { position, instruction, label } => {
                writeln!(f, "Gate {} is applied to the circuit at position {:?}", label.blue(), position)?;
                for elements in instruction {
                    writeln!(f, "{}", elements)?;
                }
                Ok(())
            }
            IStruct::MEASURE(q_bits, cl_bits) => {
                write!(f, "Measure is applied to the qbits {:?} and clbits {:?}", q_bits, cl_bits)
            }
            IStruct::SWAP { qbit1, qbit2 } => {
                write!(f, "SWAP gate is applied to the qbits {:?} and {:?}", qbit1, qbit2)
            }
            IStruct::RZ { angle, target } => {
                write!(f, "RZ gate with angle {} is applied to the qbit {:?}", angle, target)
            }
            IStruct::RX { angle, target } => {
                write!(f, "RX gate with angle {} is applied to the qbit {:?}", angle, target)
            }
            IStruct::RY { angle, target } => {
                write!(f, "RY gate with angle {} is applied to the qbit {:?}", angle, target)
            }
            _ => {
                write!(f, "Display trait to gate is not implemented")
            }
        }
    }
}