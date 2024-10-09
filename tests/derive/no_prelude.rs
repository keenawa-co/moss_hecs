#![no_implicit_prelude]

#[derive(::moss_hecs::Bundle)]
struct Foo {
    foo: (),
}

#[derive(::moss_hecs::Bundle)]
struct Bar<T> {
    foo: T,
}

#[derive(::moss_hecs::Bundle)]
struct Baz;

#[derive(::moss_hecs::Query)]
struct Quux<'a> {
    foo: &'a (),
}

fn main() {}
