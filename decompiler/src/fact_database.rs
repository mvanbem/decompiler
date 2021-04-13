use std::any::TypeId;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::iter::empty;

use crate::fact::{DefaultFact, Fact};

/// Facts may only be inserted, though they may carry mutable state.
pub struct FactDatabase {
    facts_by_addr: BTreeMap<u32, HashMap<TypeId, Box<dyn Fact>>>,
    facts_by_type_id: HashMap<TypeId, BTreeSet<u32>>,
}

impl FactDatabase {
    pub fn new() -> FactDatabase {
        FactDatabase {
            facts_by_addr: BTreeMap::new(),
            facts_by_type_id: HashMap::new(),
        }
    }

    pub fn get_fact<T: Fact>(&self, addr: u32) -> Option<&T> {
        Some(
            self.facts_by_addr
                .get(&addr)?
                .get(&TypeId::of::<T>())?
                .as_any()
                .downcast_ref()
                .unwrap(),
        )
    }

    pub fn get_fact_mut<T: Fact>(&mut self, addr: u32) -> Option<&mut T> {
        Some(
            self.facts_by_addr
                .get_mut(&addr)?
                .get_mut(&TypeId::of::<T>())?
                .as_any_mut()
                .downcast_mut()
                .unwrap(),
        )
    }

    pub fn fact_or_default<T: DefaultFact>(&mut self, addr: u32) -> &mut T {
        self.facts_by_type_id
            .entry(TypeId::of::<T>())
            .or_default()
            .insert(addr);
        self.facts_by_addr
            .entry(addr)
            .or_default()
            .entry(TypeId::of::<T>())
            .or_insert_with(|| T::default())
            .as_any_mut()
            .downcast_mut()
            .unwrap()
    }

    pub fn insert_fact_with<T, F>(&mut self, addr: u32, f: F)
    where
        T: Fact,
        F: FnOnce() -> T,
    {
        self.facts_by_type_id
            .entry(TypeId::of::<T>())
            .or_default()
            .insert(addr);
        self.facts_by_addr
            .entry(addr)
            .or_default()
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(f()));
    }

    pub fn iter_facts(
        &self,
    ) -> impl Iterator<Item = (u32, impl Clone + Iterator<Item = &dyn Fact> + '_)> + '_ {
        self.facts_by_addr.iter().map(|(addr, facts)| {
            let facts = facts.values().map(|boxed_fact| &**boxed_fact);
            (*addr, facts)
        })
    }

    pub fn iter_facts_with_type<T: Fact>(&self) -> Box<dyn Iterator<Item = u32> + '_> {
        match self.facts_by_type_id.get(&TypeId::of::<T>()) {
            Some(addrs) => Box::new(addrs.iter().copied()),
            None => Box::new(empty()),
        }
    }
}
