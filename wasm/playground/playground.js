// openjp2 playground — drop-decode, encode-anything, codestream explorer.
//
// The heavy wasm decode/encode runs in a Web Worker (worker.js); if the
// worker cannot load (e.g. `file://` without a server), the page falls
// back to running the wasm on the main thread.
import init, {
  decode_probe_opt,
  encode_rgba_opt,
  decode_info,
  layer_byte_thresholds,
  DecodeOptions,
  EncodeOptions,
  version,
} from "./pkg/openjp2_wasm.js";
import {
  MARKERS,
  KINDS,
  SPEC_URL,
  fmtBytes,
  buildStructure,
  structurallyComplete,
  codestreamHeaderInfo,
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
let reduce = 0; // resolution reduction factor (0 = full)
let numResolutions = 1; // from decode_info (header)
let headerInfo = null; // decode_info metadata (header/coding defaults)
let comments = []; // COM marker texts (encoder identification)

/// The decoder settings (mirrors the wasm `DecodeOptions`).
function decodeOpts() {
  return {
    strict: $("decode-strict").checked,
    reduce: reduce,
    layers: Number($("decode-layers").value) || 0,
    x0: Number($("area-x0").value) || 0,
    y0: Number($("area-y0").value) || 0,
    x1: Number($("area-x1").value) || 0,
    y1: Number($("area-y1").value) || 0,
    ignore_pclr_cmap_cdef: $("decode-ignore-pclr").checked,
  };
}

/// The encoder settings (mirrors the wasm `EncodeOptions`).
function encodeOpts() {
  return {
    lossless: $("encode-lossless").checked,
    num_resolutions: Number($("encode-res").value) || 6,
    num_layers: Number($("encode-layers").value) || 1,
    progression: Number($("encode-order").value) || 0,
    tile_w: Number($("encode-tilew").value) || 0,
    tile_h: Number($("encode-tileh").value) || 0,
    rate: Number($("encode-rate").value) || 0,
    rates: $("encode-rates").value.trim() || null,
    comment: $("encode-comment").value || null,
    jp2: $("encode-format").value === "jp2",
  };
}

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
async function probeDecode(data, opts, seq, tag) {
  active.decode += 1;
  updateBusy();
  try {
    if (workerUp) {
      const copy = new Uint8Array(data); // must be standalone to transfer
      const res = await rpc({ type: "decode", tag, opts, data: copy.buffer }, [copy.buffer]);
      if (seq !== undefined && seq !== decodeSeq) return null; // stale
      return {
        width: res.width,
        height: res.height,
        complete: res.complete,
        components: res.components,
        color_space: res.color_space,
        pixels: res.pixels,
      };
    }
    const probe = decode_probe_opt(data, fallbackDecodeOpts(opts));
    return {
      width: probe.width,
      height: probe.height,
      complete: probe.complete,
      components: probe.components,
      color_space: probe.color_space,
      pixels: new Uint8Array(probe.pixels),
    };
  } finally {
    active.decode -= 1;
    updateBusy();
  }
}

/// Builds the wasm DecodeOptions for the main-thread fallback path.
function fallbackDecodeOpts(o) {
  const opts = new DecodeOptions();
  opts.strict = o.strict;
  opts.reduce = o.reduce;
  opts.layers = o.layers;
  opts.x0 = o.x0;
  opts.y0 = o.y0;
  opts.x1 = o.x1;
  opts.y1 = o.y1;
  opts.ignore_pclr_cmap_cdef = o.ignore_pclr_cmap_cdef;
  return opts;
}

async function workerEncode(width, height, rgba, opts) {
  active.encode += 1;
  updateBusy();
  try {
    if (workerUp) {
      const copy = new Uint8Array(rgba);
      const res = await rpc(
        { type: "encode", opts, width, height, data: copy.buffer },
        [copy.buffer],
      );
      return new Uint8Array(res.bytes);
    }
    const e = new EncodeOptions();
    e.lossless = opts.lossless;
    e.num_resolutions = opts.num_resolutions;
    e.num_layers = opts.num_layers;
    e.progression = opts.progression;
    e.tile_w = opts.tile_w;
    e.tile_h = opts.tile_h;
    e.rate = opts.rate;
    e.rates = opts.rates;
    e.comment = opts.comment;
    e.jp2 = opts.jp2;
    return encode_rgba_opt(width, height, rgba, e);
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
let hexSpans = []; // .byte spans by byte offset (for in-place cut updates)
let currentCut = 0; // current truncation point in bytes

/// Styles one hex byte span for the given truncation point.
function styleHexSpan(s, i, cut) {
  s.dataset.offset = i;
  if (i >= cut) {
    s.className = "cut";
    s.title = "beyond the truncation point";
    delete s.dataset.range;
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

function renderHex(data, cut) {
  const view = $("hex-view");
  view.textContent = "";
  hexSpans = [];
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
        styleHexSpan(s, i, cut);
        hexSpans[i] = s;
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

/// Applies the zoom selection to a canvas: 0 = fit (CSS max-size
/// scaling), >=1 = natural/magnified size with CSS px, scrolling the box.
/// Both dimensions are set explicitly so the aspect ratio never depends
/// on the CSS max-* clamps.
function applyZoom(canvas, box, zoom) {
  if (!box) return;
  if (zoom > 0) {
    canvas.style.width = `${canvas.width * zoom}px`;
    canvas.style.height = `${canvas.height * zoom}px`;
    box.classList.add("zoomed");
  } else {
    canvas.style.width = "";
    canvas.style.height = "";
    box.classList.remove("zoomed");
  }
}

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
    const probe = await probeDecode(sub, decodeOpts(), seq, "explorer");
    if (!probe || myLoad !== loadSeq) return; // superseded / new file
    const ms = performance.now() - t0;
    drawProbe(probe.width, probe.height, probe.pixels, explorerCanvas);
    applyZoom(explorerCanvas, $("explorer-preview"), Number($("explorer-zoom").value) || 0);
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

/// Applies a new truncation point: syncs the slider/byte field/label,
/// updates the hex cut highlight and tree dimming in place (no re-render,
/// so nothing jumps), and debounces the re-decode.
function setCut(cut, decode = true) {
  if (!codestream) return;
  cut = Math.max(0, Math.min(cut, codestream.length));
  const prev = currentCut;
  currentCut = cut;
  const pct = codestream.length ? (cut / codestream.length) * 100 : 100;
  $("trunc-slider").value = pct;
  $("trunc-bytes").value = cut;
  $("trunc-label").textContent = `${pct.toFixed(2)}%`;
  if (prev !== cut) updateCutSpans(prev, cut);
  updateTreeCut(cut);
  if (decode) {
    clearTimeout(truncTimer);
    truncTimer = setTimeout(() => explorerDecode(cut), 120);
  }
}

/// Restyles only the hex bytes between the old and the new cut point.
function updateCutSpans(prev, cut) {
  const n = Math.min(codestream.length, HEX_CAP);
  const lo = Math.max(0, Math.min(prev, cut));
  const hi = Math.min(n, Math.max(prev, cut));
  for (let i = lo; i < hi; i++) {
    const s = hexSpans[i];
    if (s) styleHexSpan(s, i, cut);
  }
}

/// Dims the tree nodes beyond the cut point without rebuilding the tree
/// (keeps the user's expand/collapse state).
function updateTreeCut(cut) {
  for (const node of document.querySelectorAll("#tree-view .tree-node[data-start]")) {
    node.classList.toggle("beyond-cut", Number(node.dataset.start) >= cut);
  }
}

$("trunc-slider").addEventListener("input", (ev) => {
  if (!codestream) return;
  // the slider is in percent with 0.01% steps
  const pct = Number(ev.target.value);
  setCut(Math.round((pct / 100) * codestream.length));
});

$("trunc-bytes").addEventListener("input", (ev) => {
  if (!codestream) return;
  const v = Number(ev.target.value);
  if (Number.isFinite(v)) setCut(v);
});

for (const b of document.querySelectorAll("#trunc-steps button")) {
  b.addEventListener("click", () => {
    if (!codestream) return;
    setCut(currentCut + Number(b.dataset.delta));
  });
}

// ---------------------------------------------------------------------------
// quality-layer byte thresholds

/// Fetches the per-layer prefix lengths (worker when available) and
/// renders the table; each row links the truncation point to that size.
async function computeLayerSizes(data) {
  const el = $("layer-sizes");
  el.textContent = "computing layer sizes…";
  let thresholds;
  try {
    if (workerUp) {
      const copy = new Uint8Array(data);
      const res = await rpc({ type: "layerbytes", data: copy.buffer }, [copy.buffer]);
      thresholds = res.thresholds;
    } else {
      await init();
      thresholds = layer_byte_thresholds(data, false);
    }
  } catch (e) {
    el.textContent = `layer sizes unavailable: ${e}`;
    return;
  }
  if (!thresholds || !thresholds.length) {
    el.textContent = "";
    return;
  }
  const rows = thresholds.map((bytes, i) => {
    const pct = ((bytes / data.length) * 100).toFixed(1);
    const tr = document.createElement("tr");
    const l = document.createElement("td");
    l.textContent = `${i + 1}`;
    const b = document.createElement("td");
    b.textContent = fmtBytes(bytes);
    const p = document.createElement("td");
    p.textContent = `${pct}%`;
    tr.append(l, b, p);
    tr.title = `click: truncate the file to ${bytes} bytes (fully decodes ${i + 1} layer${i === 0 ? "" : "s"})`;
    tr.addEventListener("click", () => setCut(Math.min(bytes, codestream.length)));
    return tr;
  });
  const table = document.createElement("table");
  table.append(
    (() => {
      const head = document.createElement("tr");
      for (const h of ["layer", "needs ≥", "of file"]) {
        const th = document.createElement("th");
        th.textContent = h;
        head.appendChild(th);
      }
      return head;
    })(),
  );
  for (const r of rows) table.appendChild(r);
  el.replaceChildren(
    (() => {
      const cap = document.createElement("div");
      cap.className = "hint";
      cap.textContent = "To fully decode N quality layers, a truncated file needs at least:";
      return cap;
    })(),
    table,
  );
}

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
      setCut(size);
    });
    el.appendChild(b);
  }
}

// zoom selectors re-apply the current zoom to the last drawn canvas
$("decode-zoom").addEventListener("change", () =>
  applyZoom(decodeCanvas, $("decode-drop"), Number($("decode-zoom").value) || 0),
);
$("explorer-zoom").addEventListener("change", () =>
  applyZoom(explorerCanvas, $("explorer-preview"), Number($("explorer-zoom").value) || 0),
);

// ---------------------------------------------------------------------------
// decode panel

const decodeCanvas = $("decode-canvas");

async function showDecode(data) {
  const t0 = performance.now();
  const myLoad = loadSeq;
  // No truncation-seq guard: the panel always shows the full-file
  // decode; only a newer file load supersedes it.
  const probe = await probeDecode(data, decodeOpts(), undefined, "panel");
  if (myLoad !== loadSeq) return;
  const ms = performance.now() - t0;
  drawProbe(probe.width, probe.height, probe.pixels, decodeCanvas);
  applyZoom(decodeCanvas, $("decode-drop"), Number($("decode-zoom").value) || 0);
  const complete = structurallyComplete(data, csOffset);
  renderImageInfo(probe, complete, ms);
  $("decode-drop").classList.add("has-image");
}

const ORDER_NAMES = ["LRCP", "RLCP", "RPCL", "PCRL", "CPRL"];

/// Renders the file metadata + decode status in the Decode panel.
function renderImageInfo(probe, complete, ms) {
  const rows = [];
  const h = headerInfo;
  rows.push(["size", `${probe.width}×${probe.height}`]);
  const comps = probe.components || h?.components;
  if (comps) {
    const prec = h ? `${h.prec}-bit ${h.sgnd ? "signed" : "unsigned"}` : "";
    rows.push(["components", prec ? `${comps} (${prec})` : `${comps}`]);
  }
  const cs = probe.color_space || h?.color_space;
  if (cs) rows.push(["colour space", cs + (h?.has_icc_profile ? " (ICC profile)" : "")]);
  if (h) {
    rows.push(["resolutions", `${h.numresolutions}`]);
    const active = decodeOpts().layers;
    rows.push([
      "layers",
      (h.num_layers <= 1
        ? "1 (single-layer codestream; the setting has no effect)"
        : active > 0
          ? `decoding first ${active} of ${h.num_layers} quality layers`
          : `${h.num_layers} (all) quality layers`),
    ]);
    rows.push(["progression", ORDER_NAMES[h.progression] || "?"]);
    rows.push([
      "tiles",
      `${h.tiles_h}×${h.tiles_v} (${h.tile_w}×${h.tile_h} each)`,
    ]);
    rows.push(["code-blocks", `${2 ** h.codeblock_w}×${2 ** h.codeblock_h}`]);
    if (h.dx !== 1 || h.dy !== 1) rows.push(["subsampling", `${h.dx}×${h.dy}`]);
    if (h.mct) rows.push(["MCT", `${h.mct}`]);
  }
  if (comments.length) {
    rows.push(["encoded by", comments[0]]);
  } else {
    rows.push(["encoded by", "unknown (no COM marker)"]);
  }
  rows.push(["pixels", complete ? "complete" : "truncated (undecoded areas are black)"]);
  rows.push(["decode", `${ms.toFixed(0)} ms`]);
  setInfo($("decode-info"), rows);
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
    comments = structure.comments || [];
    codestream = data;
    loadSeq += 1;
    fetchInfo(data);
    showDecode(data).catch((e) => setInfo($("decode-info"), [["error", String(e)]]));
    // explorer
    currentCut = data.length;
    $("trunc-slider").value = 100;
    $("trunc-bytes").max = data.length;
    $("trunc-bytes").value = data.length;
    $("trunc-label").textContent = "100.00%";
    renderHex(data, data.length);
    renderTree(treeRoot, data.length);
    renderLegend();
    renderPresets();
    explorerDecode(data.length);
    computeLayerSizes(data);
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
let encodeLoaded = false;

function loadSourceImage(file) {
  const url = URL.createObjectURL(file);
  const img = new Image();
  img.onload = () => {
    encodeCanvas.width = img.naturalWidth;
    encodeCanvas.height = img.naturalHeight;
    encodeCanvas.getContext("2d").drawImage(img, 0, 0);
    URL.revokeObjectURL(url);
    $("encode-drop").classList.add("has-image");
    encodeLoaded = true;
    setInfo($("encode-info"), [["size", `${img.naturalWidth}×${img.naturalHeight}`]]);
    runEncode();
  };
  img.src = url;
}

const TILE_TEXT = (w, h) => (w > 0 && h > 0 ? `${w}×${h} tile` : "single tile");

/// The quality-layer ladder the encoder will use for the current inputs.
/// Mirrors the encoder's precedence: rates > rate > automatic ladder.
function autoLadder(n) {
  n = Math.min(Math.max(1, n), 100);
  return Array.from({ length: n }, (_, i) =>
    i === n - 1 ? 0 : 100 ** (1 - i / (n - 1)),
  );
}

function fmtRatio(r) {
  return r === 0 ? "lossless" : `${+r.toFixed(2)}:1`;
}

function effectiveEncodeConfig() {
  const layersIn = Math.max(1, Number($("encode-layers").value) || 1);
  const rate = Number($("encode-rate").value) || 0;
  const ratesRaw = $("encode-rates").value.trim();
  const parsed = ratesRaw ? ratesRaw.split(",").map((s) => s.trim()).filter(Boolean) : [];
  const explicit = parsed.map(Number).filter((n) => Number.isFinite(n) && n >= 0);
  const dropped = parsed.length - explicit.length;
  const warnings = [];
  let ladder;
  if (explicit.length) {
    ladder = explicit.slice(0, 100);
    if (layersIn !== explicit.length) {
      warnings.push(
        `"layers" is ignored: the rates list defines ${explicit.length} layer${explicit.length === 1 ? "" : "s"}.`,
      );
    }
    if (explicit.length > 100) {
      warnings.push("Only the first 100 rates are used (the format's layer limit).");
    }
    if (dropped) {
      warnings.push(`${dropped} invalid rate entr${dropped === 1 ? "y" : "ies"} ignored.`);
    }
  } else if (rate > 0) {
    ladder = [rate];
    if (layersIn !== 1) {
      warnings.push('"layers" is ignored: a single ratio layer is used.');
    }
    if (ratesRaw) {
      warnings.push('"rates" has no valid entries; the single ratio is used.');
    }
  } else if (layersIn > 1) {
    ladder = autoLadder(layersIn);
    if (layersIn > 100) {
      warnings.push("Layers are capped at 100 (the format's limit).");
    }
  } else {
    ladder = [0];
  }
  return {
    ladder,
    warnings,
    lossless: $("encode-lossless").checked,
    resolutions: Number($("encode-res").value) || 6,
    order: Number($("encode-order").value) || 0,
    tileW: Number($("encode-tilew").value) || 0,
    tileH: Number($("encode-tileh").value) || 0,
  };
}

/// Shows the effective settings and warnings, plus (once encoded) what
/// the produced file's header actually declares.
function renderEffective(file) {
  const el = $("encode-effective");
  const cfg = effectiveEncodeConfig();
  const layerWord = cfg.ladder.length === 1 ? "layer" : "layers";
  const rows = [];
  rows.push(
    `<div><b>Effective:</b> ${cfg.ladder.length} ${layerWord} — ` +
      `${cfg.ladder.map(fmtRatio).join(", ")} · ${cfg.resolutions} resolutions · ` +
      `${ORDER_NAMES[cfg.order]} · ${TILE_TEXT(cfg.tileW, cfg.tileH)} · ` +
      `${cfg.lossless ? "lossless 5-3" : "lossy 9-7"}</div>`,
  );
  if (file) {
    rows.push(
      `<div class="file">File: ${file.layers} ${file.layers === 1 ? "layer" : "layers"} · ` +
        `${file.resolutions} resolutions · ${ORDER_NAMES[file.progression] || "?"} · ` +
        `${file.transform === 1 ? "lossless 5-3" : "lossy 9-7"} · ` +
        `${TILE_TEXT(file.tile_w === file.width ? 0 : file.tile_w, file.tile_h === file.height ? 0 : file.tile_h)}` +
        `${file.codeblock ? ` · ${file.codeblock}² code-blocks` : ""}</div>`,
    );
  }
  for (const w of cfg.warnings) {
    rows.push(`<div class="warn">⚠ ${w}</div>`);
  }
  el.innerHTML = rows.join("");
}

async function runEncode() {
  renderEffective();
  if (!encodeLoaded) return;
  const ctx = encodeCanvas.getContext("2d");
  const { width, height } = encodeCanvas;
  const rgba = ctx.getImageData(0, 0, width, height).data;
  const t0 = performance.now();
  try {
    encoded = await workerEncode(width, height, rgba, encodeOpts());
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
  renderEffective(codestreamHeaderInfo(encoded));
}

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
    showDecode(codestream).catch((e) => setInfo($("decode-info"), [["error", String(e)]]));
  }
});

// The '?' help badges inside <label>s must not toggle the setting.
for (const h of document.querySelectorAll(".help")) {
  h.addEventListener("click", (e) => e.preventDefault());
}

// ---------------------------------------------------------------------------
// decode/encode settings

/// Fetches the header info (numresolutions drives the resolution buttons).
async function fetchInfo(data) {
  let info;
  try {
    if (workerUp) {
      const copy = new Uint8Array(data);
      info = await rpc({ type: "info", data: copy.buffer }, [copy.buffer]);
    } else {
      await init();
      info = decode_info(data);
    }
  } catch {
    return;
  }
  headerInfo = info;
  numResolutions = Math.max(1, info.numresolutions || 1);
  reduce = Math.min(reduce, numResolutions - 1);
  renderResolutions();
}

/// Resolution buttons: decode the codestream at each of its resolutions
/// (reduce 0..numresolutions-1), like the truncation presets.
function renderResolutions() {
  const el = $("decode-resolutions");
  el.replaceChildren();
  if (numResolutions <= 1) {
    const span = document.createElement("span");
    span.className = "muted";
    span.textContent = "single resolution";
    el.appendChild(span);
    return;
  }
  for (let r = 0; r < numResolutions; r++) {
    const b = document.createElement("button");
    b.type = "button";
    b.textContent = r === 0 ? "full" : `1/${2 ** r}`;
    b.classList.toggle("active", r === reduce);
    b.title =
      r === 0
        ? "decode at the full resolution"
        : `decode at 1/${2 ** r} resolution (the codestream's ${r + 1}. resolution)`;
    b.addEventListener("click", () => {
      reduce = r;
      renderResolutions();
      if (codestream) {
        showDecode(codestream).catch((e) =>
          setInfo($("decode-info"), [["error", String(e)]]),
        );
        explorerDecode(currentCut);
      }
    });
    el.appendChild(b);
  }
}

let decodeSettingTimer = null;
function decodeSettingsChanged() {
  if (!codestream) return;
  clearTimeout(decodeSettingTimer);
  decodeSettingTimer = setTimeout(() => {
    showDecode(codestream).catch((e) => setInfo($("decode-info"), [["error", String(e)]]));
    explorerDecode(currentCut);
  }, 200);
}

for (const id of ["decode-layers", "area-x0", "area-y0", "area-x1", "area-y1"]) {
  $(id).addEventListener("input", decodeSettingsChanged);
}
$("decode-ignore-pclr").addEventListener("change", () => {
  if (codestream) {
    showDecode(codestream).catch((e) => setInfo($("decode-info"), [["error", String(e)]]));
    explorerDecode(currentCut);
  }
});

let encodeSettingTimer = null;
function encodeSettingsChanged() {
  clearTimeout(encodeSettingTimer);
  encodeSettingTimer = setTimeout(runEncode, 200);
}
for (const id of [
  "encode-lossless",
  "encode-format",
  "encode-order",
  "encode-res",
  "encode-layers",
  "encode-tilew",
  "encode-tileh",
  "encode-rate",
  "encode-rates",
  "encode-comment",
]) {
  const el = $(id);
  el.addEventListener(el.tagName === "INPUT" && el.type === "text" ? "input" : "change", encodeSettingsChanged);
  if (el.tagName === "INPUT" && el.type === "number") {
    el.addEventListener("input", encodeSettingsChanged);
  }
}

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
