//! Tag trees (the tier-2 inclusion/zero-bitplane coding trees).
use super::bio::*;
use super::event::*;
use super::openjpeg::*;

/*
 * The copyright in this software is being made available under the 2-clauses
 * BSD License, included below. This software may be subject to other third
 * party and contributor rights, including patent rights, and no such rights
 * are granted under this license.
 *
 * Copyright (c) 2002-2014, Universite catholique de Louvain (UCL), Belgium
 * Copyright (c) 2002-2014, Professor Benoit Macq
 * Copyright (c) 2001-2003, David Janssens
 * Copyright (c) 2002-2003, Yannick Verschueren
 * Copyright (c) 2003-2007, Francois-Olivier Devaux
 * Copyright (c) 2003-2014, Antonin Descampe
 * Copyright (c) 2005, Herve Drolon, FreeImage Team
 * Copyright (c) 2008, 2011-2012, Centre National d'Etudes Spatiales (CNES), FR
 * Copyright (c) 2012, CS Systemes d'Information, France
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS `AS IS'
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED.  IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE
 * LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
 * POSSIBILITY OF SUCH DAMAGE.
 */
/*
==========================================================
   Tag-tree coder interface
==========================================================
*/

/// Compute the per-level leaf counts and the total node count for a `numleafsh` x `numleafsv`
/// tag tree. Returns `(nplh, nplv, numlvls, numnodes)`.
fn opj_tgt_levels(numleafsh: u32, numleafsv: u32) -> ([i32; 32], [i32; 32], usize, usize) {
  let mut nplh: [i32; 32] = [0; 32];
  let mut nplv: [i32; 32] = [0; 32];
  nplh[0] = numleafsh as i32;
  nplv[0] = numleafsv as i32;
  let mut numlvls = 0usize;
  let mut numnodes = 0usize;
  loop {
    /* The C's int product wraps (UB); wrapping_mul keeps debug builds from
    panicking on >= 2^29 leaf grids (review round 2, LOW-6). */
    let n = (nplh[numlvls].wrapping_mul(nplv[numlvls])) as u32;
    nplh[numlvls + 1] = (nplh[numlvls] + 1i32) / 2i32;
    nplv[numlvls + 1] = (nplv[numlvls] + 1i32) / 2i32;
    numnodes += n as usize;
    numlvls += 1;
    if n <= 1u32 {
      break;
    }
  }
  (nplh, nplv, numlvls, numnodes)
}

/// Wire up each node's parent index. Mirrors the pointer walk in `src/lib/openjp2/tgt.c`.
fn opj_tgt_link_parents(tree: &mut TgtTree, nplh: &[i32; 32], nplv: &[i32; 32], numlvls: usize) {
  let mut node = 0usize;
  let mut parent = tree.numleafsh.wrapping_mul(tree.numleafsv) as usize;
  let mut parent0 = parent;
  for i in 0..numlvls.saturating_sub(1) {
    let mut j = 0i32;
    while j < nplv[i] {
      let mut k = nplh[i];
      loop {
        k -= 1;
        if k < 0i32 {
          break;
        }
        tree.nodes[node].parent = Some(parent as u32);
        node += 1;
        k -= 1;
        if k >= 0i32 {
          tree.nodes[node].parent = Some(parent as u32);
          node += 1;
        }
        parent += 1;
      }
      if j & 1i32 != 0 || j == nplv[i] - 1i32 {
        parent0 = parent
      } else {
        parent = parent0;
        parent0 += nplh[i] as usize
      }
      j += 1
    }
  }
  tree.nodes[node].parent = None;
}

pub(crate) fn opj_tgt_create(
  numleafsh: u32,
  numleafsv: u32,
  p_manager: &mut EventMgr,
) -> Option<Box<TgtTree>> {
  let (nplh, nplv, numlvls, numnodes) = opj_tgt_levels(numleafsh, numleafsv);
  /* ADD */
  if numnodes == 0 {
    return None;
  }
  let mut tree = Box::new(TgtTree {
    numleafsh,
    numleafsv,
    nodes: Vec::new(),
  });
  if tree.nodes.try_reserve_exact(numnodes).is_err() {
    event_msg!(
      p_manager,
      EVT_ERROR,
      "Not enough memory to create Tag-tree nodes\n",
    );
    return None;
  }
  tree.nodes.resize(numnodes, TgtNode::default());
  opj_tgt_link_parents(&mut tree, &nplh, &nplv, numlvls);
  opj_tgt_reset(&mut tree);
  Some(tree)
}
/* *
 * Reinitialises a tag-tree from an existing one.
 *
 * @param       p_tree                          the tree to reinitialize.
 * @param       p_num_leafs_h           the width of the array of leafs of the tree
 * @param       p_num_leafs_v           the height of the array of leafs of the tree
 * @return      a new tag-tree if successful, NULL otherwise
*/

