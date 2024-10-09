#![allow(deprecated)]
// Copyright 2019 Google LLC
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::borrow::Cow;

use moss_hecs::*;

#[test]
fn random_access() {
    let mut frame = Frame::new();

    let e = frame.spawn(("abc", 123));
    let f = frame.spawn(("def", 456, true));
    assert_eq!(*frame.get::<&&str>(e).unwrap(), "abc");
    assert_eq!(*frame.get::<&i32>(e).unwrap(), 123);
    assert_eq!(*frame.get::<&&str>(f).unwrap(), "def");
    assert_eq!(*frame.get::<&i32>(f).unwrap(), 456);
    *frame.get::<&mut i32>(f).unwrap() = 42;
    assert_eq!(*frame.get::<&i32>(f).unwrap(), 42);
}

#[test]
fn despawn() {
    let mut frame = Frame::new();
    let e = frame.spawn(("abc", 123));
    let f = frame.spawn(("def", 456));
    assert_eq!(frame.query::<()>().iter().count(), 2);
    frame.despawn(e).unwrap();
    assert_eq!(frame.query::<()>().iter().count(), 1);
    assert!(frame.get::<&&str>(e).is_err());
    assert!(frame.get::<&i32>(e).is_err());
    assert_eq!(*frame.get::<&&str>(f).unwrap(), "def");
    assert_eq!(*frame.get::<&i32>(f).unwrap(), 456);
}

#[test]
fn query_all() {
    let mut frame = Frame::new();
    let e = frame.spawn(("abc", 123));
    let f = frame.spawn(("def", 456));

    let ents = frame
        .query::<(&i32, &&str)>()
        .iter()
        .map(|(e, (&i, &s))| (e, i, s))
        .collect::<Vec<_>>();
    assert_eq!(ents.len(), 2);
    assert!(ents.contains(&(e, 123, "abc")));
    assert!(ents.contains(&(f, 456, "def")));

    let ents = frame.query::<()>().iter().collect::<Vec<_>>();
    assert_eq!(ents.len(), 2);
    assert!(ents.contains(&(e, ())));
    assert!(ents.contains(&(f, ())));
}

#[test]
#[cfg(feature = "macros")]
fn derived_query() {
    #[derive(Query, Debug, PartialEq)]
    struct Foo<'a> {
        x: &'a i32,
        y: &'a mut bool,
    }

    let mut frame = Frame::new();
    let e = frame.spawn((42, false));
    assert_eq!(
        frame.query_one_mut::<Foo>(e).unwrap(),
        Foo {
            x: &42,
            y: &mut false
        }
    );
}

#[test]
#[cfg(feature = "macros")]
fn derived_bundle_clone() {
    #[derive(Bundle, DynamicBundleClone)]
    struct Foo<T: Clone + Component> {
        x: i32,
        y: bool,
        z: T,
    }

    #[derive(PartialEq, Debug, Query)]
    struct FooQuery<'a> {
        x: &'a i32,
        y: &'a bool,
        z: &'a String,
    }

    let mut frame = Frame::new();
    let mut builder = EntityBuilderClone::new();
    builder.add_bundle(Foo {
        x: 42,
        y: false,
        z: String::from("Foo"),
    });

    let entity = builder.build();
    let e = frame.spawn(&entity);
    assert_eq!(
        frame.query_one_mut::<FooQuery>(e).unwrap(),
        FooQuery {
            x: &42,
            y: &false,
            z: &String::from("Foo"),
        }
    );
}

#[test]
fn query_single_component() {
    let mut frame = Frame::new();
    let e = frame.spawn(("abc", 123));
    let f = frame.spawn(("def", 456, true));
    let ents = frame
        .query::<&i32>()
        .iter()
        .map(|(e, &i)| (e, i))
        .collect::<Vec<_>>();
    assert_eq!(ents.len(), 2);
    assert!(ents.contains(&(e, 123)));
    assert!(ents.contains(&(f, 456)));
}

#[test]
fn query_missing_component() {
    let mut frame = Frame::new();
    frame.spawn(("abc", 123));
    frame.spawn(("def", 456));
    assert!(frame.query::<(&bool, &i32)>().iter().next().is_none());
}

#[test]
fn query_sparse_component() {
    let mut frame = Frame::new();
    frame.spawn(("abc", 123));
    let f = frame.spawn(("def", 456, true));
    let ents = frame
        .query::<&bool>()
        .iter()
        .map(|(e, &b)| (e, b))
        .collect::<Vec<_>>();
    assert_eq!(ents, &[(f, true)]);
}

