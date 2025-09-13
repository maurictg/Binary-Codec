
# binary-codec-derive Usage Guide

`binary-codec-derive` provides macros for bit-level serialization and deserialization of Rust structs and enums using the `binary-codec` crate. This guide explains usage, attributes, and how bits are packed into bytes.

---

## Table of Contents

- [Getting Started](#getting-started)
- [Bit Packing](#example-bit-packing)
- [Supported Attributes](#supported-attributes)
- [Attribute Priority & Inheritance](#attribute-priority--inheritance)
- [Enum Example](#enum-example)
- [Dynamic Length Example](#dynamic-length-example)
- [Option and Toggled Example](#option-and-toggled-example)
- [Arrays & Vecs](#arrays)
- [Advanced Use Cases](#advanced-use-cases)
- [ZigZag Encoding](#zigzag-encoding)
- [Error Handling](#error-handling)
- [Full Example](#full-example)

---

## Getting Started

Add to your `Cargo.toml`:

```toml
[dependencies]
binary-codec = "0.1.0"
```

Import macros:

```rust
use plabble_derive::{ToBytes, FromBytes};
use plabble_codec::SerializationConfig;
```

## Example: Bit Packing

```rust
#[derive(ToBytes, FromBytes, Debug, PartialEq)]
struct Example {
	#[bits = 3]
	a: u8,           // 3 bits
	#[bits = 5]
	b: i8,           // 5 bits, zigzag encoded
	flag: bool,      // 1 bit
}

let config = SerializationConfig::default();
let value = Example { a: 5, b: -7, flag: true };
let bytes = value.to_bytes(&config).unwrap();
let decoded = Example::from_bytes(&bytes, &config).unwrap();
assert_eq!(value, decoded);
```

### How Bits Are Packed

Fields are packed left-to-right, lowest bits first. When the total bits exceed 8, the next byte is used. For the above struct:

- `a` (3 bits): `0b101`
- `b` (5 bits, zigzag): `-7` → zigzag encode → `13` → `0b01101`
- `flag` (1 bit): `1`

**Packing order:**

| Byte 0         |
|----------------|
| a (3) | b (5)  |
| 101   | 01101  |

| Byte 1         |
|----------------|
| flag (1) | pad |
| 1       | 0000000 |

If the sum of field bits in a struct is not a multiple of 8, the last byte is padded with zeros.

If another value is put in byte 1 and it is bigger than 7 bits, it will be but in a new byte and the serializer will 'waste' 7 bits. So the order of properties in your struct is very important!

#### Multi-Byte Example

```rust
#[derive(ToBytes, FromBytes, Debug, PartialEq)]
struct MultiByte {
	#[bits = 4]
	a: u8, // 4 bits
	#[bits = 4]
	b: u8, // 4 bits
	#[bits = 6]
	c: u8, // 6 bits
	#[bits = 2]
	d: u8, // 2 bits
}

// Packing:
// a: 0b1010 (4 bits)
// b: 0b1100 (4 bits)
// c: 0b111100 (6 bits)
// d: 0b11 (2 bits)

// Byte 0: a (4) | b (4) => 0b11001010
// Byte 1: c (6) | d (2) => 0b11111100
```

## Supported Attributes

- `#[bits = N]`: Use N bits for this integer field (1 ≤ N ≤ 7 for u8/i8).
- `#[dynamic]`: Use dynamic integer encoding (see `dyn_int.rs` in plabble-codec).
- `#[dynamic_len]`: Prefix Vec, String, or object with a dynamic length field (using dynamic integer encoding)
- `#[length_determined_by = "field"]`: Use another field to determine the length of a Vec or String. You can also use `field.0` if the field is an array or Vec.
- `#[toggled_by = "field"]`: Option is present only if the referenced field is true (should be a bool). You can also use `field.0` if the field is an array or Vec.
- `#[variant_by = "field"]`: For enums, select variant by another field's value. You can also use `field.0` if the field is an array or Vec.
- `#[no_disc_prefix]`: For enums, do not write a discriminant prefix. This is needed if you use the variant_by.

## Attribute Priority & Inheritance

Attributes are processed in the following order of priority:

1. `#[bits = N]` (highest priority for integer fields)
2. `#[dynamic]` (overrides bits for dynamic encoding)
3. `#[dynamic_len]` (applies to Vec/array element count, or to the length of a nested element)
4. `#[length_determined_by = "field"]` (overrides dynamic_len if present)
5. `#[toggled_by = "field"]` (controls Option presence)
6. `#[variant_by = "field"]` (for enums)
7. `#[no_disc_prefix]` (for enums)

### Inheritance Rules

- In `Option<T>`, all attributes inherit to the inner type.
- In `Vec<T>` or `[T; N]`, only `bits`, `dynamic`, and `dynamic_len` can inherit, and `dynamic_len` requires a depth argument for nested Vecs.

### Attribute Precedence Example

```rust
#[derive(ToBytes, FromBytes)]
struct Example {
	#[bits = 3]
	a: u8,           // 3 bits

	#[dynamic]
	b: u8,           // dynamic encoding, bits ignored

	#[dynamic_len]
	data: Vec<u8>,   // dynamic length prefix for element count

	#[length_determined_by = "a"]
	fixed_data: Vec<u8>, // length determined by field 'a'
}
```

---

## Enum Example

```rust
#[derive(ToBytes, FromBytes, Debug, PartialEq)]
enum MyEnum {
	A,
	B(u8),
	C { x: i32 },
}
// Discriminant (variant index) is written as the first byte unless #[no_disc_prefix] is used.
// You can use #[variant_by = "field"] to select the variant based on another field's value.
```

### Enum with Variant By Example

```rust
#[derive(ToBytes, FromBytes, Debug, PartialEq)]
struct Container {
    kind: u8,
    #[variant_by = "kind"]
    value: MyEnum,
}
```

## Dynamic Length Example

```rust
#[derive(ToBytes, FromBytes, Debug, PartialEq)]
struct WithVec {
	#[dynamic_len]
	data: Vec<u8>,
}
// The length of `data` is encoded as a dynamic integer before the actual bytes.
```

### Dynamic Length with Depth Example

```rust
#[derive(ToBytes, FromBytes, Debug, PartialEq)]
struct DeepVec {
    #[dynamic_len(3)]
    data: Vec<Vec<String>>,
}
// The outer Vec's length is encoded as a dynamic integer, then each inner Vec's length is also encoded dynamically. The string length is also encoded dynamically.
// Binary structure will look like this:
// [ elem count vec, elem count first vec, string length,..., elem count second vec, string length etc ]
```

## Option and Toggled Example

```rust
#[derive(ToBytes, FromBytes, Debug, PartialEq)]
struct WithOption {
	flag: bool,
	#[toggled_by = "flag"]
	maybe: Option<u8>,
}
// If flag is false, maybe is not deserialized.
```

### Option with Nested Attributes

```rust
#[derive(ToBytes, FromBytes, Debug, PartialEq)]
struct NestedOption {
	flag: bool,

	#[toggled_by = "flag"]
	#[bits = 4]
	maybe: Option<u8>, // If flag is true, maybe is present and uses 4 bits
}
```

## Arrays

Arrays are supported and serialized element by element. You can use `#[bits = N]` on array elements for compact encoding. If you are serializing array of a dynamic length type, you need to put `#[dynamic_len]` on top.

### Array Example

```rust
#[derive(ToBytes, FromBytes, Debug, PartialEq)]
struct ArrayExample {
	#[bits = 2]
	arr: [u8; 4], // Each element uses 2 bits
}
```

## Vecs

Vecs work like arrays, BUT the `#[dynamic_len]` attribute will apply to the ELEMENT COUNT and not to the individual structs like arrays do. If you want that, use `#[dynamic_len(2)]` so it will be applied to the first level of children.
If you for instance have `Vec<Vec<String>>` you need `#[dynamic_len(3)]`. So for the dynamic_len attribute, inheritance level needs to be specified. Other attributes then `dynamic`, `dynamic_len` and `bits` DO NOT inherit in a Vec or array. In an `Option` however, all attributes inherit without decreasing the inheritance level. 

### Vec Example

```rust
#[derive(ToBytes, FromBytes, Debug, PartialEq)]
struct VecExample {
	#[dynamic_len]
	values: Vec<u16>, // Length prefix, then each value as u16
}
```

### Nested Vec Example

```rust
#[derive(ToBytes, FromBytes, Debug, PartialEq)]
struct NestedVecExample {
	#[dynamic_len(2)]
	values: Vec<Vec<u8>>,
}
```
## Advanced Use Cases

### Combining Attributes

```rust
#[derive(ToBytes, FromBytes, Debug, PartialEq)]
struct Complex {
	#[bits = 3]
	a: u8,

	#[dynamic]
	b: u32,

	#[dynamic_len]
	data: Vec<u8>,

	#[length_determined_by = "a"]
	fixed_data: Vec<u8>,

	#[toggled_by = "flag"]
	flag: bool,

	#[toggled_by = "flag"]
	maybe: Option<u8>,
}
```

### Nested Option and Vec (error)

```rust
#[derive(ToBytes, FromBytes, Debug, PartialEq)]
struct Deep {
	#[dynamic_len(2)]
	values: Vec<Option<Vec<u8>>>,
    // !!! THIS IS NOT POSSIBLE !!!
    // Because we need to know if the option is present or not
}

// Instead, wrap in a struct:
struct OptionalVec {
    is_present: bool,

    #[dynamic_len]
    #[toggled_by = "is_present"]
    data: Option<Vec<u8>>
}
```

## ZigZag Encoding

Signed integers with `#[bits = N]` or `#[dynamic]` use zigzag encoding for efficient bit packing:

- Positive: `n → n << 1`
- Negative: `n → (n << 1) ^ (-1)`

Example: `-3` → zigzag encode → `5` → `0b101`

---

## Error Handling

All serialization and deserialization methods return a `Result<T, SerializationError>` or `Result<T, DeserializationError>`. Errors include out-of-bounds values, unexpected lengths, and unknown enum discriminants.

---

## Full Example

```rust
#[derive(ToBytes, FromBytes, Debug, PartialEq)]
struct Demo {
	#[bits = 3]
	a: u8,

	#[bits = 5]
	b: i8,

	flag: bool,
	
	#[dynamic_len]
	data: Vec<u8>,
}

let config = SerializationConfig::default();
let demo = Demo { a: 7, b: -4, flag: true, data: vec![1,2,3] };
let bytes = demo.to_bytes(&config).unwrap();
let decoded = Demo::from_bytes(&bytes, &config).unwrap();
assert_eq!(demo, decoded);
```