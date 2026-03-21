//! Module d'accès à l'API IBM Quantum.
//!
//! Fournit tous les composants nécessaires pour s'authentifier,
//! interagir avec les backends, soumettre des jobs et récupérer les résultats.
//!
//! # Modules
//!
//! - [`device`] — Structure représentant un backend IBM (réel ou simulateur)
//! - [`iam`] — Token d'accès IAM retourné après authentification
//! - [`service`] — Client HTTP principal : authentification, requêtes et gestion des backends
//! - [`structs`] — Types partagés (canaux, instances, ressources IBM Cloud)
//! - [`backend_manager`] — Gestionnaire de backends avec filtrage et sélection
//! - [`job`] — Soumission, polling et récupération des résultats de jobs
//! - [`job_struct`] — Structures de désérialisation des réponses API (jobs, résultats)
//! - [`job_builder`] — Builders pour construire les payloads Sampler et Estimator

/// Structure représentant un backend IBM Quantum (réel ou simulateur).
pub mod device;

/// Token d'accès IBM Cloud IAM retourné après authentification par API key.
pub mod iam;

/// Client HTTP principal pour l'API IBM Quantum.
///
/// Gère l'authentification IAM, l'instance CRN et expose les méthodes
/// `get`, `post` ainsi que la transpilation et la gestion des backends.
pub mod service;

/// Types partagés utilisés par le service IBM Quantum.
///
/// Contient les énumérations de canaux (`IbmQuantumPlatform`, `IbmCloud`...)
/// et les structures de réponse pour les instances IBM Cloud.
pub mod structs;

/// Gestionnaire de backends IBM Quantum.
///
/// Expose des méthodes pour lister, filtrer par type et sélectionner
/// le backend le moins chargé parmi les backends disponibles.
pub mod backend_manager;

/// Soumission et suivi des jobs IBM Quantum.
///
/// Contient l'enum [`job::JobStatus`] et les méthodes [`Service::submit_job`],
/// [`Service::wait_for_job`], [`Service::run_and_collect`] et [`Service::get_job_result`].
pub mod job;

/// Structures de désérialisation des réponses de l'API IBM Quantum.
///
/// Couvre les jobs (`JobRoot`, `JobListRoot`), les résultats (`ResultRoot`, `PubResult`)
/// et la réponse de soumission (`SubmitJobResponse`).
pub mod job_struct;

/// Builders pour construire les payloads de jobs IBM Quantum.
///
/// Fournit [`job_builder::SamplerJobBuilder`], [`job_builder::EstimatorJobBuilder`]
/// et [`job_builder::JobOptionsBuilder`] pour composer les requêtes Sampler et Estimator.
pub mod job_builder;