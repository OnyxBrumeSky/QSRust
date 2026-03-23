use std::f64::consts::PI;
use crate::instruction::i_struct::IStruct;
use crate::transpiler::coupling_map::TwoQubitGate;

/// Porte universelle à 1 qubit `U3(θ, φ, λ)`.
///
/// Toute porte à 1 qubit peut s'exprimer sous cette forme.
/// Elle est traduite en `rz · sx · rz · sx · rz` lors de la génération QASM.
#[derive(Debug, Clone)]
pub struct U3Gate {
    /// Angle de rotation θ
    pub theta: f64,
    /// Angle de phase φ
    pub phi: f64,
    /// Angle de rotation λ
    pub lambda: f64,
    /// Indice du qubit cible
    pub target: usize,
}

/// Porte CNOT à 2 qubits.
#[derive(Debug, Clone)]
pub struct CXGate {
    /// Indice du qubit contrôle
    pub control: usize,
    /// Indice du qubit cible
    pub target: usize,
}

/// Mesure d'un qubit vers un bit classique.
#[derive(Debug, Clone)]
pub struct MeasureGate {
    /// Indice du qubit à mesurer
    pub qubit: usize,
    /// Indice du bit classique de destination
    pub clbit: usize,
}

/// Représente une porte après décomposition.
///
/// Toutes les portes du circuit sont réduites à cette forme avant
/// le placement, le routing et la génération QASM.
#[derive(Debug, Clone)]
pub enum DecomposedGate {
    /// Porte universelle à 1 qubit
    U3(U3Gate),
    /// Porte CNOT à 2 qubits
    CX(CXGate),
    /// Mesure classique
    Measure(MeasureGate),
}

impl DecomposedGate {
    /// Génère la représentation QASM de la porte avec les qubits physiques du layout.
    ///
    /// La décomposition de `CX` varie selon la [`TwoQubitGate`] native du backend :
    /// - [`TwoQubitGate::CX`] → `cx $c, $t;`
    /// - [`TwoQubitGate::CZ`] → `H(t) · CZ(c,t) · H(t)` via `rz/sx` : `U3(θ,φ,λ) = RZ(λ) · SX · RZ(θ+π) · SX · RZ(φ)`
    /// - [`TwoQubitGate::ECR`] → `RZ(-π/2,c) · SX(c) · RZ(π/2,t) · ECR(c,t) · SX(t)` (nécessite [`extra_header`])
    ///
    /// [`extra_header`]: DecomposedGate::extra_header
    pub fn to_qasm(&self, layout: &[u32], two_qubit: &TwoQubitGate) -> String {
        match self {
            DecomposedGate::U3(u) => {
                let p = layout[u.target];
                format!(
                    "rz({:.6}) ${};\nsx ${};\nrz({:.6}) ${};\nsx ${};\nrz({:.6}) ${};",
                    u.lambda,      p,
                    p,
                    u.theta + PI,  p,
                    p,
                    u.phi,         p,
                )
            }

            DecomposedGate::CX(cx) => {
                let c = layout[cx.control] as usize;
                let t = layout[cx.target]  as usize;

                match two_qubit {
                    TwoQubitGate::CX => format!("cx ${}, ${};", c, t),
                    TwoQubitGate::CZ => format!(
                        "rz(pi/2) ${};\nsx ${};\nrz(pi/2) ${};\ncz ${}, ${};\nrz(pi/2) ${};\nsx ${};\nrz(pi/2) ${};",
                        t, t, t,
                        c, t,
                        t, t, t,
                    ),
                    TwoQubitGate::ECR => format!(
                        "rz(-pi/2) ${};\nsx ${};\nrz(pi/2) ${};\necr ${}, ${};\nsx ${};",
                        c, c,
                        t,
                        c, t,
                        t,
                    ),
                }
            }

            DecomposedGate::Measure(m) => {
                let p = layout[m.qubit];
                format!("c[{}] = measure ${};", m.clbit, p)
            }
        }
    }

