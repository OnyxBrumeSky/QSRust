use std::fmt;
use crate::instruction::i_struct::IStruct; 
use crate::instruction::q_logic::QLogic; 
use crate::instruction::to_gate::ToGate;
use colored::*;

/// Message constant utilisé pour indiquer qu'une instruction a été ajoutée avec succès.
const INSTRUCTION_ADDED: &str = "Instruction succesfully added";

/// Structure représentant un circuit quantique.
/// 
/// # Champs
/// - `qubits`: Nombre de qubits dans le circuit.
/// - `clbits`: Nombre de bits classiques dans le circuit.
/// - `instructions`: Liste des instructions (de type `IStruct`) appliquées dans le circuit.
#[derive(Clone)]
pub struct QuantumCircuit {
    qubits: usize,         
    clbits: usize,        
    instructions: Vec<IStruct>, 
}


impl QuantumCircuit {

    /// Crée un nouveau circuit quantique.
    ///
    /// # Arguments
    /// * `q_bits` - Nombre de qubits dans le circuit.
    /// * `cl_bits` - Nombre de bits classiques pour la mesure.
    ///
    /// # Retour
    /// Un `QuantumCircuit` vide prêt à recevoir des instructions.
    pub fn new(q_bits : usize, cl_bits : usize) -> Self{
        QuantumCircuit { qubits: q_bits, clbits: cl_bits, instructions: Vec::new() }
    }

    /// Retourne le nombre de qubits dans le circuit.
    pub fn get_q_bits(&self) -> usize {
        self.qubits
    }

    pub fn get_instructions(&self) -> &Vec<IStruct> {
        &self.instructions
    }

    /// Retourne le nombre de bits classiques dans le circuit.
    pub fn get_cl_bits(&self) -> usize {
        self.clbits
    }

    /// Applique la porte Hadamard (H) sur le qubit spécifié.
    ///
    /// # Arguments
    /// * `e` - Index du qubit cible.
    ///
    /// # Erreurs
    /// Renvoie une erreur si le qubit n'existe pas.
    pub fn h(&mut self, e : usize) -> Result<&str, ColoredString> {
        if e >= self.qubits {
            Err("tried to apply H gate to non existent qbits.\n".red())
        } else {
            self.instructions.push(IStruct::H(e));
            Ok(INSTRUCTION_ADDED)
        }
    }

    /// Applique la porte Pauli-X sur le qubit spécifié.
    pub fn x(&mut self, e : usize) -> Result<&str, ColoredString> {
        if e >= self.qubits {
            Err("tried to apply X gate to non existent qbits.\n".red())
        } else {
            self.instructions.push(IStruct::X(e));
            Ok(INSTRUCTION_ADDED)
        }
    }

    /// Applique la porte CNOT (CX) avec le qubit de contrôle et de cible spécifiés.
    ///
    /// # Arguments
    /// * `control` - Index du qubit de contrôle.
    /// * `target` - Index du qubit cible.
    ///
    /// # Erreurs
    /// Renvoie une erreur si les qubits n'existent pas ou si contrôle = cible.
    pub fn cx(&mut self, control : usize, target : usize) -> Result<&str, ColoredString>{
        if control >= self.qubits || target >= self.qubits {
            Err("tried to apply cx gate to non existent qbits.\n".red())
        } else if control == target {
            Err("tried to apply cx gate to same qbits.\n".red())
        } else {
            self.instructions.push(IStruct::CX{control, target});
            Ok(INSTRUCTION_ADDED)
        }
    }

    /// Applique la porte Pauli-Y sur le qubit spécifié.
    pub fn y(&mut self, e : usize) -> Result<&str, ColoredString>{
        if e >= self.qubits {
            Err("tried to apply Y gate to non existent qbits.\n".red())
        } else {
            self.instructions.push(IStruct::Y(e));
            Ok(INSTRUCTION_ADDED)
        }
    }

    /// Applique la porte Pauli-Z sur le qubit spécifié.
    pub fn z(&mut self, e : usize) ->  Result<&str, ColoredString>{
        if e >= self.qubits {
            Err("tried to apply Z gate to non existent qbits.\n".red())
        } else {
            self.instructions.push(IStruct::Z(e));
            Ok(INSTRUCTION_ADDED)
        }
    }

    /// Applique la porte SWAP entre deux qubits.
    ///
    /// # Arguments
    /// * `qbit1` - Index du premier qubit.
    /// * `qbit2` - Index du second qubit.
    ///
    /// # Erreurs
    /// Renvoie une erreur si les qubits n'existent pas ou si les deux qubits sont identiques.
    pub fn swap(&mut self, qbit1 : usize, qbit2 : usize) -> Result<&str, ColoredString>{
        if qbit1 >= self.qubits || qbit2 >= self.qubits {
            Err("tried to apply swap gate to non existent qbits.\n".red())
        } else if qbit1 == qbit2 {
            Err("tried to apply swap gate to same qbits.\n".red())
        } else {
            self.instructions.push(IStruct::SWAP{qbit1, qbit2});
            Ok(INSTRUCTION_ADDED)
        }
    }

