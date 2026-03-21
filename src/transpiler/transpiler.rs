use crate::circuit::quantum_circuit::QuantumCircuit;
use crate::api::service::Service;
use crate::transpiler::coupling_map::CouplingMap;
use crate::transpiler::gate_decomposer::{decompose, DecomposedGate};
use crate::transpiler::placement::{place, check_layout};
use crate::transpiler::router::route;

/// Résultat d'une transpilation.
pub struct TranspileResult {
    /// Circuit QASM transpilé, prêt à être soumis à IBM Quantum
    pub qasm: String,
    /// Mapping final `virtuel → physique` (ex: `[0, 1]` signifie `q[0]→$0, q[1]→$1`)
    pub layout: Vec<u32>,
    /// Nombre de SWAPs insérés par le router pour corriger la connectivité
    pub n_swaps_inserted: usize,
}

/// Transpile un [`QuantumCircuit`] pour un backend donné.
///
/// Exécute le pipeline complet en quatre étapes :
/// 1. **Décomposition** — toutes les portes sont réduites en `U3 + CX`
/// 2. **Placement** — les qubits virtuels sont assignés à des qubits physiques connectés
/// 3. **Routing** — des SWAPs sont insérés si des portes CX tombent sur des paires non connectées
/// 4. **Génération QASM** — le circuit physique est sérialisé en OpenQASM 3.0,
///    avec la porte native adaptée au backend (`cz`, `ecr` ou `cx`)
///
/// Retourne une erreur si le placement ou le routing échouent.
///
/// # Exemple
/// ```rust
/// let coupling_map = service.get_coupling_map("ibm_fez").await?;
/// let result = transpile(&circuit, &coupling_map)?;
/// println!("{}", result.qasm);
/// ```
pub fn transpile(
    circuit: &QuantumCircuit,
    coupling_map: &CouplingMap,
) -> Result<TranspileResult, String> {

    let n_qubits = circuit.get_q_bits() as usize;
    let n_clbits = circuit.get_cl_bits() as usize;

    let decomposed = decompose(circuit.get_instructions());
    let layout = place(n_qubits, &decomposed, coupling_map)?;
    let issues = check_layout(&decomposed, &layout, coupling_map);
    let n_swaps_inserted;

    let routed: Vec<DecomposedGate> = if issues.is_empty() {
        n_swaps_inserted = 0;
        decomposed.into_iter()
            .map(|g| apply_layout(g, &layout.mapping))
            .collect()
    } else {
        n_swaps_inserted = issues.len();
        route(decomposed, &layout, coupling_map)
            .map_err(|e| format!("Routing échoué : {}", e))?
    };

    let two_qubit = coupling_map.two_qubit_gate();
    let identity: Vec<u32> = (0..coupling_map.n_qubits).collect();

    let mut qasm = String::new();
    qasm.push_str("OPENQASM 3.0;\n");
    qasm.push_str("include \"stdgates.inc\";\n");

    let extra = DecomposedGate::extra_header(&two_qubit);
    if !extra.is_empty() {
        qasm.push_str(extra);
    }

    qasm.push_str(&format!("bit[{}] c;\n", n_clbits));
    qasm.push_str("// Layout : ");
    for (virt, &phys) in layout.mapping.iter().enumerate() {
        qasm.push_str(&format!("q[{}]→${} ", virt, phys));
    }
    qasm.push('\n');

    for gate in &routed {
        qasm.push_str(&gate.to_qasm(&identity, &two_qubit));
        qasm.push('\n');
    }

    Ok(TranspileResult {
        qasm,
        layout: layout.mapping,
        n_swaps_inserted,
    })
}

/// Applique un layout statique à une porte décomposée.
///
/// Traduit les indices virtuels en indices physiques via `mapping`.
/// Utilisé quand aucun routing n'est nécessaire.
fn apply_layout(gate: DecomposedGate, mapping: &[u32]) -> DecomposedGate {
    match gate {
        DecomposedGate::U3(mut u) => {
            u.target   = mapping[u.target] as usize;
            DecomposedGate::U3(u)
        }
        DecomposedGate::CX(mut cx) => {
            cx.control = mapping[cx.control] as usize;
            cx.target  = mapping[cx.target]  as usize;
            DecomposedGate::CX(cx)
        }
        DecomposedGate::Measure(mut m) => {
            m.qubit    = mapping[m.qubit] as usize;
            DecomposedGate::Measure(m)
        }
    }
}

impl Service {
    /// Récupère le coupling map du backend puis transpile le circuit.
    ///
    /// Combine [`get_coupling_map`] et [`transpile`] en un seul appel.
    ///
    /// [`get_coupling_map`]: Service::get_coupling_map
    ///
    /// # Exemple
    /// ```rust
    /// let result = service.transpile_circuit(&circuit, "ibm_fez").await?;
    /// println!("{}", result.qasm);
    /// ```
    pub async fn transpile_circuit(
        &self,
        circuit: &QuantumCircuit,
        backend_name: &str,
    ) -> Result<TranspileResult, Box<dyn std::error::Error>> {
        let coupling_map = self.get_coupling_map(backend_name).await?;
        let result = transpile(circuit, &coupling_map)
            .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

        Ok(result)
    }
}