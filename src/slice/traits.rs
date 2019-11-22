/*! General trait implementations for `BitSlice`.

The operator traits are defined in the `ops` module.
!*/

use super::*;

use crate::{
	cursor::Cursor,
	store::BitStore,
};

use core::{
	cmp::{
		self,
		Ordering,
	},
	fmt::{
		self,
		Binary,
		Debug,
		Display,
		Formatter,
		LowerHex,
		Octal,
		UpperHex,
	},
	hash::{
		Hash,
		Hasher,
	},
	hint::unreachable_unchecked,
};

use either::Either;

#[cfg(feature = "alloc")]
use {
	crate::vec::BitVec,
	alloc::borrow::ToOwned,
};

#[cfg(feature = "std")]
use std::{
	io::{
		self,
		Read,
	},
	mem,
};

#[cfg(feature = "alloc")]
impl<C, T> ToOwned for BitSlice<C, T>
where C: Cursor, T: BitStore {
	type Owned = BitVec<C, T>;

	fn to_owned(&self) -> Self::Owned {
		BitVec::from_bitslice(self)
	}
}

impl<C, T> Eq for BitSlice<C, T>
where C: Cursor, T: BitStore {}

impl<C, T> Ord for BitSlice<C, T>
where C: Cursor, T: BitStore {
	fn cmp(&self, rhs: &Self) -> Ordering {
		self.partial_cmp(rhs)
			//  `BitSlice` has a total ordering, and never returns `None`.
			.unwrap_or_else(|| unsafe { unreachable_unchecked() })
	}
}

/** Tests if two `BitSlice`s are semantically — not bitwise — equal.

It is valid to compare two slices of different cursor or element types.

The equality condition requires that they have the same number of total bits and
that each pair of bits in semantic order are identical.
**/
impl<A, B, C, D> PartialEq<BitSlice<C, D>> for BitSlice<A, B>
where A: Cursor, B: BitStore, C: Cursor, D: BitStore {
	/// Performas a comparison by `==`.
	///
	/// # Examples
	///
	/// ```rust
	/// use bitvec::prelude::*;
	///
	/// let lsrc = [8u8, 16, 32, 0];
	/// let rsrc = 0x10_08_04_00u32;
	/// let lbits = lsrc.bits::<LittleEndian>();
	/// let rbits = rsrc.bits::<BigEndian>();
	///
	/// assert_eq!(lbits, rbits);
	/// ```
	fn eq(&self, rhs: &BitSlice<C, D>) -> bool {
		if self.len() != rhs.len() {
			return false;
		}
		self.iter().zip(rhs.iter()).all(|(l, r)| l == r)
	}
}

impl<A, B, C, D> PartialEq<BitSlice<C, D>> for &BitSlice<A, B>
where A: Cursor, B: BitStore, C: Cursor, D: BitStore {
	fn eq(&self, rhs: &BitSlice<C, D>) -> bool {
		(*self).eq(rhs)
	}
}

impl<A, B, C, D> PartialEq<&BitSlice<C, D>> for BitSlice<A, B>
where A: Cursor, B: BitStore, C: Cursor, D: BitStore {
	fn eq(&self, rhs: &&BitSlice<C, D>) -> bool {
		self.eq(*rhs)
	}
}

/** Compares two `BitSlice`s by semantic — not bitwise — ordering.

The comparison sorts by testing each index for one slice to have a set bit where
the other has an unset bit. If the slices are different, the slice with the set
bit sorts greater than the slice with the unset bit.

If one of the slices is exhausted before they differ, the longer slice is
greater.
**/
impl<A, B, C, D> PartialOrd<BitSlice<C, D>> for BitSlice<A, B>
where A: Cursor, B: BitStore, C: Cursor, D: BitStore {
	/// Performs a comparison by `<` or `>`.
	///
	/// # Examples
	///
	/// ```rust
	/// use bitvec::prelude::*;
	///
	/// let src = 0x45u8;
	/// let bits = src.bits::<BigEndian>();
	/// let a = &bits[0 .. 3]; // 010
	/// let b = &bits[0 .. 4]; // 0100
	/// let c = &bits[0 .. 5]; // 01000
	/// let d = &bits[4 .. 8]; // 0101
	///
	/// assert!(a < b);
	/// assert!(b < c);
	/// assert!(c < d);
	/// ```
	fn partial_cmp(&self, rhs: &BitSlice<C, D>) -> Option<Ordering> {
		for (l, r) in self.iter().zip(rhs.iter()) {
			match (l, r) {
				(true, false) => return Some(Ordering::Greater),
				(false, true) => return Some(Ordering::Less),
				_ => continue,
			}
		}
		self.len().partial_cmp(&rhs.len())
	}
}

