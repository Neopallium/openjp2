// openjp2 playground — drop-decode, encode-anything, codestream explorer.
import init, {
  decode_probe,
  encode_rgba_to_jp2,
  encode_rgba_to_j2k,
  version,
} from "./pkg/openjp2_wasm.js";
import {
  MARKERS,
  KINDS,
  SPEC_URL,
  fmtBytes,
  buildStructure,
  structurallyComplete,
} from "./structure.js";

// ---------------------------------------------------------------------------
// element helpers
const $ = (id) => document.getElementById(id);

let codestream = null; // Uint8Array of the currently loaded file
let csOffset = 0; // where the codestream starts inside `codestream`
let ranges = []; // { start, end, kind, label, code?, spec? }
let treeRoot = null; // tree node

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

// ---------------------------------------------------------------------------
// hex view

const HEX_CAP = 96 * 1024;
const HEX_ROW = 16;

/// Per-byte index into `ranges` for the displayed region (Uint32Array).
/// Iterates forwards so later (inner, more specific) ranges override
/// earlier (outer) ones — e.g. markers override the enclosing jp2c box.
function buildLabelIndex(data) {
  const n = Math.min(data.length, HEX_CAP);
  const idx = new Uint32Array(n).fill(0xffffffff);
  for (let ri = 0; ri < ranges.length; ri++) {
    const r = ranges[ri];
    if (r.end <= 0 || r.start >= n) continue;
    for (let i = Math.max(r.start, 0); i < Math.min(r.end, n); i++) {
      idx[i] = ri;
    }
  }
  return idx;
}

let labelIndex = null;

