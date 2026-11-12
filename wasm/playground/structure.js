// Codestream/JP2 structure walking: box + marker range labelling and the
// file tree.  Pure functions (no DOM), exported for reuse and testing.

// JPEG 2000 markers (ISO/IEC 15444-1 Annex A section numbers; the public
// ITU-T T.800 recommendation is textually identical).
export const MARKERS = {
  0x4f: ["SOC", "Start of codestream", "A.4.1"],
  0x51: ["SIZ", "Image and tile size", "A.5.1"],
  0x52: ["COD", "Coding style default", "A.6.1"],
  0x53: ["COC", "Coding style component", "A.6.2"],
  0x54: ["TLM", "Tile-part lengths", "A.7.2"],
  0x55: ["PLM", "Packet length, main header", "A.7.4"],
  0x56: ["PLT", "Packet length, tile-part header", "A.7.3"],
  0x57: ["QCD", "Quantization default", "A.6.4"],
  0x58: ["QCC", "Quantization component", "A.6.5"],
  0x5b: ["RGN", "Region of interest", "A.6.3"],
  0x5c: ["POC", "Progression order change", "A.7.1"],
  0x5d: ["PPM", "Packed packet headers, main header", "A.8.1"],
  0x5e: ["PPT", "Packed packet headers, tile-part header", "A.8.2"],
  0x63: ["CRG", "Component registration", "A.10.3"],
  0x64: ["COM", "Comment", "A.10.2"],
  0x75: ["MCC", "Multiple component transform collection (Part 2)", "M.13"],
  0x76: ["MCO", "Multiple component transform ordering (Part 2)", "M.14"],
  0x78: ["CBD", "Component bit depth definition (Part 2)", "M.15"],
  0x90: ["SOT", "Start of tile-part", "A.4.2"],
  0x91: ["SOP", "Start of packet", "A.9.2"],
  0x92: ["EPH", "End of packet header", "A.9.3"],
  0x93: ["SOD", "Start of data", "A.4.3"],
  0xd9: ["EOC", "End of codestream", "A.4.4"],
};
export const SPEC_URL = "https://www.itu.int/rec/T-REC-T.800";

/// Packet progression order names (COD SGcod index).
export const PROGRESSIONS = ["LRCP", "RLCP", "RPCL", "PCRL", "CPRL"];

// JP2 boxes (ISO/IEC 15444-1 Annex I).
export const BOXES = {
  "jP  ": ["JP2 signature", "The 12-byte file signature box (I.5.1)."],
  ftyp: ["File type", "Brand, compatibility list and minor version (I.5.2)."],
  jp2h: ["JP2 header", "Superbox holding the image header boxes (I.5.3)."],
  ihdr: ["Image header", "Height, width, components, precision (I.5.3.1)."],
  bpcc: ["Bits per component", "Per-component bit depths (I.5.3.2)."],
  colr: ["Colour specification", "Colour space of the image (I.5.3.3)."],
  pclr: ["Palette", "Indexed colour palette (I.5.3.4)."],
  cmap: ["Component mapping", "Maps palette channels to components (I.5.3.5)."],
  cdef: ["Channel definition", "Channel types and associations (I.5.3.6)."],
  "res ": ["Resolution", "Capture/display resolution (I.5.3.7)."],
  jp2c: ["Codestream", "The JPEG 2000 codestream (I.5.4)."],
  uuid: ["UUID", "Vendor extension box (I.7.2)."],
  "xml ": ["XML", "Vendor extension XML box (I.7.2)."],
};

// Range kinds → CSS class + human label (used by the hex view and legend).
export const KINDS = {
  "box": { cls: "box", label: "box header" },
  "box-body": { cls: "box-body", label: "box contents" },
  "marker": { cls: "marker", label: "marker header" },
  "marker-body": { cls: "marker-body", label: "marker segment body" },
  "tile-part": { cls: "tile-part", label: "tile-part header" },
  "tile-data": { cls: "tile-data", label: "tile-part data (packets)" },
  "codestream": { cls: "codestream", label: "codestream data" },
  "corrupt": { cls: "corrupt", label: "unrecognized / corrupt" },
};