#[test]
fn query_optional_component() {
    let mut frame = Frame::new();
    let e = frame.spawn(("abc", 123));
    let f = frame.spawn(("def", 456, true));
    let ents = frame
        .query::<(Option<&bool>, &i32)>()
        .iter()
        .map(|(e, (b, &i))| (e, b.copied(), i))
        .collect::<Vec<_>>();
    assert_eq!(ents.len(), 2);
    assert!(ents.contains(&(e, None, 123)));
    assert!(ents.contains(&(f, Some(true), 456)));
}

#[test]
fn prepare_query() {
    let mut frame = Frame::new();
    let e = frame.spawn(("abc", 123));
    let f = frame.spawn(("def", 456));

    let mut query = PreparedQuery::<(&i32, &&str)>::default();

    let ents = query
        .query(&frame)
        .iter()
        .map(|(e, (&i, &s))| (e, i, s))
        .collect::<Vec<_>>();
    assert_eq!(ents.len(), 2);
    assert!(ents.contains(&(e, 123, "abc")));
    assert!(ents.contains(&(f, 456, "def")));

    let ents = query
        .query_mut(&mut frame)
        .map(|(e, (&i, &s))| (e, i, s))
        .collect::<Vec<_>>();
    assert_eq!(ents.len(), 2);
    assert!(ents.contains(&(e, 123, "abc")));
    assert!(ents.contains(&(f, 456, "def")));
}

#[test]
fn invalidate_prepared_query() {
    let mut frame = Frame::new();
    let e = frame.spawn(("abc", 123));
    let f = frame.spawn(("def", 456));

    let mut query = PreparedQuery::<(&i32, &&str)>::default();

    let ents = query
        .query(&frame)
        .iter()
        .map(|(e, (&i, &s))| (e, i, s))
        .collect::<Vec<_>>();
    assert_eq!(ents.len(), 2);
    assert!(ents.contains(&(e, 123, "abc")));
    assert!(ents.contains(&(f, 456, "def")));

    frame.spawn((true,));
    let g = frame.spawn(("ghi", 789));

    let ents = query
        .query_mut(&mut frame)
        .map(|(e, (&i, &s))| (e, i, s))
        .collect::<Vec<_>>();
    assert_eq!(ents.len(), 3);
    assert!(ents.contains(&(e, 123, "abc")));
    assert!(ents.contains(&(f, 456, "def")));
    assert!(ents.contains(&(g, 789, "ghi")));
}

#[test]
fn random_access_via_view() {
    let mut frame = Frame::new();
    let e = frame.spawn(("abc", 123));
    let f = frame.spawn(("def",));

    let mut query = PreparedQuery::<(&i32, &&str)>::default();
    let mut query = query.query(&frame);
    let mut view = query.view();

    let (i, s) = view.get(e).unwrap();
    assert_eq!(*i, 123);
    assert_eq!(*s, "abc");

    assert!(view.get_mut(f).is_none());
}

#[test]
fn random_access_via_view_mut() {
    let mut frame = Frame::new();
    let e = frame.spawn(("abc", 123));
    let f = frame.spawn(("def",));

    let mut query = PreparedQuery::<(&i32, &&str)>::default();
    let mut view = query.view_mut(&mut frame);

    let (i, s) = view.get(e).unwrap();
    assert_eq!(*i, 123);
    assert_eq!(*s, "abc");

    assert!(view.get_mut(f).is_none());

    assert!(view.contains(e));
    assert!(!view.contains(f));
}

#[test]
fn view_borrow_on_frame() {
    let mut frame = Frame::new();
    let e0 = frame.spawn((3, "hello"));
    let e1 = frame.spawn((6.0, "frame"));
    let e2 = frame.spawn((12,));

    {
        let str_view = frame.view::<&&str>();

        assert_eq!(*str_view.get(e0).unwrap(), "hello");
        assert_eq!(*str_view.get(e1).unwrap(), "frame");
        assert_eq!(str_view.get(e2), None);
    }

    {
        let mut int_view = frame.view::<&mut i32>();
        assert_eq!(*int_view.get_mut(e0).unwrap(), 3);
        assert_eq!(int_view.get_mut(e1), None);
        assert_eq!(*int_view.get_mut(e2).unwrap(), 12);

        // edit some value
        *int_view.get_mut(e0).unwrap() = 100;
    }

    {
        let mut int_str_view = frame.view::<(&&str, &mut i32)>();
        let (s, i) = int_str_view.get_mut(e0).unwrap();
        assert_eq!(*s, "hello");
        assert_eq!(*i, 100);
        assert_eq!(int_str_view.get_mut(e1), None);
        assert_eq!(int_str_view.get_mut(e2), None);
    }
}

