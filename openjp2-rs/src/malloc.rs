/*
 * The copyright in this software is being made available under the 2-clauses
 * BSD License, included below. This software may be subject to other third
 * party and contributor rights, including patent rights, and no such rights
 * are granted under this license.
 *
 * Copyright (c) 2005, Herve Drolon, FreeImage Team
 * All rights reserved.
 */

//! Minimal C-allocator helpers.
//!
//! These two survive for the `#[repr(C)]` encoder parameter buffers
//! (`opj_cparameters_t::cp_matrice`/`mct_data`) that C callers allocate and
//! free with the same malloc family. Everything else in the codec owns its
//! buffers through Vec/Box.

