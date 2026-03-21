use serde::Serialize;
use serde_json::{json, Value};


#[derive(Debug, Clone, Default, Serialize)]
pub struct DynamicalDecouplingOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence_type: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ZneOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extrapolator: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noise_factors: Option<Vec<u32>>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ResilienceOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub measure_mitigation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zne_mitigation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zne: Option<ZneOptions>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct TranspilationOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimization_level: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_transpilation: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct JobOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamical_decoupling: Option<DynamicalDecouplingOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resilience: Option<ResilienceOptions>,

}

#[derive(Default)]
pub struct JobOptionsBuilder {
    opts: JobOptions,
}

impl JobOptionsBuilder {
    pub fn new() -> Self { Self::default() }

    pub fn dynamical_decoupling(mut self, enable: bool) -> Self {
        self.opts.dynamical_decoupling = Some(DynamicalDecouplingOptions {
            enable: Some(enable),
            ..Default::default()
        });
        self
    }

    pub fn dynamical_decoupling_sequence(mut self, seq: impl Into<String>) -> Self {
        self.opts.dynamical_decoupling
            .get_or_insert_with(Default::default)
            .sequence_type = Some(seq.into());
        self
    }

    pub fn measure_mitigation(mut self, enable: bool) -> Self {
        self.opts.resilience
            .get_or_insert_with(Default::default)
            .measure_mitigation = Some(enable);
        self
    }

    pub fn zne_mitigation(mut self, enable: bool) -> Self {
        self.opts.resilience
            .get_or_insert_with(Default::default)
            .zne_mitigation = Some(enable);
        self
    }

    pub fn zne(mut self, extrapolators: Vec<&str>, noise_factors: Vec<u32>) -> Self {
        self.opts.resilience
            .get_or_insert_with(Default::default)
            .zne = Some(ZneOptions {
                extrapolator: Some(extrapolators.iter().map(|s| s.to_string()).collect()),
                noise_factors: Some(noise_factors),
            });
        self
    }



    pub fn build(self) -> JobOptions { self.opts }
}


pub struct SamplerPub {
    circuit: String,
    shots: Option<u32>,
    parameter_values: Option<Vec<Vec<f64>>>,
}

impl SamplerPub {
    pub fn new(qasm: impl Into<String>) -> Self {
        Self { circuit: qasm.into(), shots: None, parameter_values: None }
    }

    pub fn shots(mut self, shots: u32) -> Self {
        self.shots = Some(shots);
        self
    }

    pub fn parameter_values(mut self, values: Vec<Vec<f64>>) -> Self {
        self.parameter_values = Some(values);
        self
    }

    fn to_json(&self) -> Value {
        match (&self.parameter_values, &self.shots) {
            (None,     None    ) => json!([self.circuit]),
            (Some(pv), None    ) => json!([self.circuit, pv]),
            (None,     Some(s) ) => json!([self.circuit, null, s]),
            (Some(pv), Some(s) ) => json!([self.circuit, pv, s]),
        }
    }
}

pub struct EstimatorPub {
    circuit: String,
    observables: Value,
    parameter_values: Option<Vec<Vec<f64>>>,
}

impl EstimatorPub {
    pub fn new(qasm: impl Into<String>, observables: Value) -> Self {
        Self { circuit: qasm.into(), observables, parameter_values: None }
    }

    pub fn parameter_values(mut self, values: Vec<Vec<f64>>) -> Self {
        self.parameter_values = Some(values);
        self
    }

    fn to_json(&self) -> Value {
        match &self.parameter_values {
            None     => json!([self.circuit, self.observables]),
            Some(pv) => json!([self.circuit, self.observables, pv]),
        }
    }
}


pub enum JobRequest {
    Sampler(SamplerJobBuilder),
    Estimator(EstimatorJobBuilder),
}

impl JobRequest {
    pub fn build(self) -> Result<Value, String> {
        match self {
            JobRequest::Sampler(b)   => b.build(),
            JobRequest::Estimator(b) => b.build(),
        }
    }
}


pub struct SamplerJobBuilder {
    backend: String,
    pubs: Vec<SamplerPub>,
    session_id: Option<String>,
    private: Option<bool>,
    options: Option<JobOptions>,
}

impl SamplerJobBuilder {
    pub fn new(backend: impl Into<String>) -> Self {
        Self {
            backend: backend.into(),
            pubs: vec![],
            session_id: None,
            private: None,
            options: None,
        }
    }

    pub fn add_pub(mut self, pub_: SamplerPub) -> Self {
        self.pubs.push(pub_);
        self
    }

    pub fn session_id(mut self, id: impl Into<String>) -> Self {
        self.session_id = Some(id.into());
        self
    }

    pub fn private(mut self, private: bool) -> Self {
        self.private = Some(private);
        self
    }

    pub fn options(mut self, opts: JobOptions) -> Self {
        self.options = Some(opts);
        self
    }

    pub fn build(self) -> Result<Value, String> {
        if self.pubs.is_empty() {
            return Err("Au moins un PUB est requis".into());
        }

        let pubs: Vec<Value> = self.pubs.iter().map(|p| p.to_json()).collect();
        let mut params = json!({ "version": 2, "pubs": pubs });

        if let Some(opts) = &self.options {
            params["options"] = serde_json::to_value(opts).unwrap();
        }

        let mut body = json!({
            "program_id": "sampler",
            "backend": self.backend,
            "params": params,
        });

        if let Some(sid) = self.session_id { body["session_id"] = json!(sid); }
        if let Some(p) = self.private     { body["private"]     = json!(p);   }

        Ok(body)
    }
}


pub struct EstimatorJobBuilder {
    backend: String,
    pubs: Vec<EstimatorPub>,
    session_id: Option<String>,
    private: Option<bool>,
    resilience_level: Option<u8>,
    options: Option<JobOptions>,
}

impl EstimatorJobBuilder {
    pub fn new(backend: impl Into<String>) -> Self {
        Self {
            backend: backend.into(),
            pubs: vec![],
            session_id: None,
            private: None,
            resilience_level: None,
            options: None,
        }
    }

    pub fn add_pub(mut self, pub_: EstimatorPub) -> Self {
        self.pubs.push(pub_);
        self
    }

    pub fn session_id(mut self, id: impl Into<String>) -> Self {
        self.session_id = Some(id.into());
        self
    }

    pub fn private(mut self, private: bool) -> Self {
        self.private = Some(private);
        self
    }

    pub fn resilience_level(mut self, level: u8) -> Self {
        self.resilience_level = Some(level);
        self
    }

    pub fn options(mut self, opts: JobOptions) -> Self {
        self.options = Some(opts);
        self
    }

    pub fn build(self) -> Result<Value, String> {
        if self.pubs.is_empty() {
            return Err("Au moins un PUB est requis".into());
        }

        let pubs: Vec<Value> = self.pubs.iter().map(|p| p.to_json()).collect();
        let mut params = json!({ "version": 2, "pubs": pubs });

        if let Some(level) = self.resilience_level {
            params["resilience_level"] = json!(level);
        }
        if let Some(opts) = &self.options {
            params["options"] = serde_json::to_value(opts).unwrap();
        }

        let mut body = json!({
            "program_id": "estimator",
            "backend": self.backend,
            "params": params,
        });

        if let Some(sid) = self.session_id { body["session_id"] = json!(sid); }
        if let Some(p)   = self.private    { body["private"]    = json!(p);   }

        Ok(body)
    }
}