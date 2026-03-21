use crate::transpiler::coupling_map::CouplingMap;
use crate::transpiler::gate_decomposer::{DecomposedGate, CXGate};
use crate::transpiler::placement::Layout;

/// Layout mutable qui évolue au fil de l'insertion des SWAPs.
///
/// Maintient deux tables de correspondance synchronisées :
/// - `virt_to_phys` : qubit virtuel → qubit physique courant
/// - `phys_to_virt` : qubit physique → qubit virtuel courant (ou `None` si libre)
///
/// Chaque SWAP physique mis à jour est reflété dans les deux tables.
#[derive(Debug, Clone)]
struct DynamicLayout {
    virt_to_phys: Vec<u32>,
    phys_to_virt: Vec<Option<usize>>,
    _n_physical: usize,
}

impl DynamicLayout {
    /// Initialise depuis un [`Layout`] statique.
    fn from_layout(layout: &Layout, _n_physical: usize) -> Self {
        let mut phys_to_virt = vec![None; _n_physical];
        for (virt, &phys) in layout.mapping.iter().enumerate() {
            phys_to_virt[phys as usize] = Some(virt);
        }
        Self {
            virt_to_phys: layout.mapping.clone(),
            phys_to_virt,
            _n_physical,
        }
    }

    /// Retourne le qubit physique courant d'un qubit virtuel.
    fn physical(&self, virt: usize) -> u32 {
        self.virt_to_phys[virt]
    }

    /// Applique un SWAP physique entre deux qubits et met à jour les deux tables.
    fn apply_swap(&mut self, phys_a: u32, phys_b: u32) {
        let virt_a = self.phys_to_virt[phys_a as usize];
        let virt_b = self.phys_to_virt[phys_b as usize];

        self.phys_to_virt[phys_a as usize] = virt_b;
        self.phys_to_virt[phys_b as usize] = virt_a;

        if let Some(va) = virt_a { self.virt_to_phys[va] = phys_b; }
        if let Some(vb) = virt_b { self.virt_to_phys[vb] = phys_a; }
    }
}

/// Insère des SWAPs pour corriger les portes CX sur des paires non connectées.
///
/// Pour chaque porte CX du circuit, trois cas sont traités :
/// 1. **Connectés** — la porte est émise telle quelle.
/// 2. **Connexion inverse** — la porte est inversée via `H · CX(t,c) · H`.
/// 3. **Non connectés** — le chemin BFS le plus court est calculé et des SWAPs
///    sont insérés pour rapprocher le qubit contrôle du qubit cible.
///    Le [`DynamicLayout`] est mis à jour après chaque SWAP.
///
/// Les portes `U3` et `Measure` sont simplement remappées selon le layout courant.
///
/// Retourne une erreur si aucun chemin n'existe entre deux qubits dans le coupling map.
pub fn route(
    gates: Vec<DecomposedGate>,
    layout: &Layout,
    coupling_map: &CouplingMap,
) -> Result<Vec<DecomposedGate>, String> {
    let n_physical = coupling_map.n_qubits as usize;
    let mut dyn_layout = DynamicLayout::from_layout(layout, n_physical);
    let mut output: Vec<DecomposedGate> = vec![];

    for gate in gates {
        match &gate {
            DecomposedGate::CX(cx) => {
                let phys_c = dyn_layout.physical(cx.control);
                let phys_t = dyn_layout.physical(cx.target);

                if coupling_map.is_connected(phys_c, phys_t) {
                    output.push(DecomposedGate::CX(CXGate {
                        control: phys_c as usize,
                        target:  phys_t as usize,
                    }));
                } else if coupling_map.is_connected(phys_t, phys_c) {
                    output.extend(emit_reverse_cx(phys_c, phys_t));
                } else {
                    let path = coupling_map
                        .shortest_path(phys_c, phys_t)
                        .ok_or_else(|| format!(
                            "Aucun chemin entre ${} et ${} dans le coupling map",
                            phys_c, phys_t
                        ))?;

                    for window in path.windows(2).take(path.len() - 2) {
                        let a = window[0];
                        let b = window[1];
                        output.extend(emit_swap(a, b));
                        dyn_layout.apply_swap(a, b);
                    }

                    let new_phys_c = dyn_layout.physical(cx.control);
                    let new_phys_t = dyn_layout.physical(cx.target);

                    if coupling_map.is_connected(new_phys_c, new_phys_t) {
                        output.push(DecomposedGate::CX(CXGate {
                            control: new_phys_c as usize,
                            target:  new_phys_t as usize,
                        }));
                    } else if coupling_map.is_connected(new_phys_t, new_phys_c) {
                        output.extend(emit_reverse_cx(new_phys_c, new_phys_t));
                    } else {
                        return Err(format!(
                            "Routing échoué : impossible de connecter ${} → ${}",
                            new_phys_c, new_phys_t
                        ));
                    }
                }
            }

            DecomposedGate::U3(u) => {
                let mut updated = u.clone();
                updated.target = dyn_layout.physical(u.target) as usize;
                output.push(DecomposedGate::U3(updated));
            }

            DecomposedGate::Measure(m) => {
                let mut updated = m.clone();
                updated.qubit = dyn_layout.physical(m.qubit) as usize;
                output.push(DecomposedGate::Measure(updated));
            }
        }
    }

    Ok(output)
}

/// Émet un SWAP physique entre deux qubits sous forme de trois CX.
///
/// `SWAP(a, b) = CX(a,b) · CX(b,a) · CX(a,b)`
fn emit_swap(a: u32, b: u32) -> Vec<DecomposedGate> {
    vec![
        DecomposedGate::CX(CXGate { control: a as usize, target: b as usize }),
        DecomposedGate::CX(CXGate { control: b as usize, target: a as usize }),
        DecomposedGate::CX(CXGate { control: a as usize, target: b as usize }),
    ]
}

/// Inverse le sens d'une porte CX quand seule la connexion `target → control` existe.
///
/// Identité : `CX(c,t) = H(c) · H(t) · CX(t,c) · H(c) · H(t)`
fn emit_reverse_cx(phys_c: u32, phys_t: u32) -> Vec<DecomposedGate> {
    use crate::transpiler::gate_decomposer::U3Gate;
    use std::f64::consts::PI;

    let h = |q: u32| DecomposedGate::U3(U3Gate {
        theta: PI / 2.0, phi: 0.0, lambda: PI,
        target: q as usize,
    });

    vec![
        h(phys_c),
        h(phys_t),
        DecomposedGate::CX(CXGate {
            control: phys_t as usize,
            target:  phys_c as usize,
        }),
        h(phys_c),
        h(phys_t),
    ]
}