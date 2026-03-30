use std::collections::HashMap;
use std::f64::consts::SQRT_2;
use num_complex::Complex64;
use crate::instruction::i_struct::IStruct;

type C = Complex64;

/// Résultat d'une simulation statevector.
///
/// Contient le vecteur d'état final et les probabilités exactes de chaque état de base.
#[derive(Debug)]
pub struct StatevectorResult {
    /// Vecteur d'état complexe — `amplitudes[i]` est l'amplitude de l'état `|i⟩`
    pub amplitudes: Vec<C>,
    /// Nombre de qubits simulés
    pub n_qubits: usize,
}

impl StatevectorResult {
    /// Retourne la probabilité exacte de mesurer l'état de base `index`.
    ///
    /// `P(i) = |amplitude[i]|²`
    pub fn probability(&self, index: usize) -> f64 {
        self.amplitudes[index].norm_sqr()
    }

    /// Retourne toutes les probabilités non nulles sous forme d'histogramme.
    ///
    /// Les clés sont des bitstrings (ex: `"00"`, `"11"`).
    /// Les états avec probabilité < `threshold` sont omis.
    pub fn probabilities(&self, threshold: f64) -> HashMap<String, f64> {
        let mut map = HashMap::new();
        for (i, amp) in self.amplitudes.iter().enumerate() {
            let prob = amp.norm_sqr();
            if prob > threshold {
                let bits = format!("{:0>width$b}", i, width = self.n_qubits);
                map.insert(bits, prob);
            }
        }
        map
    }

    /// Simule `shots` mesures en tirant aléatoirement selon les probabilités.
    ///
    /// Retourne un histogramme de counts, identique au format retourné par IBM.
    pub fn sample(&self, shots: usize) -> HashMap<String, u32> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut counts: HashMap<String, u32> = HashMap::new();

        // Construire la CDF (fonction de répartition) une seule fois
        let probs: Vec<f64> = self.amplitudes.iter().map(|a| a.norm_sqr()).collect();

        for _ in 0..shots {
            let r: f64 = rng.gen();
            let mut cumul = 0.0;
            let mut chosen = 0;
            for (i, &p) in probs.iter().enumerate() {
                cumul += p;
                if r <= cumul {
                    chosen = i;
                    break;
                }
            }
            let bits = format!("{:0>width$b}", chosen, width = self.n_qubits);
            *counts.entry(bits).or_insert(0) += 1;
        }

        counts
    }
}

