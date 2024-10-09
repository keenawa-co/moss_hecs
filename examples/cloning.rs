//! This example demonstrates using the [ColumnBatch][moss_hecs::ColumnBatch] API to efficiently clone
//! the entities in a [Frame] along with some or all components.
//!
//! Note that the cloned frame may have different iteration order and/or newly created entity ids
//! may diverge between the original and newly created frames. If that is a dealbreaker for you,
//! see https://github.com/Ralith/moss_hecs/issues/332 for some pointers on preserving entity allocator
//! state; as of time of writing, you'll need to patch `moss_hecs`.

use std::any::TypeId;

use moss_hecs::{
    Archetype, ColumnBatchBuilder, ColumnBatchType, Component, Frame, TypeIdMap, TypeInfo,
};

struct ComponentCloneMetadata {
    type_info: TypeInfo,
    insert_into_batch_func: &'static dyn Fn(&Archetype, &mut ColumnBatchBuilder),
}

/// Clones frame entities along with registered components when [Self::clone_frame()] is called.
///
/// Unregistered components are omitted from the cloned frame. Entities containing unregistered
/// components will still be cloned.
///
/// Note that entity allocator state may differ in the cloned frame - so for example a new entity
/// spawned in each frame may end up with different entity ids, entity iteration order may be
/// different, etc.
#[derive(Default)]
struct FrameCloner {
    registry: TypeIdMap<ComponentCloneMetadata>,
}

impl FrameCloner {
    pub fn register<T: Component + Clone>(&mut self) {
        self.registry.insert(
            TypeId::of::<T>(),
            ComponentCloneMetadata {
                type_info: TypeInfo::of::<T>(),
                insert_into_batch_func: &|src, dest| {
                    let mut column = dest.writer::<T>().unwrap();
                    for component in &*src.get::<&T>().unwrap() {
                        _ = column.push(component.clone());
                    }
                },
            },
        );
    }

    fn clone_frame(&self, frame: &Frame) -> Frame {
        let mut cloned = Frame::new();

        for archetype in frame.archetypes() {
            let mut batch_type = ColumnBatchType::new();
            for (&type_id, clone_metadata) in self.registry.iter() {
                if archetype.has_dynamic(type_id) {
                    batch_type.add_dynamic(clone_metadata.type_info);
                }
            }

            let mut batch_builder = batch_type.into_batch(archetype.ids().len() as u32);
            for (&type_id, clone_metadata) in self.registry.iter() {
                if archetype.has_dynamic(type_id) {
                    (clone_metadata.insert_into_batch_func)(archetype, &mut batch_builder)
                }
            }

            let batch = batch_builder.build().expect("batch should be complete");
            let handles = &cloned
                .reserve_entities(archetype.ids().len() as u32)
                .collect::<Vec<_>>();
            cloned.flush();
            cloned.spawn_column_batch_at(handles, batch);
        }

        cloned
    }
}

pub fn main() {
    let int0 = 0;
    let int1 = 1;
    let str0 = "Ada".to_owned();
    let str1 = "Bob".to_owned();
    let str2 = "Cal".to_owned();

    let mut frame0 = Frame::new();
    let entity0 = frame0.spawn((int0, str0));
    let entity1 = frame0.spawn((int1, str1));
    let entity2 = frame0.spawn((str2,));
    let entity3 = frame0.spawn((0u8,)); // unregistered component

    let mut cloner = FrameCloner::default();
    cloner.register::<i32>();
    cloner.register::<String>();

    let frame1 = cloner.clone_frame(&frame0);

    assert_eq!(
        frame0.len(),
        frame1.len(),
        "cloned frame should have same entity count as original frame"
    );

    // NB: unregistered components don't get cloned
    assert!(
        frame0
            .entity(entity3)
            .expect("w0 entity3 should exist")
            .has::<u8>(),
        "original frame entity has u8 component"
    );
    assert!(
        !frame1
            .entity(entity3)
            .expect("w1 entity3 should exist")
            .has::<u8>(),
        "cloned frame entity does not have u8 component because it was not registered"
    );

    type AllRegisteredComponentsQuery = (&'static i32, &'static String);
    for entity in [entity0, entity1] {
        let w0_e = frame0.entity(entity).expect("w0 entity should exist");
        let w1_e = frame1.entity(entity).expect("w1 entity should exist");
        assert!(w0_e.satisfies::<AllRegisteredComponentsQuery>());
        assert!(w1_e.satisfies::<AllRegisteredComponentsQuery>());

        assert_eq!(
            w0_e.query::<AllRegisteredComponentsQuery>().get().unwrap(),
            w1_e.query::<AllRegisteredComponentsQuery>().get().unwrap()
        );
    }

    type SomeRegisteredComponentsQuery = (&'static String,);
    let w0_e = frame0.entity(entity2).expect("w0 entity2 should exist");
    let w1_e = frame1.entity(entity2).expect("w1 entity2 should exist");
    assert!(w0_e.satisfies::<SomeRegisteredComponentsQuery>());
    assert!(w1_e.satisfies::<SomeRegisteredComponentsQuery>());

    assert_eq!(
        w0_e.query::<SomeRegisteredComponentsQuery>().get().unwrap(),
        w1_e.query::<SomeRegisteredComponentsQuery>().get().unwrap()
    );
}