impl<A, B, C, D> PartialOrd<BitSlice<C, D>> for &BitSlice<A, B>
where A: Cursor, B: BitStore, C: Cursor, D: BitStore {
	fn partial_cmp(&self, rhs: &BitSlice<C, D>) -> Option<Ordering> {
		(*self).partial_cmp(rhs)
	}
}

impl<A, B, C, D> PartialOrd<&BitSlice<C, D>> for BitSlice<A, B>
where A: Cursor, B: BitStore, C: Cursor, D: BitStore {
	fn partial_cmp(&self, rhs: &&BitSlice<C, D>) -> Option<Ordering> {
		self.partial_cmp(*rhs)
	}
}

/// Provides write access to all fully-owned elements in the underlying memory
/// buffer. This excludes the edge elements if they are partially-owned.
impl<C, T> AsMut<[T]> for BitSlice<C, T>
where C: Cursor, T: BitStore {
	fn as_mut(&mut self) -> &mut [T] {
		self.as_mut_slice()
	}
}

/// Provides read-only access to all fully-owned elements in the underlying
/// memory buffer. This excludes the edge elements if they are partially-owned.
impl<C, T> AsRef<[T]> for BitSlice<C, T>
where C: Cursor, T: BitStore {
	fn as_ref(&self) -> &[T] {
		self.as_slice()
	}
}

impl<'a, C, T> From<&'a T> for &'a BitSlice<C, T>
where C: Cursor, T: 'a + BitStore {
	fn from(src: &'a T) -> Self {
		BitSlice::<C, T>::from_element(src)
	}
}

impl<'a, C, T> From<&'a [T]> for &'a BitSlice<C, T>
where C: Cursor, T: 'a + BitStore {
	fn from(src: &'a [T]) -> Self {
		BitSlice::<C, T>::from_slice(src)
	}
}

impl<'a, C, T> From<&'a mut T> for &'a mut BitSlice<C, T>
where C: Cursor, T: 'a + BitStore {
	fn from(src: &'a mut T) -> Self {
		BitSlice::<C, T>::from_element_mut(src)
	}
}

impl<'a, C, T> From<&'a mut [T]> for &'a mut BitSlice<C, T>
where C: Cursor, T: 'a + BitStore {
	fn from(src: &'a mut [T]) -> Self {
		BitSlice::<C, T>::from_slice_mut(src)
	}
}

impl<'a, C, T> Default for &'a BitSlice<C, T>
where C: Cursor, T: 'a + BitStore {
	fn default() -> Self {
		BitSlice::empty()
	}
}

impl<'a, C, T> Default for &'a mut BitSlice<C, T>
where C: Cursor, T: 'a + BitStore {
	fn default() -> Self {
		BitSlice::empty_mut()
	}
}

