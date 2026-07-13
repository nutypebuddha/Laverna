use serde::{Deserialize, Serialize};
use std::fmt;

mod aspects;
pub mod barnum;
mod classifier;
mod constants;
mod elements;
mod houses;
mod modalities;
mod rulers;
mod signs;
mod vedic;

pub use aspects::*;
pub use classifier::*;
pub use constants::*;
pub use elements::*;
pub use houses::*;
pub use modalities::*;
pub use rulers::*;
pub use signs::*;
pub use vedic::*;

/// The 7-axis classification of any atom (token, formula, entity).
///
/// Every classification is a weighted distribution, not a one-hot vector.
/// All values are in [0.0, 1.0] and do not need to sum to 1.0 — they
/// represent independent activation strengths across each axis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomClassification {
    pub signs: [f64; 12],
    pub elements: [f64; 4],
    pub modalities: [f64; 3],
    pub rulers: [f64; 7],
    pub houses: [f64; 12],
    pub aspects: [f64; 5],
    pub polarity: f64,
    pub vedic: VedicClassification,
}

impl Default for AtomClassification {
    fn default() -> Self {
        AtomClassification {
            signs: [0.0; 12],
            elements: [0.0; 4],
            modalities: [0.0; 3],
            rulers: [0.0; 7],
            houses: [0.0; 12],
            aspects: [0.0; 5],
            polarity: 0.5,
            vedic: VedicClassification::new(),
        }
    }
}

impl AtomClassification {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_sign(mut self, sign: Sign, value: f64) -> Self {
        self.signs[sign.index()] = value.clamp(0.0, 1.0);
        self
    }

    pub fn with_element(mut self, element: Element, value: f64) -> Self {
        self.elements[element.index()] = value.clamp(0.0, 1.0);
        self
    }

    pub fn with_modality(mut self, modality: Modality, value: f64) -> Self {
        self.modalities[modality.index()] = value.clamp(0.0, 1.0);
        self
    }

    pub fn with_ruler(mut self, ruler: PlanetaryRuler, value: f64) -> Self {
        self.rulers[ruler.index()] = value.clamp(0.0, 1.0);
        self
    }

    pub fn with_polarity(mut self, value: f64) -> Self {
        self.polarity = value.clamp(0.0, 1.0);
        self
    }

    pub fn with_graha(mut self, graha: Graha, value: f64) -> Self {
        self.vedic = self.vedic.with_graha(graha, value);
        self
    }

    pub fn with_nakshatra(mut self, nak: Nakshatra, value: f64) -> Self {
        self.vedic = self.vedic.with_nakshatra(nak, value);
        self
    }

    pub fn with_guna(mut self, guna: Guna, value: f64) -> Self {
        self.vedic = self.vedic.with_guna(guna, value);
        self
    }

    pub fn with_vedic_element(mut self, elem: VedicElement, value: f64) -> Self {
        self.vedic = self.vedic.with_vedic_element(elem, value);
        self
    }

    pub fn similarity(&self, other: &AtomClassification) -> f64 {
        let mut sim = 0.0;
        for i in 0..12 {
            sim += self.signs[i] * other.signs[i];
        }
        for i in 0..4 {
            sim += self.elements[i] * other.elements[i];
        }
        for i in 0..3 {
            sim += self.modalities[i] * other.modalities[i];
        }
        for i in 0..7 {
            sim += self.rulers[i] * other.rulers[i];
        }
        for i in 0..12 {
            sim += self.houses[i] * other.houses[i];
        }
        for i in 0..5 {
            sim += self.aspects[i] * other.aspects[i];
        }
        sim + self.polarity * other.polarity
    }