/// Simule un circuit quantique en représentation statevector.
///
/// Le statevector est un vecteur de `2ⁿ` amplitudes complexes.
/// Chaque porte est appliquée comme un produit matriciel sur ce vecteur.
///
/// # Exemple
/// ```rust
/// use crate::instruction::i_struct::IStruct;
///
/// let instructions = vec![
///     IStruct::H(0),
///     IStruct::CX { control: 0, target: 1 },
///     IStruct::MEASURE(vec![0, 1], vec![0, 1]),
/// ];
///
/// let result = simulate_statevector(&instructions, 2)?;
///
/// // Probabilités exactes du Bell state
/// let probs = result.probabilities(1e-10);
/// // {"00": 0.5, "11": 0.5}
///
/// // Simulation de 1024 shots
/// let counts = result.sample(1024);
/// ```
pub fn simulate_statevector(
    instructions: &[IStruct],
    n_qubits: usize,
) -> Result<StatevectorResult, String> {
    let dim = 1 << n_qubits; // 2^n

    // État initial |00...0⟩
    let mut state = vec![C::new(0.0, 0.0); dim];
    state[0] = C::new(1.0, 0.0);

    for instruction in instructions {
        match instruction {
            IStruct::H(q)    => apply_single(&mut state, n_qubits, *q, &gate_h()),
            IStruct::X(q)    => apply_single(&mut state, n_qubits, *q, &gate_x()),
            IStruct::Y(q)    => apply_single(&mut state, n_qubits, *q, &gate_y()),
            IStruct::Z(q)    => apply_single(&mut state, n_qubits, *q, &gate_z()),

            IStruct::RX { angle, target } => apply_single(&mut state, n_qubits, *target, &gate_rx(*angle)),
            IStruct::RY { angle, target } => apply_single(&mut state, n_qubits, *target, &gate_ry(*angle)),
            IStruct::RZ { angle, target } => apply_single(&mut state, n_qubits, *target, &gate_rz(*angle)),

            IStruct::CX { control, target } => apply_cx(&mut state, n_qubits, *control, *target),
            IStruct::CZ { control, target } => apply_cz(&mut state, n_qubits, *control, *target),

            IStruct::SWAP { qbit1, qbit2 } => {
                // SWAP = CX(a,b) · CX(b,a) · CX(a,b)
                apply_cx(&mut state, n_qubits, *qbit1, *qbit2);
                apply_cx(&mut state, n_qubits, *qbit2, *qbit1);
                apply_cx(&mut state, n_qubits, *qbit1, *qbit2);
            }

            // MEASURE : dans le statevector on ignore — les probas sont calculées en fin
            IStruct::MEASURE(..) => {}

            IStruct::GATE { position, instruction, .. } => {
                // Décomposer récursivement avec remapping des positions
                let remapped = remap_instructions(instruction, position);
                let sub = simulate_statevector(&remapped, n_qubits)?;
                state = sub.amplitudes;
            }

            IStruct::U { matrix, target } => {
                // Porte arbitraire 1-qubit via sa matrice 2×2
                if target.len() == 1 {
                    let mat = extract_2x2(matrix)?;
                    apply_single(&mut state, n_qubits, target[0], &mat);
                } else {
                    eprintln!("Porte U multi-qubit non supportée");
                }
            }

            IStruct::CONTROLLED { controls, target, gate } => {
                apply_controlled(&mut state, n_qubits, controls, *target, gate)?;
            }

            IStruct::ANY() => {}
        }
    }

    Ok(StatevectorResult { amplitudes: state, n_qubits })
}

/// Applique une porte 1-qubit `mat` (2×2) au qubit `q` du statevector.
///
/// Itère sur tous les états de base par paires `(|...0...⟩, |...1...⟩)` au bit `q`.
fn apply_single(state: &mut Vec<C>, n: usize, q: usize, mat: &[[C; 2]; 2]) {
    let bit = 1 << (n - 1 - q); // masque du qubit q (MSB = qubit 0)
    let dim = state.len();

    let mut i = 0;
    while i < dim {
        if i & bit == 0 {
            let j = i | bit;
            let a = state[i];
            let b = state[j];
            state[i] = mat[0][0] * a + mat[0][1] * b;
            state[j] = mat[1][0] * a + mat[1][1] * b;
        }
        i += 1;
        // Sauter le prochain si on vient de traiter la paire
        if i & bit != 0 { i += bit; i -= bit; } // ← no-op, juste pour clarté
    }
}

/// Applique CX(control, target) au statevector.
///
/// Flip le bit `target` seulement quand le bit `control` est à 1.
fn apply_cx(state: &mut Vec<C>, n: usize, control: usize, target: usize) {
    let ctrl_bit   = 1 << (n - 1 - control);
    let target_bit = 1 << (n - 1 - target);
    let dim = state.len();

    for i in 0..dim {
        if i & ctrl_bit != 0 && i & target_bit == 0 {
            let j = i | target_bit;
            state.swap(i, j);
        }
    }
}

/// Applique CZ(control, target) au statevector.
///
/// Ajoute une phase -1 quand les deux bits sont à 1.
fn apply_cz(state: &mut Vec<C>, n: usize, control: usize, target: usize) {
    let ctrl_bit   = 1 << (n - 1 - control);
    let target_bit = 1 << (n - 1 - target);
    let dim = state.len();

    for i in 0..dim {
        if i & ctrl_bit != 0 && i & target_bit != 0 {
            state[i] = -state[i];
        }
    }
}

