use swc_next_allocator::{Allocator, hash_map::HashMap as ArenaHashMap, vec::Vec as ArenaVec};

/// Tracks names used in one collision domain.
///
/// Names are copied into arena-owned storage. The map value is the next suffix
/// to try for that name, so repeated allocation does not rescan earlier
/// candidates. An empty state has a zero-capacity map and does not allocate
/// until the first name is observed.
pub struct UsedNameState<'a> {
    allocator: &'a Allocator,
    names: ArenaHashMap<'a, ArenaVec<'a, u8>, u64>,
}

impl std::fmt::Debug for UsedNameState<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UsedNameState")
            .field("names", &self.names)
            .finish()
    }
}

impl<'a> UsedNameState<'a> {
    /// Creates an empty name state backed by `allocator`.
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            allocator,
            names: ArenaHashMap::new_in(allocator),
        }
    }

    /// Records `name` and initializes its suffix cursor.
    pub fn observe(&mut self, name: &[u8]) {
        if !self.names.contains_key(name) {
            self.insert(name, 1);
        }
    }

    /// Returns whether `name` is observed or reserved.
    pub fn contains(&self, name: &[u8]) -> bool {
        self.names.contains_key(name)
    }

    /// Reserves and returns an available generated name based on `base`.
    ///
    /// The unsuffixed base is preferred. If it is occupied, suffixes start at
    /// `min_suffix`, for example `next_name("_temp", 2)` produces `_temp2`.
    pub fn next_name(&mut self, base: &str, min_suffix: u64) -> &str {
        assert!(min_suffix > 0, "generated name suffix must be positive");
        if !self.contains(base.as_bytes()) {
            self.insert(base.as_bytes(), min_suffix);
            return self.stored_name(base.as_bytes());
        }

        let mut suffix = self.names[base.as_bytes()].max(min_suffix);
        loop {
            let candidate = format!("{base}{suffix}");
            if !self.contains(candidate.as_bytes()) {
                self.insert(candidate.as_bytes(), min_suffix);
                let next_suffix = suffix
                    .checked_add(1)
                    .expect("generated name suffix overflow");
                *self
                    .names
                    .get_mut(base.as_bytes())
                    .expect("base name exists") = next_suffix;
                return self.stored_name(candidate.as_bytes());
            }
            suffix = suffix
                .checked_add(1)
                .expect("generated name suffix overflow");
        }
    }

    /// Iterates all currently observed or reserved names.
    pub fn names(&self) -> impl Iterator<Item = &[u8]> {
        self.names.keys().map(ArenaVec::as_slice)
    }

    fn insert(&mut self, name: &[u8], next_suffix: u64) {
        let mut stored = ArenaVec::with_capacity_in(name.len(), self.allocator);
        stored.extend_from_slice(name);
        self.names.insert(stored, next_suffix);
    }

    fn stored_name(&self, name: &[u8]) -> &str {
        let (stored, _) = self
            .names
            .get_key_value(name)
            .expect("used name must exist after insertion");
        std::str::from_utf8(stored).expect("generated names must be UTF-8")
    }
}
