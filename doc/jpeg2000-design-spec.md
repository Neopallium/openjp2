# JPEG 2000 Design Specification

*Format-and-algorithms reference for the openjp2 Rust port. Written from the
JPEG 2000 standards and public material; it deliberately contains no
references to any implementation (neither the upstream C library nor the
Rust port). The code outline document maps this spec onto the source.*

## 1. Standards

| Document | Title |
|---|---|
| ISO/IEC 15444-1 / ITU-T T.800 | JPEG 2000 core coding system |
| ISO/IEC 15444-2 / ITU-T T.801 | Extensions (Part 2) |
| ISO/IEC 15444-15 / ITU-T T.814 | HTJ2K: high-throughput JPEG 2000 |
| ISO/IEC 15444-4 | Conformance |
| ISO 15076-1 / IEC 61966-2-1 | ICC profiles (used by the colr box) |

Relevant amendments: ROI max-shift (AMD 1), extended capabilities (AMD 3,
single-tile broadcast profile).

## 2. Codestream structure (Part 1)

A codestream is a sequence of marker segments and bitstream data.

### 2.1 Markers

All markers are two bytes: `0xFF` followed by a marker code. Delimiting
markers carry no length; informational markers are followed by a 2-byte
big-endian length that includes the 2 length bytes themselves.

| Marker | Code | Description |
|---|---|---|
| SOC | FF4F | Start of codestream |
| SOT | FF90 | Start of tile-part |
| SOD | FF93 | Start of data |
| EOC | FFD9 | End of codestream |
| SIZ | FF51 | Image and tile size |
| COD | FF52 | Coding style default |
| COC | FF53 | Coding style component |
| TLM | FF55 | Tile-part lengths (main header) |
| PLM | FF57 | Packet length, main header |
| PLT | FF58 | Packet length, tile-part header |
| QCD | FF5C | Quantization default |
| QCC | FF5D | Quantization component |
| RGN | FF5E | Region of interest |
| POC | FF5F | Progression order change |
| PPM | FF60 | Packed packet headers, main header |
| PPT | FF61 | Packed packet headers, tile-part header |
| CRG | FF63 | Component registration |
| COM | FF64 | Comment |
| CBD | FF78 | Component bit depth (Part 2) |
| MCT | FF74 | Multiple component transform (Part 2) |
| MCC | FF75 | Multiple component collection (Part 2) |
| MCO | FF77 | Multiple component transform ordering (Part 2) |
| CAP | FF50 | Extended capabilities (AMD 3) |
| CPF | FF59 | Coding profile (AMD 3) |

In-tile "markers" FF90/FF92 are the in-band SOP/EPH markers, written *inside*
packets; every `0xFF` inside a packet must be followed by a zero byte
(bit-stuffing) except for a genuine marker, so decoders can resynchronise.

### 2.2 Main header

`SOC` `SIZ` (`COD`) (`COC`) (`QCD`) (`QCC`) (`RGN`) (`POC`) (`PPM`|`TLM`|`PLM`)
(`CRG`) (`COM`) ... — one or more tile-part headers (each: `SOT` (`COD`)
(`COC`) (`QCD`) (`QCC`) (`RGN`) (`POC`) (`PPT`|`PLT`) (`COM`) `SOD` bitstream),
then `EOC`.

- **SIZ**: `Rsiz` (profile flags), `Xsiz/Ysiz` (reference grid size),
  `XOsiz/YOsiz` (image offset), `XTsiz/YTsiz` (tile size),
  `XTOsiz/YTOsiz` (tile origin), `Csiz` (components) and per component
  `Ssiz` (precision and signedness), `XRsiz/YRsiz` (subsampling factors).
- **COD**: coding style `Scod` (precincts used, SOP/EPH used), progression
  order `SGcod`, number of layers, MCT flag, decomposition levels
  (`SPcod`), code-block width/height exponents, code-block style, transform
  (5-3 reversible / 9-7 irreversible), and per-resolution precinct sizes
  when precincts are used.
- **COC**: the component-scoped override of the COD fields.
- **QCD/QCC**: `Sqcd`: quantisation style (no quantisation / scalar derived
  / scalar expounded) and the step sizes, or the guard-bit count. One
  step size per subband, ordered by the standard's subband ordering.

The main header is read fully before any tile data; tile-part headers are
interleaved with tile bitstreams.

