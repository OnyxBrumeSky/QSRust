/// Ce module est le point d'entrée de la bibliothèque.
/// 
/// Il regroupe les différents modules nécessaires pour la simulation et la manipulation
/// des circuits quantiques. Chaque module est conçu pour une responsabilité spécifique :
///
/// - `circuit` : Définit les structures et les fonctionnalités pour créer et manipuler des circuits quantiques.
/// - `simulator` : Contient les outils pour simuler le comportement des circuits quantiques.
/// - `instruction` : Définit les instructions et opérations utilisées dans les circuits quantiques.
pub mod circuit;
pub mod simulator;
pub mod	instruction;
pub mod api;
