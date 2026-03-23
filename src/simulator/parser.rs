use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;
use std::f64::consts::PI;
use crate::instruction::i_struct::IStruct;

/// Parser généré par pest depuis la grammaire `qasm.pest`.
#[derive(Parser)]
#[grammar = "simulator/qasm.pest"]
pub struct QasmParser;

/// Erreur de parsing QASM.
#[derive(Debug)]
pub enum QasmError {
    /// Erreur de syntaxe détectée par pest
    ParseError(String),
    /// Instruction reconnue mais non supportée
    Unsupported(String),
}

impl std::fmt::Display for QasmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QasmError::ParseError(s)  => write!(f, "Erreur QASM : {}", s),
            QasmError::Unsupported(s) => write!(f, "Non supporté : {}", s),
        }
    }
}

impl std::error::Error for QasmError {}

/// Parse un programme OpenQASM 3 et retourne une liste d'[`IStruct`].
///
/// Les déclarations (`OPENQASM`, `include`, `qubit[n]`, `bit[n]`, `gate`)
/// sont ignorées — seules les instructions quantiques sont converties.
///
/// # Exemple
/// ```rust
/// let qasm = "OPENQASM 3.0;\ninclude \"stdgates.inc\";\nbit[2] c;\nh $0;\ncx $0, $1;\nc[0] = measure $0;\n";
/// let instructions = parse_qasm(qasm)?;
/// ```
pub fn parse_qasm(source: &str) -> Result<Vec<IStruct>, QasmError> {
    let pairs = QasmParser::parse(Rule::program, source)
        .map_err(|e| QasmError::ParseError(e.to_string()))?;

    let mut instructions = vec![];

    for pair in pairs.into_iter().next().unwrap().into_inner() {
        match pair.as_rule() {
            Rule::instruction => {
                if let Some(inst) = parse_instruction(pair)? {
                    instructions.push(inst);
                }
            }
            Rule::version | Rule::include | Rule::decl_qubit |
            Rule::decl_bit | Rule::decl_gate | Rule::EOI => {}
            _ => {}
        }
    }

    Ok(instructions)
}

/// Résultat complet du parsing d'un programme QASM.
pub struct ParseResult {
    /// Instructions quantiques parsées
    pub instructions: Vec<IStruct>,
    /// Nombre de qubits déduit automatiquement
    pub n_qubits: usize,
    /// Nombre de bits classiques déduit automatiquement
    pub n_clbits: usize,
}

/// Déduit le nombre de bits classiques depuis les instructions.
pub fn count_clbits(instructions: &[IStruct]) -> usize {
    let mut max = 0usize;
    for inst in instructions {
        if let IStruct::MEASURE(_, cl_bits) = inst {
            if let Some(&m) = cl_bits.iter().max() {
                if m > max { max = m; }
            }
        }
    }
    if max == 0 && instructions.iter().any(|i| matches!(i, IStruct::MEASURE(..))) {
        1
    } else {
        max + 1
    }
}

/// Parse et déduit automatiquement les dimensions du circuit.
pub fn parse_qasm_full(source: &str) -> Result<ParseResult, QasmError> {
    let instructions = parse_qasm(source)?;
    let n_qubits = count_qubits(&instructions);
    let n_clbits = count_clbits(&instructions);
    Ok(ParseResult { instructions, n_qubits, n_clbits })
}

fn parse_instruction(pair: Pair<Rule>) -> Result<Option<IStruct>, QasmError> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::gate_h    => Ok(Some(IStruct::H(parse_qubit(inner)?))),
        Rule::gate_x    => Ok(Some(IStruct::X(parse_qubit(inner)?))),
        Rule::gate_y    => Ok(Some(IStruct::Y(parse_qubit(inner)?))),
        Rule::gate_z    => Ok(Some(IStruct::Z(parse_qubit(inner)?))),
        Rule::gate_sx   => Ok(Some(parse_sx(inner)?)),
        Rule::gate_id   => Ok(None),

        Rule::gate_rx   => Ok(Some(parse_rotation(inner, "rx")?)),
        Rule::gate_ry   => Ok(Some(parse_rotation(inner, "ry")?)),
        Rule::gate_rz   => Ok(Some(parse_rotation(inner, "rz")?)),

        Rule::gate_cx   => Ok(Some(parse_two_qubit_cx(inner)?)),
        // ✅ CZ retourne maintenant IStruct::CZ au lieu de CX
        Rule::gate_cz   => Ok(Some(parse_two_qubit_cz(inner)?)),
        Rule::gate_swap => Ok(Some(parse_swap(inner)?)),
        Rule::gate_ecr  => {
            eprintln!("ECR ignoré — non représentable dans IStruct");
            Ok(None)
        }

        Rule::measure      => Ok(Some(parse_measure(inner)?)),
        Rule::gate_custom  => Ok(Some(parse_custom(inner)?)),

        r => Err(QasmError::Unsupported(format!("{:?}", r))),
    }
}