## 3. Decoding pipeline

1. **Parse main header**; derive the image/tile geometry, coding and
   quantisation parameters. Validate against profile constraints.
2. **For each tile(-part)**: read the tile-part header, then decode the
   packets.
3. **Per packet (tier 2)**: parse the packet header (code-block inclusion,
   zero bit-plane count, number of coding passes, segment lengths),
   extract the code-block bitstream segments.
4. **Per code-block (tier 1)**: arithmetic-decode the bit planes and
   reconstruct the quantised subband coefficients.
5. **Inverse quantisation**, **inverse DWT** per component.
6. **Inverse component transform** (RCT/ICT) if the MCT flag is set.
7. **Inverse DC level shift**; assemble the output image; optional colour
   transform from the JP2 `colr`/`pclr`/`cmap`/`cdef` boxes.

### 3.1 DC level shift

Unsigned samples of precision `P` are level-shifted by `-2^(P-1)` before
the forward transform so that sample values are symmetric about zero;
the inverse shift adds it back.

### 3.2 Component transforms (Part 1 MCT)

The default transform, signalled by the MCT flag in COD and applied when
there are exactly three components:

- **RCT** (reversible, 5-3 path):
  ```
  Y  = floor((I0 + 2*I1 + I2) / 4)
  U  = I2 - I1
  V  = I0 - I1
  ```
- **ICT** (irreversible, 9-7 path): a fixed real-valued 3x3 matrix.

Part 2 extends this with arbitrary multiple-component transforms declared
through MCT/MCC/MCO marker segments.

### 3.3 Discrete wavelet transform

Two standard kernels, both implemented as lifting factorisations:

**5-3 reversible (Le Gall):**

```
Y(2n+1) = X(2n+1) - floor((X(2n) + X(2n+2)) / 2)      (predict)
Y(2n)   = X(2n)   + floor((Y(2n-1) + Y(2n+1) + 2) / 4) (update)
```

**9-7 irreversible (CDF 9/7)**, with lifting steps

```
Y(2n+1) = X(2n+1) + alpha*(X(2n) + X(2n+2))
Y(2n)   = X(2n)   + beta *(Y(2n-1) + Y(2n+1))
Y(2n+1) = Y(2n+1) + gamma*(Y(2n) + Y(2n+2))
Y(2n)   = Y(2n)   + delta*(Y(2n-1) + Y(2n+1))
Y(2n+1) = K     * Y(2n+1)
Y(2n)   = (1/K) * Y(2n)
```

The transform is applied per tile-component across `NL` decomposition
levels: each level splits the current low-pass subband into LL, HL, LH, HH.
With `NL` levels there are `3*NL + 1` subbands (LL plus three per level);
the standard allows up to 33 levels.

### 3.4 Quantisation

- **Reversible path**: no quantisation (the integer lifting keeps
  everything exact). The QCD style is "no quantisation"; a guard-bit count
  `numgbits` is used for ROI scaling instead.
- **Irreversible path**: scalar quantisation per subband:
  ```
  q(c) = sign(c) * floor(|c| / step)
  ```
  with `step = 2^(Rb - epsb) * (1 + mu/2^11)`, where `Rb` is the nominal
  dynamic range of the subband (component precision plus analysis gain),
  and `(epsb, mu)` is the 16-bit exponent/mantissa pair stored per subband
  in QCD/QCC. "Scalar derived" stores only the exponent (mantissa 0);
  "scalar expounded" stores both.

### 3.5 Code-blocks and precincts

Each subband of each resolution is partitioned into **precincts** (only
meaningful when precincts are signalled in COD/COC; otherwise the precinct
is the whole subband), and each precinct into **code-blocks** — rectangular
arrays of at most `2^cblkw x 2^cblkh` coefficients (default 64x64), each
coded independently by the arithmetic coder.

### 3.6 ROI

- **Max-shift** (Part 1/AMD 1): all coefficients inside the ROI (scaled by
  `2^s`, `s` = the ROI shift stored in RGN) are shifted above the
  background coefficients; the decoder recovers the ROI first when
  decoding partially. Signalled by the code-block style bit `R`.
- **Implicit/scaling-based** (Part 2): the ROI shift scales the
  quantisation step sizes of the ROI background.

