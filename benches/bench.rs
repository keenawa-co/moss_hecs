// Copyright 2019 Google LLC
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use bencher::{benchmark_group, benchmark_main, Bencher};
use moss_hecs::*;

#[derive(Clone)]
struct Position(f32);
#[derive(Clone)]
struct Velocity(f32);

fn spawn_tuple(b: &mut Bencher) {
    let mut frame = Frame::new();
    b.iter(|| {
        frame.spawn((Position(0.0), Velocity(0.0)));
    });
}

fn spawn_static(b: &mut Bencher) {
    #[derive(Bundle)]
    struct Bundle {
        pos: Position,
        vel: Velocity,
    }

    let mut frame = Frame::new();
    b.iter(|| {
        frame.spawn(Bundle {
            pos: Position(0.0),
            vel: Velocity(0.0),
        });
    });
}

fn spawn_batch(b: &mut Bencher) {
    #[derive(Bundle)]
    struct Bundle {
        pos: Position,
        vel: Velocity,
    }

    let mut frame = Frame::new();
    b.iter(|| {
        frame
            .spawn_batch((0..1_000).map(|_| Bundle {
                pos: Position(0.0),
                vel: Velocity(0.0),
            }))
            .for_each(|_| {});
        frame.clear();
    });
}

fn remove(b: &mut Bencher) {
    let mut frame = Frame::new();
    b.iter(|| {
        // This really shouldn't be counted as part of the benchmark, but bencher doesn't seem to
        // support that.
        let entities = frame
            .spawn_batch((0..1_000).map(|_| (Position(0.0), Velocity(0.0))))
            .collect::<Vec<_>>();
        for e in entities {
            frame.remove_one::<Velocity>(e).unwrap();
        }
        frame.clear();
    });
}

fn insert(b: &mut Bencher) {
    let mut frame = Frame::new();
    b.iter(|| {
        // This really shouldn't be counted as part of the benchmark, but bencher doesn't seem to
        // support that.
        let entities = frame
            .spawn_batch((0..1_000).map(|_| (Position(0.0),)))
            .collect::<Vec<_>>();
        for e in entities {
            frame.insert_one(e, Velocity(0.0)).unwrap();
        }
        frame.clear();
    });
}

fn insert_remove(b: &mut Bencher) {
    let mut frame = Frame::new();
    let entities = frame
        .spawn_batch((0..1_000).map(|_| (Position(0.0), Velocity(0.0))))
        .collect::<Vec<_>>();
    let mut entities = entities.iter().cycle();
    b.iter(|| {
        let e = *entities.next().unwrap();
        frame.remove_one::<Velocity>(e).unwrap();
        frame.insert_one(e, true).unwrap();
        frame.remove_one::<bool>(e).unwrap();
        frame.insert_one(e, Velocity(0.0)).unwrap();
    });
}

fn exchange(b: &mut Bencher) {
    let mut frame = Frame::new();
    let entities = frame
        .spawn_batch((0..1_000).map(|_| (Position(0.0), Velocity(0.0))))
        .collect::<Vec<_>>();
    let mut entities = entities.iter().cycle();
    b.iter(|| {
        let e = *entities.next().unwrap();
        frame.exchange_one::<Velocity, _>(e, true).unwrap();
        frame.exchange_one::<bool, _>(e, Velocity(0.0)).unwrap();
    });
}

fn iterate_100k(b: &mut Bencher) {
    let mut frame = Frame::new();
    for i in 0..100_000 {
        frame.spawn((Position(-(i as f32)), Velocity(i as f32)));
    }
    b.iter(|| {
        for (_, (pos, vel)) in &mut frame.query::<(&mut Position, &Velocity)>() {
            pos.0 += vel.0;
        }
    })
}

fn iterate_mut_100k(b: &mut Bencher) {
    let mut frame = Frame::new();
    for i in 0..100_000 {
        frame.spawn((Position(-(i as f32)), Velocity(i as f32)));
    }
    b.iter(|| {
        for (_, (pos, vel)) in frame.query_mut::<(&mut Position, &Velocity)>() {
            pos.0 += vel.0;
        }
    })
}