function renderHex(data, cut) {
  const view = $("hex-view");
  view.textContent = "";
  if (!data || !data.length) {
    view.innerHTML = '<div class="hex-empty">decode a file first</div>';
    return;
  }
  labelIndex = buildLabelIndex(data);
  const n = Math.min(data.length, HEX_CAP);
  const frag = document.createDocumentFragment();
  const rows = Math.ceil(n / HEX_ROW);
  for (let r = 0; r < rows; r++) {
    const line = document.createElement("div");
    const addr = document.createElement("span");
    addr.className = "addr";
    addr.textContent = (r * HEX_ROW).toString(16).padStart(8, "0") + "  ";
    line.appendChild(addr);
    for (let c = 0; c < HEX_ROW; c++) {
      const i = r * HEX_ROW + c;
      const s = document.createElement("span");
      if (i >= n) {
        s.textContent = "   ";
      } else {
        const b = data[i];
        s.textContent = b.toString(16).padStart(2, "0") + " ";
        if (i >= cut) {
          s.className = "cut";
          s.title = "beyond the truncation point";
        } else {
          const ri = labelIndex[i];
          const range = ri !== 0xffffffff ? ranges[ri] : null;
          const kind = range ? KINDS[range.kind] : KINDS.corrupt;
          s.className = `byte ${kind.cls}`;
          if (range) {
            s.dataset.range = ri;
            s.title = `${range.label} (${range.start}–${range.end})`;
          } else {
            s.title = "unrecognized / corrupt data";
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

function showDetail(range) {
  const el = $("marker-detail");
  el.replaceChildren();
  const name = document.createElement("span");
  name.className = "marker-name";
  name.textContent = range.label;
  const kind = KINDS[range.kind]?.label || range.kind;
  const text = document.createElement("span");
  text.textContent = ` — ${kind}, bytes ${range.start}–${range.end} (${fmtBytes(range.end - range.start)})`;
  el.append(name, text);
  if (range.code !== undefined && MARKERS[range.code]) {
    el.append(document.createElement("br"));
    const link = document.createElement("a");
    link.href = SPEC_URL;
    link.target = "_blank";
    link.rel = "noopener";
    link.textContent = `ISO/IEC 15444-1 Annex ${MARKERS[range.code][2]} (ITU-T T.800)`;
    el.append(link);
    const desc = document.createElement("div");
    desc.className = "marker-spec";
    desc.textContent = MARKERS[range.code][1];
    el.append(desc);
  }
  if (range.spec) {
    const desc = document.createElement("div");
    desc.className = "marker-spec";
    desc.textContent = range.spec;
    el.append(desc);
  }
}

function highlightRange(range) {
  document.querySelectorAll(".byte.selected").forEach((s) => s.classList.remove("selected"));
  if (!range || !labelIndex) return;
  document.querySelectorAll("#hex-view .byte[data-range]").forEach((s) => {
    const ri = Number(s.dataset.range);
    if (ri < ranges.length) {
      const r = ranges[ri];
      if (r.start === range.start && r.end === range.end) s.classList.add("selected");
    }
  });
  const first = document.querySelector("#hex-view .byte.selected");
  if (first) first.scrollIntoView({ block: "center" });
}

// ---------------------------------------------------------------------------
// tree view

function renderTree(root, cut) {
  const el = $("tree-view");
  el.textContent = "";
  if (!root) return;
  el.appendChild(treeEl(root, cut, true));
}

function treeEl(node, cut, open) {
  const li = document.createElement("li");
  const head = document.createElement("div");
  head.className = "tree-node";
  if (node.children.length) {
    const det = document.createElement("details");
    det.open = open;
    const sum = document.createElement("summary");
    sum.textContent = node.label;
    const ul = document.createElement("ul");
    for (const child of node.children) ul.appendChild(treeEl(child, cut, false));
    det.append(sum, ul);
    head.appendChild(det);
  } else {
    head.textContent = node.label;
  }
  const span = document.createElement("span");
  span.className = "tree-range";
  span.textContent = `${fmtBytes(node.end - node.start)}`;
  head.appendChild(span);
  if (node.start >= cut) {
    head.classList.add("beyond-cut");
    head.title = "beyond the truncation point";
  }
  head.addEventListener("click", (ev) => {
    if (ev.target.closest("summary")) return; // let details toggle
    const range = {
      start: node.start,
      end: node.end,
      kind: node.kind || "corrupt",
      label: node.label,
      code: node.code,
      spec: node.spec,
    };
    showDetail(range);
    highlightRange(range);
  });
  li.appendChild(head);
  return li;
}

// ---------------------------------------------------------------------------
// explorer image view + truncation

const explorerCanvas = $("explorer-canvas");

function drawProbe(probe, canvas) {
  const ctx = canvas.getContext("2d");
  canvas.width = probe.width;
  canvas.height = probe.height;
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
}

function explorerDecode(cut) {
  const sub = codestream.subarray(0, cut);
  const t0 = performance.now();
  const complete = structurallyComplete(sub, csOffset);
  let status;
  try {
    const probe = decode_probe(sub, false);
    const ms = performance.now() - t0;
    drawProbe(probe, explorerCanvas);
    setInfo($("explorer-info"), [
      ["size", `${probe.width}×${probe.height}`],
      ["pixels", complete ? "complete" : "truncated (black = not decoded)"],
      ["decode", `${ms.toFixed(0)} ms`],
    ]);
    status = complete
      ? `decoded ${probe.width}×${probe.height} in ${ms.toFixed(0)} ms ✓`
      : probe.pixels.length
        ? "truncated: pixels decoded up to the cut point"
        : "header only (no pixel data survives at this length)";
  } catch (e) {
    setInfo($("explorer-info"), [["error", String(e)]]);
    status = `not decodable: ${e}`;
  }
  $("trunc-status").textContent = `${fmtBytes(cut)} → ${status}`;
}

let truncTimer = null;

function applyCut(cut) {
  $("trunc-label").textContent = `${Math.round((cut / codestream.length) * 100)}%`;
  renderHex(codestream, cut);
  renderTree(treeRoot, cut);
  clearTimeout(truncTimer);
  truncTimer = setTimeout(() => explorerDecode(cut), 150);
}

$("trunc-slider").addEventListener("input", (ev) => {
  if (!codestream) return;
  applyCut(Number(ev.target.value));
});

// ---------------------------------------------------------------------------
// decode panel

const decodeCanvas = $("decode-canvas");

function showDecode(data, strict) {
  const t0 = performance.now();
  const probe = decode_probe(data, strict);
  const ms = performance.now() - t0;
  drawProbe(probe, decodeCanvas);
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
    if (data.length < 2 || !(data[0] === 0xff && data[1] === 0x4f)) {
      // quick check for a JP2 signature box (12 bytes, type 'jP  ')
      if (
        data.length < 12 ||
        ((data[0] << 24) | (data[1] << 16) | (data[2] << 8) | data[3]) !== 12 ||
        ((data[4] << 24) | (data[5] << 16) | (data[6] << 8) | data[7]) !== 0x6a502020
      ) {
        setInfo($("decode-info"), [["error", "not a JP2/J2K file"]]);
        return;
      }
    }
    const structure = buildStructure(data);
    ranges = structure.ranges;
    treeRoot = structure.tree;
    csOffset = structure.csOffset;
    codestream = data;
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
    renderTree(treeRoot, data.length);
    renderLegend();
    explorerDecode(data.length);
  });
}

// ---------------------------------------------------------------------------
// hex legend

function renderLegend() {
  const el = $("hex-legend");
  el.replaceChildren();
  for (const { cls, label } of Object.values(KINDS)) {
    const item = document.createElement("span");
    item.className = `legend-item ${cls}`;
    item.textContent = label;
    el.appendChild(item);
  }
}

// ---------------------------------------------------------------------------
// byte clicks

$("hex-view").addEventListener("click", (ev) => {
  const span = ev.target.closest("[data-range]");
  if (!span) return;
  const range = ranges[Number(span.dataset.range)];
  showDetail(range);
  highlightRange(range);
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
    setInfo($("encode-info"), [["size", `${img.naturalWidth}×${img.naturalHeight}`]]);
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
