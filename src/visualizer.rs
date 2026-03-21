use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};
use crate::api::job_struct::JobRoot;

/// Métadonnées associées à un job IBM Quantum.
///
/// Regroupe toutes les informations utiles sur un job exécuté :
/// identifiant, backend, nombre de shots, date de création,
/// durée d'exécution estimée et circuit QASM soumis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetadata {
    /// Identifiant unique du job (ex: `"d6utt4k69uic73cim4l0"`)
    pub job_id: String,
    /// Nom du backend utilisé (ex: `"ibm_fez"`)
    pub backend: String,
    /// Nombre de shots exécutés
    pub shots: u32,
    /// Date de création du job au format ISO8601
    pub created: String,
    /// Temps d'exécution estimé en secondes
    pub execution_time_seconds: f64,
    /// Circuit QASM soumis, si disponible
    pub qasm: Option<String>,
}

impl JobMetadata {
    /// Construit un [`JobMetadata`] depuis un [`JobRoot`] et le nombre de shots.
    ///
    /// Extrait automatiquement le QASM depuis `params.pubs[0]`, en gérant
    /// les deux formats possibles retournés par l'API IBM :
    /// - string QASM directe
    /// - tableau `["qasm", null, shots]`
    ///
    /// # Exemple
    /// ```rust
    /// let meta = JobMetadata::from_job(&job, 100);
    /// ```
    pub fn from_job(job: &JobRoot, shots: u32) -> Self {
        let qasm = job.params.pubs.first().and_then(|pub_val| {
            if let Some(s) = pub_val.as_str() {
                return Some(s.to_string());
            }
            if let Some(arr) = pub_val.as_array() {
                if let Some(s) = arr.first().and_then(|v| v.as_str()) {
                    return Some(s.to_string());
                }
            }
            None
        });

        Self {
            job_id:                 job.id.clone(),
            backend:                job.backend.clone(),
            shots,
            created:                job.created.clone(),
            execution_time_seconds: job.estimated_running_time_seconds,
            qasm,
        }
    }

    /// Construit un [`JobMetadata`] manuellement sans [`JobRoot`].
    ///
    /// Utile quand on ne dispose pas du job complet, par exemple après
    /// avoir récupéré uniquement les résultats via [`ResultRoot`].
    ///
    /// # Exemple
    /// ```rust
    /// let meta = JobMetadata::new("job_id", "ibm_fez", 100, Some(qasm));
    /// ```
    pub fn new(
        job_id: impl Into<String>,
        backend: impl Into<String>,
        shots: u32,
        qasm: Option<String>,
    ) -> Self {
        Self {
            job_id:                 job_id.into(),
            backend:                backend.into(),
            shots,
            created:                String::new(),
            execution_time_seconds: 0.0,
            qasm,
        }
    }
}

/// Structure de sérialisation complète pour l'export JSON.
///
/// Contient les métadonnées du job, les counts bruts et
/// les probabilités calculées pour chaque état mesuré.
#[derive(Debug, Serialize, Deserialize)]
struct JobExport {
    /// Métadonnées du job
    metadata: JobMetadata,
    /// Nombre d'occurrences par état (ex: `{"00": 47, "11": 49}`)
    counts: HashMap<String, u32>,
    /// Probabilité de chaque état (ex: `{"00": 0.47, "11": 0.49}`)
    probabilities: HashMap<String, f64>,
}

/// Exporte les counts et les métadonnées du job dans un fichier JSON.
///
/// Le fichier produit contient trois sections : `metadata`, `counts` et `probabilities`.
/// Les répertoires parents sont créés automatiquement si nécessaire.
///
/// # Arguments
/// - `counts` — histogramme des mesures
/// - `meta` — métadonnées du job
/// - `path` — chemin de sortie (ex: `"results/bell.json"`)
///
/// # Exemple
/// ```rust
/// export_json(&counts, &meta, "results/bell.json")?;
/// ```
pub fn export_json(
    counts: &HashMap<String, u32>,
    meta: &JobMetadata,
    path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let total: u32 = counts.values().sum();

    let probabilities: HashMap<String, f64> = counts.iter()
        .map(|(k, &v)| (k.clone(), v as f64 / total as f64))
        .collect();

    let export = JobExport {
        metadata: meta.clone(),
        counts: counts.clone(),
        probabilities,
    };

    if let Some(parent) = path.as_ref().parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    fs::write(&path, serde_json::to_string_pretty(&export)?)?;
    println!("JSON exporté → {}", path.as_ref().display());
    Ok(())
}

