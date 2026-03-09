/// Le trait `QLogic` représente une logique quantique abstraite.
///
/// Ce trait définit une interface commune pour les objets qui possèdent une taille
/// ou une dimension, comme des circuits quantiques ou des composants logiques.
///
/// # Méthodes
pub trait QLogic {
    /// Retourne la taille ou la dimension de l'objet.
    ///
    /// # Retourne
    ///
    /// * `usize` - La taille ou la dimension de l'objet.
    fn get_size(&self) -> usize;
}