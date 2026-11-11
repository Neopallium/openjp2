// Web Worker backend for the playground: runs the wasm decode/encode off
// the main thread so the UI stays responsive.
//
// Decode requests come in two flavours:
// - "panel" jobs (the Decode section, strict toggle): always run.
// - "explorer" jobs (the truncation slider): coalesced — only the latest
//   queued truncation point is decoded, and every superseded request id
//   receives that result (the main thread discards stale ones).

import init, {
  decode_probe,
  encode_rgba_to_jp2,
  encode_rgba_to_j2k,
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

async function runJob(job) {
  const { id, data, strict } = job;
  const probe = decode_probe(new Uint8Array(data), strict);
  return {
    id,
    ok: true,
    width: probe.width,
    height: probe.height,
    complete: probe.complete,
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
      if (msg.tag === "panel") {
        panelQueue.push({ id: msg.id, data: msg.data, strict: msg.strict });
      } else {
        explorerQueue.push({ id: msg.id, data: msg.data, strict: msg.strict });
      }
      drain();
      break;
    }
    case "encode": {
      ready()
        .then(() => {
          const bytes =
            msg.format === "j2k"
              ? encode_rgba_to_j2k(msg.width, msg.height, new Uint8Array(msg.data), msg.lossless)
              : encode_rgba_to_jp2(msg.width, msg.height, new Uint8Array(msg.data), msg.lossless);
          self.postMessage({ id: msg.id, ok: true, bytes }, [bytes.buffer]);
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
