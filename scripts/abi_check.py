#!/usr/bin/env python3
"""C-ABI compatibility check: Rust `libopenjp2` vs the C reference build.

Checks:
1. Exported surface: every `opj_*` function the reference `openjpeg.h`
   declares (and the C library implements) must be exported by the Rust
   library, and the Rust library must not export `opj_*` functions the
   header does not declare.
2. Function signatures (coarse ABI shape): for the shared public
   functions, parameter/return shapes (pointer vs integer size vs float
   vs struct vs void) must match.  Typedef/opaque-pointer naming
   differences are ignored by construction (they are ABI-equivalent).
3. Public struct layouts: for each `typedef struct ...` in
   `openjpeg.h`, the Rust mirror must have the same size, field order
   and field offsets (field-name keyword escapes like `type_` are
   normalized away).

Type information comes from libabigail's `abidw` XML dumps of both
libraries (built with debug info).  Without `abidw` only the symbol
check runs (`--symbols-only`).

Usage:
  abi_check.py --header openjpeg.h --c-lib libopenjp2.so \\
               --rust-lib libopenjp2.so [--abidw abidw] [--symbols-only]
"""

import argparse
import os
import re
import shutil
import subprocess
import sys
import tempfile
import xml.etree.ElementTree as ET

# Header structs that are intentionally not mirrored by the Rust library:
# the deprecated v1 codestream-info structs (no exported function uses
# them in 2.5.x) and the JP2 metadata/index structs (their two accessors
# are declared in the header but not implemented by the C reference).
ABSENT_STRUCTS_OK = {
    "opj_tp_info",
    "opj_tile_info",
    "opj_codestream_info",
    "opj_jp2_metadata",
    "opj_jp2_index",
}

# Struct tag aliases (the Rust mirror uses a different order of the
# version suffix).
STRUCT_ALIASES = {"opj_tile_v2_info": "opj_tile_info_v2"}


def die(msg):
    print(f"error: {msg}", file=sys.stderr)
    sys.exit(2)


def run(cmd, **kw):
    return subprocess.run(cmd, check=True, text=True, **kw)


def parse_header(path):
    text = open(path, encoding="utf-8").read()
    # drop comments so commented-out declarations do not count
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    fns = set(re.findall(r"OPJ_API\s+[^;]*?\b(opj_[A-Za-z0-9_]+)\s*\(", text))
    structs = re.findall(r"typedef struct\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{", text)
    return fns, structs


def exported_symbols(lib):
    out = run(["nm", "-D", "--defined-only", lib], capture_output=True).stdout
    names = set()
    for line in out.splitlines():
        parts = line.split()
        if len(parts) >= 3 and parts[2].startswith("opj_"):
            names.add(parts[2])
    return names


def abidw_xml(abidw, lib, out_path):
    # libabigail <= 2.4 has no `-o` option: capture stdout instead.
    proc = subprocess.run(
        [abidw, "--no-show-locs", lib],
        check=True,
        text=True,
        capture_output=True,
    )
    with open(out_path, "w", encoding="utf-8") as f:
        f.write(proc.stdout)


def num(v, default=0):
    try:
        return int(v)
    except (TypeError, ValueError):
        return default


def load_xml(path):
    root = ET.parse(path).getroot()
    types = {el.get("id"): el for el in root.iter() if el.get("id")}
    return root, types


def canonical(tid, types, depth=0):
    """Coarse, ABI-relevant shape of a type (naming/typedefs ignored)."""
    if tid is None or depth > 16:
        return "?"
    el = types.get(tid)
    if el is None:
        return "?"
    tag = el.tag
    if tag == "pointer-type-def":
        inner = types.get(el.get("type-id"))
        return "fnptr" if inner is not None and inner.tag == "function-type" else "ptr"
    if tag in ("typedef-decl", "qualified-type-def", "restrict-type-def"):
        return canonical(el.get("type-id"), types, depth + 1)
    if tag in ("class-decl", "union-decl", "struct-decl", "type-decl"):
        name = el.get("name", "")
        # Rust represents `extern "C" fn` parameters as Option<fn..>
        return "fnptr" if name.startswith("Option<") and "fn(" in name else "struct"
    if tag == "enum-type-decl":
        return "enum%d" % num(el.get("size-in-bits"))
    if tag == "function-type":
        return "fnptr"
    if tag == "array-type":
        return "array"
    if tag == "void-type-decl":
        return "void"
    if tag in ("base-type", "int-type-decl", "char-type-decl", "bool-type-decl"):
        return "int%d" % num(el.get("size-in-bits"))
    if tag == "float-type-decl":
        return "float%d" % num(el.get("size-in-bits"))
    return tag


def functions(root, types):
    out = {}
    for fd in root.iter("function-decl"):
        params = tuple(canonical(p.get("type-id"), types) for p in fd.findall("parameter"))
        ret = fd.find("return")
        out[fd.get("name")] = (
            params,
            canonical(ret.get("type-id"), types) if ret is not None else "?",
        )
    return out


