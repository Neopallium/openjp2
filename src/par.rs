//! Parallel helpers: rayon when the `rayon` feature is enabled (some
//! platforms/targets cannot use rayon); everything falls back to the
//! sequential iterators otherwise, with identical results.
//!
/// The current rayon pool's worker count (1 without the `rayon` feature).
#[cfg(feature = "rayon")]
pub(crate) fn current_num_threads() -> usize {
  rayon::current_num_threads()
}

/// The current rayon pool's worker count (1 without the `rayon` feature).
#[cfg(not(feature = "rayon"))]
pub(crate) fn current_num_threads() -> usize {
  1
}

/// The calling rayon worker's index, or `None` outside a rayon job (always
/// `None` without the `rayon` feature).
#[cfg(feature = "rayon")]
pub(crate) fn current_thread_index() -> Option<usize> {
  rayon::current_thread_index()
}

/// The calling rayon worker's index, or `None` outside a rayon job (always
/// `None` without the `rayon` feature).
#[cfg(not(feature = "rayon"))]
pub(crate) fn current_thread_index() -> Option<usize> {
  None
}

#[cfg(feature = "rayon")]
thread_local! {
  // The pool that with_current_pool installs the parallel helpers on, set
  // per-thread by set_current_pool (the per-decode thread limit,
  // DecodeParameters::threads).  None = the ambient pool (the default).
  static CURRENT_POOL: std::cell::RefCell<Option<std::rc::Rc<rayon::ThreadPool>>> =
    const { std::cell::RefCell::new(None) };
}

/// Runs `f` on the per-thread current pool when one is set (the ambient
/// rayon pool otherwise).  Every parallel helper funnels through here so a
/// decode wrapped by `set_current_pool` runs ALL its parallelism (the tile
/// driver and the nested codeblock work) on the caller's chosen pool.
#[cfg(feature = "rayon")]
pub(crate) fn with_current_pool<R: Send>(f: impl FnOnce() -> R + Send) -> R {
  CURRENT_POOL.with(|p| match p.borrow().as_ref() {
    Some(pool) => pool.install(f),
    None => f(),
  })
}

/// Installs `pool` as the per-thread pool for the parallel helpers until
/// the returned guard is dropped (restoring the previous setting).
#[cfg(feature = "rayon")]
pub(crate) fn set_current_pool(pool: &std::rc::Rc<rayon::ThreadPool>) -> CurrentPoolGuard {
  let prev = CURRENT_POOL.with(|p| p.borrow_mut().replace(pool.clone()));
  CurrentPoolGuard(prev)
}

/// RAII restore for `set_current_pool` (also runs when a decode panics).
#[cfg(feature = "rayon")]
pub(crate) struct CurrentPoolGuard(Option<std::rc::Rc<rayon::ThreadPool>>);

#[cfg(feature = "rayon")]
impl Drop for CurrentPoolGuard {
  fn drop(&mut self) {
    CURRENT_POOL.with(|p| *p.borrow_mut() = self.0.take());
  }
}

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
    with_current_pool(|| {
      items
        .par_iter_mut()
        .enumerate()
        .map(|(i, x)| f(i, x))
        .collect()
    })
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
  with_current_pool(|| {
    items.par_iter_mut().enumerate().for_each(|(i, x)| f(i, x));
  });
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
  with_current_pool(|| {
    a.par_iter_mut()
      .enumerate()
      .take(n)
      .zip(b.par_iter_mut())
      .map(|((i, x), y)| f(i, x, y))
      .collect()
  })
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
  with_current_pool(|| {
    (0..n).into_par_iter().for_each(f);
  });
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
  with_current_pool(|| {
    data
      .par_chunks_mut(chunk)
      .enumerate()
      .for_each(|(i, s)| f(i, s));
  });
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
/// `DecodeParameters::threads` and the CLI tools' env-based setup.
#[cfg(feature = "rayon")]
pub(crate) fn configure_pool(n: usize) {
  if n > 0 {
    let _ = rayon::ThreadPoolBuilder::new()
      .num_threads(n)
      .build_global();
  }
}

/// No-op pool sizing without the `rayon` feature (the sequential
/// iterators are used).
#[cfg(not(feature = "rayon"))]
pub(crate) fn configure_pool(_n: usize) {}