macro_rules! fmt {
	( $trait:ident, $base:expr, $pfx:expr, $blksz:expr ) => {
/** Write out the contents of a `BitSlice` as a numeric format.

These implementations render the bits of memory governed by a `BitSlice` as one
of the three numeric bases the Rust format system supports:

- `Binary` renders each bit individually as `0` or `1`,
- `Octal` renders clusters of three bits as the numbers `0` through `7`,
- `Hex` renders clusters of four bits as the numbers `[0-9A-F]`.

The formatters produce a word for each `T` element of memory. The chunked
formats (octal and hexadecimal) operate somewhat peculiarly: they show the
semantic value of the memory as interpreted by the `Cursor` type parameter’s
implementation, and not the raw value of the memory as you might observe with a
debugger.

Specifically, the chunked formats read between zero and three (octal) or four
(hexadecimal) bits in `Cursor` order out of a memory element, store those bits
in first-high/last-low order, and then interpret that sequence as a number in
their respective bases. This means that, for instance, the byte `3` (bit pattern
`0b0000_0011`), read in `LittleEndian` order, will produce the numerals `"600"`
(`110 000 00`) in octal, and `"C0"` (`1100 0000`) in hexadecimal.

If the memory element is exhausted before a chunk is filled with three or four
bits, then the number produced will have a lower value. The byte `0xFFu8` will
always produce the octal numeral `"773"` (`111 111 11`).

The decision to chunk numeral words by memory element, even though it breaks the
octal chunking pattern was made so that the rendered text will still show memory
boundaries for easier inspection.
**/
impl<C, T> $trait for BitSlice<C, T>
where C: Cursor, T: BitStore {
	fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
		let start = if fmt.alternate() { 0 } else { 2 };
		let mut dbg = fmt.debug_list();
		let mut w: [u8; (64 / $blksz) + 2] = [b'0'; (64 / $blksz) + 2];
		w[1] = $pfx;
		let mut writer = |bits: &Self| {
			let mut end = 2;
			for (idx, chunk) in bits.chunks($blksz).enumerate() {
				let mut val = 0u8;
				for bit in chunk {
					val <<= 1;
					val |= *bit as u8;
				}
				w[2 + idx] = match val {
					v @ 0 ..= 9 => b'0' + v,
					v @ 10 ..= 16 => $base + (v - 10),
					_ => unsafe { unreachable_unchecked() },
				};
				end += 1;
			}
			dbg.entry(&RenderPart(unsafe {
				str::from_utf8_unchecked(&w[start .. end])
			}));
		};
		match self.bitptr().domain().splat() {
			Either::Right((head, elt, tail)) => {
				let (h, e, t) = (*head as usize, elt.load(), *tail as usize);
				let bits = Self::from_element(&e);
				writer(&bits[h .. t]);
			},
			Either::Left((h, b, t)) => {
				if let Some((h, head)) = h {
					writer(&Self::from_element(&head.load())[*h as usize ..]);
				}
				if let Some(body) = b {
					for elt in body.iter().map(BitAccess::load) {
						writer(Self::from_element(&elt));
					}
				}
				if let Some((tail, t)) = t {
					writer(&Self::from_element(&tail.load())[.. *t as usize]);
				}
			},
		}
		dbg.finish()
	}
}
	};
}

/** Prints the `BitSlice` for debugging.

The output is of the form `BitSlice<C, T> [ELT, *]` where `<C, T>` is the cursor
and element type, with square brackets on each end of the bits and all the
elements of the array printed in binary. The printout is always in semantic
order, and may not reflect the underlying buffer. To see the underlying buffer,
use `.as_total_slice()`.

The alternate character `{:#?}` prints each element on its own line, rather than
having all elements on the same line.
**/
impl<C, T> Debug for BitSlice<C, T>
where C: Cursor, T: BitStore {
	/// Renders the `BitSlice` type header and contents for debug.
	///
	/// # Examples
	///
	/// ```rust
	/// # #[cfg(feature = "alloc")] {
	/// use bitvec::prelude::*;
	///
	/// let src = [0b0101_0000_1111_0101u16, 0b00000000_0000_0010];
	/// let bits = &src.bits::<LittleEndian>()[.. 18];
	/// assert_eq!(
    ///     "BitSlice<LittleEndian, u16> [1010111100001010, 01]",
	///     &format!("{:?}", bits),
	/// );
	/// # }
	/// ```
	fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
		fmt.write_str("BitSlice<")?;
		fmt.write_str(C::TYPENAME)?;
		fmt.write_str(", ")?;
		fmt.write_str(T::TYPENAME)?;
		fmt.write_str("> ")?;
		Binary::fmt(self, fmt)
	}
}

/** Prints the `BitSlice` for displaying.

This prints each element in turn, formatted in binary in semantic order (so the
first bit seen is printed first and the last bit seen is printed last). Each
element of storage is separated by a space for ease of reading.

The alternate character `{:#}` prints each element on its own line.

To see the in-memory representation, use `.as_total_slice()` to get access to
the raw elements and print that slice instead.
**/
impl<C, T> Display for BitSlice<C, T>
where C: Cursor, T: BitStore {
	fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
		Binary::fmt(self, fmt)
	}
}

fmt![Binary, b'0', b'b', 1];
fmt![Octal, b'0', b'o', 3];
fmt![LowerHex, b'a', b'x', 4];
fmt![UpperHex, b'A', b'x', 4];

/** Wrapper for inserting pre-rendered text into a formatting stream.

The numeric formatters write text into a buffer, which a formatter then reads
directly. The formatter only takes `&dyn Debug` objects, so this translates the
text buffer into a compatible trait object.
**/
struct RenderPart<'a>(&'a str);
impl Debug for RenderPart<'_> {
	fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
		fmt.write_str(&self.0)
	}
}

