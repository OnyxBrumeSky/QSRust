use crate::circuit::quantum_circuit::QuantumCircuit;
use crate::instruction::i_struct::IStruct;

pub trait ToQasm{
    /// Convertit l'objet en une représentation de type QASM.
    ///
    /// # Arguments
    ///
    /// * `position` - Un vecteur contenant les indices des qubits sur lesquels l'instruction doit s'appliquer.
    /// * `label` - Une chaîne de caractères optionnelle représentant un label ou un nom pour l'instruction.
    ///
    /// # Retourne
    ///
    /// Une chaîne de caractères représentant l'instruction au format QASM.
    fn to_qasm(&self) -> String;
}


impl ToQasm for IStruct {
    fn to_qasm(&self) -> String {
        match self {
            IStruct::H(qbits) => format!("h q[{}];", qbits),
            IStruct::X(qbits) => format!("x q[{}];", qbits),
            IStruct::Y(qbits) => format!("y q[{}];", qbits),
            IStruct::Z(qbits) => format!("z q[{}];", qbits),
            IStruct::CX { control, target } => format!("cx q[{}], q[{}];", control, target),
            IStruct::U { matrix, target } => {
                let rows: Vec<String> = matrix
                    .row_iter()
                    .map(|row| {
                        let elems: Vec<String> = row.iter()
                            .map(|v| {
                                if v.im == 0.0 {
                                    format!("{}", v.re)
                                } else if v.re == 0.0 {
                                    format!("{}i", v.im)
                                } else {
                                    format!("{}+{}i", v.re, v.im)
                                }
                            })
                            .collect();
                        format!("[{}]", elems.join(", "))
                    })
                    .collect();
            
                let qubits = target
                    .iter()
                    .map(|q| format!("q[{}]", q))
                    .collect::<Vec<_>>()
                    .join(",");
            
                format!("unitary [{}] {};", rows.join(","), qubits)
            }            IStruct::MEASURE(q_bits, cl_bits) => format!("measure q[{}] -> c[{}];", q_bits[0], cl_bits[0]),
            IStruct::GATE { position : _, instruction, label : _ } => {
                let mut qasm = String::new();
                for elements in instruction {
                    qasm.push_str(&elements.to_qasm());
                    qasm.push_str("\n");
                }
                qasm
            }
            IStruct::SWAP { qbit1, qbit2 } => format!("swap q[{}], q[{}];", qbit1, qbit2),
            IStruct::RZ { angle, target } => format!("rz({}) q[{}];", angle, target),
            IStruct::RX { angle, target } => format!("rx({}) q[{}];", angle, target),
            IStruct::RY { angle, target } => format!("ry({}) q[{}];", angle, target),
            IStruct::ANY() => String::from("// ANY gate is not defined in QASM"),
            _=> String::from("// This instruction is not defined in QASM"),
        }
    }

}


impl ToQasm for QuantumCircuit {
    fn to_qasm(&self) -> String {
        let mut qasm = String::new();
        qasm.push_str("OPENQASM 3.0;\n");
        qasm.push_str(&format!("qubit q[{}];\n", self.get_q_bits()));
        qasm.push_str(&format!("bit c[{}];\n", self.get_cl_bits()));
        for instruction in self.get_instructions() {
            qasm.push_str(&instruction.to_qasm());
            qasm.push_str("\n");
        }
        qasm
    }
}