export function fmtBytes(n) {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KiB`;
  return `${(n / 1024 / 1024).toFixed(2)} MiB`;
}

export function markerName(code) {
  return MARKERS[code] ? MARKERS[code][0] : `FF${code.toString(16).toUpperCase()}`;
}

function u32(data, off) {
  return (data[off] << 24) | (data[off + 1] << 16) | (data[off + 2] << 8) | data[off + 3];
}
function u16(data, off) {
  return (data[off] << 8) | data[off + 1];
}

// A range is { start, end, kind, label, code?, spec? }; a tree node is
// { label, start, end, kind, code?, spec?, children: [] }.

function makeNode(label, start, end, kind, code, spec) {
  return { label, start, end, kind, code, spec, children: [] };
}

function pushRange(ranges, start, end, kind, label, code, spec) {
  if (end <= start) return;
  ranges.push({ start, end, kind, label, code, spec });
}

/// Walks the JP2 boxes; returns the `jp2c` box node (with the codestream
/// offset/end), or null.  Boxes after `jp2c` are walked too.
function walkBoxes(data, ranges, parent, off, end) {
  let jp2c = null;
  while (off + 8 <= end) {
    const rawLen = u32(data, off);
    const type = String.fromCharCode(
      data[off + 4], data[off + 5], data[off + 6], data[off + 7],
    );
    const [bname, bdesc] = BOXES[type] || [type.trim() || "?", "Unknown JP2 box."];
    // LBox == 0: the box extends to the end of the (super)box/file.
    const len = rawLen === 0 ? end - off : rawLen;
    if (len < 8 || off + len > end) {
      // broken length: the rest of the box content is corrupt
      const node = makeNode(`${bname} (broken length ${rawLen})`, off, end, "corrupt");
      parent.children.push(node);
      pushRange(ranges, off, end, "corrupt", `${bname} (broken length ${rawLen})`);
      return jp2c;
    }
    const node = makeNode(bname, off, off + len, "box", null, bdesc);
    parent.children.push(node);
    pushRange(ranges, off, off + 8, "box", `${bname} box header`, null, bdesc);
    pushRange(ranges, off + 8, off + len, "box-body", `${bname} box contents`, null, bdesc);
    if (type === "jp2h") {
      // superbox: walk the sub-boxes (bounded by the superbox)
      walkBoxes(data, ranges, node, off + 8, off + len);
    } else if (type === "jp2c") {
      jp2c = node;
    }
    off += len;
  }
  return jp2c;
}

/// Walks the codestream markers; tile-part payloads are skipped via SOT's
/// Psot.  Nesting goes into `parent` (the jp2c node, or the tree root).
function walkMarkers(data, ranges, cs, end, parent, comments) {
  if (cs + 2 > data.length || data[cs] !== 0xff || data[cs + 1] !== 0x4f) {
    const node = makeNode("(no codestream found)", cs, end, "corrupt");
    parent.children.push(node);
    pushRange(ranges, cs, end, "corrupt", "unrecognized data");
    return;
  }
  let pos = cs;
  const mark = (name, start, len, kind, code, spec) => {
    const node = makeNode(name, start, start + len, kind, code, spec);
    parent.children.push(node);
    return node;
  };
  mark("SOC", pos, 2, "marker", 0x4f, MARKERS[0x4f][2]);
  pushRange(ranges, pos, pos + 2, "marker", "SOC", 0x4f, MARKERS[0x4f][2]);
  pos += 2;
  while (pos + 1 < end) {
    if (data[pos] !== 0xff) {
      // bytes outside any segment: corrupt
      let j = pos;
      while (j + 1 < end && data[j] !== 0xff) j += 1;
      pushRange(ranges, pos, j, "corrupt", "unrecognized data (not a marker)");
      pos = j;
      continue;
    }
    const code = data[pos + 1];
    const name = markerName(code);
    if (code === 0xd9) {
      mark("EOC", pos, 2, "marker", 0xd9, MARKERS[0xd9][2]);
      pushRange(ranges, pos, pos + 2, "marker", "EOC", 0xd9, MARKERS[0xd9][2]);
      pos += 2;
      if (pos < end) {
        pushRange(ranges, pos, end, "corrupt", "bytes after EOC");
      }
      return;
    }
    if (code === 0x90) {
      // SOT: FF90 Lsot(2) Isot(2) Psot(4) TPsot(1) TNsot(1) = 12 bytes
      const psot = u32(data, pos + 6);
      const isot = u16(data, pos + 4);
      const tpsot = data[pos + 10];
      const tnsot = data[pos + 11];
      const label = `SOT (tile ${isot}, part ${tpsot + 1}/${tnsot})`;
      const node = mark(label, pos, 12, "tile-part", 0x90, MARKERS[0x90][2]);
      pushRange(ranges, pos, pos + 12, "tile-part", label, 0x90, MARKERS[0x90][2]);
      pos += 12;
      const dataEnd = psot > 12 ? Math.min(pos + psot - 12, end) : end;
      node.children.push(makeNode(
        `tile data (${fmtBytes(dataEnd - pos)})`, pos, dataEnd, "tile-data",
      ));
      pushRange(ranges, pos, dataEnd, "tile-data", `tile ${isot} part ${tpsot + 1} data`);
      pos = dataEnd;
      continue;
    }
    if (code === 0x93) {
      mark("SOD", pos, 1, "marker", 0x93, MARKERS[0x93][2]);
      pushRange(ranges, pos, pos + 1, "marker", "SOD", 0x93, MARKERS[0x93][2]);
      pos += 1;
      continue;
    }
    if (code === 0x64) {
      // COM: Rcom(2) + Ccom text
      const l = u16(data, pos + 2);
      if (l > 4 && pos + 2 + l <= data.length) {
        const rcom = u16(data, pos + 4);
        const text = new TextDecoder("utf-8", { fatal: false }).decode(
          data.subarray(pos + 6, pos + 2 + l),
        );
        const clean = text.replace(/\0+$/, "").trim();
        if (rcom === 1 && clean) comments.push(clean);
      }
    }
    // length markers
    const l = u16(data, pos + 2);
    if (l === 0) {
      mark(`${name} (to EOC)`, pos, end - pos, "marker", code, MARKERS[code]?.[2]);
      pushRange(ranges, pos, end, "marker", `${name} (to EOC)`, code, MARKERS[code]?.[2]);
      pos = end;
      continue;
    }
    const segEnd = Math.min(pos + 2 + l, end);
    const node = mark(name, pos, segEnd - pos, "marker", code, MARKERS[code]?.[2]);
    pushRange(ranges, pos, pos + 2, "marker", `${name} marker`, code, MARKERS[code]?.[2]);
    pushRange(ranges, pos + 2, segEnd, "marker-body", `${name} segment body`);
    if (code === 0x52 && segEnd > pos + 10) {
      // COD: expose the coding defaults as tree children (same range)
      const body = pos + 4;
      const prog = PROGRESSIONS[data[body + 1]] || "?";
      const layers = u16(data, body + 2);
      const res = data[body + 5] + 1;
      const cb = 1 << (data[body + 6] + 2);
      const transform = data[body + 9];
      const meta = (label) =>
        makeNode(label, pos, segEnd, "marker", 0x52, MARKERS[0x52][2]);
      node.children.push(meta(`progression: ${prog}`));
      node.children.push(meta(`quality layers: ${layers}`));
      node.children.push(meta(`resolutions: ${res}`));
      node.children.push(meta(`code-blocks: ${cb}×${cb}`));
      node.children.push(
        meta(`transform: ${transform === 1 ? "reversible 5-3" : "irreversible 9-7"}`),
      );
      node.meta = true;
    }
    pos = segEnd;
  }
  if (pos < end) {
    pushRange(ranges, pos, end, "corrupt", "truncated / unrecognized data");
  }
}

/// Builds the flat ranges and the tree for the loaded file; returns
/// `{ ranges, tree, csOffset }`.
export function buildStructure(data) {
  const ranges = [];
  const root = makeNode(
    data[0] === 0xff && data[1] === 0x4f ? "J2K codestream" : "JP2 file",
    0,
    data.length,
    "codestream",
  );
  const comments = [];
  let cs = -1;
  if (data[0] === 0xff && data[1] === 0x4f) {
    cs = 0;
    walkMarkers(data, ranges, 0, data.length, root, comments);
  } else {
    const jp2c = walkBoxes(data, ranges, root, 0, data.length);
    if (jp2c) {
      cs = jp2c.start + 8;
      walkMarkers(data, ranges, cs, jp2c.end, jp2c, comments);
    }
  }
  // cover any leftover gaps
  ranges.sort((a, b) => a.start - b.start || b.end - a.end);
  let cursor = 0;
  const extra = [];
  for (const r of ranges) {
    if (r.start > cursor) {
      extra.push({ start: cursor, end: r.start, kind: "corrupt", label: "unrecognized data" });
    }
    cursor = Math.max(cursor, r.end);
  }
  if (cursor < data.length) {
    extra.push({ start: cursor, end: data.length, kind: "corrupt", label: "unrecognized data" });
  }
  ranges.push(...extra);
  ranges.sort((a, b) => a.start - b.start || b.end - a.end);
  return { ranges, tree: root, csOffset: cs, comments };
}

/// Lightweight marker walk (no ranges/tree) for the completeness check:
/// true when the scan reaches an EOC within the data.
export function structurallyComplete(data, cs) {
  if (cs < 0 || cs + 2 > data.length || data[cs] !== 0xff || data[cs + 1] !== 0x4f) {
    return false;
  }
  let pos = cs + 2;
  while (pos + 1 < data.length) {
    if (data[pos] !== 0xff) {
      pos += 1;
      continue;
    }
    const code = data[pos + 1];
    if (code === 0xd9) return pos + 2 <= data.length;
    if (code === 0x90) {
      const psot = u32(data, pos + 6);
      pos += 12;
      if (psot > 12) pos += psot - 12;
      else return false;
      continue;
    }
    if (code === 0x93) {
      pos += 1;
      continue;
    }
    const l = u16(data, pos + 2);
    if (l === 0) return false;
    pos += 2 + l;
  }
  return false;
}

/// Reads the SIZ/COD fields of a JP2/J2K file and returns what the
/// codestream actually declares (the ground truth for the encoder UI):
/// `{ width, height, components, layers, progression, resolutions,
///    transform, codeblock, tile_w, tile_h }`.  Returns null when the
/// header is unreadable.
export function codestreamHeaderInfo(data) {
  // locate the codestream (raw SOC or the jp2c box body)
  let cs = -1;
  if (data.length >= 2 && data[0] === 0xff && data[1] === 0x4f) {
    cs = 0;
  } else {
    let off = 0;
    while (off + 8 <= data.length) {
      const len = u32(data, off) === 0 ? data.length - off : u32(data, off);
      const type = String.fromCharCode(
        data[off + 4], data[off + 5], data[off + 6], data[off + 7],
      );
      if (len < 8) break;
      if (type === "jp2c") {
        cs = off + 8;
        break;
      }
      off += len;
    }
  }
  if (cs < 0 || cs + 2 > data.length || data[cs] !== 0xff || data[cs + 1] !== 0x4f) {
    return null;
  }
  const out = {
    width: 0,
    height: 0,
    components: 0,
    layers: 0,
    progression: 0,
    resolutions: 0,
    transform: 1,
    codeblock: 0,
    tile_w: 0,
    tile_h: 0,
  };
  let pos = cs + 2;
  while (pos + 1 < data.length) {
    if (data[pos] !== 0xff) break;
    const code = data[pos + 1];
    if (code === 0x93 || code === 0x90 || code === 0xd9) break;
    const l = u16(data, pos + 2);
    if (l < 2) break;
    const body = pos + 4;
    if (code === 0x51 && body + 38 <= data.length) {
      // SIZ: Rsiz, Xsiz, Ysiz, XOsiz, YOsiz, XTsiz, YTsiz, XTOsiz, YTOsiz, Csiz
      out.width = u32(data, body + 2) - u32(data, body + 10);
      out.height = u32(data, body + 6) - u32(data, body + 14);
      out.tile_w = u32(data, body + 18);
      out.tile_h = u32(data, body + 22);
      out.components = u16(data, body + 34);
    } else if (code === 0x52 && body + 10 <= data.length) {
      // COD: Scod, prog, numlayers(2), mct, SPcod: numdecomp, cblk w/h
      // exponents (stored - 2), style, transform
      out.progression = data[body + 1];
      out.layers = u16(data, body + 2);
      out.resolutions = data[body + 5] + 1;
      out.codeblock = 1 << (data[body + 6] + 2);
      out.transform = data[body + 9]; // 1 = reversible 5-3, 0 = 9-7
    }
    pos += 2 + l;
  }
  return out;
}

/// Attaches packet entries (flat sextuples: start, end, layno, resno,
/// compno, precno) to the tree's tile-data nodes, grouped
/// layer > resolution > component > packet.  Returns the packet count.
export function attachPackets(tree, packets) {
  const count = Math.floor(packets.length / 6);
  if (!count) return 0;
  const MAX_DETAIL = 20000;
  const tileNodes = [];
  (function walk(n) {
    if (n.kind === "tile-data") tileNodes.push(n);
    for (const c of n.children) walk(c);
  })(tree);
  if (!tileNodes.length) return 0;
  const buckets = tileNodes.map(() => []);
  for (let i = 0; i < count; i++) {
    const o = i * 6;
    const start = packets[o];
    const end = packets[o + 1];
    let bi = tileNodes.findIndex((n) => start >= n.start && start < n.end);
    if (bi < 0) bi = tileNodes.length - 1;
    buckets[bi].push({
      start,
      end,
      layno: packets[o + 2],
      resno: packets[o + 3],
      compno: packets[o + 4],
      precno: packets[o + 5],
    });
  }
  const span = (list) => [
    list.reduce((m, p) => Math.min(m, p.start), Infinity),
    list.reduce((m, p) => Math.max(m, p.end), 0),
  ];
  const node = (label, start, end) => makeNode(label, start, end, "tile-data");
  for (let t = 0; t < tileNodes.length; t++) {
    const list = buckets[t];
    const parent = tileNodes[t];
    if (!list.length) continue;
    // avoid duplicate subtrees when re-attached
    if (parent.children.some((c) => c.label.startsWith("layer "))) continue;
    if (count > MAX_DETAIL) {
      const [s, e] = span(list);
      parent.children.push(node(`${list.length} packets (too many to list)`, s, e));
      continue;
    }
    const byLayer = new Map();
    for (const p of list) {
      if (!byLayer.has(p.layno)) byLayer.set(p.layno, []);
      byLayer.get(p.layno).push(p);
    }
    for (const layno of [...byLayer.keys()].sort((a, b) => a - b)) {
      const layerPkts = byLayer.get(layno);
      const [ls, le] = span(layerPkts);
      const layerNode = node(`layer ${layno + 1} (${layerPkts.length} packets)`, ls, le);
      const byRes = new Map();
      for (const p of layerPkts) {
        if (!byRes.has(p.resno)) byRes.set(p.resno, []);
        byRes.get(p.resno).push(p);
      }
      for (const resno of [...byRes.keys()].sort((a, b) => a - b)) {
        const resPkts = byRes.get(resno);
        const [rs, re] = span(resPkts);
        const resNode = node(`resolution ${resno} (${resPkts.length})`, rs, re);
        const byComp = new Map();
        for (const p of resPkts) {
          if (!byComp.has(p.compno)) byComp.set(p.compno, []);
          byComp.get(p.compno).push(p);
        }
        for (const compno of [...byComp.keys()].sort((a, b) => a - b)) {
          const compPkts = byComp.get(compno).sort((a, b) => a.precno - b.precno);
          const [cs, ce] = span(compPkts);
          const compNode = node(`component ${compno} (${compPkts.length})`, cs, ce);
          for (const p of compPkts) {
            compNode.children.push(
              node(
                `L${layno + 1} R${resno} C${compno} P${p.precno} — ${fmtBytes(p.end - p.start)}`,
                p.start,
                p.end,
              ),
            );
          }
          resNode.children.push(compNode);
        }
        layerNode.children.push(resNode);
      }
      parent.children.push(layerNode);
    }
    parent.packets = true;
  }
  return count;
}

/// For a codestream's progression order, groups the packet entries by the
/// outermost progression dimension (layer for LRCP, resolution for
/// RLCP/RPCL, precinct for PCRL, component for CPRL) and returns the
/// cumulative byte threshold of each group — i.e. the prefix length that
/// fully contains the first N values of that dimension.
///
/// `packets` is the flat sextuple array (start, end, layno, resno,
/// compno, precno).  Returns `{ dimension, groups: [{ value, bytes }] }`.
export function progressionGroups(packets, order) {
  // sextuple indexes: 2 = layer, 3 = resolution, 4 = component, 5 = precinct
  const dim = order === 0 ? 2 : order === 1 || order === 2 ? 3 : order === 3 ? 5 : 4;
  const names = ["layer", "resolution", "resolution", "precinct", "component"];
  const byValue = new Map();
  for (let i = 0; i < packets.length; i += 6) {
    const v = packets[i + dim];
    const end = packets[i + 1];
    byValue.set(v, Math.max(byValue.get(v) || 0, end));
  }
  const values = [...byValue.keys()].sort((a, b) => a - b);
  let acc = 0;
  const groups = values.map((value) => {
    acc = Math.max(acc, byValue.get(value));
    return { value, bytes: acc };
  });
  return { dimension: names[order] || "layer", groups };
}
