// Web Worker backend for the playground: runs the wasm decode/encode off
// the main thread so the UI stays responsive.
//
// Decode requests come in two flavours:
// - "panel" jobs (the Decode section, settings changes): always run.
// - "explorer" jobs (the truncation slider): coalesced — only the latest
//   queued truncation point is decoded, and every superseded request id
//   receives that result (the main thread discards stale ones).

import init, {
  decode_probe_opt,
  decode_info,
  encode_rgba_opt,
  layer_byte_thresholds,
  DecodeOptions,
  EncodeOptions,
  version,
} from "./pkg/openjp2_wasm.js";

let readyPromise = null;
function ready() {
  if (!readyPromise) readyPromise = init();
  return readyPromise;
}

let panelQueue = [];
let explorerQueue = [];
let running = false;

function optionsFrom(msg) {
  const opts = new DecodeOptions();
  opts.strict = msg.opts.strict;
  opts.reduce = msg.opts.reduce;
  opts.layers = msg.opts.layers;
  opts.x0 = msg.opts.x0;
  opts.y0 = msg.opts.y0;
  opts.x1 = msg.opts.x1;
  opts.y1 = msg.opts.y1;
  opts.ignore_pclr_cmap_cdef = msg.opts.ignore_pclr_cmap_cdef;
  return opts;
}

async function runJob(job) {
  const probe = decode_probe_opt(new Uint8Array(job.data), optionsFrom(job));
  return {
    id: job.id,
    ok: true,
    width: probe.width,
    height: probe.height,
    complete: probe.complete,
    components: probe.components,
    color_space: probe.color_space,
    pixels: probe.pixels,
  };
}

async function drain() {
  if (running) return;
  running = true;
  try {
    while (panelQueue.length || explorerQueue.length) {
      let job;
      if (panelQueue.length) {
        // panel jobs always run, in order
        job = panelQueue.shift();
      } else {
        // coalesce the explorer backlog into the latest request
        const jobs = explorerQueue;
        explorerQueue = [];
        job = jobs[jobs.length - 1];
        // reply the dropped ids with the same result afterwards
        job.dropIds = jobs.slice(0, -1).map((j) => j.id);
      }
      try {
        await ready();
        const result = await runJob(job);
        // dropped ids get a plain (cloned) copy first; the transferable
        // buffer is handed over only once, with the main reply
        for (const id of job.dropIds || []) {
          self.postMessage({ ...result, id });
        }
        self.postMessage(result, [result.pixels.buffer]);
      } catch (e) {
        self.postMessage({ id: job.id, ok: false, error: String(e) });
        for (const id of job.dropIds || []) {
          self.postMessage({ id, ok: false, error: String(e) });
        }
      }
    }
  } finally {
    running = false;
  }
}

self.onmessage = (ev) => {
  const msg = ev.data;
  switch (msg.type) {
    case "decode": {
      const job = { id: msg.id, data: msg.data, opts: msg.opts };
      if (msg.tag === "panel") {
        panelQueue.push(job);
      } else {
        explorerQueue.push(job);
      }
      drain();
      break;
    }
    case "info": {
      ready()
        .then(() => {
          const info = decode_info(new Uint8Array(msg.data));
          self.postMessage({
            id: msg.id,
            ok: true,
            width: info.width,
            height: info.height,
            components: info.components,
            prec: info.prec,
            sgnd: info.sgnd,
            numresolutions: info.numresolutions,
            color_space: info.color_space,
            has_icc_profile: info.has_icc_profile,
            num_layers: info.num_layers,
            progression: info.progression,
            mct: info.mct,
            tiles_h: info.tiles_h,
            tiles_v: info.tiles_v,
            tile_w: info.tile_w,
            tile_h: info.tile_h,
            codeblock_w: info.codeblock_w,
            codeblock_h: info.codeblock_h,
            dx: info.dx,
            dy: info.dy,
          });
        })
        .catch((e) => self.postMessage({ id: msg.id, ok: false, error: String(e) }));
      break;
    }
    case "encode": {
      ready()
        .then(() => {
          const opts = new EncodeOptions();
          opts.lossless = msg.opts.lossless;
          opts.num_resolutions = msg.opts.num_resolutions;
          opts.num_layers = msg.opts.num_layers;
          opts.progression = msg.opts.progression;
          opts.tile_w = msg.opts.tile_w;
          opts.tile_h = msg.opts.tile_h;
          opts.rate = msg.opts.rate;
          opts.rates = msg.opts.rates || null;
          opts.comment = msg.opts.comment || null;
          opts.jp2 = msg.opts.jp2;
          const bytes = encode_rgba_opt(
            msg.width,
            msg.height,
            new Uint8Array(msg.data),
            opts,
          );
          self.postMessage({ id: msg.id, ok: true, bytes }, [bytes.buffer]);
        })
        .catch((e) => self.postMessage({ id: msg.id, ok: false, error: String(e) }));
      break;
    }
    case "layerbytes": {
      ready()
        .then(() => {
          const thresholds = layer_byte_thresholds(
            new Uint8Array(msg.data),
            !!msg.strict,
          );
          self.postMessage({ id: msg.id, ok: true, thresholds });
        })
        .catch((e) => self.postMessage({ id: msg.id, ok: false, error: String(e) }));
      break;
    }
    case "version": {
      ready()
        .then(() => self.postMessage({ id: msg.id, ok: true, version: version() }))
        .catch((e) => self.postMessage({ id: msg.id, ok: false, error: String(e) }));
      break;
    }
  }
};