### 3.7 Tier-1 coding: bit-plane arithmetic coding (Part 1)

Each code-block is coded bit-plane by bit-plane, from the most significant
plane with any significant coefficient down to plane 0.

The context model uses a 3x3 neighbourhood of the current sample plus the
sample's significance state in the previous bit plane ("sigma"). Contexts
(19 in total) are formed from the horizontal/vertical/diagonal significance
counts. The coder processes three passes per bit plane:

1. **Significance propagation** (`SIG`): coefficients that are
   insignificant but have significant neighbours.
2. **Magnitude refinement** (`REF`): coefficients already significant.
3. **Cleanup** (`CLN`): everything else.

Each decision (significance, sign, refinement) is coded by the **MQ
arithmetic coder** (see below).

Coding options per code-block (`cblksty`):

- **Bypass** (`L`): after the first four planes, RAW and MQ decisions in
  REF/CLN use the raw (uncoded) path — fixed 1-bit symbols.
- **Reset** (`R`): contexts are reset to their initial probabilities at
  the start of each pass.
- **Terminate each pass** (`T`): the MQ coder is terminated and flushed
  after every pass, giving a segment length per pass.
- **Vertically stripe-causal** (`V`): the causal neighbourhood is the
  stripe above the current row instead of the full quadrant.
- **Predictable termination** (`P`): the number of bytes the coder *would*
  emit is tracked, so truncation points can be derived without storing
  per-pass lengths.
- **Segmentation symbols** (`S`): a fixed symbol pattern is coded at the
  end of each cleanup pass, letting decoders verify synchronisation.

**Skip path**: when a code-block is entirely insignificant down to its
zero bit-plane count, only the count (from tier 2) is coded.

#### MQ arithmetic coder

An adaptive binary arithmetic coder over 19 contexts:

- Each context holds a 16-bit probability `Qe` and a most-probable symbol
  (MPS/LPS convention).
- **Encoding**: the interval register `A` is split into the MPS/LPS
  sub-intervals; on an LPS the interval becomes `Qe` and the probability
  index transitions to the LPS state, on an MPS the interval becomes
  `A - Qe` with the MPS state transition. When `A < 0x8000`,
  **renormalisation** shifts `A` and the code register `C` left and emits
  bytes through the **bit-stuffing** rule: after a `0xFF` byte, the next
  byte's MSB is forced to 0 so marker codes never appear in-codestream.
- **Decoding** is the inverse: compare `(C >> 16)` against the LPS
  interval and renormalise by shifting in bytes.
- **Termination** (per code-block, and per pass when the `T` style is
  set): flush the interval with a minimal number of bytes, using the
  partial-byte technique.
- The probability states are the standard 47-state table indexed by
  (MPS, probability); initial state per context is set by the coding
  style (all-terminating style `T` uses the "fast attack" states).
- MPS and RAW symbols in bypass mode are written directly into the
  byte stream (`C += d << ct`), with a separate byte-stuffing path.

### 3.8 Tier-2 coding: packets, layers, progression

- A **packet** is the collection of code-block contributions for one
  layer, one resolution, one precinct and one component. The header
  describes, with tag-tree coded values, which code-blocks are included
  (their zero bit-plane counts) and how many bytes/passes each occupies.
- **Tag trees** encode the minimum of a set of integers over a quad-tree,
  a highly compact form for the inclusion/zero-plane information.
- **Layers** group packets by quality; truncating after layer `L` yields a
  lower-quality rendition. Rates per layer are signalled in the encoder
  parameters (distortion/rate allocation picks the layer boundaries).
- **Progression orders** order the packets in the codestream:
  - LRCP (layer, resolution, component, precinct) — quality-progressive,
  - RLCP (resolution, layer, component, precinct) — resolution-progressive,
  - RPCL (resolution, precinct, component, layer),
  - PCRL (precinct, component, resolution, layer),
  - CPRL (component, precinct, resolution, layer).
  POC marker segments may switch the order at given tile/resolution/
  component/layer boundaries.

### 3.9 Tile-parts

A tile may be split into several tile-part headers + bitstreams (SOT
`TPsot`/`TNsot`). A tile-part may carry any subset of the tile's packets;
this enables parallelism and incremental transfer. `PPT` (packed packet
headers) and `PLT` (packet lengths) in the tile-part headers, and `PPM`/
`PLM`/`TLM` in the main header, let decoders index packets/tile-parts
without parsing everything.