    /// Retourne un header QASM supplémentaire si la porte native nécessite une définition.
    ///
    /// Actuellement utilisé pour [`TwoQubitGate::ECR`], absent de `stdgates.inc`.
    pub fn extra_header(two_qubit: &TwoQubitGate) -> &'static str {
        match two_qubit {
            TwoQubitGate::ECR => "gate ecr q0, q1 { rzx(pi/4) q0, q1; x q0; rzx(-pi/4) q0, q1; }\n",
            _ => "",
        }
    }
}

/// Construit une [`DecomposedGate::U3`] avec les paramètres donnés.
fn u3(theta: f64, phi: f64, lambda: f64, target: usize) -> DecomposedGate {
    DecomposedGate::U3(U3Gate { theta, phi, lambda, target })
}

/// Construit une [`DecomposedGate::CX`] entre deux qubits.
fn cx(control: usize, target: usize) -> DecomposedGate {
    DecomposedGate::CX(CXGate { control, target })
}

/// Décompose `H` en `U3(π/2, 0, π)`.
pub fn decompose_h(target: usize)              -> Vec<DecomposedGate> { vec![u3(PI/2.0, 0.0, PI, target)] }
/// Décompose `X` en `U3(π, 0, π)`.
pub fn decompose_x(target: usize)              -> Vec<DecomposedGate> { vec![u3(PI, 0.0, PI, target)] }
/// Décompose `Y` en `U3(π, π/2, π/2)`.
pub fn decompose_y(target: usize)              -> Vec<DecomposedGate> { vec![u3(PI, PI/2.0, PI/2.0, target)] }
/// Décompose `Z` en `U3(0, 0, π)`.
pub fn decompose_z(target: usize)              -> Vec<DecomposedGate> { vec![u3(0.0, 0.0, PI, target)] }
/// Décompose `T` en `U3(0, 0, π/4)`.
pub fn decompose_t(target: usize)              -> Vec<DecomposedGate> { vec![u3(0.0, 0.0, PI/4.0, target)] }
/// Décompose `T†` en `U3(0, 0, -π/4)`.
pub fn decompose_tdg(target: usize)            -> Vec<DecomposedGate> { vec![u3(0.0, 0.0, -PI/4.0, target)] }
/// Décompose `RZ(angle)` en `U3(0, 0, angle)`.
pub fn decompose_rz(angle: f64, target: usize) -> Vec<DecomposedGate> { vec![u3(0.0, 0.0, angle, target)] }
/// Décompose `RX(angle)` en `U3(angle, -π/2, π/2)`.
pub fn decompose_rx(angle: f64, target: usize) -> Vec<DecomposedGate> { vec![u3(angle, -PI/2.0, PI/2.0, target)] }
/// Décompose `RY(angle)` en `U3(angle, 0, 0)`.
pub fn decompose_ry(angle: f64, target: usize) -> Vec<DecomposedGate> { vec![u3(angle, 0.0, 0.0, target)] }

/// Décompose `SWAP` en trois `CX` : `CX(a,b) · CX(b,a) · CX(a,b)`.
pub fn decompose_swap(q1: usize, q2: usize) -> Vec<DecomposedGate> {
    vec![cx(q1, q2), cx(q2, q1), cx(q1, q2)]
}

/// Décompose `CZ` en `H(t) · CX(c,t) · H(t)`.
///
/// `CZ` n'est pas une porte primitive de [`DecomposedGate`] — elle est toujours
/// réduite à une séquence `H · CX · H` avant le placement et le routing.
pub fn decompose_cz(control: usize, target: usize) -> Vec<DecomposedGate> {
    let mut g = decompose_h(target);
    g.push(cx(control, target));
    g.extend(decompose_h(target));
    g
}

/// Décompose la porte de Toffoli (CCX) en 6 CX selon la décomposition standard.
pub fn decompose_toffoli(c1: usize, c2: usize, target: usize) -> Vec<DecomposedGate> {
    let mut g = vec![];
    g.extend(decompose_h(target));
    g.push(cx(c2, target));
    g.extend(decompose_tdg(target));
    g.push(cx(c1, target));
    g.extend(decompose_t(target));
    g.push(cx(c2, target));
    g.extend(decompose_tdg(target));
    g.push(cx(c1, target));
    g.extend(decompose_t(c2));
    g.extend(decompose_t(target));
    g.push(cx(c1, c2));
    g.extend(decompose_h(target));
    g.extend(decompose_t(c1));
    g.extend(decompose_tdg(c2));
    g.push(cx(c1, c2));
    g
}

