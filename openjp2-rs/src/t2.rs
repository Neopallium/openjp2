use ::libc;
use ::c2rust_bitfields;
extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    pub type opj_thread_pool_t;
    #[no_mangle]
    fn memset(_: *mut libc::c_void, _: libc::c_int, _: libc::c_ulong)
     -> *mut libc::c_void;
    #[no_mangle]
    fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong)
     -> *mut libc::c_void;
    #[no_mangle]
    static mut stderr: *mut FILE;
    #[no_mangle]
    fn __assert_fail(__assertion: *const libc::c_char,
                     __file: *const libc::c_char, __line: libc::c_uint,
                     __function: *const libc::c_char) -> !;
    #[no_mangle]
    fn opj_malloc(size: size_t) -> *mut libc::c_void;
    #[no_mangle]
    fn opj_calloc(numOfElements: size_t, sizeOfElements: size_t)
     -> *mut libc::c_void;
    #[no_mangle]
    fn opj_realloc(m: *mut libc::c_void, s: size_t) -> *mut libc::c_void;
    #[no_mangle]
    fn opj_free(m: *mut libc::c_void);
    #[no_mangle]
    fn opj_event_msg(event_mgr: *mut opj_event_mgr_t, event_type: OPJ_INT32,
                     fmt: *const libc::c_char, _: ...) -> OPJ_BOOL;
    #[no_mangle]
    fn opj_bio_create() -> *mut opj_bio_t;
    #[no_mangle]
    fn opj_bio_destroy(bio: *mut opj_bio_t);
    #[no_mangle]
    fn opj_bio_numbytes(bio: *mut opj_bio_t) -> ptrdiff_t;
    #[no_mangle]
    fn opj_bio_init_enc(bio: *mut opj_bio_t, bp: *mut OPJ_BYTE,
                        len: OPJ_UINT32);
    #[no_mangle]
    fn opj_bio_init_dec(bio: *mut opj_bio_t, bp: *mut OPJ_BYTE,
                        len: OPJ_UINT32);
    #[no_mangle]
    fn opj_bio_write(bio: *mut opj_bio_t, v: OPJ_UINT32, n: OPJ_UINT32);
    #[no_mangle]
    fn opj_bio_read(bio: *mut opj_bio_t, n: OPJ_UINT32) -> OPJ_UINT32;
    #[no_mangle]
    fn opj_bio_flush(bio: *mut opj_bio_t) -> OPJ_BOOL;
    #[no_mangle]
    fn opj_bio_inalign(bio: *mut opj_bio_t) -> OPJ_BOOL;
    #[no_mangle]
    fn opj_pi_initialise_encode(image: *const opj_image_t, cp: *mut opj_cp_t,
                                tileno: OPJ_UINT32, t2_mode: J2K_T2_MODE,
                                manager: *mut opj_event_mgr_t)
     -> *mut opj_pi_iterator_t;
    #[no_mangle]
    fn opj_pi_create_encode(pi: *mut opj_pi_iterator_t, cp: *mut opj_cp_t,
                            tileno: OPJ_UINT32, pino: OPJ_UINT32,
                            tpnum: OPJ_UINT32, tppos: OPJ_INT32,
                            t2_mode: J2K_T2_MODE);
    #[no_mangle]
    fn opj_pi_create_decode(image: *mut opj_image_t, cp: *mut opj_cp_t,
                            tileno: OPJ_UINT32, manager: *mut opj_event_mgr_t)
     -> *mut opj_pi_iterator_t;
    #[no_mangle]
    fn opj_pi_destroy(p_pi: *mut opj_pi_iterator_t,
                      p_nb_elements: OPJ_UINT32);
    #[no_mangle]
    fn opj_pi_next(pi: *mut opj_pi_iterator_t) -> OPJ_BOOL;
    #[no_mangle]
    fn opj_get_encoding_packet_count(p_image: *const opj_image_t,
                                     p_cp: *const opj_cp_t,
                                     p_tile_no: OPJ_UINT32) -> OPJ_UINT32;
    #[no_mangle]
    fn opj_tgt_reset(tree: *mut opj_tgt_tree_t);
    #[no_mangle]
    fn opj_tgt_setvalue(tree: *mut opj_tgt_tree_t, leafno: OPJ_UINT32,
                        value: OPJ_INT32);
    #[no_mangle]
    fn opj_tgt_encode(bio: *mut opj_bio_t, tree: *mut opj_tgt_tree_t,
                      leafno: OPJ_UINT32, threshold: OPJ_INT32);
    #[no_mangle]
    fn opj_tgt_decode(bio: *mut opj_bio_t, tree: *mut opj_tgt_tree_t,
                      leafno: OPJ_UINT32, threshold: OPJ_INT32) -> OPJ_UINT32;
    #[no_mangle]
    fn opj_tcd_is_band_empty(band: *mut opj_tcd_band_t) -> OPJ_BOOL;
    #[no_mangle]
    fn opj_tcd_reinit_segment(seg: *mut opj_tcd_seg_t);
    #[no_mangle]
    fn opj_tcd_is_subband_area_of_interest(tcd: *mut opj_tcd_t,
                                           compno: OPJ_UINT32,
                                           resno: OPJ_UINT32,
                                           bandno: OPJ_UINT32, x0: OPJ_UINT32,
                                           y0: OPJ_UINT32, x1: OPJ_UINT32,
                                           y1: OPJ_UINT32) -> OPJ_BOOL;
}
pub type size_t = libc::c_ulong;
pub type __int16_t = libc::c_short;
pub type __uint16_t = libc::c_ushort;
pub type __int32_t = libc::c_int;
pub type __uint32_t = libc::c_uint;
pub type __int64_t = libc::c_long;
pub type __uint64_t = libc::c_ulong;
pub type __off_t = libc::c_long;
pub type __off64_t = libc::c_long;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _IO_FILE {
    pub _flags: libc::c_int,
    pub _IO_read_ptr: *mut libc::c_char,
    pub _IO_read_end: *mut libc::c_char,
    pub _IO_read_base: *mut libc::c_char,
    pub _IO_write_base: *mut libc::c_char,
    pub _IO_write_ptr: *mut libc::c_char,
    pub _IO_write_end: *mut libc::c_char,
    pub _IO_buf_base: *mut libc::c_char,
    pub _IO_buf_end: *mut libc::c_char,
    pub _IO_save_base: *mut libc::c_char,
    pub _IO_backup_base: *mut libc::c_char,
    pub _IO_save_end: *mut libc::c_char,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: libc::c_int,
    pub _flags2: libc::c_int,
    pub _old_offset: __off_t,
    pub _cur_column: libc::c_ushort,
    pub _vtable_offset: libc::c_schar,
    pub _shortbuf: [libc::c_char; 1],
    pub _lock: *mut libc::c_void,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut libc::c_void,
    pub __pad5: size_t,
    pub _mode: libc::c_int,
    pub _unused2: [libc::c_char; 20],
}
pub type _IO_lock_t = ();
pub type FILE = _IO_FILE;
pub type OPJ_BOOL = libc::c_int;
pub type OPJ_CHAR = libc::c_char;
pub type OPJ_FLOAT32 = libc::c_float;
pub type OPJ_FLOAT64 = libc::c_double;
pub type OPJ_BYTE = libc::c_uchar;
pub type int16_t = __int16_t;
pub type int32_t = __int32_t;
pub type int64_t = __int64_t;
pub type uint16_t = __uint16_t;
pub type uint32_t = __uint32_t;
pub type uint64_t = __uint64_t;
pub type OPJ_INT16 = int16_t;
pub type OPJ_UINT16 = uint16_t;
pub type OPJ_INT32 = int32_t;
pub type OPJ_UINT32 = uint32_t;
pub type OPJ_UINT64 = uint64_t;
pub type OPJ_OFF_T = int64_t;
pub type OPJ_SIZE_T = size_t;
pub type PROG_ORDER = libc::c_int;
pub const OPJ_CPRL: PROG_ORDER = 4;
pub const OPJ_PCRL: PROG_ORDER = 3;
pub const OPJ_RPCL: PROG_ORDER = 2;
pub const OPJ_RLCP: PROG_ORDER = 1;
pub const OPJ_LRCP: PROG_ORDER = 0;
pub const OPJ_PROG_UNKNOWN: PROG_ORDER = -1;
pub type OPJ_PROG_ORDER = PROG_ORDER;
pub type COLOR_SPACE = libc::c_int;
pub const OPJ_CLRSPC_CMYK: COLOR_SPACE = 5;
pub const OPJ_CLRSPC_EYCC: COLOR_SPACE = 4;
pub const OPJ_CLRSPC_SYCC: COLOR_SPACE = 3;
pub const OPJ_CLRSPC_GRAY: COLOR_SPACE = 2;
pub const OPJ_CLRSPC_SRGB: COLOR_SPACE = 1;
pub const OPJ_CLRSPC_UNSPECIFIED: COLOR_SPACE = 0;
pub const OPJ_CLRSPC_UNKNOWN: COLOR_SPACE = -1;
pub type OPJ_COLOR_SPACE = COLOR_SPACE;
pub type opj_msg_callback
    =
    Option<unsafe extern "C" fn(_: *const libc::c_char, _: *mut libc::c_void)
               -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_poc {
    pub resno0: OPJ_UINT32,
    pub compno0: OPJ_UINT32,
    pub layno1: OPJ_UINT32,
    pub resno1: OPJ_UINT32,
    pub compno1: OPJ_UINT32,
    pub layno0: OPJ_UINT32,
    pub precno0: OPJ_UINT32,
    pub precno1: OPJ_UINT32,
    pub prg1: OPJ_PROG_ORDER,
    pub prg: OPJ_PROG_ORDER,
    pub progorder: [OPJ_CHAR; 5],
    pub tile: OPJ_UINT32,
    pub tx0: OPJ_INT32,
    pub tx1: OPJ_INT32,
    pub ty0: OPJ_INT32,
    pub ty1: OPJ_INT32,
    pub layS: OPJ_UINT32,
    pub resS: OPJ_UINT32,
    pub compS: OPJ_UINT32,
    pub prcS: OPJ_UINT32,
    pub layE: OPJ_UINT32,
    pub resE: OPJ_UINT32,
    pub compE: OPJ_UINT32,
    pub prcE: OPJ_UINT32,
    pub txS: OPJ_UINT32,
    pub txE: OPJ_UINT32,
    pub tyS: OPJ_UINT32,
    pub tyE: OPJ_UINT32,
    pub dx: OPJ_UINT32,
    pub dy: OPJ_UINT32,
    pub lay_t: OPJ_UINT32,
    pub res_t: OPJ_UINT32,
    pub comp_t: OPJ_UINT32,
    pub prc_t: OPJ_UINT32,
    pub tx0_t: OPJ_UINT32,
    pub ty0_t: OPJ_UINT32,
}
pub type opj_poc_t = opj_poc;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_image_comp {
    pub dx: OPJ_UINT32,
    pub dy: OPJ_UINT32,
    pub w: OPJ_UINT32,
    pub h: OPJ_UINT32,
    pub x0: OPJ_UINT32,
    pub y0: OPJ_UINT32,
    pub prec: OPJ_UINT32,
    pub bpp: OPJ_UINT32,
    pub sgnd: OPJ_UINT32,
    pub resno_decoded: OPJ_UINT32,
    pub factor: OPJ_UINT32,
    pub data: *mut OPJ_INT32,
    pub alpha: OPJ_UINT16,
}
pub type opj_image_comp_t = opj_image_comp;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_image {
    pub x0: OPJ_UINT32,
    pub y0: OPJ_UINT32,
    pub x1: OPJ_UINT32,
    pub y1: OPJ_UINT32,
    pub numcomps: OPJ_UINT32,
    pub color_space: OPJ_COLOR_SPACE,
    pub comps: *mut opj_image_comp_t,
    pub icc_profile_buf: *mut OPJ_BYTE,
    pub icc_profile_len: OPJ_UINT32,
}
pub type opj_image_t = opj_image;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_packet_info {
    pub start_pos: OPJ_OFF_T,
    pub end_ph_pos: OPJ_OFF_T,
    pub end_pos: OPJ_OFF_T,
    pub disto: libc::c_double,
}
pub type opj_packet_info_t = opj_packet_info;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_marker_info {
    pub type_0: libc::c_ushort,
    pub pos: OPJ_OFF_T,
    pub len: libc::c_int,
}
pub type opj_marker_info_t = opj_marker_info;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_tp_info {
    pub tp_start_pos: libc::c_int,
    pub tp_end_header: libc::c_int,
    pub tp_end_pos: libc::c_int,
    pub tp_start_pack: libc::c_int,
    pub tp_numpacks: libc::c_int,
}
pub type opj_tp_info_t = opj_tp_info;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_tile_info {
    pub thresh: *mut libc::c_double,
    pub tileno: libc::c_int,
    pub start_pos: libc::c_int,
    pub end_header: libc::c_int,
    pub end_pos: libc::c_int,
    pub pw: [libc::c_int; 33],
    pub ph: [libc::c_int; 33],
    pub pdx: [libc::c_int; 33],
    pub pdy: [libc::c_int; 33],
    pub packet: *mut opj_packet_info_t,
    pub numpix: libc::c_int,
    pub distotile: libc::c_double,
    pub marknum: libc::c_int,
    pub marker: *mut opj_marker_info_t,
    pub maxmarknum: libc::c_int,
    pub num_tps: libc::c_int,
    pub tp: *mut opj_tp_info_t,
}
pub type opj_tile_info_t = opj_tile_info;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_codestream_info {
    pub D_max: libc::c_double,
    pub packno: libc::c_int,
    pub index_write: libc::c_int,
    pub image_w: libc::c_int,
    pub image_h: libc::c_int,
    pub prog: OPJ_PROG_ORDER,
    pub tile_x: libc::c_int,
    pub tile_y: libc::c_int,
    pub tile_Ox: libc::c_int,
    pub tile_Oy: libc::c_int,
    pub tw: libc::c_int,
    pub th: libc::c_int,
    pub numcomps: libc::c_int,
    pub numlayers: libc::c_int,
    pub numdecompos: *mut libc::c_int,
    pub marknum: libc::c_int,
    pub marker: *mut opj_marker_info_t,
    pub maxmarknum: libc::c_int,
    pub main_head_start: libc::c_int,
    pub main_head_end: libc::c_int,
    pub codestream_size: libc::c_int,
    pub tile: *mut opj_tile_info_t,
}
pub type opj_codestream_info_t = opj_codestream_info;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_tp_index {
    pub start_pos: OPJ_OFF_T,
    pub end_header: OPJ_OFF_T,
    pub end_pos: OPJ_OFF_T,
}
pub type opj_tp_index_t = opj_tp_index;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_tile_index {
    pub tileno: OPJ_UINT32,
    pub nb_tps: OPJ_UINT32,
    pub current_nb_tps: OPJ_UINT32,
    pub current_tpsno: OPJ_UINT32,
    pub tp_index: *mut opj_tp_index_t,
    pub marknum: OPJ_UINT32,
    pub marker: *mut opj_marker_info_t,
    pub maxmarknum: OPJ_UINT32,
    pub nb_packet: OPJ_UINT32,
    pub packet_index: *mut opj_packet_info_t,
}
pub type opj_tile_index_t = opj_tile_index;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_codestream_index {
    pub main_head_start: OPJ_OFF_T,
    pub main_head_end: OPJ_OFF_T,
    pub codestream_size: OPJ_UINT64,
    pub marknum: OPJ_UINT32,
    pub marker: *mut opj_marker_info_t,
    pub maxmarknum: OPJ_UINT32,
    pub nb_of_tiles: OPJ_UINT32,
    pub tile_index: *mut opj_tile_index_t,
}
pub type opj_codestream_index_t = opj_codestream_index;
pub type OPJ_BITFIELD = libc::c_uint;
pub type ptrdiff_t = libc::c_long;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_event_mgr {
    pub m_error_data: *mut libc::c_void,
    pub m_warning_data: *mut libc::c_void,
    pub m_info_data: *mut libc::c_void,
    pub error_handler: opj_msg_callback,
    pub warning_handler: opj_msg_callback,
    pub info_handler: opj_msg_callback,
}
pub type opj_event_mgr_t = opj_event_mgr;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_bio {
    pub start: *mut OPJ_BYTE,
    pub end: *mut OPJ_BYTE,
    pub bp: *mut OPJ_BYTE,
    pub buf: OPJ_UINT32,
    pub ct: OPJ_UINT32,
}
pub type opj_bio_t = opj_bio;
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct opj_cp {
    pub rsiz: OPJ_UINT16,
    pub tx0: OPJ_UINT32,
    pub ty0: OPJ_UINT32,
    pub tdx: OPJ_UINT32,
    pub tdy: OPJ_UINT32,
    pub comment: *mut OPJ_CHAR,
    pub tw: OPJ_UINT32,
    pub th: OPJ_UINT32,
    pub ppm_markers_count: OPJ_UINT32,
    pub ppm_markers: *mut opj_ppx,
    pub ppm_data: *mut OPJ_BYTE,
    pub ppm_len: OPJ_UINT32,
    pub ppm_data_read: OPJ_UINT32,
    pub ppm_data_current: *mut OPJ_BYTE,
    pub ppm_buffer: *mut OPJ_BYTE,
    pub ppm_data_first: *mut OPJ_BYTE,
    pub ppm_data_size: OPJ_UINT32,
    pub ppm_store: OPJ_INT32,
    pub ppm_previous: OPJ_INT32,
    pub tcps: *mut opj_tcp_t,
    pub m_specific_param: C2RustUnnamed,
    pub strict: OPJ_BOOL,
    #[bitfield(name = "ppm", ty = "OPJ_BITFIELD", bits = "0..=0")]
    #[bitfield(name = "m_is_decoder", ty = "OPJ_BITFIELD", bits = "1..=1")]
    #[bitfield(name = "allow_different_bit_depth_sign", ty = "OPJ_BITFIELD",
               bits = "2..=2")]
    pub ppm_m_is_decoder_allow_different_bit_depth_sign: [u8; 1],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 3],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub m_dec: opj_decoding_param_t,
    pub m_enc: opj_encoding_param_t,
}
pub type opj_encoding_param_t = opj_encoding_param;
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct opj_encoding_param {
    pub m_max_comp_size: OPJ_UINT32,
    pub m_tp_pos: OPJ_INT32,
    pub m_matrice: *mut OPJ_INT32,
    pub m_tp_flag: OPJ_BYTE,
    #[bitfield(name = "m_disto_alloc", ty = "OPJ_BITFIELD", bits = "0..=0")]
    #[bitfield(name = "m_fixed_alloc", ty = "OPJ_BITFIELD", bits = "1..=1")]
    #[bitfield(name = "m_fixed_quality", ty = "OPJ_BITFIELD", bits = "2..=2")]
    #[bitfield(name = "m_tp_on", ty = "OPJ_BITFIELD", bits = "3..=3")]
    pub m_disto_alloc_m_fixed_alloc_m_fixed_quality_m_tp_on: [u8; 1],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 6],
}
pub type opj_decoding_param_t = opj_decoding_param;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_decoding_param {
    pub m_reduce: OPJ_UINT32,
    pub m_layer: OPJ_UINT32,
}
pub type opj_tcp_t = opj_tcp;
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct opj_tcp {
    pub csty: OPJ_UINT32,
    pub prg: OPJ_PROG_ORDER,
    pub numlayers: OPJ_UINT32,
    pub num_layers_to_decode: OPJ_UINT32,
    pub mct: OPJ_UINT32,
    pub rates: [OPJ_FLOAT32; 100],
    pub numpocs: OPJ_UINT32,
    pub pocs: [opj_poc_t; 32],
    pub ppt_markers_count: OPJ_UINT32,
    pub ppt_markers: *mut opj_ppx,
    pub ppt_data: *mut OPJ_BYTE,
    pub ppt_buffer: *mut OPJ_BYTE,
    pub ppt_data_size: OPJ_UINT32,
    pub ppt_len: OPJ_UINT32,
    pub distoratio: [OPJ_FLOAT32; 100],
    pub tccps: *mut opj_tccp_t,
    pub m_current_tile_part_number: OPJ_INT32,
    pub m_nb_tile_parts: OPJ_UINT32,
    pub m_data: *mut OPJ_BYTE,
    pub m_data_size: OPJ_UINT32,
    pub mct_norms: *mut OPJ_FLOAT64,
    pub m_mct_decoding_matrix: *mut OPJ_FLOAT32,
    pub m_mct_coding_matrix: *mut OPJ_FLOAT32,
    pub m_mct_records: *mut opj_mct_data_t,
    pub m_nb_mct_records: OPJ_UINT32,
    pub m_nb_max_mct_records: OPJ_UINT32,
    pub m_mcc_records: *mut opj_simple_mcc_decorrelation_data_t,
    pub m_nb_mcc_records: OPJ_UINT32,
    pub m_nb_max_mcc_records: OPJ_UINT32,
    #[bitfield(name = "cod", ty = "OPJ_BITFIELD", bits = "0..=0")]
    #[bitfield(name = "ppt", ty = "OPJ_BITFIELD", bits = "1..=1")]
    #[bitfield(name = "POC", ty = "OPJ_BITFIELD", bits = "2..=2")]
    pub cod_ppt_POC: [u8; 1],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 7],
}
pub type opj_simple_mcc_decorrelation_data_t
    =
    opj_simple_mcc_decorrelation_data;
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct opj_simple_mcc_decorrelation_data {
    pub m_index: OPJ_UINT32,
    pub m_nb_comps: OPJ_UINT32,
    pub m_decorrelation_array: *mut opj_mct_data_t,
    pub m_offset_array: *mut opj_mct_data_t,
    #[bitfield(name = "m_is_irreversible", ty = "OPJ_BITFIELD", bits =
               "0..=0")]
    pub m_is_irreversible: [u8; 1],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 7],
}
pub type opj_mct_data_t = opj_mct_data;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_mct_data {
    pub m_element_type: J2K_MCT_ELEMENT_TYPE,
    pub m_array_type: J2K_MCT_ARRAY_TYPE,
    pub m_index: OPJ_UINT32,
    pub m_data: *mut OPJ_BYTE,
    pub m_data_size: OPJ_UINT32,
}
pub type J2K_MCT_ARRAY_TYPE = MCT_ARRAY_TYPE;
pub type MCT_ARRAY_TYPE = libc::c_uint;
pub const MCT_TYPE_OFFSET: MCT_ARRAY_TYPE = 2;
pub const MCT_TYPE_DECORRELATION: MCT_ARRAY_TYPE = 1;
pub const MCT_TYPE_DEPENDENCY: MCT_ARRAY_TYPE = 0;
pub type J2K_MCT_ELEMENT_TYPE = MCT_ELEMENT_TYPE;
pub type MCT_ELEMENT_TYPE = libc::c_uint;
pub const MCT_TYPE_DOUBLE: MCT_ELEMENT_TYPE = 3;
pub const MCT_TYPE_FLOAT: MCT_ELEMENT_TYPE = 2;
pub const MCT_TYPE_INT32: MCT_ELEMENT_TYPE = 1;
pub const MCT_TYPE_INT16: MCT_ELEMENT_TYPE = 0;
pub type opj_tccp_t = opj_tccp;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_tccp {
    pub csty: OPJ_UINT32,
    pub numresolutions: OPJ_UINT32,
    pub cblkw: OPJ_UINT32,
    pub cblkh: OPJ_UINT32,
    pub cblksty: OPJ_UINT32,
    pub qmfbid: OPJ_UINT32,
    pub qntsty: OPJ_UINT32,
    pub stepsizes: [opj_stepsize_t; 97],
    pub numgbits: OPJ_UINT32,
    pub roishift: OPJ_INT32,
    pub prcw: [OPJ_UINT32; 33],
    pub prch: [OPJ_UINT32; 33],
    pub m_dc_level_shift: OPJ_INT32,
}
pub type opj_stepsize_t = opj_stepsize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_stepsize {
    pub expn: OPJ_INT32,
    pub mant: OPJ_INT32,
}
pub type opj_ppx = opj_ppx_struct;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_ppx_struct {
    pub m_data: *mut OPJ_BYTE,
    pub m_data_size: OPJ_UINT32,
}
pub type T2_MODE = libc::c_uint;
pub const FINAL_PASS: T2_MODE = 1;
pub const THRESH_CALC: T2_MODE = 0;
pub type J2K_T2_MODE = T2_MODE;
pub type opj_cp_t = opj_cp;
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct opj_tcd {
    pub tp_pos: OPJ_INT32,
    pub tp_num: OPJ_UINT32,
    pub cur_tp_num: OPJ_UINT32,
    pub cur_totnum_tp: OPJ_UINT32,
    pub cur_pino: OPJ_UINT32,
    pub tcd_image: *mut opj_tcd_image_t,
    pub image: *mut opj_image_t,
    pub cp: *mut opj_cp_t,
    pub tcp: *mut opj_tcp_t,
    pub tcd_tileno: OPJ_UINT32,
    #[bitfield(name = "m_is_decoder", ty = "OPJ_BITFIELD", bits = "0..=0")]
    pub m_is_decoder: [u8; 1],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 3],
    pub thread_pool: *mut opj_thread_pool_t,
    pub win_x0: OPJ_UINT32,
    pub win_y0: OPJ_UINT32,
    pub win_x1: OPJ_UINT32,
    pub win_y1: OPJ_UINT32,
    pub whole_tile_decoding: OPJ_BOOL,
    pub used_component: *mut OPJ_BOOL,
}
pub type opj_tcd_image_t = opj_tcd_image;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_tcd_image {
    pub tiles: *mut opj_tcd_tile_t,
}
pub type opj_tcd_tile_t = opj_tcd_tile;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_tcd_tile {
    pub x0: OPJ_INT32,
    pub y0: OPJ_INT32,
    pub x1: OPJ_INT32,
    pub y1: OPJ_INT32,
    pub numcomps: OPJ_UINT32,
    pub comps: *mut opj_tcd_tilecomp_t,
    pub numpix: OPJ_INT32,
    pub distotile: OPJ_FLOAT64,
    pub distolayer: [OPJ_FLOAT64; 100],
    pub packno: OPJ_UINT32,
}
pub type opj_tcd_tilecomp_t = opj_tcd_tilecomp;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_tcd_tilecomp {
    pub x0: OPJ_INT32,
    pub y0: OPJ_INT32,
    pub x1: OPJ_INT32,
    pub y1: OPJ_INT32,
    pub compno: OPJ_UINT32,
    pub numresolutions: OPJ_UINT32,
    pub minimum_num_resolutions: OPJ_UINT32,
    pub resolutions: *mut opj_tcd_resolution_t,
    pub resolutions_size: OPJ_UINT32,
    pub data: *mut OPJ_INT32,
    pub ownsData: OPJ_BOOL,
    pub data_size_needed: size_t,
    pub data_size: size_t,
    pub data_win: *mut OPJ_INT32,
    pub win_x0: OPJ_UINT32,
    pub win_y0: OPJ_UINT32,
    pub win_x1: OPJ_UINT32,
    pub win_y1: OPJ_UINT32,
    pub numpix: OPJ_INT32,
}
pub type opj_tcd_resolution_t = opj_tcd_resolution;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_tcd_resolution {
    pub x0: OPJ_INT32,
    pub y0: OPJ_INT32,
    pub x1: OPJ_INT32,
    pub y1: OPJ_INT32,
    pub pw: OPJ_UINT32,
    pub ph: OPJ_UINT32,
    pub numbands: OPJ_UINT32,
    pub bands: [opj_tcd_band_t; 3],
    pub win_x0: OPJ_UINT32,
    pub win_y0: OPJ_UINT32,
    pub win_x1: OPJ_UINT32,
    pub win_y1: OPJ_UINT32,
}
pub type opj_tcd_band_t = opj_tcd_band;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_tcd_band {
    pub x0: OPJ_INT32,
    pub y0: OPJ_INT32,
    pub x1: OPJ_INT32,
    pub y1: OPJ_INT32,
    pub bandno: OPJ_UINT32,
    pub precincts: *mut opj_tcd_precinct_t,
    pub precincts_data_size: OPJ_UINT32,
    pub numbps: OPJ_INT32,
    pub stepsize: OPJ_FLOAT32,
}
pub type opj_tcd_precinct_t = opj_tcd_precinct;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_tcd_precinct {
    pub x0: OPJ_INT32,
    pub y0: OPJ_INT32,
    pub x1: OPJ_INT32,
    pub y1: OPJ_INT32,
    pub cw: OPJ_UINT32,
    pub ch: OPJ_UINT32,
    pub cblks: C2RustUnnamed_0,
    pub block_size: OPJ_UINT32,
    pub incltree: *mut opj_tgt_tree_t,
    pub imsbtree: *mut opj_tgt_tree_t,
}
pub type opj_tgt_tree_t = opj_tgt_tree;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_tgt_tree {
    pub numleafsh: OPJ_UINT32,
    pub numleafsv: OPJ_UINT32,
    pub numnodes: OPJ_UINT32,
    pub nodes: *mut opj_tgt_node_t,
    pub nodes_size: OPJ_UINT32,
}
pub type opj_tgt_node_t = opj_tgt_node;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_tgt_node {
    pub parent: *mut opj_tgt_node,
    pub value: OPJ_INT32,
    pub low: OPJ_INT32,
    pub known: OPJ_UINT32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_0 {
    pub enc: *mut opj_tcd_cblk_enc_t,
    pub dec: *mut opj_tcd_cblk_dec_t,
    pub blocks: *mut libc::c_void,
}
pub type opj_tcd_cblk_dec_t = opj_tcd_cblk_dec;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_tcd_cblk_dec {
    pub segs: *mut opj_tcd_seg_t,
    pub chunks: *mut opj_tcd_seg_data_chunk_t,
    pub x0: OPJ_INT32,
    pub y0: OPJ_INT32,
    pub x1: OPJ_INT32,
    pub y1: OPJ_INT32,
    pub Mb: OPJ_UINT32,
    pub numbps: OPJ_UINT32,
    pub numlenbits: OPJ_UINT32,
    pub numnewpasses: OPJ_UINT32,
    pub numsegs: OPJ_UINT32,
    pub real_num_segs: OPJ_UINT32,
    pub m_current_max_segs: OPJ_UINT32,
    pub numchunks: OPJ_UINT32,
    pub numchunksalloc: OPJ_UINT32,
    pub decoded_data: *mut OPJ_INT32,
}
pub type opj_tcd_seg_data_chunk_t = opj_tcd_seg_data_chunk;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_tcd_seg_data_chunk {
    pub data: *mut OPJ_BYTE,
    pub len: OPJ_UINT32,
}
pub type opj_tcd_seg_t = opj_tcd_seg;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_tcd_seg {
    pub len: OPJ_UINT32,
    pub numpasses: OPJ_UINT32,
    pub real_num_passes: OPJ_UINT32,
    pub maxpasses: OPJ_UINT32,
    pub numnewpasses: OPJ_UINT32,
    pub newlen: OPJ_UINT32,
}
pub type opj_tcd_cblk_enc_t = opj_tcd_cblk_enc;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_tcd_cblk_enc {
    pub data: *mut OPJ_BYTE,
    pub layers: *mut opj_tcd_layer_t,
    pub passes: *mut opj_tcd_pass_t,
    pub x0: OPJ_INT32,
    pub y0: OPJ_INT32,
    pub x1: OPJ_INT32,
    pub y1: OPJ_INT32,
    pub numbps: OPJ_UINT32,
    pub numlenbits: OPJ_UINT32,
    pub data_size: OPJ_UINT32,
    pub numpasses: OPJ_UINT32,
    pub numpassesinlayers: OPJ_UINT32,
    pub totalpasses: OPJ_UINT32,
}
pub type opj_tcd_pass_t = opj_tcd_pass;
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct opj_tcd_pass {
    pub rate: OPJ_UINT32,
    pub distortiondec: OPJ_FLOAT64,
    pub len: OPJ_UINT32,
    #[bitfield(name = "term", ty = "OPJ_BITFIELD", bits = "0..=0")]
    pub term: [u8; 1],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 3],
}
pub type opj_tcd_layer_t = opj_tcd_layer;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_tcd_layer {
    pub numpasses: OPJ_UINT32,
    pub len: OPJ_UINT32,
    pub disto: OPJ_FLOAT64,
    pub data: *mut OPJ_BYTE,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_pi_resolution {
    pub pdx: OPJ_UINT32,
    pub pdy: OPJ_UINT32,
    pub pw: OPJ_UINT32,
    pub ph: OPJ_UINT32,
}
pub type opj_pi_resolution_t = opj_pi_resolution;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_pi_comp {
    pub dx: OPJ_UINT32,
    pub dy: OPJ_UINT32,
    pub numresolutions: OPJ_UINT32,
    pub resolutions: *mut opj_pi_resolution_t,
}
pub type opj_pi_comp_t = opj_pi_comp;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_pi_iterator {
    pub tp_on: OPJ_BYTE,
    pub include: *mut OPJ_INT16,
    pub include_size: OPJ_UINT32,
    pub step_l: OPJ_UINT32,
    pub step_r: OPJ_UINT32,
    pub step_c: OPJ_UINT32,
    pub step_p: OPJ_UINT32,
    pub compno: OPJ_UINT32,
    pub resno: OPJ_UINT32,
    pub precno: OPJ_UINT32,
    pub layno: OPJ_UINT32,
    pub first: OPJ_BOOL,
    pub poc: opj_poc_t,
    pub numcomps: OPJ_UINT32,
    pub comps: *mut opj_pi_comp_t,
    pub tx0: OPJ_UINT32,
    pub ty0: OPJ_UINT32,
    pub tx1: OPJ_UINT32,
    pub ty1: OPJ_UINT32,
    pub x: OPJ_UINT32,
    pub y: OPJ_UINT32,
    pub dx: OPJ_UINT32,
    pub dy: OPJ_UINT32,
    pub manager: *mut opj_event_mgr_t,
}
pub type opj_pi_iterator_t = opj_pi_iterator;
pub type opj_tcd_t = opj_tcd;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_tcd_marker_info {
    pub need_PLT: OPJ_BOOL,
    pub packet_count: OPJ_UINT32,
    pub p_packet_size: *mut OPJ_UINT32,
}
pub type opj_tcd_marker_info_t = opj_tcd_marker_info;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_t2 {
    pub image: *mut opj_image_t,
    pub cp: *mut opj_cp_t,
}
pub type opj_t2_t = opj_t2;
#[inline]
unsafe extern "C" fn opj_int_max(mut a: OPJ_INT32, mut b: OPJ_INT32)
 -> OPJ_INT32 {
    return if a > b { a } else { b };
}
#[inline]
unsafe extern "C" fn opj_int_min(mut a: OPJ_INT32, mut b: OPJ_INT32)
 -> OPJ_INT32 {
    return if a < b { a } else { b };
}
#[inline]
unsafe extern "C" fn opj_uint_max(mut a: OPJ_UINT32, mut b: OPJ_UINT32)
 -> OPJ_UINT32 {
    return if a > b { a } else { b };
}
#[inline]
unsafe extern "C" fn opj_int_floorlog2(mut a: OPJ_INT32) -> OPJ_INT32 {
    let mut l: OPJ_INT32 = 0;
    l = 0 as libc::c_int;
    while a > 1 as libc::c_int { a >>= 1 as libc::c_int; l += 1 }
    return l;
}
#[inline]
unsafe extern "C" fn opj_uint_floorlog2(mut a: OPJ_UINT32) -> OPJ_UINT32 {
    let mut l: OPJ_UINT32 = 0;
    l = 0 as libc::c_int as OPJ_UINT32;
    while a > 1 as libc::c_int as libc::c_uint {
        a >>= 1 as libc::c_int;
        l = l.wrapping_add(1)
    }
    return l;
}
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
 * Copyright (c) 2017, IntoPIX SA <support@intopix.com>
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
/* * @defgroup T2 T2 - Implementation of a tier-2 coding */
/*@{*/
/* * @name Local static functions */
/*@{*/
/*@}*/
/*@}*/
/* ----------------------------------------------------------------------- */
/* #define RESTART 0x04 */
unsafe extern "C" fn opj_t2_putcommacode(mut bio: *mut opj_bio_t,
                                         mut n: OPJ_INT32) {
    loop  {
        n -= 1;
        if !(n >= 0 as libc::c_int) { break ; }
        opj_bio_write(bio, 1 as libc::c_int as OPJ_UINT32,
                      1 as libc::c_int as OPJ_UINT32);
    }
    opj_bio_write(bio, 0 as libc::c_int as OPJ_UINT32,
                  1 as libc::c_int as OPJ_UINT32);
}
unsafe extern "C" fn opj_t2_getcommacode(mut bio: *mut opj_bio_t)
 -> OPJ_UINT32 {
    let mut n = 0 as libc::c_int as OPJ_UINT32;
    while opj_bio_read(bio, 1 as libc::c_int as OPJ_UINT32) != 0 {
        n = n.wrapping_add(1)
    }
    return n;
}
/* *
Variable length code for signalling delta Zil (truncation point)
@param bio  Bit Input/Output component
@param n    delta Zil
*/
unsafe extern "C" fn opj_t2_putnumpasses(mut bio: *mut opj_bio_t,
                                         mut n: OPJ_UINT32) {
    if n == 1 as libc::c_int as libc::c_uint {
        opj_bio_write(bio, 0 as libc::c_int as OPJ_UINT32,
                      1 as libc::c_int as OPJ_UINT32);
    } else if n == 2 as libc::c_int as libc::c_uint {
        opj_bio_write(bio, 2 as libc::c_int as OPJ_UINT32,
                      2 as libc::c_int as OPJ_UINT32);
    } else if n <= 5 as libc::c_int as libc::c_uint {
        opj_bio_write(bio,
                      0xc as libc::c_int as libc::c_uint |
                          n.wrapping_sub(3 as libc::c_int as libc::c_uint),
                      4 as libc::c_int as OPJ_UINT32);
    } else if n <= 36 as libc::c_int as libc::c_uint {
        opj_bio_write(bio,
                      0x1e0 as libc::c_int as libc::c_uint |
                          n.wrapping_sub(6 as libc::c_int as libc::c_uint),
                      9 as libc::c_int as OPJ_UINT32);
    } else if n <= 164 as libc::c_int as libc::c_uint {
        opj_bio_write(bio,
                      0xff80 as libc::c_int as libc::c_uint |
                          n.wrapping_sub(37 as libc::c_int as libc::c_uint),
                      16 as libc::c_int as OPJ_UINT32);
    };
}
unsafe extern "C" fn opj_t2_getnumpasses(mut bio: *mut opj_bio_t)
 -> OPJ_UINT32 {
    let mut n: OPJ_UINT32 = 0;
    if opj_bio_read(bio, 1 as libc::c_int as OPJ_UINT32) == 0 {
        return 1 as libc::c_int as OPJ_UINT32
    }
    if opj_bio_read(bio, 1 as libc::c_int as OPJ_UINT32) == 0 {
        return 2 as libc::c_int as OPJ_UINT32
    }
    n = opj_bio_read(bio, 2 as libc::c_int as OPJ_UINT32);
    if n != 3 as libc::c_int as libc::c_uint {
        return (3 as libc::c_int as libc::c_uint).wrapping_add(n)
    }
    n = opj_bio_read(bio, 5 as libc::c_int as OPJ_UINT32);
    if n != 31 as libc::c_int as libc::c_uint {
        return (6 as libc::c_int as libc::c_uint).wrapping_add(n)
    }
    return (37 as libc::c_int as
                libc::c_uint).wrapping_add(opj_bio_read(bio,
                                                        7 as libc::c_int as
                                                            OPJ_UINT32));
}
/* ----------------------------------------------------------------------- */
#[no_mangle]
pub unsafe extern "C" fn opj_t2_encode_packets(mut p_t2: *mut opj_t2_t,
                                               mut p_tile_no: OPJ_UINT32,
                                               mut p_tile:
                                                   *mut opj_tcd_tile_t,
                                               mut p_maxlayers: OPJ_UINT32,
                                               mut p_dest: *mut OPJ_BYTE,
                                               mut p_data_written:
                                                   *mut OPJ_UINT32,
                                               mut p_max_len: OPJ_UINT32,
                                               mut cstr_info:
                                                   *mut opj_codestream_info_t,
                                               mut p_marker_info:
                                                   *mut opj_tcd_marker_info_t,
                                               mut p_tp_num: OPJ_UINT32,
                                               mut p_tp_pos: OPJ_INT32,
                                               mut p_pino: OPJ_UINT32,
                                               mut p_t2_mode: J2K_T2_MODE,
                                               mut p_manager:
                                                   *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut l_current_data = p_dest; /* t2_mode == FINAL_PASS  */
    let mut l_nb_bytes = 0 as libc::c_int as OPJ_UINT32;
    let mut compno: OPJ_UINT32 = 0;
    let mut poc: OPJ_UINT32 = 0;
    let mut l_pi = 0 as *mut opj_pi_iterator_t;
    let mut l_current_pi = 0 as *mut opj_pi_iterator_t;
    let mut l_image = (*p_t2).image;
    let mut l_cp = (*p_t2).cp;
    let mut l_tcp: *mut opj_tcp_t =
        &mut *(*l_cp).tcps.offset(p_tile_no as isize) as *mut opj_tcp_t;
    let mut pocno =
        if (*l_cp).rsiz as libc::c_int == 0x4 as libc::c_int {
            2 as libc::c_int
        } else { 1 as libc::c_int } as OPJ_UINT32;
    let mut l_max_comp =
        if (*l_cp).m_specific_param.m_enc.m_max_comp_size >
               0 as libc::c_int as libc::c_uint {
            (*l_image).numcomps
        } else { 1 as libc::c_int as libc::c_uint };
    let mut l_nb_pocs =
        (*l_tcp).numpocs.wrapping_add(1 as libc::c_int as libc::c_uint);
    l_pi =
        opj_pi_initialise_encode(l_image, l_cp, p_tile_no, p_t2_mode,
                                 p_manager);
    if l_pi.is_null() { return 0 as libc::c_int }
    *p_data_written = 0 as libc::c_int as OPJ_UINT32;
    if p_t2_mode as libc::c_uint == THRESH_CALC as libc::c_int as libc::c_uint
       {
        /* Calculating threshold */
        l_current_pi = l_pi;
        compno = 0 as libc::c_int as OPJ_UINT32;
        while compno < l_max_comp {
            let mut l_comp_len = 0 as libc::c_int as OPJ_UINT32;
            l_current_pi = l_pi;
            poc = 0 as libc::c_int as OPJ_UINT32;
            while poc < pocno {
                let mut l_tp_num = compno;
                /* TODO MSD : check why this function cannot fail (cf. v1) */
                opj_pi_create_encode(l_pi, l_cp, p_tile_no, poc, l_tp_num,
                                     p_tp_pos, p_t2_mode);
                if (*l_current_pi).poc.prg as libc::c_int ==
                       OPJ_PROG_UNKNOWN as libc::c_int {
                    /* TODO ADE : add an error */
                    opj_pi_destroy(l_pi, l_nb_pocs);
                    return 0 as libc::c_int
                }
                while opj_pi_next(l_current_pi) != 0 {
                    if (*l_current_pi).layno < p_maxlayers {
                        l_nb_bytes = 0 as libc::c_int as OPJ_UINT32;
                        if opj_t2_encode_packet(p_tile_no, p_tile, l_tcp,
                                                l_current_pi, l_current_data,
                                                &mut l_nb_bytes, p_max_len,
                                                cstr_info, p_t2_mode,
                                                p_manager) == 0 {
                            opj_pi_destroy(l_pi, l_nb_pocs);
                            return 0 as libc::c_int
                        }
                        l_comp_len =
                            (l_comp_len as
                                 libc::c_uint).wrapping_add(l_nb_bytes) as
                                OPJ_UINT32 as OPJ_UINT32;
                        l_current_data =
                            l_current_data.offset(l_nb_bytes as isize);
                        p_max_len =
                            (p_max_len as
                                 libc::c_uint).wrapping_sub(l_nb_bytes) as
                                OPJ_UINT32 as OPJ_UINT32;
                        *p_data_written =
                            (*p_data_written as
                                 libc::c_uint).wrapping_add(l_nb_bytes) as
                                OPJ_UINT32 as OPJ_UINT32
                    }
                }
                if (*l_cp).m_specific_param.m_enc.m_max_comp_size != 0 {
                    if l_comp_len >
                           (*l_cp).m_specific_param.m_enc.m_max_comp_size {
                        opj_pi_destroy(l_pi, l_nb_pocs);
                        return 0 as libc::c_int
                    }
                }
                l_current_pi = l_current_pi.offset(1);
                poc = poc.wrapping_add(1)
            }
            compno = compno.wrapping_add(1)
        }
    } else {
        opj_pi_create_encode(l_pi, l_cp, p_tile_no, p_pino, p_tp_num,
                             p_tp_pos, p_t2_mode);
        l_current_pi =
            &mut *l_pi.offset(p_pino as isize) as *mut opj_pi_iterator_t;
        if (*l_current_pi).poc.prg as libc::c_int ==
               OPJ_PROG_UNKNOWN as libc::c_int {
            /* TODO ADE : add an error */
            opj_pi_destroy(l_pi, l_nb_pocs);
            return 0 as libc::c_int
        }
        if !p_marker_info.is_null() && (*p_marker_info).need_PLT != 0 {
            /* One time use intended */
            if (*p_marker_info).packet_count ==
                   0 as libc::c_int as libc::c_uint {
            } else {
                __assert_fail(b"p_marker_info->packet_count == 0\x00" as
                                  *const u8 as *const libc::c_char,
                              b"/opt/openjpeg/src/lib/openjp2/t2.c\x00" as
                                  *const u8 as *const libc::c_char,
                              317 as libc::c_int as libc::c_uint,
                              (*::std::mem::transmute::<&[u8; 240],
                                                        &[libc::c_char; 240]>(b"OPJ_BOOL opj_t2_encode_packets(opj_t2_t *, OPJ_UINT32, opj_tcd_tile_t *, OPJ_UINT32, OPJ_BYTE *, OPJ_UINT32 *, OPJ_UINT32, opj_codestream_info_t *, opj_tcd_marker_info_t *, OPJ_UINT32, OPJ_INT32, OPJ_UINT32, J2K_T2_MODE, opj_event_mgr_t *)\x00")).as_ptr());
            }
            if (*p_marker_info).p_packet_size.is_null() {
            } else {
                __assert_fail(b"p_marker_info->p_packet_size == NULL\x00" as
                                  *const u8 as *const libc::c_char,
                              b"/opt/openjpeg/src/lib/openjp2/t2.c\x00" as
                                  *const u8 as *const libc::c_char,
                              318 as libc::c_int as libc::c_uint,
                              (*::std::mem::transmute::<&[u8; 240],
                                                        &[libc::c_char; 240]>(b"OPJ_BOOL opj_t2_encode_packets(opj_t2_t *, OPJ_UINT32, opj_tcd_tile_t *, OPJ_UINT32, OPJ_BYTE *, OPJ_UINT32 *, OPJ_UINT32, opj_codestream_info_t *, opj_tcd_marker_info_t *, OPJ_UINT32, OPJ_INT32, OPJ_UINT32, J2K_T2_MODE, opj_event_mgr_t *)\x00")).as_ptr());
            }
            (*p_marker_info).p_packet_size =
                opj_malloc((opj_get_encoding_packet_count(l_image, l_cp,
                                                          p_tile_no) as
                                libc::c_ulong).wrapping_mul(::std::mem::size_of::<OPJ_UINT32>()
                                                                as
                                                                libc::c_ulong))
                    as *mut OPJ_UINT32;
            if (*p_marker_info).p_packet_size.is_null() {
                opj_pi_destroy(l_pi, l_nb_pocs);
                return 0 as libc::c_int
            }
        }
        while opj_pi_next(l_current_pi) != 0 {
            if (*l_current_pi).layno < p_maxlayers {
                l_nb_bytes = 0 as libc::c_int as OPJ_UINT32;
                if opj_t2_encode_packet(p_tile_no, p_tile, l_tcp,
                                        l_current_pi, l_current_data,
                                        &mut l_nb_bytes, p_max_len, cstr_info,
                                        p_t2_mode, p_manager) == 0 {
                    opj_pi_destroy(l_pi, l_nb_pocs);
                    return 0 as libc::c_int
                }
                l_current_data = l_current_data.offset(l_nb_bytes as isize);
                p_max_len =
                    (p_max_len as libc::c_uint).wrapping_sub(l_nb_bytes) as
                        OPJ_UINT32 as OPJ_UINT32;
                *p_data_written =
                    (*p_data_written as libc::c_uint).wrapping_add(l_nb_bytes)
                        as OPJ_UINT32 as OPJ_UINT32;
                if !p_marker_info.is_null() && (*p_marker_info).need_PLT != 0
                   {
                    *(*p_marker_info).p_packet_size.offset((*p_marker_info).packet_count
                                                               as isize) =
                        l_nb_bytes;
                    (*p_marker_info).packet_count =
                        (*p_marker_info).packet_count.wrapping_add(1)
                }
                /* INDEX >> */
                if !cstr_info.is_null() {
                    if (*cstr_info).index_write != 0 {
                        let mut info_TL: *mut opj_tile_info_t =
                            &mut *(*cstr_info).tile.offset(p_tile_no as isize)
                                as *mut opj_tile_info_t;
                        let mut info_PK: *mut opj_packet_info_t =
                            &mut *(*info_TL).packet.offset((*cstr_info).packno
                                                               as isize) as
                                *mut opj_packet_info_t;
                        if (*cstr_info).packno == 0 {
                            (*info_PK).start_pos =
                                ((*info_TL).end_header + 1 as libc::c_int) as
                                    OPJ_OFF_T
                        } else {
                            (*info_PK).start_pos =
                                if (*l_cp).m_specific_param.m_enc.m_tp_on() as
                                       libc::c_int |
                                       (*l_tcp).POC() as libc::c_int != 0 &&
                                       (*info_PK).start_pos != 0 {
                                    (*info_PK).start_pos
                                } else {
                                    ((*(*info_TL).packet.offset(((*cstr_info).packno
                                                                     -
                                                                     1 as
                                                                         libc::c_int)
                                                                    as
                                                                    isize)).end_pos)
                                        + 1 as libc::c_int as libc::c_long
                                }
                        }
                        (*info_PK).end_pos =
                            (*info_PK).start_pos + l_nb_bytes as libc::c_long
                                - 1 as libc::c_int as libc::c_long;
                        (*info_PK).end_ph_pos +=
                            (*info_PK).start_pos -
                                1 as libc::c_int as libc::c_long
                        /* End of packet header which now only represents the distance
                                                                                                                                                                                                                                                   to start of packet is incremented by value of start of packet*/
                    }
                    (*cstr_info).packno += 1
                }
                /* << INDEX */
                (*p_tile).packno = (*p_tile).packno.wrapping_add(1)
            }
        }
    }
    opj_pi_destroy(l_pi, l_nb_pocs);
    return 1 as libc::c_int;
}
/* see issue 80 */
/* issue 290 */
unsafe extern "C" fn opj_null_jas_fprintf(mut _file: *mut FILE,
                                          mut _format: *const libc::c_char,
                                          mut _args: ...) {
}
#[no_mangle]
pub unsafe extern "C" fn opj_t2_decode_packets(mut tcd: *mut opj_tcd_t,
                                               mut p_t2: *mut opj_t2_t,
                                               mut p_tile_no: OPJ_UINT32,
                                               mut p_tile:
                                                   *mut opj_tcd_tile_t,
                                               mut p_src: *mut OPJ_BYTE,
                                               mut p_data_read:
                                                   *mut OPJ_UINT32,
                                               mut p_max_len: OPJ_UINT32,
                                               mut _p_cstr_index:
                                                   *mut opj_codestream_index_t,
                                               mut p_manager:
                                                   *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut l_current_data = p_src;
    let mut l_pi = 0 as *mut opj_pi_iterator_t;
    let mut pino: OPJ_UINT32 = 0;
    let mut l_image = (*p_t2).image;
    let mut l_cp = (*p_t2).cp;
    let mut l_tcp: *mut opj_tcp_t =
        &mut *(*(*p_t2).cp).tcps.offset(p_tile_no as isize) as *mut opj_tcp_t;
    let mut l_nb_bytes_read: OPJ_UINT32 = 0;
    let mut l_nb_pocs =
        (*l_tcp).numpocs.wrapping_add(1 as libc::c_int as libc::c_uint);
    let mut l_current_pi = 0 as *mut opj_pi_iterator_t;
    let mut l_pack_info = 0 as *mut opj_packet_info_t;
    let mut l_img_comp = 0 as *mut opj_image_comp_t;
    /* create a packet iterator */
    l_pi = opj_pi_create_decode(l_image, l_cp, p_tile_no, p_manager);
    if l_pi.is_null() { return 0 as libc::c_int }
    l_current_pi = l_pi;
    pino = 0 as libc::c_int as OPJ_UINT32;
    while pino <= (*l_tcp).numpocs {
        /* if the resolution needed is too low, one dim of the tilec could be equal to zero
         * and no packets are used to decode this resolution and
         * l_current_pi->resno is always >= p_tile->comps[l_current_pi->compno].minimum_num_resolutions
         * and no l_img_comp->resno_decoded are computed
         */
        let mut first_pass_failed = 0 as *mut OPJ_BOOL;
        if (*l_current_pi).poc.prg as libc::c_int ==
               OPJ_PROG_UNKNOWN as libc::c_int {
            /* TODO ADE : add an error */
            opj_pi_destroy(l_pi, l_nb_pocs);
            return 0 as libc::c_int
        }
        first_pass_failed =
            opj_malloc(((*l_image).numcomps as
                            libc::c_ulong).wrapping_mul(::std::mem::size_of::<OPJ_BOOL>()
                                                            as libc::c_ulong))
                as *mut OPJ_BOOL;
        if first_pass_failed.is_null() {
            opj_pi_destroy(l_pi, l_nb_pocs);
            return 0 as libc::c_int
        }
        memset(first_pass_failed as *mut libc::c_void, 1 as libc::c_int,
               ((*l_image).numcomps as
                    libc::c_ulong).wrapping_mul(::std::mem::size_of::<OPJ_BOOL>()
                                                    as libc::c_ulong));
        while opj_pi_next(l_current_pi) != 0 {
            let mut skip_packet = 0 as libc::c_int;
            opj_null_jas_fprintf(stderr,
                                 b"packet offset=00000166 prg=%d cmptno=%02d rlvlno=%02d prcno=%03d lyrno=%02d\n\n\x00"
                                     as *const u8 as *const libc::c_char,
                                 (*l_current_pi).poc.prg1 as libc::c_int,
                                 (*l_current_pi).compno,
                                 (*l_current_pi).resno,
                                 (*l_current_pi).precno,
                                 (*l_current_pi).layno);
            /* INDEX >> */
            /* << INDEX */
            if (*l_current_pi).layno >= (*l_tcp).num_layers_to_decode {
                skip_packet = 1 as libc::c_int
            } else if (*l_current_pi).resno >=
                          (*(*p_tile).comps.offset((*l_current_pi).compno as
                                                       isize)).minimum_num_resolutions
             {
                skip_packet = 1 as libc::c_int
            } else {
                /* If the packet layer is greater or equal than the maximum */
            /* number of layers, skip the packet */
                /* If the packet resolution number is greater than the minimum */
            /* number of resolution allowed, skip the packet */
                /* If no precincts of any band intersects the area of interest, */
                /* skip the packet */
                let mut bandno: OPJ_UINT32 = 0;
                let mut tilec: *mut opj_tcd_tilecomp_t =
                    &mut *(*p_tile).comps.offset((*l_current_pi).compno as
                                                     isize) as
                        *mut opj_tcd_tilecomp_t;
                let mut res: *mut opj_tcd_resolution_t =
                    &mut *(*tilec).resolutions.offset((*l_current_pi).resno as
                                                          isize) as
                        *mut opj_tcd_resolution_t;
                skip_packet = 1 as libc::c_int;
                bandno = 0 as libc::c_int as OPJ_UINT32;
                while bandno < (*res).numbands {
                    let mut band: *mut opj_tcd_band_t =
                        &mut *(*res).bands.as_mut_ptr().offset(bandno as
                                                                   isize) as
                            *mut opj_tcd_band_t;
                    let mut prec: *mut opj_tcd_precinct_t =
                        &mut *(*band).precincts.offset((*l_current_pi).precno
                                                           as isize) as
                            *mut opj_tcd_precinct_t;
                    if opj_tcd_is_subband_area_of_interest(tcd,
                                                           (*l_current_pi).compno,
                                                           (*l_current_pi).resno,
                                                           (*band).bandno,
                                                           (*prec).x0 as
                                                               OPJ_UINT32,
                                                           (*prec).y0 as
                                                               OPJ_UINT32,
                                                           (*prec).x1 as
                                                               OPJ_UINT32,
                                                           (*prec).y1 as
                                                               OPJ_UINT32) !=
                           0 {
                        skip_packet = 0 as libc::c_int;
                        break ;
                    } else { bandno = bandno.wrapping_add(1) }
                }
            }
            if skip_packet == 0 {
                l_nb_bytes_read = 0 as libc::c_int as OPJ_UINT32;
                *first_pass_failed.offset((*l_current_pi).compno as isize) =
                    0 as libc::c_int;
                if opj_t2_decode_packet(p_t2, p_tile, l_tcp, l_current_pi,
                                        l_current_data, &mut l_nb_bytes_read,
                                        p_max_len, l_pack_info, p_manager) ==
                       0 {
                    opj_pi_destroy(l_pi, l_nb_pocs);
                    opj_free(first_pass_failed as *mut libc::c_void);
                    return 0 as libc::c_int
                }
                l_img_comp =
                    &mut *(*l_image).comps.offset((*l_current_pi).compno as
                                                      isize) as
                        *mut opj_image_comp_t;
                (*l_img_comp).resno_decoded =
                    opj_uint_max((*l_current_pi).resno,
                                 (*l_img_comp).resno_decoded)
            } else {
                l_nb_bytes_read = 0 as libc::c_int as OPJ_UINT32;
                if opj_t2_skip_packet(p_t2, p_tile, l_tcp, l_current_pi,
                                      l_current_data, &mut l_nb_bytes_read,
                                      p_max_len, l_pack_info, p_manager) == 0
                   {
                    opj_pi_destroy(l_pi, l_nb_pocs);
                    opj_free(first_pass_failed as *mut libc::c_void);
                    return 0 as libc::c_int
                }
            }
            if *first_pass_failed.offset((*l_current_pi).compno as isize) != 0
               {
                l_img_comp =
                    &mut *(*l_image).comps.offset((*l_current_pi).compno as
                                                      isize) as
                        *mut opj_image_comp_t;
                if (*l_img_comp).resno_decoded ==
                       0 as libc::c_int as libc::c_uint {
                    (*l_img_comp).resno_decoded =
                        (*(*p_tile).comps.offset((*l_current_pi).compno as
                                                     isize)).minimum_num_resolutions.wrapping_sub(1
                                                                                                      as
                                                                                                      libc::c_int
                                                                                                      as
                                                                                                      libc::c_uint)
                }
            }
            l_current_data = l_current_data.offset(l_nb_bytes_read as isize);
            p_max_len =
                (p_max_len as libc::c_uint).wrapping_sub(l_nb_bytes_read) as
                    OPJ_UINT32 as OPJ_UINT32
        }
        l_current_pi = l_current_pi.offset(1);
        opj_free(first_pass_failed as *mut libc::c_void);
        pino = pino.wrapping_add(1)
    }
    /* INDEX >> */
    /* << INDEX */
    /* don't forget to release pi */
    opj_pi_destroy(l_pi, l_nb_pocs);
    *p_data_read =
        l_current_data.wrapping_offset_from(p_src) as libc::c_long as
            OPJ_UINT32;
    return 1 as libc::c_int;
}
/* ----------------------------------------------------------------------- */
/* *
 * Creates a Tier 2 handle
 *
 * @param       p_image         Source or destination image
 * @param       p_cp            Image coding parameters.
 * @return              a new T2 handle if successful, NULL otherwise.
*/
#[no_mangle]
pub unsafe extern "C" fn opj_t2_create(mut p_image: *mut opj_image_t,
                                       mut p_cp: *mut opj_cp_t)
 -> *mut opj_t2_t {
    /* create the t2 structure */
    let mut l_t2 =
        opj_calloc(1 as libc::c_int as size_t,
                   ::std::mem::size_of::<opj_t2_t>() as libc::c_ulong) as
            *mut opj_t2_t;
    if l_t2.is_null() { return 0 as *mut opj_t2_t }
    (*l_t2).image = p_image;
    (*l_t2).cp = p_cp;
    return l_t2;
}
#[no_mangle]
pub unsafe extern "C" fn opj_t2_destroy(mut t2: *mut opj_t2_t) {
    if !t2.is_null() { opj_free(t2 as *mut libc::c_void); };
}
/* *
Decode a packet of a tile from a source buffer
@param t2 T2 handle
@param tile Tile for which to write the packets
@param tcp Tile coding parameters
@param pi Packet identity
@param src Source buffer
@param data_read   FIXME DOC
@param max_length  FIXME DOC
@param pack_info Packet information
@param p_manager the user event manager

@return  FIXME DOC
*/
unsafe extern "C" fn opj_t2_decode_packet(mut p_t2: *mut opj_t2_t,
                                          mut p_tile: *mut opj_tcd_tile_t,
                                          mut p_tcp: *mut opj_tcp_t,
                                          mut p_pi: *mut opj_pi_iterator_t,
                                          mut p_src: *mut OPJ_BYTE,
                                          mut p_data_read: *mut OPJ_UINT32,
                                          mut p_max_length: OPJ_UINT32,
                                          mut p_pack_info:
                                              *mut opj_packet_info_t,
                                          mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut l_read_data: OPJ_BOOL = 0;
    let mut l_nb_bytes_read = 0 as libc::c_int as OPJ_UINT32;
    let mut l_nb_total_bytes_read = 0 as libc::c_int as OPJ_UINT32;
    *p_data_read = 0 as libc::c_int as OPJ_UINT32;
    if opj_t2_read_packet_header(p_t2, p_tile, p_tcp, p_pi, &mut l_read_data,
                                 p_src, &mut l_nb_bytes_read, p_max_length,
                                 p_pack_info, p_manager) == 0 {
        return 0 as libc::c_int
    }
    p_src = p_src.offset(l_nb_bytes_read as isize);
    l_nb_total_bytes_read =
        (l_nb_total_bytes_read as libc::c_uint).wrapping_add(l_nb_bytes_read)
            as OPJ_UINT32 as OPJ_UINT32;
    p_max_length =
        (p_max_length as libc::c_uint).wrapping_sub(l_nb_bytes_read) as
            OPJ_UINT32 as OPJ_UINT32;
    /* we should read data for the packet */
    if l_read_data != 0 {
        l_nb_bytes_read = 0 as libc::c_int as OPJ_UINT32;
        if opj_t2_read_packet_data(p_t2, p_tile, p_pi, p_src,
                                   &mut l_nb_bytes_read, p_max_length,
                                   p_pack_info, p_manager) == 0 {
            return 0 as libc::c_int
        }
        l_nb_total_bytes_read =
            (l_nb_total_bytes_read as
                 libc::c_uint).wrapping_add(l_nb_bytes_read) as OPJ_UINT32 as
                OPJ_UINT32
    }
    *p_data_read = l_nb_total_bytes_read;
    return 1 as libc::c_int;
}
/* *
Encode a packet of a tile to a destination buffer
@param tileno Number of the tile encoded
@param tile Tile for which to write the packets
@param tcp Tile coding parameters
@param pi Packet identity
@param dest Destination buffer
@param p_data_written   FIXME DOC
@param len Length of the destination buffer
@param cstr_info Codestream information structure
@param p_t2_mode If == THRESH_CALC In Threshold calculation ,If == FINAL_PASS Final pass
@param p_manager the user event manager
@return
*/
unsafe extern "C" fn opj_t2_encode_packet(mut tileno: OPJ_UINT32,
                                          mut tile: *mut opj_tcd_tile_t,
                                          mut tcp: *mut opj_tcp_t,
                                          mut pi: *mut opj_pi_iterator_t,
                                          mut dest: *mut OPJ_BYTE,
                                          mut p_data_written: *mut OPJ_UINT32,
                                          mut length: OPJ_UINT32,
                                          mut cstr_info:
                                              *mut opj_codestream_info_t,
                                          mut p_t2_mode: J2K_T2_MODE,
                                          mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut bandno: OPJ_UINT32 = 0; /* component value */
    let mut cblkno: OPJ_UINT32 = 0; /* resolution level value */
    let mut c = dest; /* precinct value */
    let mut l_nb_bytes: OPJ_UINT32 = 0; /* quality layer value */
    let mut compno = (*pi).compno; /* BIO component */
    let mut resno = (*pi).resno;
    let mut precno = (*pi).precno;
    let mut layno = (*pi).layno;
    let mut l_nb_blocks: OPJ_UINT32 = 0;
    let mut band = 0 as *mut opj_tcd_band_t;
    let mut cblk = 0 as *mut opj_tcd_cblk_enc_t;
    let mut pass = 0 as *mut opj_tcd_pass_t;
    let mut tilec: *mut opj_tcd_tilecomp_t =
        &mut *(*tile).comps.offset(compno as isize) as
            *mut opj_tcd_tilecomp_t;
    let mut res: *mut opj_tcd_resolution_t =
        &mut *(*tilec).resolutions.offset(resno as isize) as
            *mut opj_tcd_resolution_t;
    let mut bio = 0 as *mut opj_bio_t;
    let mut packet_empty = 0 as libc::c_int;
    /* <SOP 0xff91> */
    if (*tcp).csty & 0x2 as libc::c_int as libc::c_uint != 0 {
        if length < 6 as libc::c_int as libc::c_uint {
            if p_t2_mode as libc::c_uint ==
                   FINAL_PASS as libc::c_int as libc::c_uint {
                opj_event_msg(p_manager, 1 as libc::c_int,
                              b"opj_t2_encode_packet(): only %u bytes remaining in output buffer. %u needed.\n\x00"
                                  as *const u8 as *const libc::c_char, length,
                              6 as libc::c_int); /* packno is uint32_t */
            }
            return 0 as libc::c_int
        }
        *c.offset(0 as libc::c_int as isize) = 255 as libc::c_int as OPJ_BYTE;
        *c.offset(1 as libc::c_int as isize) = 145 as libc::c_int as OPJ_BYTE;
        *c.offset(2 as libc::c_int as isize) = 0 as libc::c_int as OPJ_BYTE;
        *c.offset(3 as libc::c_int as isize) = 4 as libc::c_int as OPJ_BYTE;
        *c.offset(4 as libc::c_int as isize) =
            ((*tile).packno >> 8 as libc::c_int &
                 0xff as libc::c_int as libc::c_uint) as OPJ_BYTE;
        *c.offset(5 as libc::c_int as isize) =
            ((*tile).packno & 0xff as libc::c_int as libc::c_uint) as
                OPJ_BYTE;
        c = c.offset(6 as libc::c_int as isize);
        length =
            (length as
                 libc::c_uint).wrapping_sub(6 as libc::c_int as libc::c_uint)
                as OPJ_UINT32 as OPJ_UINT32
    }
    /* </SOP> */
    if layno == 0 {
        band = (*res).bands.as_mut_ptr();
        bandno = 0 as libc::c_int as OPJ_UINT32;
        while bandno < (*res).numbands {
            let mut prc = 0 as *mut opj_tcd_precinct_t;
            /* Skip empty bands */
            if !(opj_tcd_is_band_empty(band) != 0) {
                /* Avoid out of bounds access of https://github.com/uclouvain/openjpeg/issues/1294 */
            /* but likely not a proper fix. */
                if precno >= (*res).pw.wrapping_mul((*res).ph) {
                    opj_event_msg(p_manager, 1 as libc::c_int,
                                  b"opj_t2_encode_packet(): accessing precno=%u >= %u\n\x00"
                                      as *const u8 as *const libc::c_char,
                                  precno, (*res).pw.wrapping_mul((*res).ph));
                    return 0 as libc::c_int
                }
                prc =
                    &mut *(*band).precincts.offset(precno as isize) as
                        *mut opj_tcd_precinct_t;
                opj_tgt_reset((*prc).incltree);
                opj_tgt_reset((*prc).imsbtree);
                l_nb_blocks = (*prc).cw.wrapping_mul((*prc).ch);
                cblkno = 0 as libc::c_int as OPJ_UINT32;
                while cblkno < l_nb_blocks {
                    cblk =
                        &mut *(*prc).cblks.enc.offset(cblkno as isize) as
                            *mut opj_tcd_cblk_enc_t;
                    (*cblk).numpasses = 0 as libc::c_int as OPJ_UINT32;
                    opj_tgt_setvalue((*prc).imsbtree, cblkno,
                                     (*band).numbps -
                                         (*cblk).numbps as OPJ_INT32);
                    cblkno = cblkno.wrapping_add(1)
                }
            }
            bandno = bandno.wrapping_add(1);
            band = band.offset(1)
        }
    }
    bio = opj_bio_create();
    if bio.is_null() {
        /* FIXME event manager error callback */
        return 0 as libc::c_int
    } /* Empty header bit */
    opj_bio_init_enc(bio, c, length);
    opj_bio_write(bio,
                  if packet_empty != 0 {
                      0 as libc::c_int
                  } else { 1 as libc::c_int } as OPJ_UINT32,
                  1 as libc::c_int as OPJ_UINT32);
    /* Writing Packet header */
    band = (*res).bands.as_mut_ptr();
    bandno = 0 as libc::c_int as OPJ_UINT32;
    while packet_empty == 0 && bandno < (*res).numbands {
        let mut prc_0 = 0 as *mut opj_tcd_precinct_t;
        /* Skip empty bands */
        if !(opj_tcd_is_band_empty(band) != 0) {
            /* Avoid out of bounds access of https://github.com/uclouvain/openjpeg/issues/1297 */
        /* but likely not a proper fix. */
            if precno >= (*res).pw.wrapping_mul((*res).ph) {
                opj_event_msg(p_manager, 1 as libc::c_int,
                              b"opj_t2_encode_packet(): accessing precno=%u >= %u\n\x00"
                                  as *const u8 as *const libc::c_char, precno,
                              (*res).pw.wrapping_mul((*res).ph));
                return 0 as libc::c_int
            }
            prc_0 =
                &mut *(*band).precincts.offset(precno as isize) as
                    *mut opj_tcd_precinct_t;
            l_nb_blocks = (*prc_0).cw.wrapping_mul((*prc_0).ch);
            cblk = (*prc_0).cblks.enc;
            cblkno = 0 as libc::c_int as OPJ_UINT32;
            while cblkno < l_nb_blocks {
                let mut layer: *mut opj_tcd_layer_t =
                    &mut *(*cblk).layers.offset(layno as isize) as
                        *mut opj_tcd_layer_t;
                if (*cblk).numpasses == 0 && (*layer).numpasses != 0 {
                    opj_tgt_setvalue((*prc_0).incltree, cblkno,
                                     layno as OPJ_INT32);
                }
                cblk = cblk.offset(1);
                cblkno = cblkno.wrapping_add(1)
            }
            cblk = (*prc_0).cblks.enc;
            cblkno = 0 as libc::c_int as OPJ_UINT32;
            while cblkno < l_nb_blocks {
                let mut layer_0: *mut opj_tcd_layer_t =
                    &mut *(*cblk).layers.offset(layno as isize) as
                        *mut opj_tcd_layer_t;
                let mut increment = 0 as libc::c_int as OPJ_UINT32;
                let mut nump = 0 as libc::c_int as OPJ_UINT32;
                let mut len = 0 as libc::c_int as OPJ_UINT32;
                let mut passno: OPJ_UINT32 = 0;
                let mut l_nb_passes: OPJ_UINT32 = 0;
                /* cblk inclusion bits */
                if (*cblk).numpasses == 0 {
                    opj_tgt_encode(bio, (*prc_0).incltree, cblkno,
                                   layno.wrapping_add(1 as libc::c_int as
                                                          libc::c_uint) as
                                       OPJ_INT32);
                } else {
                    opj_bio_write(bio,
                                  ((*layer_0).numpasses !=
                                       0 as libc::c_int as libc::c_uint) as
                                      libc::c_int as OPJ_UINT32,
                                  1 as libc::c_int as OPJ_UINT32);
                }
                /* if cblk not included, go to the next cblk  */
                if (*layer_0).numpasses == 0 {
                    cblk = cblk.offset(1)
                } else {
                    /* if first instance of cblk --> zero bit-planes information */
                    if (*cblk).numpasses == 0 {
                        (*cblk).numlenbits = 3 as libc::c_int as OPJ_UINT32;
                        opj_tgt_encode(bio, (*prc_0).imsbtree, cblkno,
                                       999 as libc::c_int);
                    }
                    /* number of coding passes included */
                    opj_t2_putnumpasses(bio, (*layer_0).numpasses);
                    l_nb_passes =
                        (*cblk).numpasses.wrapping_add((*layer_0).numpasses);
                    pass = (*cblk).passes.offset((*cblk).numpasses as isize);
                    /* computation of the increase of the length indicator and insertion in the header     */
                    passno = (*cblk).numpasses;
                    while passno < l_nb_passes {
                        nump = nump.wrapping_add(1);
                        len =
                            (len as libc::c_uint).wrapping_add((*pass).len) as
                                OPJ_UINT32 as OPJ_UINT32;
                        if (*pass).term() as libc::c_int != 0 ||
                               passno ==
                                   (*cblk).numpasses.wrapping_add((*layer_0).numpasses).wrapping_sub(1
                                                                                                         as
                                                                                                         libc::c_int
                                                                                                         as
                                                                                                         libc::c_uint)
                           {
                            increment =
                                opj_int_max(increment as OPJ_INT32,
                                            opj_int_floorlog2(len as
                                                                  OPJ_INT32) +
                                                1 as libc::c_int -
                                                ((*cblk).numlenbits as
                                                     OPJ_INT32 +
                                                     opj_int_floorlog2(nump as
                                                                           OPJ_INT32)))
                                    as OPJ_UINT32;
                            len = 0 as libc::c_int as OPJ_UINT32;
                            nump = 0 as libc::c_int as OPJ_UINT32
                        }
                        pass = pass.offset(1);
                        passno = passno.wrapping_add(1)
                    }
                    opj_t2_putcommacode(bio, increment as OPJ_INT32);
                    /* computation of the new Length indicator */
                    (*cblk).numlenbits =
                        ((*cblk).numlenbits as
                             libc::c_uint).wrapping_add(increment) as
                            OPJ_UINT32 as OPJ_UINT32;
                    pass = (*cblk).passes.offset((*cblk).numpasses as isize);
                    /* insertion of the codeword segment length */
                    passno = (*cblk).numpasses;
                    while passno < l_nb_passes {
                        nump = nump.wrapping_add(1);
                        len =
                            (len as libc::c_uint).wrapping_add((*pass).len) as
                                OPJ_UINT32 as OPJ_UINT32;
                        if (*pass).term() as libc::c_int != 0 ||
                               passno ==
                                   (*cblk).numpasses.wrapping_add((*layer_0).numpasses).wrapping_sub(1
                                                                                                         as
                                                                                                         libc::c_int
                                                                                                         as
                                                                                                         libc::c_uint)
                           {
                            opj_bio_write(bio, len,
                                          (*cblk).numlenbits.wrapping_add(opj_int_floorlog2(nump
                                                                                                as
                                                                                                OPJ_INT32)
                                                                              as
                                                                              OPJ_UINT32));
                            len = 0 as libc::c_int as OPJ_UINT32;
                            nump = 0 as libc::c_int as OPJ_UINT32
                        }
                        pass = pass.offset(1);
                        passno = passno.wrapping_add(1)
                    }
                    cblk = cblk.offset(1)
                }
                cblkno = cblkno.wrapping_add(1)
            }
        }
        bandno = bandno.wrapping_add(1);
        band = band.offset(1)
    }
    if opj_bio_flush(bio) == 0 {
        opj_bio_destroy(bio);
        return 0 as libc::c_int
        /* modified to eliminate longjmp !! */
    }
    l_nb_bytes = opj_bio_numbytes(bio) as OPJ_UINT32;
    c = c.offset(l_nb_bytes as isize);
    length =
        (length as libc::c_uint).wrapping_sub(l_nb_bytes) as OPJ_UINT32 as
            OPJ_UINT32;
    opj_bio_destroy(bio);
    /* <EPH 0xff92> */
    if (*tcp).csty & 0x4 as libc::c_int as libc::c_uint != 0 {
        if length < 2 as libc::c_int as libc::c_uint {
            if p_t2_mode as libc::c_uint ==
                   FINAL_PASS as libc::c_int as libc::c_uint {
                opj_event_msg(p_manager, 1 as libc::c_int,
                              b"opj_t2_encode_packet(): only %u bytes remaining in output buffer. %u needed.\n\x00"
                                  as *const u8 as *const libc::c_char, length,
                              2 as libc::c_int);
            }
            return 0 as libc::c_int
        }
        *c.offset(0 as libc::c_int as isize) = 255 as libc::c_int as OPJ_BYTE;
        *c.offset(1 as libc::c_int as isize) = 146 as libc::c_int as OPJ_BYTE;
        c = c.offset(2 as libc::c_int as isize);
        length =
            (length as
                 libc::c_uint).wrapping_sub(2 as libc::c_int as libc::c_uint)
                as OPJ_UINT32 as OPJ_UINT32
    }
    /* </EPH> */
    /* << INDEX */
    /* End of packet header position. Currently only represents the distance to start of packet
       Will be updated later by incrementing with packet start value*/
    if !cstr_info.is_null() && (*cstr_info).index_write != 0 {
        let mut info_PK: *mut opj_packet_info_t =
            &mut *(*(*cstr_info).tile.offset(tileno as
                                                 isize)).packet.offset((*cstr_info).packno
                                                                           as
                                                                           isize)
                as *mut opj_packet_info_t;
        (*info_PK).end_ph_pos =
            c.wrapping_offset_from(dest) as libc::c_long as OPJ_INT32 as
                OPJ_OFF_T
    }
    /* INDEX >> */
    /* Writing the packet body */
    band = (*res).bands.as_mut_ptr();
    bandno = 0 as libc::c_int as OPJ_UINT32;
    while packet_empty == 0 && bandno < (*res).numbands {
        let mut prc_1 = 0 as *mut opj_tcd_precinct_t;
        /* Skip empty bands */
        if !(opj_tcd_is_band_empty(band) != 0) {
            prc_1 =
                &mut *(*band).precincts.offset(precno as isize) as
                    *mut opj_tcd_precinct_t;
            l_nb_blocks = (*prc_1).cw.wrapping_mul((*prc_1).ch);
            cblk = (*prc_1).cblks.enc;
            cblkno = 0 as libc::c_int as OPJ_UINT32;
            while cblkno < l_nb_blocks {
                let mut layer_1: *mut opj_tcd_layer_t =
                    &mut *(*cblk).layers.offset(layno as isize) as
                        *mut opj_tcd_layer_t;
                if (*layer_1).numpasses == 0 {
                    cblk = cblk.offset(1)
                } else {
                    if (*layer_1).len > length {
                        if p_t2_mode as libc::c_uint ==
                               FINAL_PASS as libc::c_int as libc::c_uint {
                            opj_event_msg(p_manager, 1 as libc::c_int,
                                          b"opj_t2_encode_packet(): only %u bytes remaining in output buffer. %u needed.\n\x00"
                                              as *const u8 as
                                              *const libc::c_char, length,
                                          (*layer_1).len);
                        }
                        return 0 as libc::c_int
                    }
                    memcpy(c as *mut libc::c_void,
                           (*layer_1).data as *const libc::c_void,
                           (*layer_1).len as libc::c_ulong);
                    (*cblk).numpasses =
                        ((*cblk).numpasses as
                             libc::c_uint).wrapping_add((*layer_1).numpasses)
                            as OPJ_UINT32 as OPJ_UINT32;
                    c = c.offset((*layer_1).len as isize);
                    length =
                        (length as libc::c_uint).wrapping_sub((*layer_1).len)
                            as OPJ_UINT32 as OPJ_UINT32;
                    /* INDEX >> */
                    /* << INDEX */
                    if !cstr_info.is_null() && (*cstr_info).index_write != 0 {
                        let mut info_PK_0: *mut opj_packet_info_t =
                            &mut *(*(*cstr_info).tile.offset(tileno as
                                                                 isize)).packet.offset((*cstr_info).packno
                                                                                           as
                                                                                           isize)
                                as *mut opj_packet_info_t;
                        (*info_PK_0).disto += (*layer_1).disto;
                        if (*cstr_info).D_max < (*info_PK_0).disto {
                            (*cstr_info).D_max = (*info_PK_0).disto
                        }
                    }
                    cblk = cblk.offset(1)
                }
                cblkno = cblkno.wrapping_add(1)
            }
        }
        bandno = bandno.wrapping_add(1);
        band = band.offset(1)
    }
    if c >= dest {
    } else {
        __assert_fail(b"c >= dest\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/t2.c\x00" as *const u8
                          as *const libc::c_char,
                      1000 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 189],
                                                &[libc::c_char; 189]>(b"OPJ_BOOL opj_t2_encode_packet(OPJ_UINT32, opj_tcd_tile_t *, opj_tcp_t *, opj_pi_iterator_t *, OPJ_BYTE *, OPJ_UINT32 *, OPJ_UINT32, opj_codestream_info_t *, J2K_T2_MODE, opj_event_mgr_t *)\x00")).as_ptr());
    }
    *p_data_written =
        (*p_data_written as
             libc::c_uint).wrapping_add(c.wrapping_offset_from(dest) as
                                            libc::c_long as OPJ_UINT32) as
            OPJ_UINT32 as OPJ_UINT32;
    return 1 as libc::c_int;
}
unsafe extern "C" fn opj_t2_skip_packet(mut p_t2: *mut opj_t2_t,
                                        mut p_tile: *mut opj_tcd_tile_t,
                                        mut p_tcp: *mut opj_tcp_t,
                                        mut p_pi: *mut opj_pi_iterator_t,
                                        mut p_src: *mut OPJ_BYTE,
                                        mut p_data_read: *mut OPJ_UINT32,
                                        mut p_max_length: OPJ_UINT32,
                                        mut p_pack_info:
                                            *mut opj_packet_info_t,
                                        mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut l_read_data: OPJ_BOOL = 0;
    let mut l_nb_bytes_read = 0 as libc::c_int as OPJ_UINT32;
    let mut l_nb_total_bytes_read = 0 as libc::c_int as OPJ_UINT32;
    *p_data_read = 0 as libc::c_int as OPJ_UINT32;
    if opj_t2_read_packet_header(p_t2, p_tile, p_tcp, p_pi, &mut l_read_data,
                                 p_src, &mut l_nb_bytes_read, p_max_length,
                                 p_pack_info, p_manager) == 0 {
        return 0 as libc::c_int
    }
    p_src = p_src.offset(l_nb_bytes_read as isize);
    l_nb_total_bytes_read =
        (l_nb_total_bytes_read as libc::c_uint).wrapping_add(l_nb_bytes_read)
            as OPJ_UINT32 as OPJ_UINT32;
    p_max_length =
        (p_max_length as libc::c_uint).wrapping_sub(l_nb_bytes_read) as
            OPJ_UINT32 as OPJ_UINT32;
    /* we should read data for the packet */
    if l_read_data != 0 {
        l_nb_bytes_read = 0 as libc::c_int as OPJ_UINT32;
        if opj_t2_skip_packet_data(p_t2, p_tile, p_pi, &mut l_nb_bytes_read,
                                   p_max_length, p_pack_info, p_manager) == 0
           {
            return 0 as libc::c_int
        }
        l_nb_total_bytes_read =
            (l_nb_total_bytes_read as
                 libc::c_uint).wrapping_add(l_nb_bytes_read) as OPJ_UINT32 as
                OPJ_UINT32
    }
    *p_data_read = l_nb_total_bytes_read;
    return 1 as libc::c_int;
}
unsafe extern "C" fn opj_t2_read_packet_header(mut p_t2: *mut opj_t2_t,
                                               mut p_tile:
                                                   *mut opj_tcd_tile_t,
                                               mut p_tcp: *mut opj_tcp_t,
                                               mut p_pi:
                                                   *mut opj_pi_iterator_t,
                                               mut p_is_data_present:
                                                   *mut OPJ_BOOL,
                                               mut p_src_data: *mut OPJ_BYTE,
                                               mut p_data_read:
                                                   *mut OPJ_UINT32,
                                               mut p_max_length: OPJ_UINT32,
                                               mut p_pack_info:
                                                   *mut opj_packet_info_t,
                                               mut p_manager:
                                                   *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    /* loop */
    let mut bandno: OPJ_UINT32 = 0; /* BIO component */
    let mut cblkno: OPJ_UINT32 = 0;
    let mut l_nb_code_blocks: OPJ_UINT32 = 0;
    let mut l_remaining_length: OPJ_UINT32 = 0;
    let mut l_header_length: OPJ_UINT32 = 0;
    let mut l_modified_length_ptr = 0 as *mut OPJ_UINT32;
    let mut l_current_data = p_src_data;
    let mut l_cp = (*p_t2).cp;
    let mut l_bio = 0 as *mut opj_bio_t;
    let mut l_band = 0 as *mut opj_tcd_band_t;
    let mut l_cblk = 0 as *mut opj_tcd_cblk_dec_t;
    let mut l_res: *mut opj_tcd_resolution_t =
        &mut *(*(*p_tile).comps.offset((*p_pi).compno as
                                           isize)).resolutions.offset((*p_pi).resno
                                                                          as
                                                                          isize)
            as *mut opj_tcd_resolution_t;
    let mut l_header_data = 0 as *mut OPJ_BYTE;
    let mut l_header_data_start = 0 as *mut *mut OPJ_BYTE;
    let mut l_present: OPJ_UINT32 = 0;
    if (*p_pi).layno == 0 as libc::c_int as libc::c_uint {
        l_band = (*l_res).bands.as_mut_ptr();
        /* reset tagtrees */
        bandno = 0 as libc::c_int as OPJ_UINT32;
        while bandno < (*l_res).numbands {
            if opj_tcd_is_band_empty(l_band) == 0 {
                let mut l_prc: *mut opj_tcd_precinct_t =
                    &mut *(*l_band).precincts.offset((*p_pi).precno as isize)
                        as *mut opj_tcd_precinct_t;
                if !(((*p_pi).precno as libc::c_ulong) <
                         ((*l_band).precincts_data_size as
                              libc::c_ulong).wrapping_div(::std::mem::size_of::<opj_tcd_precinct_t>()
                                                              as
                                                              libc::c_ulong))
                   {
                    opj_event_msg(p_manager, 1 as libc::c_int,
                                  b"Invalid precinct\n\x00" as *const u8 as
                                      *const libc::c_char);
                    return 0 as libc::c_int
                }
                opj_tgt_reset((*l_prc).incltree);
                opj_tgt_reset((*l_prc).imsbtree);
                l_cblk = (*l_prc).cblks.dec;
                l_nb_code_blocks = (*l_prc).cw.wrapping_mul((*l_prc).ch);
                cblkno = 0 as libc::c_int as OPJ_UINT32;
                while cblkno < l_nb_code_blocks {
                    (*l_cblk).numsegs = 0 as libc::c_int as OPJ_UINT32;
                    (*l_cblk).real_num_segs = 0 as libc::c_int as OPJ_UINT32;
                    l_cblk = l_cblk.offset(1);
                    cblkno = cblkno.wrapping_add(1)
                }
            }
            l_band = l_band.offset(1);
            bandno = bandno.wrapping_add(1)
        }
    }
    /* SOP markers */
    if (*p_tcp).csty & 0x2 as libc::c_int as libc::c_uint != 0 {
        if p_max_length < 6 as libc::c_int as libc::c_uint {
            opj_event_msg(p_manager, 2 as libc::c_int,
                          b"Not enough space for expected SOP marker\n\x00" as
                              *const u8 as *const libc::c_char);
        } else if *l_current_data as libc::c_int != 0xff as libc::c_int ||
                      *l_current_data.offset(1 as libc::c_int as isize) as
                          libc::c_int != 0x91 as libc::c_int {
            opj_event_msg(p_manager, 2 as libc::c_int,
                          b"Expected SOP marker\n\x00" as *const u8 as
                              *const libc::c_char);
        } else {
            l_current_data = l_current_data.offset(6 as libc::c_int as isize)
        }
        /* * TODO : check the Nsop value */
    }
    /*
    When the marker PPT/PPM is used the packet header are store in PPT/PPM marker
    This part deal with this characteristic
    step 1: Read packet header in the saved structure
    step 2: Return to codestream for decoding
    */
    l_bio = opj_bio_create();
    if l_bio.is_null() { return 0 as libc::c_int }
    if (*l_cp).ppm() as libc::c_int == 1 as libc::c_int {
        /* PPM */
        l_header_data_start = &mut (*l_cp).ppm_data; /* Normal Case */
        l_header_data = *l_header_data_start;
        l_modified_length_ptr = &mut (*l_cp).ppm_len
    } else if (*p_tcp).ppt() as libc::c_int == 1 as libc::c_int {
        /* PPT */
        l_header_data_start = &mut (*p_tcp).ppt_data;
        l_header_data = *l_header_data_start;
        l_modified_length_ptr = &mut (*p_tcp).ppt_len
    } else {
        l_header_data_start = &mut l_current_data;
        l_header_data = *l_header_data_start;
        l_remaining_length =
            p_src_data.offset(p_max_length as
                                  isize).wrapping_offset_from(l_header_data)
                as libc::c_long as OPJ_UINT32;
        l_modified_length_ptr = &mut l_remaining_length
    }
    opj_bio_init_dec(l_bio, l_header_data, *l_modified_length_ptr);
    l_present = opj_bio_read(l_bio, 1 as libc::c_int as OPJ_UINT32);
    opj_null_jas_fprintf(stderr,
                         b"present=%d \n\x00" as *const u8 as
                             *const libc::c_char, l_present);
    if l_present == 0 {
        /* TODO MSD: no test to control the output of this function*/
        opj_bio_inalign(l_bio);
        l_header_data =
            l_header_data.offset(opj_bio_numbytes(l_bio) as isize);
        opj_bio_destroy(l_bio);
        /* EPH markers */
        if (*p_tcp).csty & 0x4 as libc::c_int as libc::c_uint != 0 {
            if (*l_modified_length_ptr).wrapping_sub(l_header_data.wrapping_offset_from(*l_header_data_start)
                                                         as libc::c_long as
                                                         OPJ_UINT32) <
                   2 as libc::c_uint {
                opj_event_msg(p_manager, 2 as libc::c_int,
                              b"Not enough space for expected EPH marker\n\x00"
                                  as *const u8 as *const libc::c_char);
            } else if *l_header_data as libc::c_int != 0xff as libc::c_int ||
                          *l_header_data.offset(1 as libc::c_int as isize) as
                              libc::c_int != 0x92 as libc::c_int {
                opj_event_msg(p_manager, 2 as libc::c_int,
                              b"Expected EPH marker\n\x00" as *const u8 as
                                  *const libc::c_char);
            } else {
                l_header_data =
                    l_header_data.offset(2 as libc::c_int as isize)
            }
        }
        l_header_length =
            l_header_data.wrapping_offset_from(*l_header_data_start) as
                libc::c_long as OPJ_UINT32;
        *l_modified_length_ptr =
            (*l_modified_length_ptr as
                 libc::c_uint).wrapping_sub(l_header_length) as OPJ_UINT32 as
                OPJ_UINT32;
        *l_header_data_start =
            (*l_header_data_start).offset(l_header_length as isize);
        /* << INDEX */
        /* End of packet header position. Currently only represents the distance to start of packet
           Will be updated later by incrementing with packet start value */
        if !p_pack_info.is_null() {
            (*p_pack_info).end_ph_pos =
                l_current_data.wrapping_offset_from(p_src_data) as
                    libc::c_long as OPJ_INT32 as OPJ_OFF_T
        }
        /* INDEX >> */
        *p_is_data_present = 0 as libc::c_int;
        *p_data_read =
            l_current_data.wrapping_offset_from(p_src_data) as libc::c_long as
                OPJ_UINT32;
        return 1 as libc::c_int
    }
    l_band = (*l_res).bands.as_mut_ptr();
    bandno = 0 as libc::c_int as OPJ_UINT32;
    while bandno < (*l_res).numbands {
        let mut l_prc_0: *mut opj_tcd_precinct_t =
            &mut *(*l_band).precincts.offset((*p_pi).precno as isize) as
                *mut opj_tcd_precinct_t;
        if !(opj_tcd_is_band_empty(l_band) != 0) {
            l_nb_code_blocks = (*l_prc_0).cw.wrapping_mul((*l_prc_0).ch);
            l_cblk = (*l_prc_0).cblks.dec;
            cblkno = 0 as libc::c_int as OPJ_UINT32;
            while cblkno < l_nb_code_blocks {
                let mut l_included: OPJ_UINT32 = 0;
                let mut l_increment: OPJ_UINT32 = 0;
                let mut l_segno: OPJ_UINT32 = 0;
                let mut n: OPJ_INT32 = 0;
                /* if cblk not yet included before --> inclusion tagtree */
                if (*l_cblk).numsegs == 0 {
                    l_included =
                        opj_tgt_decode(l_bio, (*l_prc_0).incltree, cblkno,
                                       (*p_pi).layno.wrapping_add(1 as
                                                                      libc::c_int
                                                                      as
                                                                      libc::c_uint)
                                           as OPJ_INT32)
                    /* else one bit */
                } else {
                    l_included =
                        opj_bio_read(l_bio, 1 as libc::c_int as OPJ_UINT32)
                }
                /* if cblk not included */
                if l_included == 0 {
                    (*l_cblk).numnewpasses = 0 as libc::c_int as OPJ_UINT32;
                    l_cblk = l_cblk.offset(1);
                    opj_null_jas_fprintf(stderr,
                                         b"included=%d \n\x00" as *const u8 as
                                             *const libc::c_char, l_included);
                } else {
                    /* if cblk not yet included --> zero-bitplane tagtree */
                    if (*l_cblk).numsegs == 0 {
                        let mut i = 0 as libc::c_int as OPJ_UINT32;
                        while opj_tgt_decode(l_bio, (*l_prc_0).imsbtree,
                                             cblkno, i as OPJ_INT32) == 0 {
                            i = i.wrapping_add(1)
                        }
                        (*l_cblk).Mb = (*l_band).numbps as OPJ_UINT32;
                        (*l_cblk).numbps =
                            ((*l_band).numbps as
                                 OPJ_UINT32).wrapping_add(1 as libc::c_int as
                                                              libc::c_uint).wrapping_sub(i);
                        (*l_cblk).numlenbits = 3 as libc::c_int as OPJ_UINT32
                    }
                    /* number of coding passes */
                    (*l_cblk).numnewpasses = opj_t2_getnumpasses(l_bio);
                    l_increment = opj_t2_getcommacode(l_bio);
                    /* length indicator increment */
                    (*l_cblk).numlenbits =
                        ((*l_cblk).numlenbits as
                             libc::c_uint).wrapping_add(l_increment) as
                            OPJ_UINT32 as OPJ_UINT32;
                    l_segno = 0 as libc::c_int as OPJ_UINT32;
                    if (*l_cblk).numsegs == 0 {
                        if opj_t2_init_seg(l_cblk, l_segno,
                                           (*(*p_tcp).tccps.offset((*p_pi).compno
                                                                       as
                                                                       isize)).cblksty,
                                           1 as libc::c_int as OPJ_UINT32) ==
                               0 {
                            opj_bio_destroy(l_bio);
                            return 0 as libc::c_int
                        }
                    } else {
                        l_segno =
                            (*l_cblk).numsegs.wrapping_sub(1 as libc::c_int as
                                                               libc::c_uint);
                        if (*(*l_cblk).segs.offset(l_segno as
                                                       isize)).numpasses ==
                               (*(*l_cblk).segs.offset(l_segno as
                                                           isize)).maxpasses {
                            l_segno = l_segno.wrapping_add(1);
                            if opj_t2_init_seg(l_cblk, l_segno,
                                               (*(*p_tcp).tccps.offset((*p_pi).compno
                                                                           as
                                                                           isize)).cblksty,
                                               0 as libc::c_int as OPJ_UINT32)
                                   == 0 {
                                opj_bio_destroy(l_bio);
                                return 0 as libc::c_int
                            }
                        }
                    }
                    n = (*l_cblk).numnewpasses as OPJ_INT32;
                    if (*(*p_tcp).tccps.offset((*p_pi).compno as
                                                   isize)).cblksty &
                           0x40 as libc::c_int as libc::c_uint !=
                           0 as libc::c_int as libc::c_uint {
                        loop  {
                            let mut bit_number: OPJ_UINT32 = 0;
                            (*(*l_cblk).segs.offset(l_segno as
                                                        isize)).numnewpasses =
                                if l_segno == 0 as libc::c_int as libc::c_uint
                                   {
                                    1 as libc::c_int as libc::c_uint
                                } else { n as OPJ_UINT32 };
                            bit_number =
                                (*l_cblk).numlenbits.wrapping_add(opj_uint_floorlog2((*(*l_cblk).segs.offset(l_segno
                                                                                                                 as
                                                                                                                 isize)).numnewpasses));
                            if bit_number > 32 as libc::c_int as libc::c_uint
                               {
                                opj_event_msg(p_manager, 1 as libc::c_int,
                                              b"Invalid bit number %d in opj_t2_read_packet_header()\n\x00"
                                                  as *const u8 as
                                                  *const libc::c_char,
                                              bit_number);
                                opj_bio_destroy(l_bio);
                                return 0 as libc::c_int
                            }
                            (*(*l_cblk).segs.offset(l_segno as isize)).newlen
                                = opj_bio_read(l_bio, bit_number);
                            opj_null_jas_fprintf(stderr,
                                                 b"included=%d numnewpasses=%d increment=%d len=%d \n\x00"
                                                     as *const u8 as
                                                     *const libc::c_char,
                                                 l_included,
                                                 (*(*l_cblk).segs.offset(l_segno
                                                                             as
                                                                             isize)).numnewpasses,
                                                 l_increment,
                                                 (*(*l_cblk).segs.offset(l_segno
                                                                             as
                                                                             isize)).newlen);
                            n -=
                                (*(*l_cblk).segs.offset(l_segno as
                                                            isize)).numnewpasses
                                    as OPJ_INT32;
                            if n > 0 as libc::c_int {
                                l_segno = l_segno.wrapping_add(1);
                                if opj_t2_init_seg(l_cblk, l_segno,
                                                   (*(*p_tcp).tccps.offset((*p_pi).compno
                                                                               as
                                                                               isize)).cblksty,
                                                   0 as libc::c_int as
                                                       OPJ_UINT32) == 0 {
                                    opj_bio_destroy(l_bio);
                                    return 0 as libc::c_int
                                }
                            }
                            if !(n > 0 as libc::c_int) { break ; }
                        }
                    } else {
                        loop  {
                            let mut bit_number_0: OPJ_UINT32 = 0;
                            (*(*l_cblk).segs.offset(l_segno as
                                                        isize)).numnewpasses =
                                opj_int_min((*(*l_cblk).segs.offset(l_segno as
                                                                        isize)).maxpasses.wrapping_sub((*(*l_cblk).segs.offset(l_segno
                                                                                                                                   as
                                                                                                                                   isize)).numpasses)
                                                as OPJ_INT32, n) as
                                    OPJ_UINT32;
                            bit_number_0 =
                                (*l_cblk).numlenbits.wrapping_add(opj_uint_floorlog2((*(*l_cblk).segs.offset(l_segno
                                                                                                                 as
                                                                                                                 isize)).numnewpasses));
                            if bit_number_0 >
                                   32 as libc::c_int as libc::c_uint {
                                opj_event_msg(p_manager, 1 as libc::c_int,
                                              b"Invalid bit number %d in opj_t2_read_packet_header()\n\x00"
                                                  as *const u8 as
                                                  *const libc::c_char,
                                              bit_number_0);
                                opj_bio_destroy(l_bio);
                                return 0 as libc::c_int
                            }
                            (*(*l_cblk).segs.offset(l_segno as isize)).newlen
                                = opj_bio_read(l_bio, bit_number_0);
                            opj_null_jas_fprintf(stderr,
                                                 b"included=%d numnewpasses=%d increment=%d len=%d \n\x00"
                                                     as *const u8 as
                                                     *const libc::c_char,
                                                 l_included,
                                                 (*(*l_cblk).segs.offset(l_segno
                                                                             as
                                                                             isize)).numnewpasses,
                                                 l_increment,
                                                 (*(*l_cblk).segs.offset(l_segno
                                                                             as
                                                                             isize)).newlen);
                            n -=
                                (*(*l_cblk).segs.offset(l_segno as
                                                            isize)).numnewpasses
                                    as OPJ_INT32;
                            if n > 0 as libc::c_int {
                                l_segno = l_segno.wrapping_add(1);
                                if opj_t2_init_seg(l_cblk, l_segno,
                                                   (*(*p_tcp).tccps.offset((*p_pi).compno
                                                                               as
                                                                               isize)).cblksty,
                                                   0 as libc::c_int as
                                                       OPJ_UINT32) == 0 {
                                    opj_bio_destroy(l_bio);
                                    return 0 as libc::c_int
                                }
                            }
                            if !(n > 0 as libc::c_int) { break ; }
                        }
                    }
                    l_cblk = l_cblk.offset(1)
                }
                cblkno = cblkno.wrapping_add(1)
            }
        }
        bandno = bandno.wrapping_add(1);
        l_band = l_band.offset(1)
    }
    if opj_bio_inalign(l_bio) == 0 {
        opj_bio_destroy(l_bio);
        return 0 as libc::c_int
    }
    l_header_data = l_header_data.offset(opj_bio_numbytes(l_bio) as isize);
    opj_bio_destroy(l_bio);
    /* EPH markers */
    if (*p_tcp).csty & 0x4 as libc::c_int as libc::c_uint != 0 {
        if (*l_modified_length_ptr).wrapping_sub(l_header_data.wrapping_offset_from(*l_header_data_start)
                                                     as libc::c_long as
                                                     OPJ_UINT32) <
               2 as libc::c_uint {
            opj_event_msg(p_manager, 2 as libc::c_int,
                          b"Not enough space for expected EPH marker\n\x00" as
                              *const u8 as *const libc::c_char);
        } else if *l_header_data as libc::c_int != 0xff as libc::c_int ||
                      *l_header_data.offset(1 as libc::c_int as isize) as
                          libc::c_int != 0x92 as libc::c_int {
            opj_event_msg(p_manager, 2 as libc::c_int,
                          b"Expected EPH marker\n\x00" as *const u8 as
                              *const libc::c_char);
        } else {
            l_header_data = l_header_data.offset(2 as libc::c_int as isize)
        }
    }
    l_header_length =
        l_header_data.wrapping_offset_from(*l_header_data_start) as
            libc::c_long as OPJ_UINT32;
    opj_null_jas_fprintf(stderr,
                         b"hdrlen=%d \n\x00" as *const u8 as
                             *const libc::c_char, l_header_length);
    opj_null_jas_fprintf(stderr,
                         b"packet body\n\x00" as *const u8 as
                             *const libc::c_char);
    *l_modified_length_ptr =
        (*l_modified_length_ptr as libc::c_uint).wrapping_sub(l_header_length)
            as OPJ_UINT32 as OPJ_UINT32;
    *l_header_data_start =
        (*l_header_data_start).offset(l_header_length as isize);
    /* << INDEX */
    /* End of packet header position. Currently only represents the distance to start of packet
     Will be updated later by incrementing with packet start value */
    if !p_pack_info.is_null() {
        (*p_pack_info).end_ph_pos =
            l_current_data.wrapping_offset_from(p_src_data) as libc::c_long as
                OPJ_INT32 as OPJ_OFF_T
    }
    /* INDEX >> */
    *p_is_data_present = 1 as libc::c_int; /* next code_block */
    *p_data_read =
        l_current_data.wrapping_offset_from(p_src_data) as libc::c_long as
            OPJ_UINT32;
    return 1 as libc::c_int;
}
unsafe extern "C" fn opj_t2_read_packet_data(mut p_t2: *mut opj_t2_t,
                                             mut p_tile: *mut opj_tcd_tile_t,
                                             mut p_pi: *mut opj_pi_iterator_t,
                                             mut p_src_data: *mut OPJ_BYTE,
                                             mut p_data_read: *mut OPJ_UINT32,
                                             mut p_max_length: OPJ_UINT32,
                                             mut _pack_info:
                                                 *mut opj_packet_info_t,
                                             mut p_manager:
                                                 *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut bandno: OPJ_UINT32 = 0;
    let mut cblkno: OPJ_UINT32 = 0;
    let mut l_nb_code_blocks: OPJ_UINT32 = 0;
    let mut l_current_data = p_src_data;
    let mut l_band = 0 as *mut opj_tcd_band_t;
    let mut l_cblk = 0 as *mut opj_tcd_cblk_dec_t;
    let mut l_res: *mut opj_tcd_resolution_t =
        &mut *(*(*p_tile).comps.offset((*p_pi).compno as
                                           isize)).resolutions.offset((*p_pi).resno
                                                                          as
                                                                          isize)
            as *mut opj_tcd_resolution_t;
    let mut partial_buffer = 0 as libc::c_int;
    l_band = (*l_res).bands.as_mut_ptr();
    bandno = 0 as libc::c_int as OPJ_UINT32;
    while bandno < (*l_res).numbands {
        let mut l_prc: *mut opj_tcd_precinct_t =
            &mut *(*l_band).precincts.offset((*p_pi).precno as isize) as
                *mut opj_tcd_precinct_t;
        if (*l_band).x1 - (*l_band).x0 == 0 as libc::c_int ||
               (*l_band).y1 - (*l_band).y0 == 0 as libc::c_int {
            l_band = l_band.offset(1)
        } else {
            l_nb_code_blocks = (*l_prc).cw.wrapping_mul((*l_prc).ch);
            l_cblk = (*l_prc).cblks.dec;
            cblkno = 0 as libc::c_int as OPJ_UINT32;
            while cblkno < l_nb_code_blocks {
                let mut l_seg = 0 as *mut opj_tcd_seg_t;
                // if we have a partial data stream, set numchunks to zero
            // since we have no data to actually decode.
                if partial_buffer != 0 {
                    (*l_cblk).numchunks = 0 as libc::c_int as OPJ_UINT32
                }
                if (*l_cblk).numnewpasses == 0 {
                    /* nothing to do */
                    l_cblk = l_cblk.offset(1)
                } else {
                    if (*l_cblk).numsegs == 0 {
                        l_seg = (*l_cblk).segs;
                        (*l_cblk).numsegs = (*l_cblk).numsegs.wrapping_add(1)
                    } else {
                        l_seg =
                            &mut *(*l_cblk).segs.offset((*l_cblk).numsegs.wrapping_sub(1
                                                                                           as
                                                                                           libc::c_int
                                                                                           as
                                                                                           libc::c_uint)
                                                            as isize) as
                                *mut opj_tcd_seg_t;
                        if (*l_seg).numpasses == (*l_seg).maxpasses {
                            l_seg = l_seg.offset(1);
                            (*l_cblk).numsegs =
                                (*l_cblk).numsegs.wrapping_add(1)
                        }
                    }
                    loop 
                         /* Check possible overflow (on l_current_data only, assumes input args already checked) then size */
                         {
                        if (l_current_data as
                                OPJ_SIZE_T).wrapping_add((*l_seg).newlen as
                                                             OPJ_SIZE_T) <
                               l_current_data as OPJ_SIZE_T ||
                               l_current_data.offset((*l_seg).newlen as isize)
                                   > p_src_data.offset(p_max_length as isize)
                               || partial_buffer != 0 {
                            if (*(*p_t2).cp).strict != 0 {
                                opj_event_msg(p_manager, 1 as libc::c_int,
                                              b"read: segment too long (%d) with max (%d) for codeblock %d (p=%d, b=%d, r=%d, c=%d)\n\x00"
                                                  as *const u8 as
                                                  *const libc::c_char,
                                              (*l_seg).newlen, p_max_length,
                                              cblkno, (*p_pi).precno, bandno,
                                              (*p_pi).resno, (*p_pi).compno);
                                return 0 as libc::c_int
                            } else {
                                opj_event_msg(p_manager, 2 as libc::c_int,
                                              b"read: segment too long (%d) with max (%d) for codeblock %d (p=%d, b=%d, r=%d, c=%d)\n\x00"
                                                  as *const u8 as
                                                  *const libc::c_char,
                                              (*l_seg).newlen, p_max_length,
                                              cblkno, (*p_pi).precno, bandno,
                                              (*p_pi).resno, (*p_pi).compno);
                                // skip this codeblock since it is a partial read
                                partial_buffer = 1 as libc::c_int;
                                (*l_cblk).numchunks =
                                    0 as libc::c_int as OPJ_UINT32;
                                (*l_seg).numpasses =
                                    ((*l_seg).numpasses as
                                         libc::c_uint).wrapping_add((*l_seg).numnewpasses)
                                        as OPJ_UINT32 as OPJ_UINT32;
                                (*l_cblk).numnewpasses =
                                    ((*l_cblk).numnewpasses as
                                         libc::c_uint).wrapping_sub((*l_seg).numnewpasses)
                                        as OPJ_UINT32 as OPJ_UINT32;
                                if (*l_cblk).numnewpasses >
                                       0 as libc::c_int as libc::c_uint {
                                    l_seg = l_seg.offset(1);
                                    (*l_cblk).numsegs =
                                        (*l_cblk).numsegs.wrapping_add(1);
                                    break ;
                                }
                            }
                        } else {
                            /* USE_JPWL */
                            if (*l_cblk).numchunks == (*l_cblk).numchunksalloc
                               {
                                let mut l_numchunksalloc =
                                    (*l_cblk).numchunksalloc.wrapping_mul(2 as
                                                                              libc::c_int
                                                                              as
                                                                              libc::c_uint).wrapping_add(1
                                                                                                             as
                                                                                                             libc::c_int
                                                                                                             as
                                                                                                             libc::c_uint);
                                let mut l_chunks =
                                    opj_realloc((*l_cblk).chunks as
                                                    *mut libc::c_void,
                                                (l_numchunksalloc as
                                                     libc::c_ulong).wrapping_mul(::std::mem::size_of::<opj_tcd_seg_data_chunk_t>()
                                                                                     as
                                                                                     libc::c_ulong))
                                        as *mut opj_tcd_seg_data_chunk_t;
                                if l_chunks.is_null() {
                                    opj_event_msg(p_manager, 1 as libc::c_int,
                                                  b"cannot allocate opj_tcd_seg_data_chunk_t* array\x00"
                                                      as *const u8 as
                                                      *const libc::c_char);
                                    return 0 as libc::c_int
                                }
                                (*l_cblk).chunks = l_chunks;
                                (*l_cblk).numchunksalloc = l_numchunksalloc
                            }
                            let ref mut fresh0 =
                                (*(*l_cblk).chunks.offset((*l_cblk).numchunks
                                                              as isize)).data;
                            *fresh0 = l_current_data;
                            (*(*l_cblk).chunks.offset((*l_cblk).numchunks as
                                                          isize)).len =
                                (*l_seg).newlen;
                            (*l_cblk).numchunks =
                                (*l_cblk).numchunks.wrapping_add(1);
                            l_current_data =
                                l_current_data.offset((*l_seg).newlen as
                                                          isize);
                            (*l_seg).len =
                                ((*l_seg).len as
                                     libc::c_uint).wrapping_add((*l_seg).newlen)
                                    as OPJ_UINT32 as OPJ_UINT32;
                            (*l_seg).numpasses =
                                ((*l_seg).numpasses as
                                     libc::c_uint).wrapping_add((*l_seg).numnewpasses)
                                    as OPJ_UINT32 as OPJ_UINT32;
                            (*l_cblk).numnewpasses =
                                ((*l_cblk).numnewpasses as
                                     libc::c_uint).wrapping_sub((*l_seg).numnewpasses)
                                    as OPJ_UINT32 as OPJ_UINT32;
                            (*l_seg).real_num_passes = (*l_seg).numpasses;
                            if (*l_cblk).numnewpasses >
                                   0 as libc::c_int as libc::c_uint {
                                l_seg = l_seg.offset(1);
                                (*l_cblk).numsegs =
                                    (*l_cblk).numsegs.wrapping_add(1)
                            }
                        }
                        if !((*l_cblk).numnewpasses >
                                 0 as libc::c_int as libc::c_uint) {
                            break ;
                        }
                    }
                    (*l_cblk).real_num_segs = (*l_cblk).numsegs;
                    l_cblk = l_cblk.offset(1)
                }
                cblkno = cblkno.wrapping_add(1)
            }
            l_band = l_band.offset(1)
        }
        bandno = bandno.wrapping_add(1)
    }
    // return the number of bytes read
    if partial_buffer != 0 {
        *p_data_read = p_max_length
    } else {
        *p_data_read =
            l_current_data.wrapping_offset_from(p_src_data) as libc::c_long as
                OPJ_UINT32
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn opj_t2_skip_packet_data(mut p_t2: *mut opj_t2_t,
                                             mut p_tile: *mut opj_tcd_tile_t,
                                             mut p_pi: *mut opj_pi_iterator_t,
                                             mut p_data_read: *mut OPJ_UINT32,
                                             mut p_max_length: OPJ_UINT32,
                                             mut _pack_info:
                                                 *mut opj_packet_info_t,
                                             mut p_manager:
                                                 *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut bandno: OPJ_UINT32 = 0;
    let mut cblkno: OPJ_UINT32 = 0;
    let mut l_nb_code_blocks: OPJ_UINT32 = 0;
    let mut l_band = 0 as *mut opj_tcd_band_t;
    let mut l_cblk = 0 as *mut opj_tcd_cblk_dec_t;
    let mut l_res: *mut opj_tcd_resolution_t =
        &mut *(*(*p_tile).comps.offset((*p_pi).compno as
                                           isize)).resolutions.offset((*p_pi).resno
                                                                          as
                                                                          isize)
            as *mut opj_tcd_resolution_t;
    *p_data_read = 0 as libc::c_int as OPJ_UINT32;
    l_band = (*l_res).bands.as_mut_ptr();
    bandno = 0 as libc::c_int as OPJ_UINT32;
    while bandno < (*l_res).numbands {
        let mut l_prc: *mut opj_tcd_precinct_t =
            &mut *(*l_band).precincts.offset((*p_pi).precno as isize) as
                *mut opj_tcd_precinct_t;
        if (*l_band).x1 - (*l_band).x0 == 0 as libc::c_int ||
               (*l_band).y1 - (*l_band).y0 == 0 as libc::c_int {
            l_band = l_band.offset(1)
        } else {
            l_nb_code_blocks = (*l_prc).cw.wrapping_mul((*l_prc).ch);
            l_cblk = (*l_prc).cblks.dec;
            cblkno = 0 as libc::c_int as OPJ_UINT32;
            while cblkno < l_nb_code_blocks {
                let mut l_seg = 0 as *mut opj_tcd_seg_t;
                if (*l_cblk).numnewpasses == 0 {
                    /* nothing to do */
                    l_cblk = l_cblk.offset(1)
                } else {
                    if (*l_cblk).numsegs == 0 {
                        l_seg = (*l_cblk).segs;
                        (*l_cblk).numsegs = (*l_cblk).numsegs.wrapping_add(1)
                    } else {
                        l_seg =
                            &mut *(*l_cblk).segs.offset((*l_cblk).numsegs.wrapping_sub(1
                                                                                           as
                                                                                           libc::c_int
                                                                                           as
                                                                                           libc::c_uint)
                                                            as isize) as
                                *mut opj_tcd_seg_t;
                        if (*l_seg).numpasses == (*l_seg).maxpasses {
                            l_seg = l_seg.offset(1);
                            (*l_cblk).numsegs =
                                (*l_cblk).numsegs.wrapping_add(1)
                        }
                    }
                    loop  {
                        /* Check possible overflow then size */
                        if (*p_data_read).wrapping_add((*l_seg).newlen) <
                               *p_data_read ||
                               (*p_data_read).wrapping_add((*l_seg).newlen) >
                                   p_max_length {
                            if (*(*p_t2).cp).strict != 0 {
                                opj_event_msg(p_manager, 1 as libc::c_int,
                                              b"skip: segment too long (%d) with max (%d) for codeblock %d (p=%d, b=%d, r=%d, c=%d)\n\x00"
                                                  as *const u8 as
                                                  *const libc::c_char,
                                              (*l_seg).newlen, p_max_length,
                                              cblkno, (*p_pi).precno, bandno,
                                              (*p_pi).resno, (*p_pi).compno);
                                return 0 as libc::c_int
                            } else {
                                opj_event_msg(p_manager, 2 as libc::c_int,
                                              b"skip: segment too long (%d) with max (%d) for codeblock %d (p=%d, b=%d, r=%d, c=%d)\n\x00"
                                                  as *const u8 as
                                                  *const libc::c_char,
                                              (*l_seg).newlen, p_max_length,
                                              cblkno, (*p_pi).precno, bandno,
                                              (*p_pi).resno, (*p_pi).compno);
                            }
                        }
                        /* USE_JPWL */
                        opj_null_jas_fprintf(stderr,
                                             b"p_data_read (%d) newlen (%d) \n\x00"
                                                 as *const u8 as
                                                 *const libc::c_char,
                                             *p_data_read, (*l_seg).newlen);
                        *p_data_read =
                            (*p_data_read as
                                 libc::c_uint).wrapping_add((*l_seg).newlen)
                                as OPJ_UINT32 as OPJ_UINT32;
                        (*l_seg).numpasses =
                            ((*l_seg).numpasses as
                                 libc::c_uint).wrapping_add((*l_seg).numnewpasses)
                                as OPJ_UINT32 as OPJ_UINT32;
                        (*l_cblk).numnewpasses =
                            ((*l_cblk).numnewpasses as
                                 libc::c_uint).wrapping_sub((*l_seg).numnewpasses)
                                as OPJ_UINT32 as OPJ_UINT32;
                        if (*l_cblk).numnewpasses >
                               0 as libc::c_int as libc::c_uint {
                            l_seg = l_seg.offset(1);
                            (*l_cblk).numsegs =
                                (*l_cblk).numsegs.wrapping_add(1)
                        }
                        if !((*l_cblk).numnewpasses >
                                 0 as libc::c_int as libc::c_uint) {
                            break ;
                        }
                    }
                    l_cblk = l_cblk.offset(1)
                }
                cblkno = cblkno.wrapping_add(1)
            }
            l_band = l_band.offset(1)
        }
        bandno = bandno.wrapping_add(1)
    }
    return 1 as libc::c_int;
}
/* *
@param cblk
@param index
@param cblksty
@param first
*/
unsafe extern "C" fn opj_t2_init_seg(mut cblk: *mut opj_tcd_cblk_dec_t,
                                     mut index: OPJ_UINT32,
                                     mut cblksty: OPJ_UINT32,
                                     mut first: OPJ_UINT32) -> OPJ_BOOL {
    let mut seg = 0 as *mut opj_tcd_seg_t;
    let mut l_nb_segs = index.wrapping_add(1 as libc::c_int as libc::c_uint);
    if l_nb_segs > (*cblk).m_current_max_segs {
        let mut new_segs = 0 as *mut opj_tcd_seg_t;
        let mut l_m_current_max_segs =
            (*cblk).m_current_max_segs.wrapping_add(10 as libc::c_int as
                                                        libc::c_uint);
        new_segs =
            opj_realloc((*cblk).segs as *mut libc::c_void,
                        (l_m_current_max_segs as
                             libc::c_ulong).wrapping_mul(::std::mem::size_of::<opj_tcd_seg_t>()
                                                             as
                                                             libc::c_ulong))
                as *mut opj_tcd_seg_t;
        if new_segs.is_null() {
            /* opj_event_msg(p_manager, EVT_ERROR, "Not enough memory to initialize segment %d\n", l_nb_segs); */
            return 0 as libc::c_int
        }
        (*cblk).segs = new_segs;
        memset(new_segs.offset((*cblk).m_current_max_segs as isize) as
                   *mut libc::c_void, 0 as libc::c_int,
               (10 as libc::c_int as
                    libc::c_ulong).wrapping_mul(::std::mem::size_of::<opj_tcd_seg_t>()
                                                    as libc::c_ulong));
        (*cblk).m_current_max_segs = l_m_current_max_segs
    }
    seg = &mut *(*cblk).segs.offset(index as isize) as *mut opj_tcd_seg_t;
    opj_tcd_reinit_segment(seg);
    if cblksty & 0x4 as libc::c_int as libc::c_uint != 0 {
        (*seg).maxpasses = 1 as libc::c_int as OPJ_UINT32
    } else if cblksty & 0x1 as libc::c_int as libc::c_uint != 0 {
        if first != 0 {
            (*seg).maxpasses = 10 as libc::c_int as OPJ_UINT32
        } else {
            (*seg).maxpasses =
                if (*seg.offset(-(1 as libc::c_int as isize))).maxpasses ==
                       1 as libc::c_int as libc::c_uint ||
                       (*seg.offset(-(1 as libc::c_int as isize))).maxpasses
                           == 10 as libc::c_int as libc::c_uint {
                    2 as libc::c_int
                } else { 1 as libc::c_int } as OPJ_UINT32
        }
    } else {
        /* See paragraph "B.10.6 Number of coding passes" of the standard.
         * Probably that 109 must be interpreted a (Mb-1)*3 + 1 with Mb=37,
         * Mb being the maximum number of bit-planes available for the
         * representation of coefficients in the sub-band */
        (*seg).maxpasses = 109 as libc::c_int as OPJ_UINT32
    }
    return 1 as libc::c_int;
}
