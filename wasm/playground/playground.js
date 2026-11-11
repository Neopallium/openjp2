// openjp2 playground — drop-decode, encode-anything, codestream explorer.
//
// The heavy wasm decode/encode runs in a Web Worker (worker.js); if the
// worker cannot load (e.g. `file://` without a server), the page falls
// back to running the wasm on the main thread.
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
// worker plumbing (with a main-thread fallback)

const worker = (() => {
  try {
    return new Worker(new URL("./worker.js", import.meta.url), { type: "module" });
  } catch {
    return null;
  }
})();

let workerUp = worker !== null;
let callId = 0;
const pending = new Map();

if (worker) {
  worker.onmessage = (ev) => {
    const { id, ok, ...rest } = ev.data;
    const p = pending.get(id);
    if (!p) return;
    pending.delete(id);
    if (ok) p.resolve(rest);
    else p.reject(new Error(rest.error));
  };
  worker.onerror = () => {
    // module worker unsupported or the script failed to load: fall back
    // to main-thread wasm.
    workerUp = false;
    for (const [, p] of pending) p.reject(new Error("worker failed"));
    pending.clear();
  };
}

function rpc(msg, transfer) {
  return new Promise((resolve, reject) => {
    const id = ++callId;
    pending.set(id, { resolve, reject });
    worker.postMessage({ id, ...msg }, transfer || []);
  });
}

let decodeSeq = 0; // truncation-point ordering (explorer)
let loadSeq = 0; // file-load generation (panel + explorer)

/// Busy indicator: count of in-flight decode/encode jobs.
const active = { decode: 0, encode: 0 };
function updateBusy() {
  const el = $("busy");
  const busy = active.decode + active.encode > 0;
  el.hidden = !busy;
  el.textContent = active.decode ? "decoding…" : active.encode ? "encoding…" : "";
}

/// Decode to an RGBA probe, on the worker when available.  `seq` guards
/// against stale results from rapid truncation changes (explorer only);
/// `tag` tells the worker whether the job may be coalesced.
async function probeDecode(data, strict, seq, tag) {
  active.decode += 1;
  updateBusy();
  try {
    if (workerUp) {
      const copy = new Uint8Array(data); // must be standalone to transfer
      const res = await rpc({ type: "decode", tag, data: copy.buffer, strict }, [copy.buffer]);
      if (seq !== undefined && seq !== decodeSeq) return null; // stale
      return { width: res.width, height: res.height, complete: res.complete, pixels: res.pixels };
    }
    const probe = decode_probe(data, strict);
    return {
      width: probe.width,
      height: probe.height,
      complete: probe.complete,
      pixels: new Uint8Array(probe.pixels),
    };
  } finally {
    active.decode -= 1;
    updateBusy();
  }
}

async function workerEncode(width, height, rgba, lossless, format) {
  active.encode += 1;
  updateBusy();
  try {
    if (workerUp) {
      const copy = new Uint8Array(rgba);
      const res = await rpc(
        { type: "encode", width, height, data: copy.buffer, lossless, format },
        [copy.buffer],
      );
      return new Uint8Array(res.bytes);
    }
    return format === "j2k"
      ? encode_rgba_to_j2k(width, height, rgba, lossless)
      : encode_rgba_to_jp2(width, height, rgba, lossless);
  } finally {
    active.encode -= 1;
    updateBusy();
  }
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
        s.dataset.offset = i;
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

/// Selects the hex bytes in [start, end) and scrolls the first into view.
function selectHexSpan(start, end) {
  document.querySelectorAll("#hex-view .byte.selected").forEach((s) => s.classList.remove("selected"));
  const cap = Math.min(codestream.length, HEX_CAP);
  const lo = Math.max(start, 0);
  const hi = Math.min(end, cap);
  if (hi <= lo) return;
  let first = null;
  document.querySelectorAll("#hex-view .byte[data-offset]").forEach((s) => {
    const off = Number(s.dataset.offset);
    if (off >= lo && off < hi) {
      s.classList.add("selected");
      if (!first) first = s;
    }
  });
  if (first) first.scrollIntoView({ block: "center" });
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
  if (range.code !== undefined && range.code !== null && MARKERS[range.code]) {
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
  head.dataset.start = node.start;
  head.dataset.end = node.end;
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
    selectHexSpan(node.start, node.end);
    selectTreeNodes(node.start, node.end, true);
  });
  li.appendChild(head);
  return li;
}