/// Décompose une porte `CONTROLLED` selon le nombre de contrôles et la porte cible.
///
/// - 1 contrôle + `X` → `CX`
/// - 1 contrôle + `Z` → `CZ`
/// - 2 contrôles + `X` → Toffoli
/// - Autres cas → ignorés avec un avertissement sur stderr
fn decompose_controlled(controls: &[usize], target: usize, gate: &IStruct) -> Vec<DecomposedGate> {
    match (controls.len(), gate) {
        (1, IStruct::X(_)) => vec![cx(controls[0], target)],
        (1, IStruct::Z(_)) => decompose_cz(controls[0], target),
        (2, IStruct::X(_)) => decompose_toffoli(controls[0], controls[1], target),
        _ => {
            eprintln!("CONTROLLED non supporté : {} contrôle(s) — ignoré", controls.len());
            vec![]
        }
    }
}

/// Décompose une liste d'instructions [`IStruct`] en [`DecomposedGate`].
///
/// Chaque porte est réduite à une combinaison de [`U3Gate`] et [`CXGate`].
/// Les portes composites (`GATE`) sont décomposées récursivement avec remapping
/// des indices locaux vers les indices globaux via `position`.
/// Les portes non supportées (`U` matricielle, cas `CONTROLLED` complexes) sont
/// ignorées avec un avertissement sur stderr.
pub fn decompose(instructions: &[IStruct]) -> Vec<DecomposedGate> {
    let mut result = vec![];

    for instruction in instructions {
        let gates: Vec<DecomposedGate> = match instruction {
            IStruct::H(q)                           => decompose_h(*q),
            IStruct::X(q)                           => decompose_x(*q),
            IStruct::Y(q)                           => decompose_y(*q),
            IStruct::Z(q)                           => decompose_z(*q),
            IStruct::CX { control, target }         => vec![cx(*control, *target)],
            // ✅ CZ décomposé en H · CX · H
            IStruct::CZ { control, target }         => decompose_cz(*control, *target),
            IStruct::SWAP { qbit1, qbit2 }          => decompose_swap(*qbit1, *qbit2),
            IStruct::RZ { angle, target }           => decompose_rz(*angle, *target),
            IStruct::RX { angle, target }           => decompose_rx(*angle, *target),
            IStruct::RY { angle, target }           => decompose_ry(*angle, *target),
            IStruct::CONTROLLED { controls, target, gate } => {
                decompose_controlled(controls, *target, gate)
            }
            IStruct::GATE { position, instruction, .. } => {
                let inner: Vec<IStruct> = instruction.iter().map(|b| *b.clone()).collect();
                decompose(&inner).into_iter().map(|g| remap_gate(g, position)).collect()
            }
            IStruct::MEASURE(q_bits, cl_bits) => {
                q_bits.iter().zip(cl_bits.iter())
                    .map(|(&q, &c)| DecomposedGate::Measure(MeasureGate { qubit: q, clbit: c }))
                    .collect()
            }
            IStruct::U { target, .. } => {
                eprintln!("Porte U (matrice arbitraire) sur {:?} — non supportée", target);
                vec![]
            }
            IStruct::ANY() => vec![],
        };

        result.extend(gates);
    }

    result
}

/// Remet à jour les indices de qubits d'une porte selon un mapping `position`.
///
/// Utilisé lors de la décomposition récursive des portes composites (`GATE`),
/// pour traduire les indices locaux `0, 1, 2...` en indices globaux du circuit.
fn remap_gate(gate: DecomposedGate, position: &[usize]) -> DecomposedGate {
    match gate {
        DecomposedGate::U3(mut u) => { u.target   = position[u.target];   DecomposedGate::U3(u) }
        DecomposedGate::CX(mut c) => { c.control  = position[c.control];
                                        c.target   = position[c.target];   DecomposedGate::CX(c) }
        DecomposedGate::Measure(mut m) => { m.qubit = position[m.qubit];   DecomposedGate::Measure(m) }
    }
}