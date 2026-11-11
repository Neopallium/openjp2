// openjp2 playground — drop-decode, encode-anything, codestream explorer.
import init, {
  decode_probe,
  encode_rgba_to_jp2,
  encode_rgba_to_j2k,
  version,
} from "./pkg/openjp2_wasm.js";

// ---------------------------------------------------------------------------
// JPEG 2000 marker dictionary (ISO/IEC 15444-1 Annex A section numbers;
// the public ITU-T T.800 recommendation is textually identical).
const MARKERS = {
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
const SPEC_URL = "https://www.itu.int/rec/T-REC-T.800";

// ---------------------------------------------------------------------------
// element helpers
const $ = (id) => document.getElementById(id);

let codestream = null; // Uint8Array of the currently loaded codestream
let markers = []; // [{ start, end, name }] from scanMarkers
let csOffset = 0; // where the codestream starts inside `codestream`

function setInfo(el, entries) {
  el.replaceChildren(
    ...entries.map(([k, v]) => {
      const dt = document.createElement("dt");
      dt.textContent = k;
      const dd = document.createElement("dd");
      dd.textContent = v;
      return [dt, dd];
    }).flat(),
  );
}

function fmtBytes(n) {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KiB`;
  return `${(n / 1024 / 1024).toFixed(2)} MiB`;
}

// ---------------------------------------------------------------------------
// codestream location + marker scan

/// Finds where the J2K codestream starts: raw SOC, or the `jp2c` box.
function codestreamOffset(data) {
  if (data.length >= 2 && data[0] === 0xff && data[1] === 0x4f) return 0;
  let off = 0;
  while (off + 8 <= data.length) {
    const len =
      (data[off] << 24) | (data[off + 1] << 16) | (data[off + 2] << 8) | data[off + 3];
    const type = String.fromCharCode(
      data[off + 4], data[off + 5], data[off + 6], data[off + 7],
    );
    if (len < 8) break;
    if (type === "jp2c") return off + 8;
    off += len;
  }
  return -1;
}

/// Scans marker segments (accurate for headers; tile-part data is skipped
/// via SOT's Psot field).
function scanMarkers(data, cs) {
  const out = [];
  if (cs < 0 || cs + 2 > data.length || data[cs] !== 0xff || data[cs + 1] !== 0x4f) {
    return out;
  }
  const name = (code) => (MARKERS[code] ? MARKERS[code][0] : `FF${code.toString(16).toUpperCase()}`);
  out.push({ start: cs, end: cs + 2, name: "SOC" });
  let pos = cs + 2;
  while (pos + 1 < data.length) {
    if (data[pos] !== 0xff) {
      pos += 1;
      continue;
    }
    const code = data[pos + 1];
    if (code === 0xd9) {
      out.push({ start: pos, end: pos + 2, name: "EOC" });
      break;
    }
    if (code === 0x90) {
      out.push({ start: pos, end: pos + 12, name: "SOT" });
      const psot =
        (data[pos + 6] << 24) | (data[pos + 7] << 16) | (data[pos + 8] << 8) | data[pos + 9];
      pos += 12;
      if (psot > 12) {
        pos += psot - 12;
      } else {
        // tile-part runs to EOC
        const eoc = data.indexOf(0xd9, pos);
        if (eoc > 0 && data[eoc - 1] === 0xff) {
          out.push({ start: eoc - 1, end: eoc + 1, name: "EOC" });
        }
        break;
      }
      continue;
    }
    if (code === 0x93) {
      out.push({ start: pos, end: pos + 1, name: "SOD" });
      pos += 1;
      continue;
    }
    const l = (data[pos + 2] << 8) | data[pos + 3];
    if (l === 0) {
      out.push({ start: pos, end: data.length, name: name(code) });
      break;
    }
    out.push({ start: pos, end: pos + 2 + l, name: name(code) });
    pos += 2 + l;
  }
  return out;
}

// ---------------------------------------------------------------------------
// hex view (capped) + marker detail

const HEX_CAP = 96 * 1024;

function renderHex(data, cut) {
  const view = $("hex-view");
  view.textContent = "";
  if (!data || !data.length) {
    view.innerHTML = '<div class="hex-empty">decode a file first</div>';
    return;
  }
  const n = Math.min(data.length, HEX_CAP);
  const frag = document.createDocumentFragment();
  const rows = Math.ceil(n / 16);
  for (let r = 0; r < rows; r++) {
    const line = document.createElement("div");
    const addr = document.createElement("span");
    addr.className = "addr";
    addr.textContent = (r * 16).toString(16).padStart(8, "0") + "  ";
    line.appendChild(addr);
    for (let c = 0; c < 16; c++) {
      const i = r * 16 + c;
      const s = document.createElement("span");
      if (i >= n) {
        s.textContent = "   ";
      } else {
        const b = data[i];
        s.textContent = b.toString(16).padStart(2, "0") + " ";
        if (i >= cut) {
          s.className = "cut";
        } else if (b === 0x00) {
          s.className = "zero";
        } else {
          s.className = "byte";
        }
        if (i < cut) {
          const m = markers.find((m) => i >= m.start && i < m.end);
          if (m) {
            s.classList.add("marker-byte");
            s.dataset.marker = m.name;
            s.dataset.start = m.start;
            s.dataset.end = m.end;
          }
        }
      }
      line.appendChild(s);
    }
    frag.appendChild(line);
  }
  if (data.length > HEX_CAP) {
    const note = document.createElement("div");
    note.className = "hex-empty";
    note.textContent = `… showing the first ${fmtBytes(HEX_CAP)} of ${fmtBytes(data.length)}`;
    frag.appendChild(note);
  }
  view.appendChild(frag);
}

function markerDetail(name, start, end) {
  const el = $("marker-detail");
  const [full, desc, section] = MARKERS[parseInt(name.slice(2), 16)] ||
    [name, "Unknown marker", null];
  el.replaceChildren();
  const span = document.createElement("span");
  span.className = "marker-name";
  span.textContent = full;
  const text = document.createElement("span");
  text.textContent = ` — ${desc} (bytes ${start}–${end})`;
  el.append(span, text);
  if (section) {
    el.append(document.createElement("br"));
    const link = document.createElement("a");
    link.href = SPEC_URL;
    link.target = "_blank";
    link.rel = "noopener";
    link.textContent = `ISO/IEC 15444-1 Annex ${section} (ITU-T T.800)`;
    el.append(link);
  }
}

// ---------------------------------------------------------------------------
// decode panel

const decodeCanvas = $("decode-canvas");

/// True when the marker scan reaches an EOC within the data: every
/// tile-part's Psot fit, so the pixels are complete.
function structurallyComplete(data, cs) {
  const ms = scanMarkers(data, cs);
  if (ms.length === 0) return false;
  const last = ms[ms.length - 1];
  return last.name === "EOC" && last.end <= data.length;
}

function showDecode(data, strict) {
  const t0 = performance.now();
  const probe = decode_probe(data, strict);
  const ms = performance.now() - t0;
  const ctx = decodeCanvas.getContext("2d");
  decodeCanvas.width = probe.width;
  decodeCanvas.height = probe.height;
  if (probe.pixels.length === probe.width * probe.height * 4) {
    const img = new ImageData(
      new Uint8ClampedArray(probe.pixels.buffer, probe.pixels.byteOffset, probe.pixels.length),
      probe.width,
      probe.height,
    );
    ctx.putImageData(img, 0, 0);
  } else {
    ctx.fillStyle = "#000";
    ctx.fillRect(0, 0, probe.width, probe.height);
  }
  const complete = structurallyComplete(data, csOffset);
  setInfo($("decode-info"), [
    ["size", `${probe.width}×${probe.height}`],
    ["pixels", complete ? "complete" : "truncated (undecoded areas are black)"],
    ["decode", `${ms.toFixed(0)} ms`],
  ]);
  $("decode-drop").classList.add("has-image");
  return probe;
}

function loadCodestream(file) {
  file.arrayBuffer().then((buf) => {
    const data = new Uint8Array(buf);
    csOffset = codestreamOffset(data);
    if (csOffset < 0) {
      setInfo($("decode-info"), [["error", "no codestream found (not a JP2/J2K file?)"]]);
      return;
    }
    codestream = data;
    markers = scanMarkers(data, csOffset);
    try {
      showDecode(data, $("decode-strict").checked);
    } catch (e) {
      setInfo($("decode-info"), [["error", String(e)]]);
    }
    // explorer
    const slider = $("trunc-slider");
    slider.max = data.length;
    slider.value = data.length;
    $("trunc-label").textContent = "100%";
    renderHex(data, data.length);
    $("trunc-status").textContent =
      `${fmtBytes(data.length)} · ${markers.length - 2} marker segments`;
  });
}

// ---------------------------------------------------------------------------
// truncation slider (lenient re-decode)

let truncTimer = null;

$("trunc-slider").addEventListener("input", (ev) => {
  if (!codestream) return;
  const cut = Number(ev.target.value);
  $("trunc-label").textContent = `${Math.round((cut / codestream.length) * 100)}%`;
  renderHex(codestream, cut);
  clearTimeout(truncTimer);
  truncTimer = setTimeout(() => {
    const sub = codestream.subarray(0, cut);
    const t0 = performance.now();
    const complete = structurallyComplete(sub, csOffset);
    try {
      const probe = decode_probe(sub, false);
      const ms = performance.now() - t0;
      const status = complete
        ? `decoded ${probe.width}×${probe.height} in ${ms.toFixed(0)} ms ✓`
        : probe.pixels.length
          ? "truncated: pixels decoded up to the cut point"
          : "header only (no pixel data survives at this length)";
      $("trunc-status").textContent = `${fmtBytes(cut)} → ${status}`;
    } catch (e) {
      $("trunc-status").textContent = `${fmtBytes(cut)} → not decodable: ${e}`;
    }
  }, 120);
});

// ---------------------------------------------------------------------------
// marker clicks

$("hex-view").addEventListener("click", (ev) => {
  const span = ev.target.closest("[data-marker]");
  if (!span) return;
  document.querySelectorAll(".marker-byte.hot").forEach((s) => s.classList.remove("hot"));
  document
    .querySelectorAll(`[data-marker="${span.dataset.marker}"][data-start="${span.dataset.start}"]`)
    .forEach((s) => s.classList.add("hot"));
  markerDetail(span.dataset.marker, Number(span.dataset.start), Number(span.dataset.end));
});

// ---------------------------------------------------------------------------
// encode panel

const encodeCanvas = $("encode-canvas");
let encoded = null;

function loadSourceImage(file) {
  const url = URL.createObjectURL(file);
  const img = new Image();
  img.onload = () => {
    encodeCanvas.width = img.naturalWidth;
    encodeCanvas.height = img.naturalHeight;
    encodeCanvas.getContext("2d").drawImage(img, 0, 0);
    URL.revokeObjectURL(url);
    $("encode-drop").classList.add("has-image");
    setInfo($("encode-info"), [
      ["size", `${img.naturalWidth}×${img.naturalHeight}`],
    ]);
    runEncode();
  };
  img.src = url;
}

function runEncode() {
  const ctx = encodeCanvas.getContext("2d");
  const { width, height } = encodeCanvas;
  const rgba = ctx.getImageData(0, 0, width, height).data;
  const lossless = $("encode-lossless").checked;
  const format = $("encode-format").value;
  const t0 = performance.now();
  try {
    encoded = format === "j2k"
      ? encode_rgba_to_j2k(width, height, rgba, lossless)
      : encode_rgba_to_jp2(width, height, rgba, lossless);
  } catch (e) {
    setInfo($("encode-info"), [["error", String(e)]]);
    return;
  }
  const ms = performance.now() - t0;
  setInfo($("encode-info"), [
    ["size", `${width}×${height}`],
    ["codestream", `${fmtBytes(encoded.length)}`],
    ["encode", `${ms.toFixed(0)} ms`],
  ]);
  $("encode-download").disabled = false;
}

$("encode-lossless").addEventListener("change", runEncode);
$("encode-format").addEventListener("change", runEncode);
$("encode-download").addEventListener("click", () => {
  if (!encoded) return;
  const ext = $("encode-format").value;
  const blob = new Blob([encoded], { type: "application/octet-stream" });
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  a.download = `encoded.${ext}`;
  a.click();
  URL.revokeObjectURL(a.href);
});

// ---------------------------------------------------------------------------
// dropzones + browse

function wireDrop(zone, input, onFile) {
  ["dragover", "dragenter"].forEach((ev) =>
    zone.addEventListener(ev, (e) => {
      e.preventDefault();
      zone.classList.add("dragover");
    }),
  );
  ["dragleave", "drop"].forEach((ev) =>
    zone.addEventListener(ev, () => zone.classList.remove("dragover")),
  );
  zone.addEventListener("drop", (e) => {
    e.preventDefault();
    if (e.dataTransfer.files.length) onFile(e.dataTransfer.files[0]);
  });
  zone.addEventListener("click", () => input.click());
  input.addEventListener("change", () => {
    if (input.files.length) onFile(input.files[0]);
  });
}

const decodeInput = document.createElement("input");
decodeInput.type = "file";
decodeInput.accept = ".jp2,.j2k,.j2c,.jpc,.jpf,.jpx";
wireDrop($("decode-drop"), decodeInput, loadCodestream);

const encodeInput = document.createElement("input");
encodeInput.type = "file";
encodeInput.accept = "image/*";
wireDrop($("encode-drop"), encodeInput, loadSourceImage);

$("decode-strict").addEventListener("change", () => {
  if (codestream) {
    try {
      showDecode(codestream, $("decode-strict").checked);
    } catch (e) {
      setInfo($("decode-info"), [["error", String(e)]]);
    }
  }
});

// ---------------------------------------------------------------------------

init().then(() => {
  $("version").textContent = `openjp2-wasm ${version()}`;
});
