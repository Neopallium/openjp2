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
    mark(name, pos, segEnd - pos, "marker", code, MARKERS[code]?.[2]);
    pushRange(ranges, pos, pos + 2, "marker", `${name} marker`, code, MARKERS[code]?.[2]);
    pushRange(ranges, pos + 2, segEnd, "marker-body", `${name} segment body`);
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