fn spawn_100_by_50(frame: &mut Frame) {
    fn spawn_two<const N: usize>(frame: &mut Frame, i: i32) {
        frame.spawn((Position(-(i as f32)), Velocity(i as f32), [(); N]));
        frame.spawn((Position(-(i as f32)), [(); N]));
    }

    for i in 0..2 {
        spawn_two::<0>(frame, i);
        spawn_two::<1>(frame, i);
        spawn_two::<2>(frame, i);
        spawn_two::<3>(frame, i);
        spawn_two::<4>(frame, i);
        spawn_two::<5>(frame, i);
        spawn_two::<6>(frame, i);
        spawn_two::<7>(frame, i);
        spawn_two::<8>(frame, i);
        spawn_two::<9>(frame, i);
        spawn_two::<10>(frame, i);
        spawn_two::<11>(frame, i);
        spawn_two::<12>(frame, i);
        spawn_two::<13>(frame, i);
        spawn_two::<14>(frame, i);
        spawn_two::<15>(frame, i);
        spawn_two::<16>(frame, i);
        spawn_two::<17>(frame, i);
        spawn_two::<18>(frame, i);
        spawn_two::<19>(frame, i);
        spawn_two::<20>(frame, i);
        spawn_two::<21>(frame, i);
        spawn_two::<22>(frame, i);
        spawn_two::<23>(frame, i);
        spawn_two::<24>(frame, i);
    }
}

fn iterate_uncached_100_by_50(b: &mut Bencher) {
    let mut frame = Frame::new();
    spawn_100_by_50(&mut frame);
    b.iter(|| {
        for (_, (pos, vel)) in frame.query::<(&mut Position, &Velocity)>().iter() {
            pos.0 += vel.0;
        }
    })
}

fn iterate_uncached_1_of_100_by_50(b: &mut Bencher) {
    let mut frame = Frame::new();
    spawn_100_by_50(&mut frame);
    b.iter(|| {
        for (_, (pos, vel)) in frame
            .query::<(&mut Position, &Velocity)>()
            .with::<&[(); 0]>()
            .iter()
        {
            pos.0 += vel.0;
        }
    })
}

fn iterate_cached_100_by_50(b: &mut Bencher) {
    let mut frame = Frame::new();
    spawn_100_by_50(&mut frame);
    let mut query = PreparedQuery::<(&mut Position, &Velocity)>::default();
    let _ = query.query(&frame).iter();
    b.iter(|| {
        for (_, (pos, vel)) in query.query(&frame).iter() {
            pos.0 += vel.0;
        }
    })
}

fn iterate_mut_uncached_100_by_50(b: &mut Bencher) {
    let mut frame = Frame::new();
    spawn_100_by_50(&mut frame);
    b.iter(|| {
        for (_, (pos, vel)) in frame.query_mut::<(&mut Position, &Velocity)>() {
            pos.0 += vel.0;
        }
    })
}

fn iterate_mut_cached_100_by_50(b: &mut Bencher) {
    let mut frame = Frame::new();
    spawn_100_by_50(&mut frame);
    let mut query = PreparedQuery::<(&mut Position, &Velocity)>::default();
    let _ = query.query_mut(&mut frame);
    b.iter(|| {
        for (_, (pos, vel)) in query.query_mut(&mut frame) {
            pos.0 += vel.0;
        }
    })
}

fn build(b: &mut Bencher) {
    let mut frame = Frame::new();
    let mut builder = EntityBuilder::new();
    b.iter(|| {
        builder.add(Position(0.0)).add(Velocity(0.0));
        frame.spawn(builder.build());
    });
}

fn build_cloneable(b: &mut Bencher) {
    let mut frame = Frame::new();
    let mut builder = EntityBuilderClone::new();
    builder.add(Position(0.0)).add(Velocity(0.0));
    let bundle = builder.build();
    b.iter(|| {
        frame.spawn(&bundle);
    });
}

fn access_view(b: &mut Bencher) {
    let mut frame = Frame::new();
    let _enta = frame.spawn((Position(0.0), Velocity(0.0)));
    let _entb = frame.spawn((true, 12));
    let entc = frame.spawn((Position(3.0),));
    let _entd = frame.spawn((13, true, 4.0));
    let mut query = PreparedQuery::<&Position>::new();
    let mut query = query.query(&frame);
    let view = query.view();
    b.iter(|| {
        let _comp = bencher::black_box(view.get(entc).unwrap());
    });
}

fn spawn_buffered(b: &mut Bencher) {
    let mut frame = Frame::new();
    let mut buffer = CommandBuffer::new();
    let ent = frame.reserve_entity();
    b.iter(|| {
        buffer.insert(ent, (Position(0.0), Velocity(0.0)));
        buffer.run_on(&mut frame);
    });
}

benchmark_group!(
    benches,
    spawn_tuple,
    spawn_static,
    spawn_batch,
    remove,
    insert,
    insert_remove,
    exchange,
    iterate_100k,
    iterate_mut_100k,
    iterate_uncached_100_by_50,
    iterate_uncached_1_of_100_by_50,
    iterate_cached_100_by_50,
    iterate_mut_uncached_100_by_50,
    iterate_mut_cached_100_by_50,
    build,
    build_cloneable,
    access_view,
    spawn_buffered,
);
benchmark_main!(benches);