    pub fn merge_max(&self, other: &AtomClassification) -> AtomClassification {
        let mut result = self.clone();
        for i in 0..12 {
            result.signs[i] = result.signs[i].max(other.signs[i]);
        }
        for i in 0..4 {
            result.elements[i] = result.elements[i].max(other.elements[i]);
        }
        for i in 0..3 {
            result.modalities[i] = result.modalities[i].max(other.modalities[i]);
        }
        for i in 0..7 {
            result.rulers[i] = result.rulers[i].max(other.rulers[i]);
        }
        for i in 0..12 {
            result.houses[i] = result.houses[i].max(other.houses[i]);
        }
        for i in 0..5 {
            result.aspects[i] = result.aspects[i].max(other.aspects[i]);
        }
        result.polarity = result.polarity.max(other.polarity);
        for i in 0..9 {
            result.vedic.grahas[i] = result.vedic.grahas[i].max(other.vedic.grahas[i]);
        }
        for i in 0..27 {
            result.vedic.nakshatras[i] = result.vedic.nakshatras[i].max(other.vedic.nakshatras[i]);
        }
        for i in 0..3 {
            result.vedic.gunas[i] = result.vedic.gunas[i].max(other.vedic.gunas[i]);
        }
        for i in 0..5 {
            result.vedic.vedic_elements[i] =
                result.vedic.vedic_elements[i].max(other.vedic.vedic_elements[i]);
        }
        result.vedic.confidence = result.vedic.confidence.max(other.vedic.confidence);
        result
    }

    pub fn merge_sum(&self, other: &AtomClassification) -> AtomClassification {
        let mut result = self.clone();
        for i in 0..12 {
            result.signs[i] = (result.signs[i] + other.signs[i]).min(1.0);
        }
        for i in 0..4 {
            result.elements[i] = (result.elements[i] + other.elements[i]).min(1.0);
        }
        for i in 0..3 {
            result.modalities[i] = (result.modalities[i] + other.modalities[i]).min(1.0);
        }
        for i in 0..7 {
            result.rulers[i] = (result.rulers[i] + other.rulers[i]).min(1.0);
        }
        for i in 0..12 {
            result.houses[i] = (result.houses[i] + other.houses[i]).min(1.0);
        }
        for i in 0..5 {
            result.aspects[i] = (result.aspects[i] + other.aspects[i]).min(1.0);
        }
        result.polarity = (result.polarity + other.polarity).min(1.0);
        for i in 0..9 {
            result.vedic.grahas[i] = (result.vedic.grahas[i] + other.vedic.grahas[i]).min(1.0);
        }
        for i in 0..27 {
            result.vedic.nakshatras[i] =
                (result.vedic.nakshatras[i] + other.vedic.nakshatras[i]).min(1.0);
        }
        for i in 0..3 {
            result.vedic.gunas[i] = (result.vedic.gunas[i] + other.vedic.gunas[i]).min(1.0);
        }
        for i in 0..5 {
            result.vedic.vedic_elements[i] =
                (result.vedic.vedic_elements[i] + other.vedic.vedic_elements[i]).min(1.0);
        }
        result.vedic.confidence = (result.vedic.confidence + other.vedic.confidence).min(1.0);
        result
    }

    pub fn dominant_sign(&self) -> Option<Sign> {
        self.signs
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .filter(|(_, &v)| v > 0.0)
            .map(|(i, _)| Sign::from_index(i))
    }

    pub fn dominant_element(&self) -> Option<Element> {
        self.elements
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .filter(|(_, &v)| v > 0.0)
            .map(|(i, _)| Element::from_index(i))
    }

    pub fn dominant_modality(&self) -> Option<Modality> {
        self.modalities
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .filter(|(_, &v)| v > 0.0)
            .map(|(i, _)| Modality::from_index(i))
    }

    pub fn dominant_ruler(&self) -> Option<PlanetaryRuler> {
        self.rulers
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .filter(|(_, &v)| v > 0.0)
            .map(|(i, _)| PlanetaryRuler::from_index(i))
    }

    pub fn dominant_house(&self) -> Option<House> {
        self.houses
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .filter(|(_, &v)| v > 0.0)
            .map(|(i, _)| House::from_index(i))
    }