    /// Applique la porte de rotation RZ sur un qubit.
    ///
    /// # Arguments
    /// * `angle` - Angle de rotation en radians.
    /// * `target` - Index du qubit cible.
    pub fn rz(&mut self, angle : f64, target : usize) -> Result<&str, ColoredString>{
        if target >= self.qubits {
            Err("tried to apply RZ gate to non existent qbits.\n".red())
        } else {
            self.instructions.push(IStruct::RZ{angle, target});
            Ok(INSTRUCTION_ADDED)
        }
    }

    /// Applique la porte de rotation RY sur un qubit.
    pub fn ry(&mut self, angle : f64, target : usize) -> Result<&str, ColoredString>{
        if target >= self.qubits {
            Err("tried to apply RY gate to non existent qbits.\n".red())
        } else {
            self.instructions.push(IStruct::RY{angle, target});
            Ok(INSTRUCTION_ADDED)
        }
    }

    /// Applique la porte de rotation RX sur un qubit.
    pub fn rx(&mut self, angle : f64, target : usize) -> Result<&str, ColoredString>{
        if target >= self.qubits {
            Err("tried to apply RX gate to non existent qbits.\n".red())
        } else {
            self.instructions.push(IStruct::RX{angle, target});
            Ok(INSTRUCTION_ADDED)
        }
    }

    /// Applique une porte contrôlée personnalisée.
    ///
    /// # Arguments
    /// * `controls` - Liste des qubits de contrôle.
    /// * `target` - Qubit cible.
    /// * `gate` - La porte à appliquer si tous les qubits de contrôle sont activés.
    ///
    /// # Erreurs
    /// Vérifie que les qubits existent et que le target n'est pas dans les contrôles.
    pub fn controlled(&mut self, controls: Vec<usize>, target: usize, gate: IStruct) -> Result<&str, ColoredString> {
        if controls.iter().any(|&c| c >= self.qubits) || target >= self.qubits {
            return Err("tried to apply controlled gate to non existent qbits.\n".red());
        }
        if controls.contains(&target) {
            return Err("control qubits cannot include the target qubit.\n".red());
        }
        self.instructions.push(IStruct::CONTROLLED {
            controls,
            target,
            gate: Box::new(gate),
        });
        Ok(INSTRUCTION_ADDED)
    }

    /// Ajoute une porte générique au circuit.
    ///
    /// # Arguments
    /// * `gate` - La porte implémentant `ToGate` et `QLogic`.
    /// * `position` - Indices des qubits où appliquer la porte (None = premiers qubits).
    /// * `label` - Nom optionnel pour la porte.
    ///
    /// # Erreurs
    /// Vérifie que la porte peut s'appliquer dans le circuit.
    pub fn append<T: ToGate + QLogic>(&mut self, gate : &T, position : Option<Vec<usize>>, label : Option<String>) -> Result<&Self, ColoredString> {
        let position = position.unwrap_or_else(|| (0..gate.get_size()).collect());
        if position.iter().any(|&x| x >= self.qubits) || gate.get_size() > self.qubits || position.len() > gate.get_size(){
            return Err("tried to apply a gate that doesn't fit the circuit.\n".red());
        }
        self.instructions.push(gate.to_gate(position, label));
        Ok(self)
    }

    /// Applique une mesure sur des qubits et stocke le résultat dans des bits classiques.
    ///
    /// # Arguments
    /// * `q_bits` - Qubits à mesurer.
    /// * `cl_bits` - Bits classiques pour stocker le résultat.
    ///
    /// # Erreurs
    /// Vérifie que les qubits et bits existent et que les vecteurs sont de même taille.
    pub fn measure(&mut self, q_bits : Vec<usize>, cl_bits : Vec<usize>) -> Result<&str, ColoredString> {
        if q_bits.iter().any(|&x| x >= self.qubits) || cl_bits.iter().any(|&x| x >= self.clbits) {
            Err("tried to apply measure to non existent qbits or clbits.\n".red())
        } else if q_bits.len() != cl_bits.len() {
            Err("tried to apply measure with different number of qbits and clbits.\n".red())
        } else {
            self.instructions.push(IStruct::MEASURE(q_bits, cl_bits));
            Ok(INSTRUCTION_ADDED)
        }
    }

