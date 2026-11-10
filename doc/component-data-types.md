# Component data typing — proposed design (f32 / i32 / u32 / high precision)

Written 2026-11-10. Owner decision: **i32-only for the 0.7 release**; this
document is the proposed design change for supporting f32/i32 (and possibly
u32 / higher-resolution samples) afterwards.

## 1. Why

- **9-7 irreversible codestreams** (`qmfbid == 0`) are float data end to end:
  the decoder's coefficients are f32, and the port currently round-trips them
  through `i32` storage (`f32::from_bits`/`to_bits`) so a decoded-then-
  re-encoded image loses the exact float values (and any float-only consumer
  — tone mapping, HDR pipelines, `half`-prec writers — must reinterpret).
  The internal DWT scratch exists precisely because the C reinterprets the
  i32 buffer as f32 in place (see `doc/internals-summary.md`); the typed-buffer
  refactor (B.1.4) removes that constraint and makes typed storage natural.
- **PCLR palettes** can declare channel precision up to 128 bits
  (JPEG 2000 Part 1, I.5.3.4); the decoder clamps samples to i32 today. A
  `U32` variant would faithfully hold the 32-38-bit range the SIZ
  validation actually permits upstream (the port caps at 31 — see the SIZ
  handler note) — though note the port's own decode rejects prec > 31, so
  `U32` support also needs a decision about lifting that cap.
- **High-resolution pixels** (future JPEG 2000 parts / HTJ2K profiles) and
  `image`-crate interop: `image` supports up to 16-bit; raw accessors with a
  real type let callers handle anything the codec can produce.

## 2. Current state

- `CoreImageComp.data: Vec<i32>` (public); `prec: u32`, `sgnd: u32` describe
  the sample range.
- 9-7 decode/encode stage f32 values through the i32 buffer via
  `from_bits`/`to_bits`; the whole-tile inverse DWT converts through
  `Tcd.dwt_f32_scratch`.
- `data_u8()`/`data_u16()` scale from i32 on access (precision-aware);
  `get_pixels`/`DynamicImage` build on those.
- `TileBuf` (internal) is `Vec<i32>`.

## 3. Proposed design

### 3.1 Storage

```rust
/// The storage type of a decoded component plane.
#[derive(Clone, Debug, PartialEq)]
pub enum ComponentData {
  I32(Vec<i32>), // reversible 5-3 output, and any input < 2^31 samples
  U32(Vec<u32>), // future: >31-bit components (PCLR/SIZ up to 38)
  F32(Vec<f32>), // irreversible 9-7 output (exact float coefficients)
}
```

`CoreImageComp` gains `data: ComponentData` plus a cached `sample_type()`.
`prec`/`sgnd` keep their current meaning for the integer variants and are
informational for `F32` (the codec's quantization already defines the range).

Decision rule at decode time: `qmfbid == 1` (5-3) → `I32`; `qmfbid == 0`
(9-7) → `F32`; `prec > 31` (if the cap is lifted) → `U32`. The tier-1
write-back writes the native type directly (no bit reinterpretation), which
is exactly what the B.1.4 `TileBuf` refactor enables.

### 3.2 Accessors (Rust API guidelines: C-NO-OUT, conversion methods as `to_*`)

```rust
impl ImageComponent {
  pub fn sample_type(&self) -> SampleType;          // I32 | U32 | F32
  pub fn data(&self) -> ComponentDataRef<'_>;       // enum of slices, no copy
  // lossless views
  pub fn as_i32(&self) -> Result<&[i32]>;           // Err for F32 (no silent conversion)
  pub fn as_f32(&self) -> Result<&[f32]>;
  // explicit, checked conversions (precision-aware)
  pub fn to_i32(&self) -> Result<Cow<'_, [i32]>>;   // owned only when converting
  pub fn to_f32(&self) -> Result<Cow<'_, [f32]>>;
  // the existing scaled iterators stay for display-oriented use
  pub fn to_u8(&self) -> impl Iterator<Item = u8>;
  pub fn to_u16(&self) -> impl Iterator<Item = u16>;
}
```

Encoding (`Image::from_core_image`) accepts any variant; the 5-3 encoder
requires integer samples (converts or rejects), the 9-7 encoder accepts
`F32` natively (no lossy staging).

### 3.3 Interop with the `image` crate (feature `image`)

`get_pixels`/`DynamicImage` convert from all three variants through the
existing precision-aware scaling (`F32` clamps/scales to the 8/16-bit
target). The current behaviour for i32 images is unchanged.

## 4. Noted shortcomings of the current jpeg2k-style `get_pixels`

(Owner decision: keep the signature for 0.7; these are recorded so the
design above can fix them later.)

- `alpha_default: Option<u32>` truncates to `u8`/`u16` on output, and a
  fill value is an un-idiomatic parameter for a read method (a separate
  `with_alpha` builder or an `AlphaPolicy` enum would be cleaner).
- `get_pixels` silently infers the format from the component count when the
  colour space is unspecified; a typed `PixelFormat` return plus an explicit
  "unknown" case would be more predictable.
- 16-bit output is only reachable when `prec > 8`, so a 9-bit source scales
  to 16-bit — correct but surprising; the `SampleType`-aware conversion can
  document the mapping explicitly.

## 5. Migration path

1. B.1.4 first: `TileBuf` becomes a typed variant (internal), 9-7 paths stop
   staging through i32 — no public change.
2. Add `ComponentData` + `sample_type` alongside `data: Vec<i32>`
   (additive; `data()` keeps returning i32 slices with a `Result` once F32
   planes exist).
3. Decode starts producing `F32` planes for 9-7; re-encode round-trips them
   bit-exactly.
4. (Optional, needs the SIZ-cap decision) `U32` for >31-bit components.
5. Deprecate/remove the raw `Vec<i32>` field access in a later release.

## 6. Open sub-questions (to decide when scheduled)

- Should `F32` planes be exposed *raw* (exact coefficients, consumers must
  know the quantization) or should the decoder offer a "normalized to
  0..1" view too?
- Lift the `prec > 31` decode cap together with `U32`, or keep the cap and
  defer `U32` entirely?
- Does the `image`-crate feature get a `DynamicImage` path for float data
  (tone-mapped), or do float consumers go through `to_f32()` manually?