#[test]
fn view_mut_on_frame() {
    let mut frame = Frame::new();
    let e0 = frame.spawn((3, "hello"));
    let e1 = frame.spawn((6.0, "frame"));
    let e2 = frame.spawn((12,));

    let str_view = frame.view_mut::<&&str>();
    assert_eq!(*str_view.get(e0).unwrap(), "hello");
    assert_eq!(*str_view.get(e1).unwrap(), "frame");
    assert_eq!(str_view.get(e2), None);

    let mut int_view = frame.view_mut::<&mut i32>();
    assert_eq!(*int_view.get_mut(e0).unwrap(), 3);
    assert_eq!(int_view.get_mut(e1), None);
    assert_eq!(*int_view.get_mut(e2).unwrap(), 12);

    // edit some value
    *int_view.get_mut(e0).unwrap() = 100;

    let mut int_str_view = frame.view_mut::<(&&str, &mut i32)>();
    let (s, i) = int_str_view.get_mut(e0).unwrap();
    assert_eq!(*s, "hello");
    assert_eq!(*i, 100);
    assert_eq!(int_str_view.get_mut(e1), None);
    assert_eq!(int_str_view.get_mut(e2), None);
}

#[should_panic]
#[test]
fn view_mut_panic() {
    let mut frame = Frame::new();
    let e = frame.spawn(('a',));

    // we should panic since we have two overlapping views:
    let mut first_view = frame.view::<&mut char>();
    let mut second_view = frame.view::<&mut char>();

    first_view.get_mut(e).unwrap();
    second_view.get_mut(e).unwrap();
}

#[test]
#[should_panic]
fn simultaneous_access_must_be_non_overlapping() {
    let mut frame = Frame::new();
    let a = frame.spawn((1,));
    let b = frame.spawn((2,));
    let c = frame.spawn((3,));
    let d = frame.spawn((4,));

    let mut query = frame.query_mut::<&mut i32>();
    let mut view = query.view();

    view.get_mut_n([a, d, c, b, a]);
}

#[test]
fn build_entity() {
    let mut frame = Frame::new();
    let mut entity = EntityBuilder::new();
    entity.add("abc");
    entity.add(123);
    let e = frame.spawn(entity.build());
    entity.add("def");
    entity.add([0u8; 1024]);
    entity.add(456);
    entity.add(789);
    let f = frame.spawn(entity.build());
    assert_eq!(*frame.get::<&&str>(e).unwrap(), "abc");
    assert_eq!(*frame.get::<&i32>(e).unwrap(), 123);
    assert_eq!(*frame.get::<&&str>(f).unwrap(), "def");
    assert_eq!(*frame.get::<&i32>(f).unwrap(), 789);
}