/// Applique une porte contrôlée générique.
fn apply_controlled(
    state: &mut Vec<C>,
    n: usize,
    controls: &[usize],
    target: usize,
    gate: &IStruct,
) -> Result<(), String> {
    match (controls.len(), gate) {
        (1, IStruct::X(_)) => { apply_cx(state, n, controls[0], target); Ok(()) }
        (1, IStruct::Z(_)) => { apply_cz(state, n, controls[0], target); Ok(()) }
        (2, IStruct::X(_)) => {
            // Toffoli : flip target si les deux contrôles sont à 1
            let c0 = 1 << (n - 1 - controls[0]);
            let c1 = 1 << (n - 1 - controls[1]);
            let t  = 1 << (n - 1 - target);
            let dim = state.len();
            for i in 0..dim {
                if i & c0 != 0 && i & c1 != 0 && i & t == 0 {
                    state.swap(i, i | t);
                }
            }
            Ok(())
        }
        _ => {
            eprintln!("CONTROLLED {:?} non supporté — ignoré", controls.len());
            Ok(())
        }
    }
}


fn c(re: f64, im: f64) -> C { C::new(re, im) }

fn gate_h() -> [[C; 2]; 2] {
    let s = 1.0 / SQRT_2;
    [[c(s, 0.0), c(s, 0.0)],
     [c(s, 0.0), c(-s, 0.0)]]
}

fn gate_x() -> [[C; 2]; 2] {
    [[c(0.0, 0.0), c(1.0, 0.0)],
     [c(1.0, 0.0), c(0.0, 0.0)]]
}

fn gate_y() -> [[C; 2]; 2] {
    [[c(0.0, 0.0), c(0.0, -1.0)],
     [c(0.0, 1.0), c(0.0,  0.0)]]
}

fn gate_z() -> [[C; 2]; 2] {
    [[c(1.0, 0.0), c(0.0,  0.0)],
     [c(0.0, 0.0), c(-1.0, 0.0)]]
}

fn gate_rx(theta: f64) -> [[C; 2]; 2] {
    let cos = (theta / 2.0).cos();
    let sin = (theta / 2.0).sin();
    [[c(cos, 0.0),   c(0.0, -sin)],
     [c(0.0, -sin),  c(cos,  0.0)]]
}

fn gate_ry(theta: f64) -> [[C; 2]; 2] {
    let cos = (theta / 2.0).cos();
    let sin = (theta / 2.0).sin();
    [[c(cos, 0.0), c(-sin, 0.0)],
     [c(sin, 0.0), c( cos, 0.0)]]
}

fn gate_rz(theta: f64) -> [[C; 2]; 2] {
    let half = theta / 2.0;
    [[c((-half).cos(), (-half).sin()), c(0.0, 0.0)],
     [c(0.0, 0.0),                    c(half.cos(), half.sin())]]
}


/// Extrait une matrice 2×2 depuis une `DMatrix<Complex32>`.
fn extract_2x2(matrix: &nalgebra::DMatrix<num_complex::Complex<f32>>) -> Result<[[C; 2]; 2], String> {
    if matrix.nrows() != 2 || matrix.ncols() != 2 {
        return Err(format!("Matrice U non 2×2 : {}×{}", matrix.nrows(), matrix.ncols()));
    }
    Ok([
        [c(matrix[(0,0)].re as f64, matrix[(0,0)].im as f64),
         c(matrix[(0,1)].re as f64, matrix[(0,1)].im as f64)],
        [c(matrix[(1,0)].re as f64, matrix[(1,0)].im as f64),
         c(matrix[(1,1)].re as f64, matrix[(1,1)].im as f64)],
    ])
}

/// Recrée des IStruct en remappant les qubits locaux → globaux pour les portes composites.
fn remap_instructions(instructions: &[Box<IStruct>], position: &[usize]) -> Vec<IStruct> {
    instructions.iter().map(|b| {
        let inst = *b.clone();
        match inst {
            IStruct::H(q)  => IStruct::H(position[q]),
            IStruct::X(q)  => IStruct::X(position[q]),
            IStruct::Y(q)  => IStruct::Y(position[q]),
            IStruct::Z(q)  => IStruct::Z(position[q]),
            IStruct::CX { control, target } => IStruct::CX {
                control: position[control],
                target:  position[target],
            },
            IStruct::RZ { angle, target } => IStruct::RZ { angle, target: position[target] },
            IStruct::RX { angle, target } => IStruct::RX { angle, target: position[target] },
            IStruct::RY { angle, target } => IStruct::RY { angle, target: position[target] },
            other => other,
        }
    }).collect()
}