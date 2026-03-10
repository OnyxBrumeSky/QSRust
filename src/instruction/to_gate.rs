use crate::instruction::i_struct::IStruct;

/// Le trait `ToGate` permet de convertir une structure en une représentation de type `IStruct`.
///
/// Ce trait est utilisé pour définir une interface commune pour les objets pouvant être
/// transformés en une instruction de circuit quantique (`IStruct`).
pub trait ToGate {
    /// Convertit l'objet en une instruction de type `IStruct`.
    ///
    /// # Arguments
    ///
    /// * `position` - Un vecteur contenant les indices des qubits sur lesquels l'instruction doit s'appliquer.
    /// * `label` - Une chaîne de caractères optionnelle représentant un label ou un nom pour l'instruction.
    ///
    /// # Retourne
    ///
    /// Une instance de `IStruct` représentant l'instruction générée.
    fn to_gate(&self, position: Vec<usize>, label: Option<String>) -> IStruct;
}


