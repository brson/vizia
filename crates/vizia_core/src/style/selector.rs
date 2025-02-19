use std::hash::Hash;

use std::cmp::{Eq, PartialEq};
use std::collections::HashSet;

use super::Specificity;

use bitflags::bitflags;

bitflags! {
    /// A bitflag of possible pseudoclasses.
    ///
    /// This type is part of the prelude.
    pub struct PseudoClass: u16 {
        const HOVER = 1;
        const OVER = 1 << 1;
        const ACTIVE = 1 << 2;
        const FOCUS = 1 << 3;
        const DISABLED = 1 << 4;
        const CHECKED = 1 << 5;
        const SELECTED = 1 << 6;
        const CUSTOM = 1 << 7;
        const FOCUS_WITHIN = 1<<8;
        const FOCUS_VISIBLE = 1 << 9;
        const ROOT = 1 << 10;
    }
}

impl Default for PseudoClass {
    fn default() -> Self {
        PseudoClass::empty()
    }
}

impl std::fmt::Display for PseudoClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.contains(PseudoClass::HOVER) {
            write!(f, ":hover")?;
        }
        if self.contains(PseudoClass::OVER) {
            write!(f, ":over")?;
        }
        if self.contains(PseudoClass::ACTIVE) {
            write!(f, ":active")?;
        }
        if self.contains(PseudoClass::FOCUS) {
            write!(f, ":focus")?;
        }
        if self.contains(PseudoClass::DISABLED) {
            write!(f, ":disabled")?;
        }
        if self.contains(PseudoClass::CHECKED) {
            write!(f, ":checked")?;
        }
        if self.contains(PseudoClass::SELECTED) {
            write!(f, ":selected")?;
        }
        if self.contains(PseudoClass::FOCUS_WITHIN) {
            write!(f, ":focus-within")?;
        }
        if self.contains(PseudoClass::FOCUS_VISIBLE) {
            write!(f, ":focus-visible")?;
        }
        if self.contains(PseudoClass::ROOT) {
            write!(f, ":root")?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum SelectorRelation {
    None,
    Ancestor,
    Parent,
}

/// A style selector.
#[derive(Clone, Debug)]
pub struct Selector {
    pub id: Option<String>,
    pub element: Option<String>,
    pub classes: HashSet<String>,
    pub pseudo_classes: PseudoClass,
    pub relation: SelectorRelation,
    pub asterisk: bool,
}

impl Default for Selector {
    fn default() -> Selector {
        Selector {
            id: None,
            element: None,
            classes: HashSet::new(),
            pseudo_classes: PseudoClass::empty(),
            relation: SelectorRelation::None,
            asterisk: false,
        }
    }
}

impl std::fmt::Display for Selector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.asterisk {
            write!(f, "*")?;
        }

        if let Some(element) = &self.element {
            write!(f, "{}", element)?;
        }

        if let Some(id) = &self.id {
            write!(f, "#{}", id)?;
        }

        for class_name in self.classes.iter() {
            write!(f, ".{}", class_name)?;
        }

        write!(f, "{}", self.pseudo_classes)?;

        match self.relation {
            SelectorRelation::None => {}
            SelectorRelation::Ancestor => write!(f, " ")?,
            SelectorRelation::Parent => write!(f, ">")?,
        }

        Ok(())
    }
}

impl Selector {
    // pub fn new() -> Self {
    //     Selector {
    //         id: None,
    //         element: None,
    //         classes: HashSet::new(),
    //         pseudo_classes: PseudoClass::empty(),
    //         relation: SelectorRelation::None,
    //         asterisk: false,
    //     }
    // }

    // pub fn element(element: &str) -> Self {
    //     //let mut s = DefaultHasher::new();
    //     //element.hash(&mut s);

    //     Selector {
    //         id: None,
    //         element: Some(element.to_owned()),
    //         classes: HashSet::new(),
    //         pseudo_classes: PseudoClass::empty(),
    //         relation: SelectorRelation::None,
    //         asterisk: false,
    //     }
    // }

    pub fn is_empty(&self) -> bool {
        self.id.is_none() && self.element.is_none() && self.classes.is_empty()
    }

    // Returns true if the selectors are identical unless either are empty
    pub fn same(&self, entity_selector: &Selector) -> bool {
        if self.is_empty() || entity_selector.is_empty() {
            return false;
        }

        if self.asterisk != entity_selector.asterisk {
            return false;
        }

        if self.id.is_some() && self.id != entity_selector.id {
            return false;
        }

        if self.element.is_some() && self.element != entity_selector.element {
            return false;
        }

        if !self.classes.is_empty() && self.classes != entity_selector.classes {
            return false;
        }

        if self.pseudo_classes != entity_selector.pseudo_classes {
            return false;
        }

        true
    }

    pub fn matches(&self, entity_selector: &Selector) -> bool {
        // Universal selector always matches
        if self.asterisk {
            if !self.pseudo_classes.is_empty()
                && !self.pseudo_classes.intersects(entity_selector.pseudo_classes)
            {
                return false;
            } else {
                return true;
            }
        }

        // Check for ID match
        if self.id.is_some() && self.id != entity_selector.id {
            return false;
        }

        // Check for element name match
        if self.element.is_some() && self.element != entity_selector.element {
            return false;
        }

        // Check for classes match
        if !self.classes.is_subset(&entity_selector.classes) {
            return false;
        }

        if !self.pseudo_classes.is_empty()
            && !self.pseudo_classes.intersects(entity_selector.pseudo_classes)
        {
            return false;
        }

        if self.asterisk != entity_selector.asterisk {
            return false;
        }

        true
    }

    pub(crate) fn specificity(&self) -> Specificity {
        Specificity([
            if self.id.is_some() { 1 } else { 0 },
            (self.classes.len() + self.pseudo_classes.bits().count_ones() as usize) as u8,
            if self.element.is_some() { 1 } else { 0 },
        ])
    }

    // pub fn id(mut self, id: &str) -> Self {
    //     let mut s = DefaultHasher::new();
    //     id.hash(&mut s);
    //     self.id = Some(s.finish());
    //     self
    // }

    // pub fn class(mut self, class: &str) -> Self {
    //     self.classes.insert(class.to_string());
    //     self
    // }

    // pub fn replace_class(&mut self, old: &str, new: &str) -> &mut Self {
    //     self.classes.remove(old);
    //     self.classes.insert(new.to_string());

    //     self
    // }

    pub fn set_id(&mut self, id: &str) -> &mut Self {
        self.id = Some(id.to_owned());
        self
    }

    pub fn set_element(&mut self, element: &str) -> &mut Self {
        self.element = Some(element.to_owned());
        self
    }
}

impl PartialEq for Selector {
    fn eq(&self, other: &Selector) -> bool {
        self.matches(other)
    }
}