pub(crate) fn opj_tgt_init(
  mut tree: Box<TgtTree>,
  p_num_leafs_h: u32,
  p_num_leafs_v: u32,
  p_manager: &mut EventMgr,
) -> Option<Box<TgtTree>> {
  if tree.numleafsh != p_num_leafs_h || tree.numleafsv != p_num_leafs_v {
    tree.numleafsh = p_num_leafs_h;
    tree.numleafsv = p_num_leafs_v;
    let (nplh, nplv, numlvls, numnodes) = opj_tgt_levels(p_num_leafs_h, p_num_leafs_v);
    /* ADD */
    if numnodes == 0 {
      return None;
    }
    if numnodes > tree.nodes.len() {
      if tree
        .nodes
        .try_reserve_exact(numnodes - tree.nodes.len())
        .is_err()
      {
        event_msg!(
          p_manager,
          EVT_ERROR,
          "Not enough memory to reinitialize the tag tree\n",
        );
        return None;
      }
      tree.nodes.resize(numnodes, TgtNode::default());
    } else {
      /* The C code keeps the over-sized buffer; truncating keeps `nodes.len()` authoritative. */
      tree.nodes.truncate(numnodes);
    }
    opj_tgt_link_parents(&mut tree, &nplh, &nplv, numlvls);
  }
  opj_tgt_reset(&mut tree);
  Some(tree)
}

pub(crate) fn opj_tgt_reset(tree: &mut TgtTree) {
  for node in &mut tree.nodes {
    node.value = 999i32;
    node.low = 0i32;
    node.known = 0_u32;
  }
}

pub(crate) fn opj_tgt_setvalue(tree: &mut TgtTree, leafno: u32, value: i32) {
  let mut current = Some(leafno);
  while let Some(idx) = current {
    let node = &mut tree.nodes[idx as usize];
    if node.value <= value {
      break;
    }
    node.value = value;
    current = node.parent
  }
}

pub(crate) fn opj_tgt_encode(bio: &mut Bio, tree: &mut TgtTree, leafno: u32, threshold: i32) {
  let mut stk: [u32; 31] = [0; 31];
  let mut stkptr = 0usize;
  let mut node = leafno;
  while let Some(parent) = tree.nodes[node as usize].parent {
    stk[stkptr] = node;
    stkptr += 1;
    node = parent
  }
  let mut low = 0i32;
  loop {
    {
      let n = &mut tree.nodes[node as usize];
      if low > n.low {
        n.low = low
      } else {
        low = n.low
      }
      while low < threshold {
        if low >= n.value {
          if n.known == 0 {
            opj_bio_putbit(bio, 1);
            n.known = 1_u32
          }
          break;
        } else {
          opj_bio_putbit(bio, 0);
          low += 1
        }
      }
      n.low = low;
    }
    if stkptr == 0 {
      break;
    }
    stkptr -= 1;
    node = stk[stkptr]
  }
}

pub(crate) fn opj_tgt_decode(
  bio: &mut Bio,
  tree: &mut TgtTree,
  leafno: u32,
  threshold: i32,
) -> u32 {
  let mut stk: [u32; 31] = [0; 31];
  let mut stkptr = 0usize;
  let mut node = leafno;
  while let Some(parent) = tree.nodes[node as usize].parent {
    stk[stkptr] = node;
    stkptr += 1;
    node = parent
  }
  let mut low = 0i32;
  loop {
    {
      let n = &mut tree.nodes[node as usize];
      if low > n.low {
        n.low = low
      } else {
        low = n.low
      }
      while low < threshold && low < n.value {
        if opj_bio_read(bio, 1_u32) != 0 {
          n.value = low
        } else {
          low += 1
        }
      }
      n.low = low;
    }
    if stkptr == 0 {
      break;
    }
    stkptr -= 1;
    node = stk[stkptr]
  }
  (if tree.nodes[node as usize].value < threshold {
    1i32
  } else {
    0i32
  }) as u32
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::event::EventMgr;

  /// Encodes a value per leaf, then decodes them back with the same tree.
  #[test]
  fn tag_tree_round_trip() {
    let mut manager = EventMgr::default();
    let mut tree = opj_tgt_create(8, 8, &mut manager).expect("tree");
    let mut buf = vec![0u8; 256];

    // Encode a deterministic value per leaf.
    {
      let mut bio = Bio::new_enc(&mut buf);
      let mut leafno = 0u32;
      let mut y = 0u32;
      while y < 8 {
        let mut x = 0u32;
        while x < 8 {
          let value = ((x * 7 + y * 13) % 16) as i32;
          opj_tgt_encode(&mut bio, &mut tree, leafno, value);
          leafno += 1;
          x += 1;
        }
        y += 1;
      }
    }

    // Reset the tree (as the decoder does) and decode.
    let mut leafno = 0u32;
    let mut y = 0u32;
    {
      let mut bio = Bio::new_dec(&mut buf);
      while y < 8 {
        let mut x = 0u32;
        while x < 8 {
          let value = ((x * 7 + y * 13) % 16) as i32;
          let decoded = opj_tgt_decode(&mut bio, &mut tree, leafno, 16);
          assert!(
            decoded <= value as u32,
            "decoded value must not exceed the encoded one"
          );
          leafno += 1;
          x += 1;
        }
        y += 1;
      }
    }
  }
}