/// Writes the contents of the `BitSlice`, in semantic bit order, into a hasher.
impl<C, T> Hash for BitSlice<C, T>
where C: Cursor, T: BitStore {
	fn hash<H>(&self, hasher: &mut H)
	where H: Hasher {
		for bit in self {
			hasher.write_u8(*bit as u8);
		}
	}
}

#[cfg(feature = "std")]
impl<C, T> Read for &BitSlice<C, T>
where C: Cursor, T: BitStore {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		let tgt = BitSlice::<C, u8>::from_slice_mut(buf);
		let len = cmp::min(self.len(), tgt.len());
		let (head, rest) = self.split_at(len);
		tgt[.. len].clone_from_slice(head);
		*self = rest;
		Ok(len)
	}

	fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
		let tgt = BitSlice::<C, u8>::from_slice_mut(buf);
		let len = tgt.len();
		if self.len() > len {
			return Err(io::Error::new(
				io::ErrorKind::UnexpectedEof,
				"failed to fill whole buffer",
			));
		}
		let (head, rest) = self.split_at(len);
		tgt.clone_from_slice(&head[.. len]);
		*self = rest;
		Ok(())
	}

	fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
		let mut bv = BitVec::<C, u8>::from_vec(mem::replace(buf, Vec::new()));
		let len = self.len();
		bv.reserve(len);
		bv.extend(self.iter().copied());
		*self = BitSlice::<C, T>::empty();
		mem::replace(buf, bv.into_vec());
		Ok(len)
	}
}

/** `BitSlice` is safe to move across thread boundaries, when atomic operations
are enabled.

Consider this (contrived) example:

```rust
# #[cfg(feature = "std")] {
use bitvec::prelude::*;
use std::thread;

static mut SRC: u8 = 0;
# {
let bits = unsafe { SRC.bits_mut::<BigEndian>() };
let (l, r) = bits.split_at_mut(4);

let a = thread::spawn(move || l.set(2, true));
let b = thread::spawn(move || r.set(2, true));
a.join();
b.join();
# }

println!("{:02X}", unsafe { SRC });
# }
```

Without atomic operations, this is logically a data race. With atomic
operations, each read/modify/write cycle is guaranteed to exclude other threads
from observing the location until the writeback completes.
**/
#[cfg(feature = "atomic")]
unsafe impl<C, T> Send for BitSlice<C, T>
where C: Cursor, T: BitStore {}

/** Reading across threads still has synchronization concerns if one thread can
mutate, so read access across threads requires atomicity in order to ensure that
write operations from one thread to an element conclude before another thread
can read from the element, even if the two `BitSlice`s do not collide.
**/
#[cfg(feature = "atomic")]
unsafe impl<C, T> Sync for BitSlice<C, T>
where C: Cursor, T: BitStore {}

#[cfg(all(test, feature = "alloc"))]
mod tests {
	use crate::{
		bits::Bits,
		cursor::BigEndian,
	};

	#[test]
	fn binary() {
		let data = [0u8, 0x0F, !0];
		let bits = data.bits::<BigEndian>();

		assert_eq!(format!("{:b}", &bits[.. 0]), "[]");
		assert_eq!(format!("{:#b}", &bits[.. 0]), "[]");

		assert_eq!(format!("{:b}", &bits[9 .. 15]), "[000111]");
		assert_eq!(format!("{:#b}", &bits[9 .. 15]),
"[
    0b000111,
]");

		assert_eq!(format!("{:b}", &bits[4 .. 20]), "[0000, 00001111, 1111]");
		assert_eq!(format!("{:#b}", &bits[4 .. 20]),
"[
    0b0000,
    0b00001111,
    0b1111,
]");

		assert_eq!(format!("{:b}", &bits[4 ..]), "[0000, 00001111, 11111111]");
		assert_eq!(format!("{:#b}", &bits[4 ..]),
"[
    0b0000,
    0b00001111,
    0b11111111,
]");

		assert_eq!(format!("{:b}", &bits[.. 20]), "[00000000, 00001111, 1111]");
		assert_eq!(format!("{:#b}", &bits[.. 20]),
"[
    0b00000000,
    0b00001111,
    0b1111,
]");

		assert_eq!(format!("{:b}", bits), "[00000000, 00001111, 11111111]");
		assert_eq!(format!("{:#b}", bits),
"[
    0b00000000,
    0b00001111,
    0b11111111,
]");
	}

