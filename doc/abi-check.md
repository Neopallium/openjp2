# C-ABI compatibility check

`openjp2-capi` provides a drop-in replacement for the upstream C library
(`libopenjp2.so`).  A C consumer compiles against `openjpeg.h` and links
the library; its ABI is therefore a hard contract:

- every `opj_*` function declared in the header must be exported;
- the calling shape of each function must match (parameter/return types);
- structs the caller allocates or reads (`opj_image_t`,
  `opj_dparameters_t`, `opj_cparameters_t`, the codestream-info structs,
  …) must have identical size, field order and offsets;
- enum constants must have the same values.

## Options considered

| Option | What it checks | Verdict |
|---|---|---|
| `nm -D` symbol-set diff | exported names only | kept as the first stage — cheap and exact for the export surface |
| C header link test (compile a program including `openjpeg.h`, link the Rust lib) | missing symbols at link time | covered by the symbol stage; C cannot check the implementation's signatures |
| `abi-compliance-checker` / `abi-tracker` (the upstream C repo's workflow) | full ABI/API diff between *released versions* | not adopted: it downloads old releases and needs the Perl tooling; it compares C-vs-C releases, not C-vs-Rust |
| `abidiff` (libabigail) direct | full DWARF signature/type diff | not adopted as-is: C's opaque typedef pointers vs Rust's `c_void` pointers / `Option<fn>` callbacks are ABI-identical but reported as incompatible, producing ~50 false positives |
| **`abidw` XML analysis** (libabigail) | chosen | dumps both libraries' ABI to XML, then compares only the public surface with ABI-level normalization |
| `napetrov/abicheck` (issue #7's suggestion) | Rust ABI (not C ABI) tooling | not adopted for the C ABI; can be re-evaluated for the Rust surface |

## What the check does

`scripts/abi_check.py` (driven by `scripts/check_abi.sh`):

1. **Exports** — parses the `OPJ_API` declarations from the reference
   `openjpeg.h`, intersects them with the C library's exports (a few
   declared-but-unimplemented functions such as `opj_get_jp2_index` are
   ignored), and requires the Rust library's exported `opj_*` set to be
   exactly that.  Missing **or** unexpected exports fail.
2. **Signatures** — for each public function, compares a coarse ABI shape
   of every parameter and the return value (`pointer`, `fnptr`,
   `int<size>`, `float<size>`, `struct`, `void`, `enum<size>`).  Typedef
   and opaque-type *names* are intentionally ignored: they do not affect
   the calling convention.
3. **Struct layouts** — for every `typedef struct …` in the header, the
   Rust mirror must have the same size, field order and field offsets
   (member sizes are implied by the offsets and are not compared
   separately; a trailing `_` keyword escape such as `type_` is
   normalized).  Structs that are unused by the implemented API (the
   deprecated v1 codestream-info structs and the unimplemented JP2
   metadata/index structs) are allow-listed.

The remaining ABI aspects are covered functionally: the **parity** CI job
builds the C tools/tests against the Rust library and runs the full
upstream ctest suite, which exercises the struct layouts and enum values
end-to-end.

## Running it

```sh
# Debian/Ubuntu: the type-level stage needs libabigail
sudo apt install abigail-tools

# build both libraries and compare (C reference in target/abi-c)
./scripts/check_abi.sh

# symbol-only check (no libabigail)
./scripts/check_abi.sh --symbols-only

# reuse existing builds
SKIP_BUILD=1 ./scripts/check_abi.sh
```

CI runs it in the `abi` job of `.github/workflows/ci.yml`.