/// Génère un fichier HTML avec un histogramme interactif (Chart.js) et les métadonnées du job.
///
/// Le fichier produit s'ouvre directement dans un navigateur et affiche :
/// - un histogramme des états mesurés avec tooltips de probabilité
/// - un tableau des counts et probabilités par état
/// - un tableau des métadonnées (job ID, backend, shots, date, durée)
/// - le circuit QASM soumis, si disponible dans les métadonnées
///
/// Les répertoires parents sont créés automatiquement si nécessaire.
///
/// # Arguments
/// - `counts` — histogramme des mesures
/// - `meta` — métadonnées du job
/// - `title` — titre affiché dans la page HTML
/// - `path` — chemin de sortie (ex: `"results/bell.html"`)
///
/// # Exemple
/// ```rust
/// export_html(&counts, &meta, "Bell State — ibm_fez", "results/bell.html")?;
/// ```
pub fn export_html(
    counts: &HashMap<String, u32>,
    meta: &JobMetadata,
    title: &str,
    path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let total: u32 = counts.values().sum();

    let mut sorted: Vec<(&String, &u32)> = counts.iter().collect();
    sorted.sort_by_key(|(k, _)| k.as_str());

    let labels        = sorted.iter().map(|(k, _)| format!("\"{}\"", k)).collect::<Vec<_>>().join(", ");
    let values        = sorted.iter().map(|(_, v)| v.to_string()).collect::<Vec<_>>().join(", ");
    let probabilities = sorted.iter().map(|(_, &v)| format!("{:.4}", v as f64 / total as f64)).collect::<Vec<_>>().join(", ");
    let colors        = sorted.iter().enumerate()
        .map(|(i, _)| if i % 2 == 0 { "\"#4C7EF4\"" } else { "\"#8B5CF6\"" })
        .collect::<Vec<_>>().join(", ");

    let dominant = sorted.iter().max_by_key(|(_, &v)| v).map(|(k, _)| k.as_str()).unwrap_or("?");

    let rows: String = sorted.iter().map(|(k, &v)| format!(
        "<tr><td><code>{}</code></td><td>{}</td><td>{:.1}%</td></tr>",
        k, v, v as f64 / total as f64 * 100.0
    )).collect::<Vec<_>>().join("\n");

    let qasm_block = match &meta.qasm {
        Some(q) => {
            let escaped = q
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;");
            format!(r#"<div class="card" style="margin-top:1.5rem">
      <h2>Circuit QASM</h2>
      <pre class="qasm">{}</pre>
    </div>"#, escaped)
        }
        None => String::new(),
    };

    let created_row = if !meta.created.is_empty() {
        format!("<tr><td>Créé le</td><td>{}</td></tr>", meta.created)
    } else { String::new() };

    let exec_row = if meta.execution_time_seconds > 0.0 {
        format!("<tr><td>Durée estimée</td><td>{:.2}s</td></tr>", meta.execution_time_seconds)
    } else { String::new() };

    let html = format!(r#"<!DOCTYPE html>
<html lang="fr">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title}</title>
<script src="https://cdn.jsdelivr.net/npm/chart.js@4"></script>
<style>
  * {{ box-sizing: border-box; margin: 0; padding: 0; }}
  body {{ font-family: system-ui, sans-serif; background: #0f0f13; color: #e2e8f0; padding: 2rem; }}
  h1 {{ font-size: 1.4rem; font-weight: 500; margin-bottom: 0.3rem; color: #f1f5f9; }}
  .job-id {{ font-size: 0.78rem; color: #475569; font-family: monospace; margin-bottom: 1.5rem; }}
  .stat {{ display: flex; flex-wrap: wrap; gap: 1rem; margin-bottom: 1.5rem; }}
  .stat-item {{ background: #1a1a24; border: 1px solid #2d2d3d; border-radius: 8px; padding: 0.65rem 1.1rem; }}
  .stat-label {{ font-size: 0.72rem; color: #64748b; text-transform: uppercase; letter-spacing: .05em; }}
  .stat-value {{ font-size: 1.15rem; font-weight: 600; color: #f1f5f9; margin-top: 2px; }}
  .grid {{ display: grid; grid-template-columns: 1fr 320px; gap: 1.5rem; align-items: start; }}
  .card {{ background: #1a1a24; border: 1px solid #2d2d3d; border-radius: 12px; padding: 1.25rem; }}
  .card h2 {{ font-size: 0.78rem; text-transform: uppercase; letter-spacing: .08em; color: #64748b; margin-bottom: 1rem; }}
  canvas {{ max-height: 300px; }}
  table {{ width: 100%; border-collapse: collapse; font-size: 0.85rem; }}
  th {{ text-align: left; padding: 0.45rem 0.7rem; color: #64748b; font-weight: 500;
        border-bottom: 1px solid #2d2d3d; font-size: 0.78rem; }}
  td {{ padding: 0.45rem 0.7rem; border-bottom: 1px solid #1e1e2e; color: #cbd5e1; }}
  tr:last-child td {{ border-bottom: none; }}
  td:first-child {{ color: #64748b; }}
  code {{ background: #0f0f13; padding: 2px 6px; border-radius: 4px; font-family: monospace; color: #a78bfa; }}
  pre.qasm {{ background: #0f0f13; border: 1px solid #2d2d3d; border-radius: 8px;
              padding: 1rem; font-family: monospace; font-size: 0.8rem; color: #7dd3fc;
              white-space: pre-wrap; word-break: break-all; line-height: 1.6; overflow-x: auto; }}
</style>
</head>
<body>

<h1>{title}</h1>
<div class="job-id">Job ID : {job_id}</div>

<div class="stat">
  <div class="stat-item"><div class="stat-label">Backend</div><div class="stat-value">{backend}</div></div>
  <div class="stat-item"><div class="stat-label">Shots</div><div class="stat-value">{shots}</div></div>
  <div class="stat-item"><div class="stat-label">États distincts</div><div class="stat-value">{n_states}</div></div>
  <div class="stat-item"><div class="stat-label">État dominant</div><div class="stat-value"><code>{dominant}</code></div></div>
</div>

<div class="grid">
  <div>
    <div class="card">
      <h2>Distribution des mesures</h2>
      <canvas id="chart"></canvas>
    </div>
    {qasm_block}
  </div>
  <div style="display:flex;flex-direction:column;gap:1.5rem">
    <div class="card">
      <h2>Counts</h2>
      <table>
        <thead><tr><th>État</th><th>Counts</th><th>Prob.</th></tr></thead>
        <tbody>{rows}</tbody>
      </table>
    </div>
    <div class="card">
      <h2>Métadonnées</h2>
      <table>
        <tbody>
          <tr><td>Job ID</td><td><code style="font-size:0.72rem;word-break:break-all">{job_id}</code></td></tr>
          <tr><td>Backend</td><td>{backend}</td></tr>
          <tr><td>Shots</td><td>{shots}</td></tr>
          {created_row}
          {exec_row}
        </tbody>
      </table>
    </div>
  </div>
</div>

<script>
new Chart(document.getElementById('chart'), {{
  type: 'bar',
  data: {{
    labels: [{labels}],
    datasets: [{{ label: 'Counts', data: [{values}], backgroundColor: [{colors}], borderRadius: 6, borderSkipped: false }}]
  }},
  options: {{
    responsive: true,
    plugins: {{
      legend: {{ display: false }},
      tooltip: {{ callbacks: {{ afterLabel: ctx => {{ const p=[{probabilities}]; return 'P = '+(p[ctx.dataIndex]*100).toFixed(1)+'%'; }} }} }}
    }},
    scales: {{
      x: {{ grid: {{ color: '#2d2d3d' }}, ticks: {{ color: '#94a3b8' }} }},
      y: {{ grid: {{ color: '#2d2d3d' }}, ticks: {{ color: '#94a3b8' }}, beginAtZero: true }}
    }}
  }}
}});
</script>
</body>
</html>"#,
        title         = title,
        job_id        = meta.job_id,
        backend       = meta.backend,
        shots         = meta.shots,
        n_states      = sorted.len(),
        dominant      = dominant,
        rows          = rows,
        qasm_block    = qasm_block,
        created_row   = created_row,
        exec_row      = exec_row,
        labels        = labels,
        values        = values,
        colors        = colors,
        probabilities = probabilities,
    );

    if let Some(parent) = path.as_ref().parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    fs::write(&path, html)?;
    println!("HTML exporté → {}", path.as_ref().display());
    Ok(())
}

/// Affiche un histogramme horizontal des mesures dans le terminal.
///
/// Chaque barre représente la fréquence relative d'un état mesuré.
/// Les métadonnées du job sont affichées dans un encadré au-dessus de l'histogramme.
/// Si un circuit QASM est disponible dans les métadonnées, il est affiché en dessous.
///
/// # Arguments
/// - `counts` — histogramme des mesures
/// - `meta` — métadonnées du job
/// - `title` — titre affiché en en-tête
/// - `bar_width` — largeur maximale des barres en caractères (ex: `40`)
///
/// # Exemple
/// ```rust
/// print_histogram(&counts, &meta, "Bell State", 40);
/// ```
pub fn print_histogram(
    counts: &HashMap<String, u32>,
    meta: &JobMetadata,
    title: &str,
    bar_width: usize,
) {
    let total      = counts.values().sum::<u32>();
    let max_count  = counts.values().copied().max().unwrap_or(1);
    let label_w    = counts.keys().map(|k| k.len()).max().unwrap_or(2);
    let sep_w      = bar_width + label_w + 22;

    let mut sorted: Vec<(&String, &u32)> = counts.iter().collect();
    sorted.sort_by_key(|(k, _)| k.as_str());

    let sep = "═".repeat(sep_w);

    println!();
    println!("  ╔{sep}╗");
    println!("  ║  {title:<sep_w$}║", title = title, sep_w = sep_w - 2);
    println!("  ╠{sep}╣");
    println!("  ║  Job ID  : {:<width$}║", meta.job_id,  width = sep_w - 12);
    println!("  ║  Backend : {:<width$}║", meta.backend, width = sep_w - 12);
    println!("  ║  Shots   : {:<width$}║", meta.shots,   width = sep_w - 12);
    if !meta.created.is_empty() {
        println!("  ║  Créé le : {:<width$}║", meta.created, width = sep_w - 12);
    }
    if meta.execution_time_seconds > 0.0 {
        println!("  ║  Durée   : {:<width$}║", format!("{:.2}s", meta.execution_time_seconds), width = sep_w - 12);
    }
    println!("  ╠{sep}╣");

    for (bitstring, &count) in &sorted {
        let filled = (count as f64 / max_count as f64 * bar_width as f64).round() as usize;
        let empty  = bar_width.saturating_sub(filled);
        let prob   = count as f64 / total as f64;
        println!(
            "  ║ {:>lw$} ┤{}{}┤ {:>5}  {:5.1}%  ║",
            bitstring,
            "█".repeat(filled),
            "░".repeat(empty),
            count,
            prob * 100.0,
            lw = label_w,
        );
    }

    println!("  ╠{sep}╣");
    println!("  ║  Total : {} shots  │  {} états distincts{:<pad$}║",
        total, sorted.len(), "", pad = sep_w - 38 - sorted.len().to_string().len() - total.to_string().len());
    println!("  ╚{sep}╝");

    if let Some(qasm) = &meta.qasm {
        println!();
        println!("  Circuit QASM :");
        println!("  {}", "─".repeat(60));
        for line in qasm.lines() {
            println!("  {}", line);
        }
        println!("  {}", "─".repeat(60));
    }

    println!();
}