#[test]
fn build_entity_clone() {
    let mut frame = Frame::new();
    let mut entity = EntityBuilderClone::new();
    entity.add("def");
    entity.add([0u8; 1024]);
    entity.add(456);
    entity.add(789);
    entity.add_bundle(("yup", 67_usize));
    entity.add_bundle((5.0_f32, String::from("Foo")));
    entity.add_bundle((7.0_f32, String::from("Bar"), 42_usize));
    let entity = entity.build();
    let e = frame.spawn(&entity);
    let f = frame.spawn(&entity);
    let g = frame.spawn(&entity);
    frame
        .insert_one(g, Cow::<'static, str>::from("after"))
        .unwrap();

    for e in [e, f, g] {
        assert_eq!(*frame.get::<&&str>(e).unwrap(), "yup");
        assert_eq!(*frame.get::<&i32>(e).unwrap(), 789);
        assert_eq!(*frame.get::<&usize>(e).unwrap(), 42);
        assert_eq!(*frame.get::<&f32>(e).unwrap(), 7.0);
        assert_eq!(*frame.get::<&String>(e).unwrap(), "Bar");
    }

    assert_eq!(*frame.get::<&Cow<'static, str>>(g).unwrap(), "after");
}

#[test]
fn build_builder_clone() {
    let mut a = EntityBuilderClone::new();
    a.add(String::from("abc"));
    a.add(123);
    let mut b = EntityBuilderClone::new();
    b.add(String::from("def"));
    b.add_bundle(&a.build());
    assert_eq!(b.get::<&String>(), Some(&String::from("abc")));
    assert_eq!(b.get::<&i32>(), Some(&123));
}

#[test]
#[allow(clippy::redundant_clone)]
fn cloned_builder() {
    let mut builder = EntityBuilderClone::new();
    builder.add(String::from("abc")).add(123);

    let mut frame = Frame::new();
    let e = frame.spawn(&builder.build().clone());
    assert_eq!(*frame.get::<&String>(e).unwrap(), "abc");
    assert_eq!(*frame.get::<&i32>(e).unwrap(), 123);
}

#[test]
#[cfg(feature = "macros")]
fn build_dynamic_bundle() {
    #[derive(Bundle, DynamicBundleClone)]
    struct Foo {
        x: i32,
        y: char,
    }

    let mut frame = Frame::new();
    let mut entity = EntityBuilderClone::new();
    entity.add_bundle(Foo { x: 5, y: 'c' });
    entity.add_bundle((String::from("Bar"), 6.0_f32));
    entity.add('a');
    let entity = entity.build();
    let e = frame.spawn(&entity);
    let f = frame.spawn(&entity);
    let g = frame.spawn(&entity);

    frame
        .insert_one(g, Cow::<'static, str>::from("after"))
        .unwrap();

    for e in [e, f, g] {
        assert_eq!(*frame.get::<&i32>(e).unwrap(), 5);
        assert_eq!(*frame.get::<&char>(e).unwrap(), 'a');
        assert_eq!(*frame.get::<&String>(e).unwrap(), "Bar");
        assert_eq!(*frame.get::<&f32>(e).unwrap(), 6.0);
    }

    assert_eq!(*frame.get::<&Cow<'static, str>>(g).unwrap(), "after");
}

#[test]
fn access_builder_components() {
    let mut frame = Frame::new();
    let mut entity = EntityBuilder::new();

    entity.add("abc");
    entity.add(123);

    assert!(entity.has::<&str>());
    assert!(entity.has::<i32>());
    assert!(!entity.has::<usize>());

    assert_eq!(*entity.get::<&&str>().unwrap(), "abc");
    assert_eq!(*entity.get::<&i32>().unwrap(), 123);
    assert_eq!(entity.get::<&usize>(), None);

    *entity.get_mut::<&mut i32>().unwrap() = 456;
    assert_eq!(*entity.get::<&i32>().unwrap(), 456);

    let g = frame.spawn(entity.build());

    assert_eq!(*frame.get::<&&str>(g).unwrap(), "abc");
    assert_eq!(*frame.get::<&i32>(g).unwrap(), 456);
}

#[test]
fn build_entity_bundle() {
    let mut frame = Frame::new();
    let mut entity = EntityBuilder::new();
    entity.add_bundle(("abc", 123));
    let e = frame.spawn(entity.build());
    entity.add(456);
    entity.add_bundle(("def", [0u8; 1024], 789));
    let f = frame.spawn(entity.build());
    assert_eq!(*frame.get::<&&str>(e).unwrap(), "abc");
    assert_eq!(*frame.get::<&i32>(e).unwrap(), 123);
    assert_eq!(*frame.get::<&&str>(f).unwrap(), "def");
    assert_eq!(*frame.get::<&i32>(f).unwrap(), 789);
}

#[test]
fn dynamic_components() {
    let mut frame = Frame::new();
    let e = frame.spawn((42,));
    frame.insert(e, (true, "abc")).unwrap();
    assert_eq!(
        frame
            .query::<(&i32, &bool)>()
            .iter()
            .map(|(e, (&i, &b))| (e, i, b))
            .collect::<Vec<_>>(),
        &[(e, 42, true)]
    );
    assert_eq!(frame.remove_one::<i32>(e), Ok(42));
    assert_eq!(
        frame
            .query::<(&i32, &bool)>()
            .iter()
            .map(|(e, (&i, &b))| (e, i, b))
            .collect::<Vec<_>>(),
        &[]
    );
    assert_eq!(
        frame
            .query::<(&bool, &&str)>()
            .iter()
            .map(|(e, (&b, &s))| (e, b, s))
            .collect::<Vec<_>>(),
        &[(e, true, "abc")]
    );
}

#[test]
fn spawn_buffered_entity() {
    let mut frame = Frame::new();
    let mut buffer = CommandBuffer::new();
    let ent = frame.reserve_entity();
    let ent1 = frame.reserve_entity();
    let ent2 = frame.reserve_entity();
    let ent3 = frame.reserve_entity();

    buffer.insert(ent, (1, true));
    buffer.insert(ent1, (13, 7.11, "moss_hecs"));
    buffer.insert(ent2, (17i8, false, 'o'));
    buffer.insert(ent3, (2u8, "qwe", 101.103, false));

    buffer.run_on(&mut frame);

    assert!(*frame.get::<&bool>(ent).unwrap());
    assert!(!*frame.get::<&bool>(ent2).unwrap());

    assert_eq!(*frame.get::<&&str>(ent1).unwrap(), "moss_hecs");
    assert_eq!(*frame.get::<&i32>(ent1).unwrap(), 13);
    assert_eq!(*frame.get::<&u8>(ent3).unwrap(), 2);
}

#[test]
fn despawn_buffered_entity() {
    let mut frame = Frame::new();
    let mut buffer = CommandBuffer::new();
    let ent = frame.spawn((1, true));
    buffer.despawn(ent);

    buffer.run_on(&mut frame);
    assert!(!frame.contains(ent));
}

#[test]
fn remove_buffered_component() {
    let mut frame = Frame::new();
    let mut buffer = CommandBuffer::new();
    let ent = frame.spawn((7, true, "moss_hecs"));

    buffer.remove::<(i32, &str)>(ent);
    buffer.run_on(&mut frame);

    assert!(frame.get::<&&str>(ent).is_err());
    assert!(frame.get::<&i32>(ent).is_err());
}

#[test]
#[should_panic(expected = "already borrowed")]
fn illegal_borrow() {
    let mut frame = Frame::new();
    frame.spawn(("abc", 123));
    frame.spawn(("def", 456));

    frame.query::<(&mut i32, &i32)>().iter();
}

#[test]
#[should_panic(expected = "already borrowed")]
fn illegal_borrow_2() {
    let mut frame = Frame::new();
    frame.spawn(("abc", 123));
    frame.spawn(("def", 456));

    frame.query::<(&mut i32, &mut i32)>().iter();
}

#[test]
#[should_panic(expected = "query violates a unique borrow")]
fn illegal_query_mut_borrow() {
    let mut frame = Frame::new();
    frame.spawn(("abc", 123));
    frame.spawn(("def", 456));

    frame.query_mut::<(&i32, &mut i32)>();
}

#[test]
#[should_panic(expected = "query violates a unique borrow")]
fn illegal_query_one_borrow() {
    let mut frame = Frame::new();
    let entity = frame.spawn(("abc", 123));

    frame.query_one::<(&mut i32, &i32)>(entity).unwrap();
}

#[test]
#[should_panic(expected = "query violates a unique borrow")]
fn illegal_query_one_borrow_2() {
    let mut frame = Frame::new();
    let entity = frame.spawn(("abc", 123));

    frame.query_one::<(&mut i32, &mut i32)>(entity).unwrap();
}

#[test]
#[should_panic(expected = "query violates a unique borrow")]
fn illegal_query_one_mut_borrow() {
    let mut frame = Frame::new();
    let entity = frame.spawn(("abc", 123));

    frame.query_one_mut::<(&mut i32, &i32)>(entity).unwrap();
}

#[test]
#[should_panic(expected = "query violates a unique borrow")]
fn illegal_query_one_mut_borrow_2() {
    let mut frame = Frame::new();
    let entity = frame.spawn(("abc", 123));

    frame.query_one_mut::<(&mut i32, &mut i32)>(entity).unwrap();
}

#[test]
fn disjoint_queries() {
    let mut frame = Frame::new();
    frame.spawn(("abc", true));
    frame.spawn(("def", 456));

    let _a = frame.query::<(&mut &str, &bool)>();
    let _b = frame.query::<(&mut &str, &i32)>();
}

#[test]
fn shared_borrow() {
    let mut frame = Frame::new();
    frame.spawn(("abc", 123));
    frame.spawn(("def", 456));

    frame.query::<(&i32, &i32)>();
}

#[test]
#[should_panic(expected = "already borrowed")]
fn illegal_random_access() {
    let mut frame = Frame::new();
    let e = frame.spawn(("abc", 123));
    let _borrow = frame.get::<&mut i32>(e).unwrap();
    frame.get::<&i32>(e).unwrap();
}

#[test]
#[cfg(feature = "macros")]
fn derived_bundle() {
    #[derive(Bundle)]
    struct Foo {
        x: i32,
        y: char,
    }

    let mut frame = Frame::new();
    let e = frame.spawn(Foo { x: 42, y: 'a' });
    assert_eq!(*frame.get::<&i32>(e).unwrap(), 42);
    assert_eq!(*frame.get::<&char>(e).unwrap(), 'a');
}

#[test]
#[cfg(feature = "macros")]
#[cfg_attr(
    debug_assertions,
    should_panic(
        expected = "attempted to allocate entity with duplicate i32 components; each type must occur at most once!"
    )
)]
#[cfg_attr(
    not(debug_assertions),
    should_panic(
        expected = "attempted to allocate entity with duplicate components; each type must occur at most once!"
    )
)]
fn bad_bundle_derive() {
    #[derive(Bundle)]
    struct Foo {
        x: i32,
        y: i32,
    }

    let mut frame = Frame::new();
    frame.spawn(Foo { x: 42, y: 42 });
}

