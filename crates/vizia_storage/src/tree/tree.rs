use crate::{TreeError, TreeExt, TreeIterator};
use vizia_id::GenerationalId;

/// The [Tree] describes the tree of entities.
///
/// This type is part of the prelude.
#[derive(Debug, Clone)]
pub struct Tree<I>
where
    I: GenerationalId,
{
    pub parent: Vec<Option<I>>,
    pub first_child: Vec<Option<I>>,
    pub next_sibling: Vec<Option<I>>,
    pub prev_sibling: Vec<Option<I>>,
    pub ignored: Vec<bool>,
    pub lock_focus_within: Vec<bool>,
    pub changed: bool,
}

impl<I> Tree<I>
where
    I: GenerationalId,
{
    /// Creates a new tree with a root entity.
    pub fn new() -> Self {
        Self {
            parent: vec![None],
            first_child: vec![None],
            next_sibling: vec![None],
            prev_sibling: vec![None],
            ignored: vec![false],
            lock_focus_within: vec![true],
            changed: true,
        }
    }

    pub fn get_child_index(&self, entity: I) -> Option<usize> {
        if let Some(parent) = self.get_parent(entity) {
            for (index, child) in parent.child_iter(self).enumerate() {
                if child == entity {
                    return Some(index);
                }
            }
        }

        None
    }

    /// Returns the last child of an entity.
    pub fn get_last_child(&self, entity: I) -> Option<I> {
        //check if entity exists
        let index = entity.index();
        if index < self.first_child.len() {
            let mut f = self.first_child[index];
            let mut r = None;
            while f != None {
                r = f;
                f = self.next_sibling[f.unwrap().index()];
            }

            return r;
        } else {
            None
        }
    }

    /// Returns the nth child of an entity.
    pub fn get_child(&self, entity: I, n: usize) -> Option<I> {
        if entity.is_null() {
            return None;
        };
        let index = entity.index();
        let mut f = self.first_child[index];
        let mut i = 0;
        while f != None {
            if i == n {
                break;
            }
            f = self.next_sibling[f.unwrap().index()];
            i += 1;
        }

        return f;
    }

    /// Returns the number of children of an entity.
    pub fn get_num_children(&self, entity: I) -> Option<u32> {
        let index = entity.index();
        if entity.is_null() {
            return None;
        };
        let mut f = self.first_child[index];
        let mut r = 0;
        while f != None {
            r += 1;
            f = self.next_sibling[f.unwrap().index()];
        }

        Some(r)
    }

    /// Returns true if the node should be skipped by layout
    pub fn is_ignored(&self, entity: I) -> bool {
        self.ignored.get(entity.index()).map_or_else(|| false, |ignored| *ignored)
    }

    /// Returns the first ancestor to have the lock_focus_within flag set
    pub fn lock_focus_within(&self, entity: I) -> I {
        entity
            .parent_iter(&self)
            .find(|&entity| self.lock_focus_within.get(entity.index()).cloned().unwrap_or_default())
            .unwrap_or(I::root())
    }

    /// Returns the first ancestor of an entity which is not ignored
    pub fn get_layout_parent(&self, entity: I) -> Option<I> {
        let mut i = self.get_parent(entity);
        while let Some(parent) = i {
            if !self.is_ignored(parent) {
                return Some(parent);
            }

            i = self.get_parent(parent);
        }
        None
    }

    /// Returns the parent of an entity.
    pub fn get_parent(&self, entity: I) -> Option<I> {
        self.parent.get(entity.index()).map_or(None, |&parent| parent)
    }

    /// Returns the first child of an entity or `None` if there isn't one.
    pub fn get_first_child(&self, entity: I) -> Option<I> {
        self.first_child.get(entity.index()).map_or(None, |&first_child| first_child)
    }

    /// Returns the next sibling of an entity or `None` if t here isn't one.
    pub fn get_next_sibling(&self, entity: I) -> Option<I> {
        self.next_sibling.get(entity.index()).map_or(None, |&next_sibling| next_sibling)
    }

    /// Returns the previous sibling of an entity or `None` if there isn't one.
    pub fn get_prev_sibling(&self, entity: I) -> Option<I> {
        self.prev_sibling.get(entity.index()).map_or(None, |&prev_sibling| prev_sibling)
    }

    /// Returns true if the entity is the first child of its parent.
    pub fn is_first_child(&self, entity: I) -> bool {
        if let Some(parent) = self.get_parent(entity) {
            if let Some(first_child) = self.get_first_child(parent) {
                if first_child == entity {
                    return true;
                } else {
                    return false;
                }
            }
        }

        false
    }

    /// Returns true if the entity is the last child of its parent.
    pub fn is_last_child(&self, entity: I) -> bool {
        if let Some(parent) = self.get_parent(entity) {
            if let Some(mut temp) = self.get_first_child(parent) {
                while let Some(next_sibling) = self.get_next_sibling(temp) {
                    temp = next_sibling;
                }

                if temp == entity {
                    return true;
                }
            }
        }

        false
    }

    // Checks if entity1 is the sibling of entity2.
    pub fn is_sibling(&self, entity1: I, entity2: I) -> bool {
        if let Some(parent1) = self.get_parent(entity1) {
            if let Some(parent2) = self.get_parent(entity2) {
                return parent1 == parent2;
            }
        }

        false
    }

    /// Returns true if the entity has children.
    pub fn has_children(&self, entity: I) -> bool {
        self.get_first_child(entity).is_some()
    }

    /// Removes an entity from the tree
    ///
    /// This method assumes that a check if the entity is alive has already been done prior to calling this method.
    pub fn remove(&mut self, entity: I) -> Result<(), TreeError> {
        // Check if the entity is null
        if entity == I::null() {
            return Err(TreeError::NullEntity);
        }

        // Check if the entity to be removed exists in the tree
        let entity_index = entity.index();

        if entity_index >= self.parent.len() {
            return Err(TreeError::NoEntity);
        }

        // If the entity was is the first child of its parent then set its next sibling to be the new first child.
        if let Some(parent) = self.get_parent(entity) {
            if self.is_first_child(entity) {
                self.first_child[parent.index()] = self.get_next_sibling(entity);
            }
        }

        // Set the next sibling of the previous sibling of the entity to the next sibling of the entity.
        // from:    [PS] -> [E] -> [NS]
        // to:      [PS] -> [NS]
        // where:   PS - Previous Sibling, E - I, NS - Next Sibling
        if let Some(prev_sibling) = self.get_prev_sibling(entity) {
            self.next_sibling[prev_sibling.index()] = self.get_next_sibling(entity);
        }

        // Set the previous sibling of the next sibling of the entity to the previous sibling of the entity.
        // from:    [PS] <- [E] <- [NS]
        // to:      [PS] <- [NS]
        // where:   PS - Previous Sibling, E - I, NS - Next Sibling
        if let Some(next_sibling) = self.get_next_sibling(entity) {
            self.prev_sibling[next_sibling.index()] = self.get_prev_sibling(entity);
        }

        // Set the next sibling, previous sibling and parent of the removed entity to None
        self.next_sibling[entity_index] = None;
        self.prev_sibling[entity_index] = None;
        self.parent[entity_index] = None;
        self.ignored[entity_index] = false;
        self.lock_focus_within[entity_index] = false;

        // Set the changed flag
        self.changed = true;

        Ok(())
    }

    /// Makes the entity the first child of its parent.
    pub fn set_first_child(&mut self, entity: I) -> Result<(), TreeError> {
        let index = entity.index();
        // Check is sibling exists in the tree
        if index >= self.parent.len() {
            return Err(TreeError::InvalidSibling);
        }

        // Check if the parent is in the tree
        if let Some(parent) = self.get_parent(entity) {
            if parent.index() >= self.parent.len() {
                return Err(TreeError::InvalidParent);
            }
        }

        let parent = self.get_parent(entity).unwrap();

        let previous_first_child = self.first_child[parent.index()];

        if previous_first_child == Some(entity) {
            return Err(TreeError::AlreadyFirstChild);
        }

        let entity_prev_sibling = self.get_prev_sibling(entity);
        let entity_next_sibling = self.get_next_sibling(entity);

        // Remove the entity from the children
        if let Some(eps) = entity_prev_sibling {
            self.next_sibling[eps.index()] = entity_next_sibling; //C
        }

        if let Some(ens) = entity_next_sibling {
            self.prev_sibling[ens.index()] = entity_prev_sibling; //F
        }

        if let Some(pfc) = previous_first_child {
            self.prev_sibling[pfc.index()] = Some(entity);
        }

        self.next_sibling[index] = previous_first_child;

        self.first_child[parent.index()] = Some(entity);

        self.changed = true;

        Ok(())
    }

    pub fn set_next_sibling(&mut self, entity: I, sibling: I) -> Result<(), TreeError> {
        if self.next_sibling[entity.index()] == Some(sibling) {
            return Err(TreeError::AlreadySibling);
        }

        // Check is sibling exists in the tree
        if sibling.index() >= self.parent.len() {
            return Err(TreeError::InvalidSibling);
        }

        // Check if sibling has the same parent
        if let Some(parent) = self.get_parent(entity) {
            if let Some(sibling_parent) = self.get_parent(entity) {
                if parent != sibling_parent {
                    return Err(TreeError::InvalidSibling);
                }
            }
        } else {
            return Err(TreeError::InvalidParent);
        }

        // Safe to unwrap because we already checked if it has a parent
        let parent = self.get_parent(entity).unwrap();

        // Temporarily store the prev_sibling of the desired sibling
        let sibling_prev_sibling = self.get_prev_sibling(sibling);
        let sibling_next_sibling = self.get_next_sibling(sibling);

        // Remove sibling
        if let Some(sps) = sibling_prev_sibling {
            self.next_sibling[sps.index()] = sibling_next_sibling; // C
        } else {
            self.first_child[parent.index()] = sibling_next_sibling;
        }

        if let Some(sns) = sibling_next_sibling {
            self.prev_sibling[sns.index()] = sibling_prev_sibling; // F
        }

        // Temporarily store the next_sibling of the entity
        let entity_next_sibling = self.get_next_sibling(entity);

        if let Some(ens) = entity_next_sibling {
            self.prev_sibling[ens.index()] = Some(sibling); //B
        }

        self.next_sibling[sibling.index()] = entity_next_sibling; //E
        self.prev_sibling[sibling.index()] = Some(entity); // D
        self.next_sibling[entity.index()] = Some(sibling); // A

        self.changed = true;

        Ok(())
    }

    pub fn set_prev_sibling(&mut self, entity: I, sibling: I) -> Result<(), TreeError> {
        if self.prev_sibling[entity.index()] == Some(sibling) {
            return Err(TreeError::InvalidSibling);
        }

        // Check is sibling exists in the tree
        if sibling.index() >= self.parent.len() {
            return Err(TreeError::InvalidSibling);
        }

        // Check if sibling has the same parent
        if let Some(parent) = self.get_parent(entity) {
            if let Some(sibling_parent) = self.get_parent(entity) {
                if parent != sibling_parent {
                    return Err(TreeError::InvalidSibling);
                }
            }
        } else {
            return Err(TreeError::InvalidParent);
        }

        // Safe to unwrap because we already checked if it has a parent
        let parent = self.get_parent(entity).unwrap();

        // Temporarily store the prev_sibling of the desired sibling
        let sibling_prev_sibling = self.get_prev_sibling(sibling);
        let sibling_next_sibling = self.get_next_sibling(sibling);

        // Remove sibling
        if let Some(sps) = sibling_prev_sibling {
            self.next_sibling[sps.index()] = sibling_next_sibling; // C
        } else {
            self.first_child[parent.index()] = sibling_next_sibling;
        }

        if let Some(sns) = sibling_next_sibling {
            self.prev_sibling[sns.index()] = sibling_prev_sibling; // F
        }

        // Temporarily store the prev_sibling of the entity
        let entity_prev_sibling = self.get_prev_sibling(entity);

        if let Some(eps) = entity_prev_sibling {
            self.next_sibling[eps.index()] = Some(sibling); // A
        } else {
            self.first_child[parent.index()] = Some(sibling);
        }

        self.next_sibling[sibling.index()] = Some(entity); //E

        self.prev_sibling[sibling.index()] = entity_prev_sibling; // D

        self.prev_sibling[entity.index()] = Some(sibling); // B

        self.changed = true;

        Ok(())
    }

    pub fn set_parent(&mut self, entity: I, parent: I) {
        if let Some(old_parent) = self.get_parent(entity) {
            if self.is_first_child(entity) {
                self.first_child[old_parent.index()] = self.get_next_sibling(entity);
            }
        }

        if let Some(prev_sibling) = self.get_prev_sibling(entity) {
            self.next_sibling[prev_sibling.index()] = self.get_next_sibling(entity);
        }

        if let Some(next_sibling) = self.get_next_sibling(entity) {
            self.prev_sibling[next_sibling.index()] = self.get_prev_sibling(entity);
        }

        if self.first_child[parent.index()] == None {
            self.first_child[parent.index()] = Some(entity);
        } else {
            let mut temp = self.first_child[parent.index()];

            loop {
                if self.next_sibling[temp.unwrap().index()] == None {
                    break;
                }

                temp = self.next_sibling[temp.unwrap().index()];
            }

            self.next_sibling[temp.unwrap().index()] = Some(entity);
            self.prev_sibling[entity.index()] = temp;
        }

        self.parent[entity.index()] = Some(parent);

        self.changed = true;
    }

    pub fn set_ignored(&mut self, entity: I, flag: bool) {
        self.ignored.get_mut(entity.index()).and_then(|ignored| Some(*ignored = flag));
    }

    pub fn set_lock_focus_within(&mut self, entity: I, flag: bool) {
        self.lock_focus_within.get_mut(entity.index()).and_then(|result| Some(*result = flag));
    }

    /// Adds an entity to the tree with the specified parent.
    pub fn add(&mut self, entity: I, parent: I) -> Result<(), TreeError> {
        if entity == I::null() || parent == I::null() {
            return Err(TreeError::NullEntity);
        }

        let parent_index = parent.index();

        if parent_index >= self.parent.len() {
            return Err(TreeError::InvalidParent);
        }

        let entity_index = entity.index();

        if entity_index >= self.parent.len() {
            self.parent.resize(entity_index + 1, None);
            self.first_child.resize(entity_index + 1, None);
            self.next_sibling.resize(entity_index + 1, None);
            self.prev_sibling.resize(entity_index + 1, None);
            self.ignored.resize(entity_index + 1, false);
            self.lock_focus_within.resize(entity_index + 1, false);
        }

        self.parent[entity_index] = Some(parent);
        self.first_child[entity_index] = None;
        self.next_sibling[entity_index] = None;
        self.prev_sibling[entity_index] = None;
        self.ignored[entity_index] = false;
        self.lock_focus_within[entity_index] = false;

        // If the parent has no first child then this entity is the first child
        if self.first_child[parent_index] == None {
            self.first_child[parent_index] = Some(entity);
        } else {
            let mut temp = self.first_child[parent_index];

            loop {
                if self.next_sibling[temp.unwrap().index()] == None {
                    break;
                }

                temp = self.next_sibling[temp.unwrap().index()];
            }

            self.next_sibling[temp.unwrap().index()] = Some(entity);
            self.prev_sibling[entity_index] = temp;
        }

        self.changed = true;

        Ok(())
    }
}

impl<'a, I> IntoIterator for &'a Tree<I>
where
    I: GenerationalId,
{
    type Item = I;
    type IntoIter = TreeIterator<'a, I>;

    fn into_iter(self) -> Self::IntoIter {
        TreeIterator::full(self)
    }
}