## 4. HTJ2K (T.814) decoding

HT blocks replace the MQ-coded cleanup pass with a variable-length coded
(MEL + VLC) representation:

- The block header holds the number of remaining bit planes and the
  segment length.
- **MEL** (Most-significant bit-plane Encoding Loop) codes the
  significance of runs of 2x2 quads with a simple adaptive unary-ish
  structure.
- **VLC** (SPP/MagSgn) codes the refinement/sign/magnitude of a 4-sample
  strip as a variable-length word per quad row, using one of a small set
  of tables keyed by the coding context.
- The `u_q` array carries the per-quad context (sigma pattern of the
  neighbours).
- HT blocks may be mixed with MQ blocks (HTMIXED style); the HT path is
  used only when every block of the tile-component is HT-coded.

## 5. JP2 file format (Part 1)

A JP2 file = a **signature box** (12-byte `\x00\x00\x00\x0CjP  \x0D\x0A\x87\x0A`),
a **file type box** (`ftyp`: brand, minor version, compatibility list), a
**header box** (`jp2h`), and exactly one **contiguous codestream box**
(`jp2c`), plus optional `xml`/`uuid` boxes.

Boxes: `LBox` (4-byte length), `TBox` (4-byte type). Length 0 means
"extends to end of file"; length 1 introduces an 8-byte `XLBox` (64-bit
length). Box types use printable 4CCs.

The `jp2h` box:

- **ihdr** (image header): height, width, number of components, bits per
  component (signedness in the MSB), compression type (7 = Part 1),
  colour space flag ("unknown" = 0, "known" = 1), IPR flag.
- **colr** (colour specification), method `Meth`:
  - 1: enumerated colour space (`EnumCS`): 16 sRGB, 17 greyscale, 18
    sYCC, 24 e-sRGB? (24 = e-sYCC in practice), 12 CMYK, 14 CIELab
    (restricted ICC profile follows);
  - 2: restricted ICC profile (`Prec`, `Approx`, profile bytes);
  - 3/4: vendor colour methods.
- **pclr** (palette), **cmap** (component mapping: palette/alpha/channel
  to codestream components), **cdef** (channel definition: the meaning of
  each component — colour, alpha, premultiplied alpha).
- **bpcc** (bits per component, when components differ).
- **res** (resolution: capture/display grid).

On decode, the JP2 header produces the final image colour space: `colr`
sets it, `pclr`+`cmap` apply palette expansion, `cdef` sets the channel
semantics, and an ICC profile overrides the enumerated colour space.

## 6. Encoder flow

1. Build the image; DC level-shift; apply the component transform (RCT/
   ICT or Part-2 MCT) when configured.
2. Per tile-component: forward DWT (`NL` levels), quantisation for the
   9-7 path.
3. Per code-block: tier-1 encode (bit planes, three passes each, MQ
   coder, optional bypass/reset/terminate/segsym styles).
4. Tier 2: build packets in the requested progression order and layer
   structure; **rate allocation** chooses, per layer, the truncation
   point of every code-block to hit the target rates, driven either by
   fixed layer rates ("fixed quality" via Lagrange multipliers over
   distortion slopes) or by fixed allocation tables (per resolution,
   layer and subband).
5. Emit main header, tile-part headers (SOT...SOD), packets, EOC; with
   PLT/PPT/TLM index markers as configured; MCT/MCC/MCO markers for
   Part-2 transforms; per-component COD/COC/QCD/QCC when non-default;
   RGN for ROI.
6. For JP2 output: wrap the codestream in signature/ftyp/jp2h(ihdr, colr
   (ICC or enumerated from the image colour space), optional
   pclr/cmap/cdef, res)/jp2c boxes.

## 7. Error handling and robustness

- Strict vs non-strict decoding: strict mode rejects malformed
  structures that a lenient decoder could tolerate (truncated tile-parts,
  inconsistent lengths, out-of-range values); the decoder must never
  write out of bounds in either mode.
- The zero bit-plane count caps the planes actually coded; segment
  lengths must stay within the packet; tile-part lengths must not exceed
  the codestream.
- Skipped ("corrupted") code-blocks decode to zero coefficients instead
  of aborting, so a damaged tile can still produce a partial image.
