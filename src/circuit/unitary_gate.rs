//! Module définissant la structure `UnitaryGate` et ses implémentations.
//!
//! Cette structure représente une porte unitaire utilisée dans les circuits quantiques. 
//! Une porte unitaire est définie par une matrice carrée complexe qui respecte les propriétés unitaires (U * U† = I).

use num::complex::Complex32; 
use nalgebra::DMatrix; 
use colored::*; 
use std::fmt;
use crate::instruction::q_logic::QLogic;
use crate::instruction::to_gate::ToGate; 
use crate::instruction::i_struct::IStruct;

/// Structure représentant une porte unitaire.
///
/// Une porte unitaire est définie par :
/// - `qubits`: Le nombre de qubits nécessaires.
/// - `matrix`: La matrice unitaire associée à la porte.
#[derive(Clone)]
pub struct UnitaryGate {
    /// Nombre de qubits nécessaires pour cette porte.
    qubits: usize,
    /// Matrice unitaire associée à la porte.
    matrix: DMatrix<Complex32>,
}

impl UnitaryGate {
    /// Crée une nouvelle instance de `UnitaryGate` à partir d'une matrice.
    ///
    /// # Arguments
    ///
    /// * `matrice` - Une tranche contenant les éléments de la matrice unitaire (en ligne).
    ///
    /// # Retourne
    ///
    /// * `Ok(UnitaryGate)` si la matrice est valide et unitaire.
    /// * `Err(ColoredString)` si la matrice n'est pas carrée, n'est pas une puissance de deux, ou n'est pas unitaire.
    ///
    /// # Exemple
    ///
    /// ```rust
    /// let matrice = vec![...]; // Matrice unitaire sous forme de tableau.
    /// let gate = UnitaryGate::new(&matrice);
    /// ```
    pub fn new(matrice: &[Complex32]) -> Result<Self, ColoredString> {
        let len = matrice.len();
        let n = (len as f64).sqrt().round() as usize;

        if n * n != len || !n.is_power_of_two() {
            return Err("Error: Tried to apply a non unitary quantique matrice".red());
        }

        let mat = DMatrix::from_row_slice(n, n, &matrice);
        let product = &mat.adjoint() * &mat;
        let identity = DMatrix::<Complex32>::identity(n, n);
        let tol = 1e-6f32;

        let max_diff = product
            .iter()
            .zip(identity.iter())
            .map(|(a, b)| (*a - *b).norm())
            .fold(0.0, f32::max);

        if max_diff > tol {
            return Err("Error: tried to apply U gate that is not unitary.\n".red());
        }

        let qubits = n.ilog2() as usize;
        Ok(UnitaryGate { qubits, matrix: mat })
    }
}

impl fmt::Display for UnitaryGate {
    /// Implémentation de l'affichage pour `UnitaryGate`.
    ///
    /// Affiche la matrice unitaire et le nombre de qubits nécessaires.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.matrix)?;
        writeln!(f, "Number of qbits needed : {}", self.qubits)?;
        Ok(())
    }
}

impl ToGate for UnitaryGate {
    /// Convertit la porte unitaire en une structure `IStruct`.
    ///
    /// # Arguments
    ///
    /// * `position` - Les positions des qubits cibles.
    /// * `label` - Une étiquette optionnelle pour la porte.
    ///
    /// # Retourne
    ///
    /// Une instance de `IStruct` représentant la porte.
    fn to_gate(&self, position: Vec<usize>, label: Option<String>) -> IStruct {
        let label = label.unwrap_or(String::from("Gate"));
        IStruct::GATE {
            position : position.clone(),
            instruction: vec![Box::new(IStruct::U {
                matrix: self.matrix.clone(),
                target: position.clone(),
            })],
            label,
        }
    }
}

impl QLogic for UnitaryGate {
    /// Retourne la taille (nombre de qubits) de la porte unitaire.
    fn get_size(&self) -> usize {
        self.qubits
    }
}