#[test]
#[cfg_attr(miri, ignore)]
fn spawn_many() {
    let mut frame = Frame::new();
    const N: usize = 100_000;
    for _ in 0..N {
        frame.spawn((42u128,));
    }
    assert_eq!(frame.iter().count(), N);
}

#[test]
fn clear() {
    let mut frame = Frame::new();
    frame.spawn(("abc", 123));
    frame.spawn(("def", 456, true));
    frame.clear();
    assert_eq!(frame.iter().count(), 0);
}

#[test]
fn remove_missing() {
    let mut frame = Frame::new();
    let e = frame.spawn(("abc", 123));
    assert!(frame.remove_one::<bool>(e).is_err());
}

#[test]
fn exchange_components() {
    let mut frame = Frame::new();

    let entity = frame.spawn(("abc".to_owned(), 123));
    assert!(frame.get::<&String>(entity).is_ok());
    assert!(frame.get::<&i32>(entity).is_ok());
    assert!(frame.get::<&bool>(entity).is_err());

    frame.exchange_one::<String, _>(entity, true).unwrap();
    assert!(frame.get::<&String>(entity).is_err());
    assert!(frame.get::<&i32>(entity).is_ok());
    assert!(frame.get::<&bool>(entity).is_ok());
}

#[test]
fn reserve() {
    let mut frame = Frame::new();
    let a = frame.reserve_entity();
    let b = frame.reserve_entity();

    assert_eq!(frame.query::<()>().iter().count(), 0);

    frame.flush();

    let entities = frame
        .query::<()>()
        .iter()
        .map(|(e, ())| e)
        .collect::<Vec<_>>();

    assert_eq!(entities.len(), 2);
    assert!(entities.contains(&a));
    assert!(entities.contains(&b));
}