/// Extrait l'indice d'un qubit depuis `qubit_ref` (physique `$N` ou registre `q[N]`).
fn parse_qubit_ref(pair: Pair<Rule>) -> Result<usize, QasmError> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::phys_qubit => {
            let s = inner.as_str().trim_start_matches('$');
            Ok(s.parse::<usize>().unwrap())
        }
        Rule::reg_qubit => {
            let idx = inner.into_inner()
                .find(|p| p.as_rule() == Rule::uint)
                .unwrap()
                .as_str()
                .parse::<usize>()
                .unwrap();
            Ok(idx)
        }
        r => Err(QasmError::ParseError(format!("qubit attendu, trouvé {:?}", r))),
    }
}

/// Extrait le premier qubit_ref d'une instruction à 1 qubit.
fn parse_qubit(pair: Pair<Rule>) -> Result<usize, QasmError> {
    let qref = pair.into_inner()
        .find(|p| p.as_rule() == Rule::qubit_ref)
        .unwrap();
    parse_qubit_ref(qref)
}

/// Extrait deux qubit_ref depuis une instruction à 2 qubits.
fn parse_two_qubits(pair: Pair<Rule>) -> Result<(usize, usize), QasmError> {
    let mut qubits = pair.into_inner()
        .filter(|p| p.as_rule() == Rule::qubit_ref);
    let a = parse_qubit_ref(qubits.next().unwrap())?;
    let b = parse_qubit_ref(qubits.next().unwrap())?;
    Ok((a, b))
}

/// Évalue une `angle_expr` en radians (f64).
///
/// Supporte : `pi`, `3.14`, `-pi/2`, `pi*2`, `2*pi/3`.
fn parse_angle(pair: Pair<Rule>) -> Result<f64, QasmError> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::pi    => Ok(PI),
        Rule::float => Ok(inner.as_str().parse::<f64>().unwrap()),
        Rule::angle_neg => {
            let atom = inner.into_inner().next().unwrap();
            Ok(-eval_angle_atom(atom))
        }
        Rule::angle_div => {
            let mut parts = inner.into_inner();
            let num = eval_angle_atom(parts.next().unwrap());
            let den = parts.next().unwrap().as_str().parse::<f64>().unwrap();
            Ok(num / den)
        }
        Rule::angle_mul => {
            let mut parts = inner.into_inner();
            let a = parts.next().unwrap().as_str().parse::<f64>().unwrap();
            Ok(a * PI)
        }
        r => Err(QasmError::ParseError(format!("angle invalide : {:?}", r))),
    }
}

fn eval_angle_atom(pair: Pair<Rule>) -> f64 {
    match pair.as_rule() {
        Rule::pi    => PI,
        Rule::float => pair.as_str().parse::<f64>().unwrap(),
        _           => 0.0,
    }
}

/// SX = RX(π/2)
fn parse_sx(pair: Pair<Rule>) -> Result<IStruct, QasmError> {
    let target = parse_qubit(pair)?;
    Ok(IStruct::RX { angle: PI / 2.0, target })
}

fn parse_rotation(pair: Pair<Rule>, gate: &str) -> Result<IStruct, QasmError> {
    let mut inner = pair.into_inner();
    let angle_pair = inner.find(|p| p.as_rule() == Rule::angle_expr).unwrap();
    let angle = parse_angle(angle_pair)?;
    let qref  = inner.find(|p| p.as_rule() == Rule::qubit_ref).unwrap();
    let target = parse_qubit_ref(qref)?;

    match gate {
        "rx" => Ok(IStruct::RX { angle, target }),
        "ry" => Ok(IStruct::RY { angle, target }),
        "rz" => Ok(IStruct::RZ { angle, target }),
        _    => unreachable!(),
    }
}

