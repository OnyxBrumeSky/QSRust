use crate::transpiler::coupling_map::CouplingMap;
use crate::transpiler::gate_decomposer::DecomposedGate;

/// Mapping entre qubits virtuels et qubits physiques.
///
/// `mapping[i]` contient l'indice du qubit physique assigné au qubit virtuel `i`.
/// Produit par [`place`] et consommé par le router et le générateur QASM.
#[derive(Debug, Clone)]
pub struct Layout {
    /// Mapping `virtuel → physique` : `mapping[virtuel] = physique`
    pub mapping: Vec<u32>,
}

impl Layout {
    /// Construit un [`Layout`] depuis un vecteur de mapping.
    pub fn new(mapping: Vec<u32>) -> Self {
        Self { mapping }
    }

    /// Retourne le qubit physique correspondant à un qubit virtuel donné.
    pub fn physical(&self, virtual_qubit: usize) -> u32 {
        self.mapping[virtual_qubit]
    }
}

/// Extrait les paires de qubits virtuels qui interagissent via une porte CX.
///
/// Retourne une liste dédoublonnée de paires `(a, b)` avec `a < b`,
/// utilisée par le placement pour garantir que ces paires sont voisines
/// dans le coupling map.
fn required_pairs(gates: &[DecomposedGate]) -> Vec<(usize, usize)> {
    let mut pairs = vec![];
    for gate in gates {
        if let DecomposedGate::CX(cx) = gate {
            let pair = if cx.control < cx.target {
                (cx.control, cx.target)
            } else {
                (cx.target, cx.control)
            };
            if !pairs.contains(&pair) {
                pairs.push(pair);
            }
        }
    }
    pairs
}

/// Assigne les qubits virtuels du circuit à des qubits physiques connectés.
///
/// Stratégie greedy en deux étapes :
/// 1. Pour chaque paire `(virt_a, virt_b)` ayant un CX, chercher une paire
///    physique connectée et disponible dans le coupling map.
/// 2. Assigner les qubits restants (sans CX) à n'importe quel qubit libre.
///
/// Retourne une erreur si le circuit contient plus de qubits que le backend,
/// ou si aucune paire physique connectée n'est disponible pour une paire CX.
///
/// # Exemple
/// ```rust
/// let layout = place(2, &decomposed_gates, &coupling_map)?;
/// println!("{:?}", layout.mapping); // [0, 1]
/// ```
pub fn place(
    n_virtual: usize,
    gates: &[DecomposedGate],
    coupling_map: &CouplingMap,
) -> Result<Layout, String> {
    if n_virtual == 0 {
        return Ok(Layout::new(vec![]));
    }
    if n_virtual > coupling_map.n_qubits as usize {
        return Err(format!(
            "Circuit trop grand : {} qubits virtuels pour {} physiques",
            n_virtual, coupling_map.n_qubits
        ));
    }

    let mut mapping: Vec<Option<u32>> = vec![None; n_virtual];
    let mut used_physical: std::collections::HashSet<u32> = std::collections::HashSet::new();

    let pairs = required_pairs(gates);

    for (virt_a, virt_b) in &pairs {
        if mapping[*virt_a].is_some() && mapping[*virt_b].is_some() {
            let pa = mapping[*virt_a].unwrap();
            let pb = mapping[*virt_b].unwrap();
            if !coupling_map.are_neighbors(pa, pb) {
                return Err(format!(
                    "Conflit : q[{}]→${} et q[{}]→${} ne sont pas connectés",
                    virt_a, pa, virt_b, pb
                ));
            }
            continue;
        }

        let found = find_connected_pair(
            mapping[*virt_a],
            mapping[*virt_b],
            coupling_map,
            &used_physical,
        );

        match found {
            Some((pa, pb)) => {
                if mapping[*virt_a].is_none() {
                    mapping[*virt_a] = Some(pa);
                    used_physical.insert(pa);
                }
                if mapping[*virt_b].is_none() {
                    mapping[*virt_b] = Some(pb);
                    used_physical.insert(pb);
                }
            }
            None => {
                return Err(format!(
                    "Aucune paire physique disponible et connectée pour q[{}] et q[{}]",
                    virt_a, virt_b
                ));
            }
        }
    }

    let mut free_physical: Vec<u32> = (0..coupling_map.n_qubits)
        .filter(|q| !used_physical.contains(q))
        .collect();

    for slot in mapping.iter_mut() {
        if slot.is_none() {
            let phys = free_physical.remove(0);
            *slot = Some(phys);
            used_physical.insert(phys);
        }
    }

    let final_mapping: Vec<u32> = mapping.into_iter().map(|s| s.unwrap()).collect();
    Ok(Layout::new(final_mapping))
}