#[test]
fn query_batched() {
    let mut frame = Frame::new();
    let a = frame.spawn(());
    let b = frame.spawn(());
    let c = frame.spawn((42,));
    assert_eq!(frame.query::<()>().iter_batched(1).count(), 3);
    assert_eq!(frame.query::<()>().iter_batched(2).count(), 2);
    assert_eq!(frame.query::<()>().iter_batched(2).flatten().count(), 3);
    // different archetypes are always in different batches
    assert_eq!(frame.query::<()>().iter_batched(3).count(), 2);
    assert_eq!(frame.query::<()>().iter_batched(3).flatten().count(), 3);
    assert_eq!(frame.query::<()>().iter_batched(4).count(), 2);
    let entities = frame
        .query::<()>()
        .iter_batched(1)
        .flatten()
        .map(|(e, ())| e)
        .collect::<Vec<_>>();
    dbg!(&entities);
    assert_eq!(entities.len(), 3);
    assert!(entities.contains(&a));
    assert!(entities.contains(&b));
    assert!(entities.contains(&c));
}

#[test]
fn query_mut_batched() {
    let mut frame = Frame::new();
    let a = frame.spawn(());
    let b = frame.spawn(());
    let c = frame.spawn((42,));
    assert_eq!(frame.query_mut::<()>().into_iter_batched(1).count(), 3);
    assert_eq!(frame.query_mut::<()>().into_iter_batched(2).count(), 2);
    assert_eq!(
        frame
            .query_mut::<()>()
            .into_iter_batched(2)
            .flatten()
            .count(),
        3
    );
    // different archetypes are always in different batches
    assert_eq!(frame.query_mut::<()>().into_iter_batched(3).count(), 2);
    assert_eq!(
        frame
            .query_mut::<()>()
            .into_iter_batched(3)
            .flatten()
            .count(),
        3
    );
    assert_eq!(frame.query_mut::<()>().into_iter_batched(4).count(), 2);
    let entities = frame
        .query_mut::<()>()
        .into_iter_batched(1)
        .flatten()
        .map(|(e, ())| e)
        .collect::<Vec<_>>();
    dbg!(&entities);
    assert_eq!(entities.len(), 3);
    assert!(entities.contains(&a));
    assert!(entities.contains(&b));
    assert!(entities.contains(&c));
}