def structs(root, types):
    out = {}
    for el in root.iter("class-decl"):
        if el.get("is-struct") != "yes":
            continue
        name = el.get("name", "").split("::")[-1]
        members = []
        for dm in el.findall("data-member"):
            vd = dm.find("var-decl")
            if vd is None:
                continue
            # offsets + total size fully define the ABI layout; member
            # *sizes* are implied and libabigail < 2.5 reports enum
            # sizes as unknown, so they are not compared.
            members.append((vd.get("name", ""), num(dm.get("layout-offset-in-bits"))))
        out.setdefault(name, []).append((num(el.get("size-in-bits")), tuple(members)))
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--header", required=True, help="reference openjpeg.h")
    ap.add_argument("--c-lib", required=True, help="reference libopenjp2.so")
    ap.add_argument("--rust-lib", required=True, help="Rust libopenjp2.so")
    ap.add_argument(
        "--abidw",
        default=os.environ.get("ABIDW", "abidw"),
        help="libabigail abidw binary (default: $ABIDW or 'abidw')",
    )
    ap.add_argument(
        "--symbols-only",
        action="store_true",
        help="skip the abidw type checks (no libabigail installed)",
    )
    args = ap.parse_args()

    header_fns, header_structs = parse_header(args.header)
    if not header_fns:
        die(f"no OPJ_API declarations found in {args.header}")

    # ---- 1. exported symbol surface -----------------------------------
    c_exports = exported_symbols(args.c_lib)
    rust_exports = exported_symbols(args.rust_lib)
    public = header_fns & c_exports
    unimplemented = sorted(header_fns - c_exports)
    missing = sorted(public - rust_exports)
    unexpected = sorted(rust_exports - header_fns)

    failures = 0
    print(f"== exported API ({len(public)} functions)")
    if unimplemented:
        print("  declared in the header but not built by the C reference (ignored):")
        print("   ", ", ".join(unimplemented))
    if missing:
        failures += 1
        print("  MISSING from the Rust library:")
        for name in missing:
            print("   ", name)
    if unexpected:
        failures += 1
        print("  UNEXPECTED exports not declared in the header:")
        for name in unexpected:
            print("   ", name)
    if not missing and not unexpected:
        print(f"  ok: all {len(public)} functions exported, no extras")

    if args.symbols_only:
        print()
        print("abi check:", "FAILED" if failures else "OK")
        return 1 if failures else 0

    if not shutil.which(args.abidw) and not os.path.isfile(args.abidw):
        die(
            "abidw not found (install libabigail, e.g. `apt install abigail-tools`), "
            "or pass --symbols-only"
        )

    with tempfile.TemporaryDirectory() as tmp:
        c_xml = os.path.join(tmp, "c.xml")
        r_xml = os.path.join(tmp, "rust.xml")
        abidw_xml(args.abidw, args.c_lib, c_xml)
        abidw_xml(args.abidw, args.rust_lib, r_xml)
        croot, ctypes = load_xml(c_xml)
        rroot, rtypes = load_xml(r_xml)

    # ---- 2. function signature shapes ---------------------------------
    c_fns = functions(croot, ctypes)
    r_fns = functions(rroot, rtypes)
    print(f"== function signatures ({len(public)} checked)")
    sig_bad = 0
    for name in sorted(public):
        if name not in c_fns or name not in r_fns:
            sig_bad += 1
            print(f"  {name}: missing from the {'C' if name not in c_fns else 'Rust'} ABI dump")
            continue
        if c_fns[name] != r_fns[name]:
            sig_bad += 1
            print(f"  {name}:")
            print(f"    C   {c_fns[name]}")
            print(f"    Rust{r_fns[name]}")
    failures += 1 if sig_bad else 0
    if not sig_bad:
        print(f"  ok: {len(public)} signatures ABI-compatible")

    # ---- 3. public struct layouts -------------------------------------
    c_structs = structs(croot, ctypes)
    r_structs = structs(rroot, rtypes)
    print(f"== public struct layouts ({len(header_structs)} in the header)")
    layout_bad = 0
    absent = []
    for tag in header_structs:
        cerr = c_structs.get(tag)
        rerr = r_structs.get(STRUCT_ALIASES.get(tag, tag))
        if not cerr or not rerr:
            absent.append(tag)
            continue
        c_size, c_members = cerr[0]
        r_size, r_members = rerr[0]
        # keyword-escape normalization (`type_` <-> `type`)
        norm = lambda members: [(n.rstrip("_"), o) for n, o in members]
        if c_size != r_size or norm(c_members) != norm(r_members):
            layout_bad += 1
            print(f"  {tag}: layout mismatch")
            if c_size != r_size:
                print(f"    size: C {c_size} bits, Rust {r_size} bits")
            for i in range(max(len(c_members), len(r_members))):
                c_m = norm(c_members)[i] if i < len(c_members) else None
                r_m = norm(r_members)[i] if i < len(r_members) else None
                if c_m != r_m:
                    print(f"    member {i}: C {c_m}, Rust {r_m}")
    for tag in absent:
        if tag in ABSENT_STRUCTS_OK:
            print(f"  ok (not mirrored, unused by the implemented API): {tag}")
        else:
            layout_bad += 1
            print(f"  MISSING from the Rust library: {tag}")
    failures += 1 if layout_bad else 0
    if not layout_bad:
        print(f"  ok: {len(header_structs) - len(absent)} layouts match")

    print()
    print("abi check:", "FAILED" if failures else "OK")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