	#[test]
	fn octal() {
		let data = [0u8, 0x0F, !0];
		let bits = data.bits::<BigEndian>();

		assert_eq!(format!("{:o}", &bits[.. 0]), "[]");
		assert_eq!(format!("{:#o}", &bits[.. 0]), "[]");

		assert_eq!(format!("{:o}", &bits[9 .. 15]), "[07]");
		assert_eq!(format!("{:#o}", &bits[9 .. 15]),
"[
    0o07,
]");

		assert_eq!(format!("{:o}", &bits[4 .. 20]), "[00, 033, 71]");
		assert_eq!(format!("{:#o}", &bits[4 .. 20]),
"[
    0o00,
    0o033,
    0o71,
]");

		assert_eq!(format!("{:o}", &bits[4 ..]), "[00, 033, 773]");
		assert_eq!(format!("{:#o}", &bits[4 ..]),
"[
    0o00,
    0o033,
    0o773,
]");

		assert_eq!(format!("{:o}", &bits[.. 20]), "[000, 033, 71]");
		assert_eq!(format!("{:#o}", &bits[.. 20]),
"[
    0o000,
    0o033,
    0o71,
]");

		assert_eq!(format!("{:o}", bits), "[000, 033, 773]");
		assert_eq!(format!("{:#o}", bits),
"[
    0o000,
    0o033,
    0o773,
]");
	}

	#[test]
	fn hex_lower() {
		let data = [0u8, 0x0F, !0];
		let bits = data.bits::<BigEndian>();

		assert_eq!(format!("{:x}", &bits[.. 0]), "[]");
		assert_eq!(format!("{:#x}", &bits[.. 0]), "[]");

		assert_eq!(format!("{:x}", &bits[9 .. 15]), "[13]");
		assert_eq!(format!("{:#x}", &bits[9 .. 15]),
"[
    0x13,
]");

		assert_eq!(format!("{:x}", &bits[4 .. 20]), "[0, 0f, f]");
		assert_eq!(format!("{:#x}", &bits[4 .. 20]),
"[
    0x0,
    0x0f,
    0xf,
]");

		assert_eq!(format!("{:x}", &bits[4 ..]), "[0, 0f, ff]");
		assert_eq!(format!("{:#x}", &bits[4 ..]),
"[
    0x0,
    0x0f,
    0xff,
]");

		assert_eq!(format!("{:x}", &bits[.. 20]), "[00, 0f, f]");
		assert_eq!(format!("{:#x}", &bits[.. 20]),
"[
    0x00,
    0x0f,
    0xf,
]");

		assert_eq!(format!("{:x}", bits), "[00, 0f, ff]");
		assert_eq!(format!("{:#x}", bits),
"[
    0x00,
    0x0f,
    0xff,
]");
	}

	#[test]
	fn hex_upper() {
		let data = [0u8, 0x0F, !0];
		let bits = data.bits::<BigEndian>();

		assert_eq!(format!("{:X}", &bits[.. 0]), "[]");
		assert_eq!(format!("{:#X}", &bits[.. 0]), "[]");

		assert_eq!(format!("{:X}", &bits[9 .. 15]), "[13]");
		assert_eq!(format!("{:#X}", &bits[9 .. 15]),
"[
    0x13,
]");

		assert_eq!(format!("{:X}", &bits[4 .. 20]), "[0, 0F, F]");
		assert_eq!(format!("{:#X}", &bits[4 .. 20]),
"[
    0x0,
    0x0F,
    0xF,
]");

		assert_eq!(format!("{:X}", &bits[4 ..]), "[0, 0F, FF]");
		assert_eq!(format!("{:#X}", &bits[4 ..]),
"[
    0x0,
    0x0F,
    0xFF,
]");

		assert_eq!(format!("{:X}", &bits[.. 20]), "[00, 0F, F]");
		assert_eq!(format!("{:#X}", &bits[.. 20]),
"[
    0x00,
    0x0F,
    0xF,
]");

		assert_eq!(format!("{:X}", bits), "[00, 0F, FF]");
		assert_eq!(format!("{:#X}", bits),
"[
    0x00,
    0x0F,
    0xFF,
]");
	}
}