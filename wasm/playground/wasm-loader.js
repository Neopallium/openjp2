// Loads the appropriate openjp2-wasm build:
// - the threaded build (pkg-mt/, SharedArrayBuffer + rayon) when the page
//   is cross-origin isolated and the user enabled threads,
// - otherwise the single-threaded build (pkg/).
//
// `init_threads` is the requested rayon pool size (1 = off).  Returns the
// pool actually in use.

let api = null;
let loaded = null;

export async function loadWasm(initThreads = 1) {
  if (loaded) return loaded;
  const wantMT =
    initThreads > 1 &&
    globalThis.crossOriginIsolated &&
    typeof SharedArrayBuffer === "function";
  if (wantMT) {
    try {
      const mod = await import("./pkg-mt/openjp2_wasm.js");
      await mod.default();
      if (mod.has_threads && mod.has_threads()) {
        await mod.init_thread_pool(initThreads);
        api = mod;
        loaded = { mt: true, threads: mod.thread_count ? mod.thread_count() : initThreads };
        return loaded;
      }
    } catch (e) {
      // no threaded build installed / failed to load: fall back below
      console.warn("threaded wasm unavailable, using the single-thread build:", e);
    }
  }
  const mod = await import("./pkg/openjp2_wasm.js");
  await mod.default();
  api = mod;
  loaded = { mt: false, threads: 1 };
  return loaded;
}

/// The loaded wasm API (throws if loadWasm has not completed).
export function wasm() {
  if (!api) throw new Error("wasm module is not loaded yet");
  return api;
}

/// Whether the browser can run the threaded build at all.
export function threadsSupported() {
  return (
    typeof SharedArrayBuffer === "function" &&
    globalThis.crossOriginIsolated === true
  );
}