    pub fn dominant_aspect(&self) -> Option<SignAspect> {
        self.aspects
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .filter(|(_, &v)| v > 0.0)
            .map(|(i, _)| match i {
                0 => SignAspect::Conjunction,
                1 => SignAspect::Sextile,
                2 => SignAspect::Trine,
                3 => SignAspect::Square,
                4 => SignAspect::Opposition,
                _ => unreachable!(),
            })
    }

    pub fn dominant_graha(&self) -> Option<Graha> {
        self.vedic.dominant_graha()
    }

    pub fn dominant_nakshatra(&self) -> Option<Nakshatra> {
        self.vedic.dominant_nakshatra()
    }

    pub fn dominant_guna(&self) -> Option<Guna> {
        self.vedic.dominant_guna()
    }

    pub fn dominant_vedic_element(&self) -> Option<VedicElement> {
        self.vedic.dominant_vedic_element()
    }
}

impl fmt::Display for AtomClassification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AtomClassification {{ ")?;
        if let Some(sign) = self.dominant_sign() {
            write!(f, "sign: {}, ", sign.symbol())?;
        }
        if let Some(elem) = self.dominant_element() {
            write!(f, "element: {}, ", elem.symbol())?;
        }
        if let Some(modality) = self.dominant_modality() {
            write!(f, "modality: {}, ", modality.symbol())?;
        }
        write!(f, "polarity: {:.2}", self.polarity)?;
        if let Some(g) = self.vedic.dominant_graha() {
            write!(f, ", graha: {}", g.name())?;
        }
        if let Some(g) = self.vedic.dominant_guna() {
            write!(f, ", guṇa: {}", g.name())?;
        }
        write!(f, " }}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_classification() {
        let c = AtomClassification::new();
        assert_eq!(c.signs, [0.0; 12]);
        assert_eq!(c.polarity, 0.5);
    }