fn parse_two_qubit_cx(pair: Pair<Rule>) -> Result<IStruct, QasmError> {
    let (control, target) = parse_two_qubits(pair)?;
    Ok(IStruct::CX { control, target })
}

/// Parse `cz $a, $b;` en [`IStruct::CZ`].
///
/// La décomposition `H · CX · H` est effectuée plus tard par [`decompose_cz`]
/// dans le `gate_decomposer`, ce qui permet de garder `CZ` intact
/// pour la simulation statevector (qui peut l'appliquer directement).
///
/// [`decompose_cz`]: crate::transpiler::gate_decomposer::decompose_cz
fn parse_two_qubit_cz(pair: Pair<Rule>) -> Result<IStruct, QasmError> {
    let (control, target) = parse_two_qubits(pair)?;
    Ok(IStruct::CZ { control, target })
}

fn parse_swap(pair: Pair<Rule>) -> Result<IStruct, QasmError> {
    let (qbit1, qbit2) = parse_two_qubits(pair)?;
    Ok(IStruct::SWAP { qbit1, qbit2 })
}

fn parse_measure(pair: Pair<Rule>) -> Result<IStruct, QasmError> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::measure_arrow => {
            let mut parts = inner.into_inner();
            let qref  = parts.next().unwrap();
            let clref = parts.next().unwrap();
            let q = parse_qubit_ref(qref)?;
            let c = parse_clbit(clref)?;
            Ok(IStruct::MEASURE(vec![q], vec![c]))
        }
        Rule::measure_eq => {
            let mut parts = inner.into_inner();
            let clref = parts.next().unwrap();
            let qref  = parts.next().unwrap();
            let c = parse_clbit(clref)?;
            let q = parse_qubit_ref(qref)?;
            Ok(IStruct::MEASURE(vec![q], vec![c]))
        }
        r => Err(QasmError::ParseError(format!("mesure invalide : {:?}", r))),
    }
}

/// Extrait l'indice d'un bit classique `c[N]`.
fn parse_clbit(pair: Pair<Rule>) -> Result<usize, QasmError> {
    let idx = pair.into_inner()
        .find(|p| p.as_rule() == Rule::uint)
        .unwrap()
        .as_str()
        .parse::<usize>()
        .unwrap();
    Ok(idx)
}

/// Parse une porte custom paramétrée ou non.
///
/// Ex: `mygate(pi/2) $0;` → `IStruct::GATE`
fn parse_custom(pair: Pair<Rule>) -> Result<IStruct, QasmError> {
    let mut inner = pair.into_inner();
    let label = inner.next().unwrap().as_str().to_string();

    let mut angles = vec![];
    let mut qubits = vec![];

    for part in inner {
        match part.as_rule() {
            Rule::gate_params => {
                for angle_pair in part.into_inner() {
                    angles.push(parse_angle(angle_pair)?);
                }
            }
            Rule::gate_qubits => {
                for qref in part.into_inner() {
                    qubits.push(parse_qubit_ref(qref)?);
                }
            }
            _ => {}
        }
    }

    Ok(IStruct::GATE {
        position:    qubits,
        instruction: vec![],
        label,
    })
}



/// Déduit le nombre de qubits nécessaires depuis une liste d'[`IStruct`].
///
/// Scanne toutes les instructions et retourne `max_qubit_index + 1`.
/// Retourne `1` si aucune instruction ne référence de qubit.
pub fn count_qubits(instructions: &[IStruct]) -> usize {
    let mut max = 0usize;

    for inst in instructions {
        let indices: Vec<usize> = match inst {
            IStruct::H(q) | IStruct::X(q) | IStruct::Y(q) | IStruct::Z(q) => vec![*q],
            IStruct::RX { target, .. } | IStruct::RY { target, .. } | IStruct::RZ { target, .. } => vec![*target],
            IStruct::CX { control, target } | IStruct::CZ { control, target } => vec![*control, *target],
            IStruct::SWAP { qbit1, qbit2 } => vec![*qbit1, *qbit2],
            IStruct::MEASURE(q_bits, _) => q_bits.clone(),
            IStruct::CONTROLLED { controls, target, .. } => {
                let mut v = controls.clone();
                v.push(*target);
                v
            }
            IStruct::GATE { position, .. } => position.clone(),
            IStruct::U { target, .. } => target.clone(),
            IStruct::ANY() => vec![],
        };

        if let Some(&m) = indices.iter().max() {
            if m > max { max = m; }
        }
    }

    max + 1
}