/// Selects the tree node(s) covering [start, end); `preferSmallest`
/// picks the innermost node when several cover the span.
function selectTreeNodes(start, end, preferSmallest) {
  document.querySelectorAll("#tree-view .tree-node.selected").forEach((s) => s.classList.remove("selected"));
  const nodes = [...document.querySelectorAll("#tree-view .tree-node[data-start]")]
    .filter((n) => Number(n.dataset.start) <= start && Number(n.dataset.end) >= end);
  if (!nodes.length) return;
  const chosen = preferSmallest
    ? nodes.sort(
        (a, b) =>
          (a.dataset.end - a.dataset.start) - (b.dataset.end - b.dataset.start),
      )[0]
    : nodes[0];
  chosen.classList.add("selected");
  chosen.scrollIntoView({ block: "nearest" });
}

// ---------------------------------------------------------------------------
// explorer image view + truncation

const explorerCanvas = $("explorer-canvas");

function drawProbe(width, height, pixels, canvas) {
  const ctx = canvas.getContext("2d");
  canvas.width = width;
  canvas.height = height;
  if (width > 0 && height > 0 && pixels.length === width * height * 4) {
    const img = new ImageData(
      new Uint8ClampedArray(pixels.buffer, pixels.byteOffset, pixels.length),
      width,
      height,
    );
    ctx.putImageData(img, 0, 0);
  } else {
    // header-only decode: no pixel data for the declared geometry
    ctx.fillStyle = "#000";
    ctx.fillRect(0, 0, width, height);
  }
}

async function explorerDecode(cut) {
  const sub = codestream.subarray(0, cut);
  const t0 = performance.now();
  const complete = structurallyComplete(sub, csOffset);
  const seq = ++decodeSeq;
  const myLoad = loadSeq;
  let status;
  try {
    const probe = await probeDecode(sub, false, seq, "explorer");
    if (!probe || myLoad !== loadSeq) return; // superseded / new file
    const ms = performance.now() - t0;
    drawProbe(probe.width, probe.height, probe.pixels, explorerCanvas);
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
// truncation presets

const PRESETS = [500, 1024, 4096, 32768, 262144, 1048576];

function renderPresets() {
  const el = $("trunc-presets");
  el.replaceChildren();
  for (const size of PRESETS) {
    const b = document.createElement("button");
    b.type = "button";
    b.textContent = fmtBytes(size);
    b.disabled = !codestream || size >= codestream.length;
    b.title = `truncate to ${size} bytes`;
    b.addEventListener("click", () => {
      const cut = Math.min(size, codestream.length);
      $("trunc-slider").value = cut;
      applyCut(cut);
    });
    el.appendChild(b);
  }
}

// ---------------------------------------------------------------------------
// decode panel

const decodeCanvas = $("decode-canvas");

async function showDecode(data, strict) {
  const t0 = performance.now();
  const myLoad = loadSeq;
  // No truncation-seq guard: the panel always shows the full-file
  // decode; only a newer file load supersedes it.
  const probe = await probeDecode(data, strict, undefined, "panel");
  if (myLoad !== loadSeq) return;
  const ms = performance.now() - t0;
  drawProbe(probe.width, probe.height, probe.pixels, decodeCanvas);
  const complete = structurallyComplete(data, csOffset);
  setInfo($("decode-info"), [
    ["size", `${probe.width}×${probe.height}`],
    ["pixels", complete ? "complete" : "truncated (undecoded areas are black)"],
    ["decode", `${ms.toFixed(0)} ms`],
  ]);
  $("decode-drop").classList.add("has-image");
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
    loadSeq += 1;
    showDecode(data, $("decode-strict").checked).catch((e) =>
      setInfo($("decode-info"), [["error", String(e)]]),
    );
    // explorer
    const slider = $("trunc-slider");
    slider.max = data.length;
    slider.value = data.length;
    $("trunc-label").textContent = "100%";
    renderHex(data, data.length);
    renderTree(treeRoot, data.length);
    renderLegend();
    renderPresets();
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
// hex <-> tree selection

$("hex-view").addEventListener("click", (ev) => {
  const span = ev.target.closest("[data-range]");
  if (!span) return;
  const range = ranges[Number(span.dataset.range)];
  showDetail(range);
  selectHexSpan(range.start, range.end);
  selectTreeNodes(range.start, range.end, true);
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

async function runEncode() {
  const ctx = encodeCanvas.getContext("2d");
  const { width, height } = encodeCanvas;
  const rgba = ctx.getImageData(0, 0, width, height).data;
  const lossless = $("encode-lossless").checked;
  const format = $("encode-format").value;
  const t0 = performance.now();
  try {
    encoded = await workerEncode(width, height, rgba, lossless, format);
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
    showDecode(codestream, $("decode-strict").checked).catch((e) =>
      setInfo($("decode-info"), [["error", String(e)]]),
    );
  }
});

// ---------------------------------------------------------------------------

(async () => {
  try {
    let v;
    if (workerUp) {
      const res = await rpc({ type: "version" });
      v = res.version;
    } else {
      await init();
      v = version();
    }
    $("version").textContent = `openjp2-wasm ${v}`;
  } catch {
    $("version").textContent = "openjp2-wasm";
  }
})();