#[test]
fn spawn_batch() {
    let mut frame = Frame::new();
    frame.spawn_batch((0..10).map(|x| (x, "abc")));
    let entity_count = frame.query::<&i32>().iter().count();
    assert_eq!(entity_count, 10);
}

#[test]
fn query_one() {
    let mut frame = Frame::new();
    let a = frame.spawn(("abc", 123));
    let b = frame.spawn(("def", 456));
    let c = frame.spawn(("ghi", 789, true));
    assert_eq!(frame.query_one::<&i32>(a).unwrap().get(), Some(&123));
    assert_eq!(frame.query_one::<&i32>(b).unwrap().get(), Some(&456));
    assert!(frame.query_one::<(&i32, &bool)>(a).unwrap().get().is_none());
    assert_eq!(
        frame.query_one::<(&i32, &bool)>(c).unwrap().get(),
        Some((&789, &true))
    );
    frame.despawn(a).unwrap();
    assert!(frame.query_one::<&i32>(a).is_err());
}

#[test]
#[cfg_attr(
    debug_assertions,
    should_panic(
        expected = "attempted to allocate entity with duplicate f32 components; each type must occur at most once!"
    )
)]
#[cfg_attr(
    not(debug_assertions),
    should_panic(
        expected = "attempted to allocate entity with duplicate components; each type must occur at most once!"
    )
)]
fn duplicate_components_panic() {
    let mut frame = Frame::new();
    frame.reserve::<(f32, i64, f32)>(1);
}