    #[test]
    fn test_with_sign() {
        let c = AtomClassification::new()
            .with_sign(Sign::Aries, 0.9)
            .with_sign(Sign::Leo, 0.3);
        assert!((c.signs[Sign::Aries.index()] - 0.9).abs() < 1e-12);
        assert!((c.signs[Sign::Leo.index()] - 0.3).abs() < 1e-12);
        assert!((c.signs[Sign::Taurus.index()] - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_dominant_sign() {
        let c = AtomClassification::new()
            .with_sign(Sign::Taurus, 0.8)
            .with_sign(Sign::Gemini, 0.2);
        assert_eq!(c.dominant_sign(), Some(Sign::Taurus));
    }

    #[test]
    fn test_dominant_sign_none() {
        let c = AtomClassification::new();
        assert_eq!(c.dominant_sign(), None);
    }

    #[test]
    fn test_similarity() {
        let a = AtomClassification::new().with_sign(Sign::Aries, 1.0);
        let b = AtomClassification::new().with_sign(Sign::Aries, 1.0);
        assert!((a.similarity(&b) - 1.25).abs() < 1e-12);
    }

    #[test]
    fn test_similarity_different() {
        let a = AtomClassification::new().with_sign(Sign::Aries, 1.0);
        let b = AtomClassification::new().with_sign(Sign::Taurus, 1.0);
        assert!((a.similarity(&b) - 0.25).abs() < 1e-12);
    }

    #[test]
    fn test_merge_max() {
        let a = AtomClassification::new().with_sign(Sign::Aries, 0.8);
        let b = AtomClassification::new().with_sign(Sign::Taurus, 0.6);
        let merged = a.merge_max(&b);
        assert!((merged.signs[Sign::Aries.index()] - 0.8).abs() < 1e-12);
        assert!((merged.signs[Sign::Taurus.index()] - 0.6).abs() < 1e-12);
    }

    #[test]
    fn test_merge_sum() {
        let a = AtomClassification::new().with_sign(Sign::Aries, 0.6);
        let b = AtomClassification::new().with_sign(Sign::Aries, 0.5);
        let merged = a.merge_sum(&b);
        assert!((merged.signs[Sign::Aries.index()] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_classification_display() {
        let c = AtomClassification::new()
            .with_sign(Sign::Aries, 0.9)
            .with_element(Element::Fire, 0.8)
            .with_modality(Modality::Cardinal, 0.7);
        let display = format!("{}", c);
        assert!(display.contains("♈"));
        assert!(display.contains("🔥"));
        assert!(display.contains("🅲"));
    }

    #[test]
    fn test_clamp_values() {
        let c = AtomClassification::new().with_sign(Sign::Aries, 1.5);
        assert!((c.signs[Sign::Aries.index()] - 1.0).abs() < 1e-12);

        let c = AtomClassification::new().with_sign(Sign::Aries, -0.5);
        assert!((c.signs[Sign::Aries.index()] - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_dominant_element() {
        let c = AtomClassification::new()
            .with_element(Element::Fire, 0.9)
            .with_element(Element::Water, 0.3);
        assert_eq!(c.dominant_element(), Some(Element::Fire));
    }

    #[test]
    fn test_dominant_modality() {
        let c = AtomClassification::new()
            .with_modality(Modality::Fixed, 0.7)
            .with_modality(Modality::Cardinal, 0.2);
        assert_eq!(c.dominant_modality(), Some(Modality::Fixed));
    }

    #[test]
    fn test_dominant_ruler() {
        let c = AtomClassification::new()
            .with_ruler(PlanetaryRuler::Mars, 0.8)
            .with_ruler(PlanetaryRuler::Venus, 0.4);
        assert_eq!(c.dominant_ruler(), Some(PlanetaryRuler::Mars));
    }

    #[test]
    fn test_dominant_graha() {
        let c = AtomClassification::new()
            .with_graha(Graha::Surya, 0.9)
            .with_graha(Graha::Chandra, 0.3);
        assert_eq!(c.dominant_graha(), Some(Graha::Surya));
    }

    #[test]
    fn test_with_graha_builder() {
        let c = AtomClassification::new()
            .with_graha(Graha::Mangala, 0.7)
            .with_guna(Guna::Rajas, 0.6)
            .with_vedic_element(VedicElement::Fire, 0.8);
        assert_eq!(c.dominant_graha(), Some(Graha::Mangala));
        assert_eq!(c.dominant_guna(), Some(Guna::Rajas));
        assert_eq!(c.dominant_vedic_element(), Some(VedicElement::Fire));
    }

    #[test]
    fn test_merge_max_vedic() {
        let a = AtomClassification::new().with_graha(Graha::Surya, 0.8);
        let b = AtomClassification::new().with_graha(Graha::Chandra, 0.5);
        let merged = a.merge_max(&b);
        assert_eq!(merged.dominant_graha(), Some(Graha::Surya));
        assert!((merged.vedic.grahas[Graha::Chandra.index()] - 0.5).abs() < 1e-12);
    }

    #[test]
    fn test_merge_sum_vedic() {
        let a = AtomClassification::new().with_graha(Graha::Surya, 0.6);
        let b = AtomClassification::new().with_graha(Graha::Surya, 0.5);
        let merged = a.merge_sum(&b);
        assert!((merged.vedic.grahas[Graha::Surya.index()] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_dominant_nakshatra() {
        let c = AtomClassification::new()
            .with_nakshatra(Nakshatra::Rohini, 0.8)
            .with_nakshatra(Nakshatra::Ashwini, 0.3);
        assert_eq!(c.dominant_nakshatra(), Some(Nakshatra::Rohini));
    }

    #[test]
    fn test_dominant_house() {
        let c = AtomClassification::new().with_polarity(0.5);
        assert_eq!(c.dominant_house(), None);
    }

    #[test]
    fn test_with_polarity() {
        let c = AtomClassification::new().with_polarity(0.0);
        assert!((c.polarity - 0.0).abs() < 1e-12);
    }
}
