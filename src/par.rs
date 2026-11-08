//! Parallel helpers: rayon when the `rayon` feature is enabled (some
//! platforms/targets cannot use rayon); everything falls back to the
//! sequential iterators otherwise, with identical results.

/// Applies `f(index, element)` to every element of `items`, collecting the
/// results. Parallel under the `rayon` feature when `items.len() >= threshold`.
#[cfg(feature = "rayon")]
pub(crate) fn map_collect<T, R, F>(items: &mut [T], threshold: usize, f: F) -> Vec<R>
where
  T: Send,
  R: Send,
  F: Fn(usize, &mut T) -> R + Sync,
{
  use rayon::prelude::*;
  if items.len() >= threshold {
    items
      .par_iter_mut()
      .enumerate()
      .map(|(i, x)| f(i, x))
      .collect()
  } else {
    items.iter_mut().enumerate().map(|(i, x)| f(i, x)).collect()
  }
}

#[cfg(not(feature = "rayon"))]
pub(crate) fn map_collect<T, R, F>(items: &mut [T], _threshold: usize, f: F) -> Vec<R>
where
  F: Fn(usize, &mut T) -> R,
{
  items.iter_mut().enumerate().map(|(i, x)| f(i, x)).collect()
}

/// Applies `f(index, element)` to every element of `items` (no return value).
#[cfg(feature = "rayon")]
pub(crate) fn for_each<T, F>(items: &mut [T], f: F)
where
  T: Send,
  F: Fn(usize, &mut T) + Sync,
{
  use rayon::prelude::*;
  items.par_iter_mut().enumerate().for_each(|(i, x)| f(i, x));
}

#[cfg(not(feature = "rayon"))]
pub(crate) fn for_each<T, F>(items: &mut [T], f: F)
where
  F: Fn(usize, &mut T),
{
  items.iter_mut().enumerate().for_each(|(i, x)| f(i, x));
}

/// Zips the first `n` elements of `a` with `b` and applies
/// `f(index, a_element, b_element)`, collecting the results.
#[cfg(feature = "rayon")]
pub(crate) fn zip_map_collect<T, U, R, F>(a: &mut [T], b: &mut [U], n: usize, f: F) -> Vec<R>
where
  T: Send,
  U: Send,
  R: Send,
  F: Fn(usize, &mut T, &mut U) -> R + Sync,
{
  use rayon::prelude::*;
  a.par_iter_mut()
    .enumerate()
    .take(n)
    .zip(b.par_iter_mut())
    .map(|((i, x), y)| f(i, x, y))
    .collect()
}

#[cfg(not(feature = "rayon"))]
pub(crate) fn zip_map_collect<T, U, R, F>(a: &mut [T], b: &mut [U], n: usize, f: F) -> Vec<R>
where
  F: Fn(usize, &mut T, &mut U) -> R,
{
  a.iter_mut()
    .enumerate()
    .take(n)
    .zip(b.iter_mut())
    .map(|((i, x), y)| f(i, x, y))
    .collect()
}

/// Applies `f(index)` for `i` in `0..n`, in parallel under the `rayon`
/// feature.
#[cfg(feature = "rayon")]
pub(crate) fn for_each_range<F>(n: usize, f: F)
where
  F: Fn(usize) + Sync + Send,
{
  use rayon::prelude::*;
  (0..n).into_par_iter().for_each(f);
}

#[cfg(not(feature = "rayon"))]
pub(crate) fn for_each_range<F>(n: usize, f: F)
where
  F: Fn(usize),
{
  for i in 0..n {
    f(i);
  }
}

/// Applies `f(chunk_index, chunk)` to each disjoint `chunk`-sized slice of
/// `data`, in parallel under the `rayon` feature.
#[cfg(feature = "rayon")]
pub(crate) fn for_each_chunks_mut<T, F>(data: &mut [T], chunk: usize, f: F)
where
  T: Send,
  F: Fn(usize, &mut [T]) + Sync,
{
  use rayon::prelude::*;
  data.par_chunks_mut(chunk).enumerate().for_each(|(i, s)| f(i, s));
}

#[cfg(not(feature = "rayon"))]
pub(crate) fn for_each_chunks_mut<T, F>(data: &mut [T], chunk: usize, f: F)
where
  F: Fn(usize, &mut [T]),
{
  for (i, s) in data.chunks_mut(chunk).enumerate() {
    f(i, s);
  }
}


/// Sizes the process-wide rayon pool (the first successful call wins:
/// `build_global` only succeeds once).  Used by the C-ABI
/// `opj_codec_set_threads` and the CLI tools' env-based setup.
#[cfg(feature = "rayon")]
pub(crate) fn configure_pool(n: usize) {
  if n > 0 {
    let _ = rayon::ThreadPoolBuilder::new().num_threads(n).build_global();
  }
}


/// No-op pool sizing without the `rayon` feature (the sequential
/// iterators are used).
#[cfg(not(feature = "rayon"))]
pub(crate) fn configure_pool(_n: usize) {}