/// Trouve une paire physique `(a, b)` connectée et non utilisée.
///
/// Si l'un des deux qubits est déjà assigné (`fixed_a` ou `fixed_b`),
/// cherche un voisin libre pour l'autre parmi les sorties puis les entrées du coupling map.
/// Si aucun n'est assigné, prend la première arête libre disponible.
fn find_connected_pair(
    fixed_a: Option<u32>,
    fixed_b: Option<u32>,
    coupling_map: &CouplingMap,
    used: &std::collections::HashSet<u32>,
) -> Option<(u32, u32)> {
    match (fixed_a, fixed_b) {
        (None, None) => {
            for &(a, b) in coupling_map.edges() {
                if !used.contains(&a) && !used.contains(&b) {
                    return Some((a, b));
                }
            }
            None
        }
        (Some(pa), None) => {
            for neighbor in coupling_map.neighbors(pa) {
                if !used.contains(&neighbor) {
                    return Some((pa, neighbor));
                }
            }
            for &(a, b) in coupling_map.edges() {
                if b == pa && !used.contains(&a) {
                    return Some((pa, a));
                }
            }
            None
        }
        (None, Some(pb)) => {
            for neighbor in coupling_map.neighbors(pb) {
                if !used.contains(&neighbor) {
                    return Some((neighbor, pb));
                }
            }
            for &(a, b) in coupling_map.edges() {
                if b == pb && !used.contains(&a) {
                    return Some((a, pb));
                }
            }
            None
        }
        (Some(pa), Some(pb)) => Some((pa, pb)),
    }
}

/// Problème de placement détecté lors de la vérification post-assignation.
///
/// Indique qu'une paire de qubits virtuels ayant une porte CX
/// a été assignée à des qubits physiques non connectés dans le coupling map.
/// Le router devra alors insérer des SWAPs pour corriger ce problème.
#[derive(Debug)]
pub struct PlacementIssue {
    /// Indice du qubit virtuel contrôle
    pub virtual_control: usize,
    /// Indice du qubit virtuel cible
    pub virtual_target: usize,
    /// Qubit physique assigné au contrôle
    pub physical_control: u32,
    /// Qubit physique assigné à la cible
    pub physical_target: u32,
}

/// Vérifie que toutes les portes CX du circuit sont sur des paires physiques connectées.
///
/// Retourne la liste des [`PlacementIssue`] détectés.
/// Si la liste est vide, aucun routing n'est nécessaire.
pub fn check_layout(
    gates: &[DecomposedGate],
    layout: &Layout,
    coupling_map: &CouplingMap,
) -> Vec<PlacementIssue> {
    let mut issues = vec![];
    for gate in gates {
        if let DecomposedGate::CX(cx) = gate {
            let phys_c = layout.physical(cx.control);
            let phys_t = layout.physical(cx.target);
            if !coupling_map.are_neighbors(phys_c, phys_t) {
                issues.push(PlacementIssue {
                    virtual_control:  cx.control,
                    virtual_target:   cx.target,
                    physical_control: phys_c,
                    physical_target:  phys_t,
                });
            }
        }
    }
    issues
}