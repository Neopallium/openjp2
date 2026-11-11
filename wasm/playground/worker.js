// Web Worker backend for the playground: runs the wasm decode/encode off
// the main thread so the UI stays responsive.  Coalesces queued decode
// requests (only the latest truncation point is decoded).

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

let decodeQueue = [];
let running = false;

async function drain() {
  if (running) return;
  running = true;
  try {
    while (decodeQueue.length) {
      // Coalesce: run only the latest queued decode, but reply to every
      // superseded request id too (the main thread discards stale ones).
      const jobs = decodeQueue;
      decodeQueue = [];
      const latest = jobs[jobs.length - 1];
      const ids = jobs.map((j) => j.id);
      const { data, strict } = latest;
      try {
        await ready();
        const probe = decode_probe(new Uint8Array(data), strict);
        for (const id of ids) {
          self.postMessage(
            {
              id,
              ok: true,
              width: probe.width,
              height: probe.height,
              complete: probe.complete,
              pixels: probe.pixels,
            },
            [probe.pixels.buffer],
          );
        }
      } catch (e) {
        for (const id of ids) {
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
      decodeQueue.push({ id: msg.id, data: msg.data, strict: msg.strict });
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
