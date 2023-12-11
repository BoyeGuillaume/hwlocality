//! Full list of objects contained within the topology

use super::{
    depth::{Depth, NormalDepth},
    TopologyObject,
};
use crate::topology::Topology;
#[allow(unused)]
#[cfg(test)]
use similar_asserts::assert_eq;
use std::iter::FusedIterator;
#[cfg(test)]
use std::sync::OnceLock;

/// # Full object list
///
/// For some use cases, especially testing, it is convenient to have a full list
/// of all objects contained within a topology. These methods provide just that.
///
/// This functionality is unique to the Rust hwloc bindings
impl Topology {
    /// Full list of objects in the topology, first normal objects ordered by
    /// increasing depth then virtual objects ordered by type
    pub fn objects(&self) -> impl FusedIterator<Item = &TopologyObject> + Clone {
        self.normal_objects().chain(self.virtual_objects())
    }

    /// Pre-computed list of objects from the test instance
    #[cfg(test)]
    pub(crate) fn test_objects() -> &'static [&'static TopologyObject] {
        static OBJECTS: OnceLock<Box<[&'static TopologyObject]>> = OnceLock::new();
        &OBJECTS.get_or_init(|| Self::test_instance().objects().collect())[..]
    }

    /// Like [`Topology::test_objects()`], but for the foreign instance
    #[cfg(test)]
    pub(crate) fn foreign_objects() -> &'static [&'static TopologyObject] {
        static OBJECTS: OnceLock<Box<[&'static TopologyObject]>> = OnceLock::new();
        &OBJECTS.get_or_init(|| Self::foreign_instance().objects().collect())[..]
    }

    /// Full list of objects contains in the normal hierarchy of the topology,
    /// ordered by increasing depth
    pub fn normal_objects(&self) -> impl FusedIterator<Item = &TopologyObject> + Clone {
        NormalDepth::iter_range(NormalDepth::MIN, self.depth())
            .flat_map(|depth| self.objects_at_depth(depth))
    }

    /// Full list of virtual bjects in the topology, ordered by type
    pub fn virtual_objects(&self) -> impl FusedIterator<Item = &TopologyObject> + Clone {
        Depth::VIRTUAL_DEPTHS
            .iter()
            .flat_map(|&depth| self.objects_at_depth(depth))
    }

    /// Full list of memory objects in the topology, ordered by type
    pub fn memory_objects(&self) -> impl FusedIterator<Item = &TopologyObject> + Clone {
        Depth::MEMORY_DEPTHS
            .iter()
            .flat_map(|&depth| self.objects_at_depth(depth))
    }

    /// Full list of I/O objects in the topology, ordered by type
    pub fn io_objects(&self) -> impl FusedIterator<Item = &TopologyObject> + Clone {
        Depth::IO_DEPTHS
            .iter()
            .flat_map(|&depth| self.objects_at_depth(depth))
    }
}

#[allow(clippy::cognitive_complexity)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::ObjectType;
    use similar_asserts::assert_eq;
    use std::collections::{HashMap, HashSet};

    /// Check that the various object lists match their definitions
    #[test]
    fn object_lists() {
        let topology = Topology::test_instance();

        fn checked_object_set<'a>(
            it: impl Iterator<Item = &'a TopologyObject>,
        ) -> HashMap<u64, &'a TopologyObject> {
            let mut set = HashMap::new();
            for obj in it {
                assert!(
                    set.insert(obj.global_persistent_index(), obj).is_none(),
                    "global_persistent_index should be unique across the topology"
                );
            }
            set
        }
        let key_set = |map: &HashMap<u64, _>| -> HashSet<u64> { map.keys().copied().collect() };

        let objects = checked_object_set(topology.objects());
        let keys = key_set(&objects);

        let normal_objects = checked_object_set(topology.normal_objects());
        assert!(normal_objects
            .values()
            .all(|obj| obj.object_type().is_normal()));
        let normal_keys = key_set(&normal_objects);

        let virtual_objects = checked_object_set(topology.virtual_objects());
        assert!(virtual_objects
            .values()
            .all(|obj| !obj.object_type().is_normal()));
        let virtual_keys = key_set(&virtual_objects);

        assert_eq!(keys, &normal_keys | &virtual_keys);
        assert_eq!(normal_keys, &keys - &virtual_keys);
        assert_eq!(virtual_keys, &keys - &normal_keys);

        let memory_objects = checked_object_set(topology.memory_objects());
        assert!(memory_objects
            .values()
            .all(|obj| obj.object_type().is_memory()));
        let memory_keys = key_set(&memory_objects);

        let io_objects = checked_object_set(topology.io_objects());
        assert!(io_objects.values().all(|obj| obj.object_type().is_io()));
        let io_keys = key_set(&io_objects);

        let misc_objects = checked_object_set(topology.objects_with_type(ObjectType::Misc));
        assert!(misc_objects
            .values()
            .all(|obj| obj.object_type() == ObjectType::Misc));
        let misc_keys = key_set(&misc_objects);

        assert_eq!(virtual_keys, &(&memory_keys | &io_keys) | &misc_keys);
        assert_eq!(memory_keys, &(&virtual_keys - &io_keys) - &misc_keys);
        assert_eq!(io_keys, &(&virtual_keys - &memory_keys) - &misc_keys);
        assert_eq!(misc_keys, &(&virtual_keys - &memory_keys) - &io_keys);

        fn compare_object_sets<'a>(
            result: impl Iterator<Item = &'a TopologyObject>,
            reference: impl Iterator<Item = &'a TopologyObject>,
        ) {
            assert_eq!(
                checked_object_set(result).keys().collect::<HashSet<_>>(),
                checked_object_set(reference).keys().collect::<HashSet<_>>()
            );
        }
        compare_object_sets(
            topology.pci_devices(),
            topology.objects_with_type(ObjectType::PCIDevice),
        );
        compare_object_sets(
            topology.os_devices(),
            topology.objects_with_type(ObjectType::OSDevice),
        );
        compare_object_sets(
            topology.bridges(),
            topology.objects_with_type(ObjectType::Bridge),
        );
    }
}