#[test]
fn spawn_column_batch() {
    let mut frame = Frame::new();
    let mut batch_ty = ColumnBatchType::new();
    batch_ty.add::<i32>().add::<bool>();

    // Unique archetype
    let b;
    {
        let mut batch = batch_ty.clone().into_batch(2);
        let mut bs = batch.writer::<bool>().unwrap();
        bs.push(true).unwrap();
        bs.push(false).unwrap();
        let mut is = batch.writer::<i32>().unwrap();
        is.push(42).unwrap();
        is.push(43).unwrap();
        let entities = frame
            .spawn_column_batch(batch.build().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(entities.len(), 2);
        assert_eq!(
            frame.query_one_mut::<(&i32, &bool)>(entities[0]).unwrap(),
            (&42, &true)
        );
        assert_eq!(
            frame.query_one_mut::<(&i32, &bool)>(entities[1]).unwrap(),
            (&43, &false)
        );
        frame.despawn(entities[0]).unwrap();
        b = entities[1];
    }

    // Duplicate archetype
    {
        let mut batch = batch_ty.clone().into_batch(2);
        let mut bs = batch.writer::<bool>().unwrap();
        bs.push(true).unwrap();
        bs.push(false).unwrap();
        let mut is = batch.writer::<i32>().unwrap();
        is.push(44).unwrap();
        is.push(45).unwrap();
        let entities = frame
            .spawn_column_batch(batch.build().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(entities.len(), 2);
        assert_eq!(*frame.get::<&i32>(b).unwrap(), 43);
        assert_eq!(*frame.get::<&i32>(entities[0]).unwrap(), 44);
        assert_eq!(*frame.get::<&i32>(entities[1]).unwrap(), 45);
    }
}

#[test]
fn columnar_access() {
    let mut frame = Frame::new();
    let e = frame.spawn(("abc", 123));
    let f = frame.spawn(("def", 456, true));
    let g = frame.spawn(("ghi", 789, false));
    let mut archetypes = frame.archetypes();
    let _empty = archetypes.next().unwrap();
    let a = archetypes.next().unwrap();
    assert_eq!(a.ids(), &[e.id()]);
    assert_eq!(*a.get::<&i32>().unwrap(), [123]);
    assert!(a.get::<&bool>().is_none());
    let b = archetypes.next().unwrap();
    assert_eq!(b.ids(), &[f.id(), g.id()]);
    assert_eq!(*b.get::<&i32>().unwrap(), [456, 789]);
}

#[test]
fn empty_entity_ref() {
    let mut frame = Frame::new();
    let e = frame.spawn(());
    let r = frame.entity(e).unwrap();
    assert_eq!(r.entity(), e);
}

#[test]
fn query_or() {
    let mut frame = Frame::new();
    let e = frame.spawn(("abc", 123));
    let _ = frame.spawn(("def",));
    let f = frame.spawn(("ghi", true));
    let g = frame.spawn(("jkl", 456, false));
    let results = frame
        .query::<(&&str, Or<&i32, &bool>)>()
        .iter()
        .map(|(handle, (&s, value))| (handle, s, value.cloned()))
        .collect::<Vec<_>>();
    assert_eq!(results.len(), 3);
    assert!(results.contains(&(e, "abc", Or::Left(123))));
    assert!(results.contains(&(f, "ghi", Or::Right(true))));
    assert!(results.contains(&(g, "jkl", Or::Both(456, false))));
}

#[test]
fn len() {
    let mut frame = Frame::new();
    let ent = frame.spawn(());
    frame.spawn(());
    frame.spawn(());
    assert_eq!(frame.len(), 3);
    frame.despawn(ent).unwrap();
    assert_eq!(frame.len(), 2);
    frame.clear();
    assert_eq!(frame.len(), 0);
}

#[test]
fn take() {
    let mut frame_a = Frame::new();
    let e = frame_a.spawn(("abc".to_string(), 42));
    let f = frame_a.spawn(("def".to_string(), 17));
    let mut frame_b = Frame::new();
    let e2 = frame_b.spawn(frame_a.take(e).unwrap());
    assert!(!frame_a.contains(e));
    assert_eq!(*frame_b.get::<&String>(e2).unwrap(), "abc");
    assert_eq!(*frame_b.get::<&i32>(e2).unwrap(), 42);
    assert_eq!(*frame_a.get::<&String>(f).unwrap(), "def");
    assert_eq!(*frame_a.get::<&i32>(f).unwrap(), 17);
    frame_b.take(e2).unwrap();
    assert!(!frame_b.contains(e2));
}

#[test]
fn empty_archetype_conflict() {
    let mut frame = Frame::new();
    let _ = frame.spawn((42, true));
    let _ = frame.spawn((17, "abc"));
    let e = frame.spawn((12, false, "def"));
    frame.despawn(e).unwrap();
    for _ in frame.query::<(&mut i32, &&str)>().iter() {
        for _ in frame.query::<(&mut i32, &bool)>().iter() {}
    }
}

#[test]
fn component_ref_map() {
    struct TestComponent {
        id: i32,
    }

    let mut frame = Frame::new();
    let e = frame.spawn((TestComponent { id: 21 },));

    let e_ref = frame.entity(e).unwrap();
    {
        let comp = e_ref.get::<&'_ TestComponent>().unwrap();
        // Test that no unbalanced releases occur when cloning refs.
        let _comp2 = comp.clone();
        let id = Ref::map(comp, |c| &c.id);
        assert_eq!(*id, 21);
    }

    {
        let comp = e_ref.get::<&'_ mut TestComponent>().unwrap();
        let mut id = RefMut::map(comp, |c| &mut c.id);
        *id = 31;
    }

    {
        let comp = e_ref.get::<&'_ TestComponent>().unwrap();
        let id = Ref::map(comp, |c| &c.id);
        assert_eq!(*id, 31);
    }
}

#[test]
fn query_many() {
    let mut frame = Frame::new();
    let a = frame.spawn((42, true));
    let b = frame.spawn((17,));
    assert_eq!(frame.query_many_mut::<&i32, 2>([a, b]), [Ok(&42), Ok(&17)]);
}

#[test]
#[should_panic]
fn query_many_duplicate() {
    let mut frame = Frame::new();
    let e = frame.spawn(());
    _ = frame.query_many_mut::<(), 2>([e, e]);
}