    /// Crée une nouvelle version du circuit avec les qubits remappés selon `mapping`.
    ///
    /// # Arguments
    /// * `mapping` - Permutation des indices des qubits.
    ///
    /// # Erreurs
    /// Vérifie que la permutation est valide et complète.
    pub fn remap(&self, mapping : Vec<usize>) -> Result<QuantumCircuit, ColoredString> {
        if mapping.len() != self.qubits || mapping.iter().any(|&x| x >= self.qubits) {
            return Err("tried to remap with invalid mapping.\n".red());
        }
        let mut new_instructions = Vec::new();
        for instruction in &self.instructions {
            match instruction {
                IStruct::H(qbits) => new_instructions.push(IStruct::H(mapping[*qbits])),
                IStruct::X(qbits) => new_instructions.push(IStruct::X(mapping[*qbits])),
                IStruct::Y(qbits) => new_instructions.push(IStruct::Y(mapping[*qbits])),
                IStruct::Z(qbits) => new_instructions.push(IStruct::Z(mapping[*qbits])),
                IStruct::CX{control, target} => new_instructions.push(IStruct::CX{control: mapping[*control], target: mapping[*target]}),
                IStruct::U{matrix, target} => new_instructions.push(IStruct::U{matrix: matrix.clone(), target: target.iter().map(|&x| mapping[x]).collect()}),
                IStruct::MEASURE(q_bits, cl_bits) => new_instructions.push(IStruct::MEASURE(q_bits.iter().map(|&x| mapping[x]).collect(), cl_bits.clone())),
                IStruct::GATE{position, instruction, label} => 
                    new_instructions.push(IStruct::GATE{position: position.iter().map(|&x| mapping[x]).collect(), instruction: instruction.clone(), label: label.to_string()}),
                _ => new_instructions.push(instruction.clone())
            }
        }
        Ok(QuantumCircuit { qubits: self.qubits, clbits: self.clbits, instructions: new_instructions })
    }

    /// Concatène un autre circuit à la fin de celui-ci, en décalant les indices des qubits.
    pub fn compose(&self, other : &QuantumCircuit) -> QuantumCircuit{
        let mut istr = self.instructions.clone();
        istr.extend(other.offset_i(self.qubits));
        QuantumCircuit { qubits: self.qubits + other.get_q_bits(), clbits: self.clbits + other.get_cl_bits(), instructions: istr }
    }

    /// Décale les indices des qubits d'un circuit de `offset`.
    ///
    /// # Note
    /// Les instructions de mesure ne sont pas décalées et génèrent un message d'erreur.
    fn offset_i (&self, offset: usize) -> Vec<IStruct>{
        let mut new = vec![];
        for element in &self.instructions{
            match element {
                IStruct::H(value) => new.push(IStruct::H(value + offset)),
                IStruct::X(value) => new.push(IStruct::X(value + offset)),
                IStruct::Y(value) => new.push(IStruct::Y(value + offset)),
                IStruct::Z(value) => new.push(IStruct::Z(value + offset)),
                IStruct::CX{control, target} => new.push(IStruct::CX{control :control + offset, target :target + offset}),
                IStruct::U { matrix, target } => new.push(IStruct::U { matrix: matrix.clone(), target: target.iter().map(|&x| x + offset).collect() }),
                IStruct::MEASURE(_a, _b) => eprint!("{}","Error : tried to offset a measure instruction.\n".red()),
                IStruct::GATE{position, instruction, label} => new.push(IStruct::GATE{position: position.iter().map(|&x| x + offset).collect(), instruction: instruction.clone(), label: label.to_string()}),
                 _ => new.push(element.clone())
            }
        }
        new
    }
}




/// Implémente l'affichage formaté pour le circuit quantique.
/// 
/// Affiche le nombre de qubits, le nombre de bits classiques, et liste toutes les instructions.
/// Utilise des couleurs pour rendre la sortie plus lisible (cyan, jaune, vert).
impl fmt::Display for QuantumCircuit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Titre
        writeln!(f, "{}","Quantum circuit :".cyan())?;
        // Nombre de qubits et bits classiques
        writeln!(
            f, 
            "nb of quantum bits {}, nb of classical bits {}.",
            self.qubits.to_string().yellow(), 
            self.clbits.to_string().yellow()
        )?;
        // Liste des instructions
        writeln!(f, "{}", "List of instructions:".bright_green())?;
        for elements in &self.instructions {
            writeln!(f, "{}", elements.to_string())?;
        }
        Ok(())
    }
}

/// Implémente la conversion d'un QuantumCircuit en une porte (`IStruct::GATE`) générique.
///
/// # Arguments
/// * `position` - Indices des qubits sur lesquels appliquer le sub-circuit.
/// * `label` - Nom optionnel de la porte, "Gate" par défaut.
///
/// # Retour
/// Une instruction `IStruct::GATE` contenant toutes les instructions du circuit.
/// Chaque instruction est clonée et encapsulée dans un `Box` pour posséder les données.
impl ToGate for QuantumCircuit {
    fn to_gate(&self, position : Vec<usize>, label: Option<String>) -> IStruct {
        let label = label.unwrap_or(String::from("Gate"));
        let boxed_instructions: Vec<Box<IStruct>> = self.instructions
            .iter()
            .map(|instr| Box::new(instr.clone()))
            .collect();
        IStruct::GATE { position, instruction: boxed_instructions, label }
    }
}

/// Implémente le trait `QLogic` pour QuantumCircuit.
///
/// Permet d'interroger la "taille" du circuit, c'est-à-dire
/// le nombre de qubits du circuit.
impl QLogic for QuantumCircuit {
    fn get_size(&self) -> usize {
        self.qubits
    }
}



