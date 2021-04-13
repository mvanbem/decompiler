use std::collections::HashSet;
use std::hash::Hash;

/// Tracks the states of a set of work items.
///
/// # Item states
///
/// Items are in one of three states:
///
/// - *unknown*, the initial state
/// - *open*
/// - *closed*, the final state
///
/// All items in the universe of `T` begin in the *unknown* state. They may be moved from *unknown*
/// to *open* with [`insert`](WorkSet::insert), from *open* to *closed* with [`pop`](WorkSet::pop),
/// or to *closed* from any state with [`close`](WorkSet::close).
///
/// Finally, [`peek`](WorkSet::peek) returns a reference to an arbitrary *open* item.
pub struct WorkSet<T> {
    known: HashSet<T>,
    open: HashSet<T>,
}

impl<T> WorkSet<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Moves the given item from the *unknown* state to the *open* state, if possible.
    ///
    /// If the item is already *open* or *closed*, this method has no effect.
    pub fn insert(&mut self, item: T)
    where
        T: Clone + Eq + Hash,
    {
        if !self.known.contains(&item) {
            self.known.insert(item.clone());
            self.open.insert(item);
        }
    }

    /// Returns a reference to an arbitrary *open* item, if there are any such items.
    ///
    /// # Example
    ///
    /// ```
    /// # use work_set::WorkSet;
    /// let mut work_set = WorkSet::new();
    /// work_set.insert(1);
    /// while let Some(item) = work_set.peek().copied() {
    ///     work_set.insert((item + 7) % 10);
    ///     work_set.close(item);
    /// }
    ///
    /// let mut results = work_set.iter_known().copied().collect::<Vec<_>>();
    /// results.sort();  // Note that WorkSet is unordered!
    /// assert_eq!(results, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    /// ```
    pub fn peek(&mut self) -> Option<&T>
    where
        T: Eq + Hash,
    {
        self.open.iter().next()
    }

    /// Finds an arbitrary *open* item, if there are any, moves it to the *closed* state, and
    /// returns a reference to it.
    ///
    /// # Example
    ///
    /// ```
    /// # use work_set::WorkSet;
    /// let mut work_set = WorkSet::new();
    /// work_set.insert(1);
    /// while let Some(item) = work_set.pop().copied() {
    ///     work_set.insert((item + 7) % 10);
    /// }
    ///
    /// let mut results = work_set.iter_known().copied().collect::<Vec<_>>();
    /// results.sort();  // Note that WorkSet is unordered!
    /// assert_eq!(results, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    /// ```
    pub fn pop(&mut self) -> Option<&T>
    where
        T: Eq + Hash,
    {
        let item_in_open_set = self.open.iter().next()?;
        let item_in_known_set = self.known.get(&item_in_open_set).unwrap();
        self.open.remove(item_in_known_set);
        Some(item_in_known_set)
    }

    /// Moves the given item from the *unknown* or *open* state to the *closed* state, if possible,
    /// and reports whether it did.
    ///
    /// If the item is already *closed*, this method has no effect and returns `false`.
    pub fn close(&mut self, item: T) -> bool
    where
        T: Eq + Hash,
    {
        // NOTE: Short-circuit evaluation does the right thing here. If removing the item from the
        // open set returns true, the item was in the open state, so it's already in the known set.
        self.open.remove(&item) || self.known.insert(item)
    }

    /// Returns an iterator over all *open* and *closed* items.
    pub fn iter_known(&self) -> impl Iterator<Item = &T> + '_ {
        self.known.iter()
    }
}

// Implement `Default` manually because we want it regardless of whether `T` is `Default`.
impl<T> Default for WorkSet<T> {
    fn default() -> Self {
        Self {
            known: HashSet::new(),
            open: HashSet::new(),
        }
    }
}

impl<A> Extend<A> for WorkSet<A>
where
    A: Clone + Eq + Hash,
{
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T) {
        for item in iter {
            self.insert(item);
        }
    }
}
