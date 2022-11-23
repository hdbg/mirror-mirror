use crate::Reflect;
use crate::{self as mirror_mirror, FromReflect, GetField, Struct, StructValue, Value};

#[derive(Reflect, Default, Clone, Eq, PartialEq, Debug)]
struct Foo {
    field: i32,
}

#[test]
fn accessing_fields() {
    let foo = Foo { field: 42 };
    let struct_ = foo.as_struct().unwrap();

    let value = struct_
        .field("field")
        .unwrap()
        .downcast_ref::<i32>()
        .unwrap();

    assert_eq!(*value, 42);
}

#[test]
fn patching() {
    let mut foo = Foo { field: 42 };

    let patch = StructValue::default().with_field("field", 1337);

    foo.patch(&patch);

    assert_eq!(foo.field, 1337);
}

#[test]
fn patching_struct_value() {
    let mut value = StructValue::default().with_field("field", 42);
    let patch = StructValue::default().with_field("field", 1337);
    value.patch(&patch);

    assert_eq!(
        value.field("field").unwrap().downcast_ref::<i32>().unwrap(),
        &1337
    );
}

#[test]
fn from_reflect() {
    let foo = Foo::default();
    let foo_reflect: &dyn Reflect = &foo;

    let foo = Foo::from_reflect(foo_reflect).unwrap();

    assert_eq!(foo.field, 0);
}

#[test]
fn serialize_deserialize() {
    let foo = Foo::default();
    let struct_value = foo.to_value();

    let json = serde_json::to_string(&struct_value).unwrap();

    let struct_value = serde_json::from_str::<Value>(&json).unwrap();
    let foo = Foo::from_reflect(&struct_value).unwrap();

    assert_eq!(foo.field, 0);
}

#[test]
fn fields() {
    let foo = Foo::default();

    for (name, value) in foo.fields() {
        if name == "field" {
            assert_eq!(foo.field, i32::from_reflect(value).unwrap());
        } else {
            panic!("Unknown field {name:?}");
        }
    }
}

#[test]
fn struct_value_from_reflect() {
    let value = StructValue::default().with_field("foo", 42);
    let reflect = value.as_reflect();

    let value = StructValue::from_reflect(reflect).unwrap();

    assert_eq!(
        value.field("foo").unwrap().downcast_ref::<i32>().unwrap(),
        &42,
    );
}

#[test]
fn box_dyn_reflect_as_reflect() {
    let foo = Foo::default();
    let mut box_dyn_reflect = Box::new(foo) as Box<dyn Reflect>;

    assert_eq!(
        box_dyn_reflect
            .as_struct()
            .unwrap()
            .field("field")
            .unwrap()
            .downcast_ref::<i32>()
            .unwrap(),
        &0,
    );

    box_dyn_reflect.patch(&StructValue::default().with_field("field", 42));

    assert_eq!(
        box_dyn_reflect
            .as_struct()
            .unwrap()
            .field("field")
            .unwrap()
            .downcast_ref::<i32>()
            .unwrap(),
        &42,
    );

    let foo = Foo::from_reflect(&box_dyn_reflect).unwrap();
    assert_eq!(foo, Foo { field: 42 });
}

#[test]
fn deeply_nested() {
    #[derive(Reflect, Clone, Debug)]
    struct Foo {
        bar: Bar,
    }

    #[derive(Reflect, Clone, Debug)]
    struct Bar {
        baz: Baz,
    }

    #[derive(Reflect, Clone, Debug)]
    struct Baz {
        qux: i32,
    }

    let foo = Foo {
        bar: Bar {
            baz: Baz { qux: 42 },
        },
    };

    let &forty_two = (|| {
        foo.get_field::<Bar>("bar")?
            .get_field::<Baz>("baz")?
            .get_field::<i32>("qux")
    })()
    .unwrap();

    assert_eq!(forty_two, 42);
}