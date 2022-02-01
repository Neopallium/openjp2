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
    fn __assert_fail(__assertion: *const libc::c_char,
                     __file: *const libc::c_char, __line: libc::c_uint,
                     __function: *const libc::c_char) -> !;
    #[no_mangle]
    fn opj_image_data_alloc(size: OPJ_SIZE_T) -> *mut libc::c_void;
    #[no_mangle]
    fn opj_image_data_free(ptr: *mut libc::c_void);
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
    fn opj_procedure_list_create() -> *mut opj_procedure_list_t;
    #[no_mangle]
    fn opj_procedure_list_destroy(p_list: *mut opj_procedure_list_t);
    #[no_mangle]
    fn opj_procedure_list_add_procedure(p_validation_list:
                                            *mut opj_procedure_list_t,
                                        p_procedure: opj_procedure,
                                        p_manager: *mut opj_event_mgr_t)
     -> OPJ_BOOL;
    #[no_mangle]
    fn opj_procedure_list_get_nb_procedures(p_validation_list:
                                                *mut opj_procedure_list_t)
     -> OPJ_UINT32;
    #[no_mangle]
    fn opj_procedure_list_get_first_procedure(p_validation_list:
                                                  *mut opj_procedure_list_t)
     -> *mut opj_procedure;
    #[no_mangle]
    fn opj_procedure_list_clear(p_validation_list: *mut opj_procedure_list_t);
    #[no_mangle]
    fn opj_write_bytes_LE(p_buffer: *mut OPJ_BYTE, p_value: OPJ_UINT32,
                          p_nb_bytes: OPJ_UINT32);
    #[no_mangle]
    fn opj_read_bytes_LE(p_buffer: *const OPJ_BYTE, p_value: *mut OPJ_UINT32,
                         p_nb_bytes: OPJ_UINT32);
    #[no_mangle]
    fn opj_stream_read_data(p_stream: *mut opj_stream_private_t,
                            p_buffer: *mut OPJ_BYTE, p_size: OPJ_SIZE_T,
                            p_event_mgr: *mut opj_event_mgr) -> OPJ_SIZE_T;
    #[no_mangle]
    fn opj_stream_write_data(p_stream: *mut opj_stream_private_t,
                             p_buffer: *const OPJ_BYTE, p_size: OPJ_SIZE_T,
                             p_event_mgr: *mut opj_event_mgr) -> OPJ_SIZE_T;
    #[no_mangle]
    fn opj_stream_skip(p_stream: *mut opj_stream_private_t, p_size: OPJ_OFF_T,
                       p_event_mgr: *mut opj_event_mgr) -> OPJ_OFF_T;
    #[no_mangle]
    fn opj_stream_tell(p_stream: *const opj_stream_private_t) -> OPJ_OFF_T;
    #[no_mangle]
    fn opj_stream_get_number_byte_left(p_stream: *const opj_stream_private_t)
     -> OPJ_OFF_T;
    #[no_mangle]
    fn opj_stream_seek(p_stream: *mut opj_stream_private_t, p_size: OPJ_OFF_T,
                       p_event_mgr: *mut opj_event_mgr) -> OPJ_BOOL;
    #[no_mangle]
    fn opj_stream_has_seek(p_stream: *const opj_stream_private_t) -> OPJ_BOOL;
    #[no_mangle]
    fn opj_j2k_setup_decoder(j2k: *mut opj_j2k_t,
                             parameters: *mut opj_dparameters_t);
    #[no_mangle]
    fn opj_j2k_decoder_set_strict_mode(j2k: *mut opj_j2k_t, strict: OPJ_BOOL);
    #[no_mangle]
    fn opj_j2k_set_threads(j2k: *mut opj_j2k_t, num_threads: OPJ_UINT32)
     -> OPJ_BOOL;
    #[no_mangle]
    fn opj_j2k_create_compress() -> *mut opj_j2k_t;
    #[no_mangle]
    fn opj_j2k_setup_encoder(p_j2k: *mut opj_j2k_t,
                             parameters: *mut opj_cparameters_t,
                             image: *mut opj_image_t,
                             p_manager: *mut opj_event_mgr_t) -> OPJ_BOOL;
    #[no_mangle]
    fn opj_j2k_end_decompress(j2k: *mut opj_j2k_t,
                              p_stream: *mut opj_stream_private_t,
                              p_manager: *mut opj_event_mgr_t) -> OPJ_BOOL;
    #[no_mangle]
    fn opj_j2k_read_header(p_stream: *mut opj_stream_private_t,
                           p_j2k: *mut opj_j2k_t,
                           p_image: *mut *mut opj_image_t,
                           p_manager: *mut opj_event_mgr_t) -> OPJ_BOOL;
    #[no_mangle]
    fn opj_j2k_destroy(p_j2k: *mut opj_j2k_t);
    #[no_mangle]
    fn opj_j2k_decode_tile(p_j2k: *mut opj_j2k_t, p_tile_index: OPJ_UINT32,
                           p_data: *mut OPJ_BYTE, p_data_size: OPJ_UINT32,
                           p_stream: *mut opj_stream_private_t,
                           p_manager: *mut opj_event_mgr_t) -> OPJ_BOOL;
    #[no_mangle]
    fn opj_j2k_read_tile_header(p_j2k: *mut opj_j2k_t,
                                p_tile_index: *mut OPJ_UINT32,
                                p_data_size: *mut OPJ_UINT32,
                                p_tile_x0: *mut OPJ_INT32,
                                p_tile_y0: *mut OPJ_INT32,
                                p_tile_x1: *mut OPJ_INT32,
                                p_tile_y1: *mut OPJ_INT32,
                                p_nb_comps: *mut OPJ_UINT32,
                                p_go_on: *mut OPJ_BOOL,
                                p_stream: *mut opj_stream_private_t,
                                p_manager: *mut opj_event_mgr_t) -> OPJ_BOOL;
    #[no_mangle]
    fn opj_j2k_set_decoded_components(p_j2k: *mut opj_j2k_t,
                                      numcomps: OPJ_UINT32,
                                      comps_indices: *const OPJ_UINT32,
                                      p_manager: *mut opj_event_mgr_t)
     -> OPJ_BOOL;
    #[no_mangle]
    fn opj_j2k_set_decode_area(p_j2k: *mut opj_j2k_t,
                               p_image: *mut opj_image_t,
                               p_start_x: OPJ_INT32, p_start_y: OPJ_INT32,
                               p_end_x: OPJ_INT32, p_end_y: OPJ_INT32,
                               p_manager: *mut opj_event_mgr_t) -> OPJ_BOOL;
    #[no_mangle]
    fn opj_j2k_create_decompress() -> *mut opj_j2k_t;
    #[no_mangle]
    fn j2k_dump(p_j2k: *mut opj_j2k_t, flag: OPJ_INT32,
                out_stream: *mut FILE);
    #[no_mangle]
    fn j2k_get_cstr_info(p_j2k: *mut opj_j2k_t)
     -> *mut opj_codestream_info_v2_t;
    #[no_mangle]
    fn j2k_get_cstr_index(p_j2k: *mut opj_j2k_t)
     -> *mut opj_codestream_index_t;
    #[no_mangle]
    fn opj_j2k_decode(j2k: *mut opj_j2k_t,
                      p_stream: *mut opj_stream_private_t,
                      p_image: *mut opj_image_t,
                      p_manager: *mut opj_event_mgr_t) -> OPJ_BOOL;
    #[no_mangle]
    fn opj_j2k_get_tile(p_j2k: *mut opj_j2k_t,
                        p_stream: *mut opj_stream_private_t,
                        p_image: *mut opj_image_t,
                        p_manager: *mut opj_event_mgr_t,
                        tile_index: OPJ_UINT32) -> OPJ_BOOL;
    #[no_mangle]
    fn opj_j2k_set_decoded_resolution_factor(p_j2k: *mut opj_j2k_t,
                                             res_factor: OPJ_UINT32,
                                             p_manager: *mut opj_event_mgr_t)
     -> OPJ_BOOL;
    #[no_mangle]
    fn opj_j2k_encoder_set_extra_options(p_j2k: *mut opj_j2k_t,
                                         p_options:
                                             *const *const libc::c_char,
                                         p_manager: *mut opj_event_mgr_t)
     -> OPJ_BOOL;
    #[no_mangle]
    fn opj_j2k_write_tile(p_j2k: *mut opj_j2k_t, p_tile_index: OPJ_UINT32,
                          p_data: *mut OPJ_BYTE, p_data_size: OPJ_UINT32,
                          p_stream: *mut opj_stream_private_t,
                          p_manager: *mut opj_event_mgr_t) -> OPJ_BOOL;
    #[no_mangle]
    fn opj_j2k_encode(p_j2k: *mut opj_j2k_t, cio: *mut opj_stream_private_t,
                      p_manager: *mut opj_event_mgr_t) -> OPJ_BOOL;
    #[no_mangle]
    fn opj_j2k_start_compress(p_j2k: *mut opj_j2k_t,
                              p_stream: *mut opj_stream_private_t,
                              p_image: *mut opj_image_t,
                              p_manager: *mut opj_event_mgr_t) -> OPJ_BOOL;
    #[no_mangle]
    fn opj_j2k_end_compress(p_j2k: *mut opj_j2k_t,
                            cio: *mut opj_stream_private_t,
                            p_manager: *mut opj_event_mgr_t) -> OPJ_BOOL;
}
pub type size_t = libc::c_ulong;
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
pub type int32_t = __int32_t;
pub type int64_t = __int64_t;
pub type uint16_t = __uint16_t;
pub type uint32_t = __uint32_t;
pub type uint64_t = __uint64_t;
pub type OPJ_UINT16 = uint16_t;
pub type OPJ_INT32 = int32_t;
pub type OPJ_UINT32 = uint32_t;
pub type OPJ_UINT64 = uint64_t;
pub type OPJ_OFF_T = int64_t;
pub type OPJ_SIZE_T = size_t;
pub type RSIZ_CAPABILITIES = libc::c_uint;
pub const OPJ_MCT: RSIZ_CAPABILITIES = 33024;
pub const OPJ_CINEMA4K: RSIZ_CAPABILITIES = 4;
pub const OPJ_CINEMA2K: RSIZ_CAPABILITIES = 3;
pub const OPJ_STD_RSIZ: RSIZ_CAPABILITIES = 0;
pub type OPJ_RSIZ_CAPABILITIES = RSIZ_CAPABILITIES;
pub type CINEMA_MODE = libc::c_uint;
pub const OPJ_CINEMA4K_24: CINEMA_MODE = 3;
pub const OPJ_CINEMA2K_48: CINEMA_MODE = 2;
pub const OPJ_CINEMA2K_24: CINEMA_MODE = 1;
pub const OPJ_OFF: CINEMA_MODE = 0;
pub type OPJ_CINEMA_MODE = CINEMA_MODE;
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
pub struct opj_cparameters {
    pub tile_size_on: OPJ_BOOL,
    pub cp_tx0: libc::c_int,
    pub cp_ty0: libc::c_int,
    pub cp_tdx: libc::c_int,
    pub cp_tdy: libc::c_int,
    pub cp_disto_alloc: libc::c_int,
    pub cp_fixed_alloc: libc::c_int,
    pub cp_fixed_quality: libc::c_int,
    pub cp_matrice: *mut libc::c_int,
    pub cp_comment: *mut libc::c_char,
    pub csty: libc::c_int,
    pub prog_order: OPJ_PROG_ORDER,
    pub POC: [opj_poc_t; 32],
    pub numpocs: OPJ_UINT32,
    pub tcp_numlayers: libc::c_int,
    pub tcp_rates: [libc::c_float; 100],
    pub tcp_distoratio: [libc::c_float; 100],
    pub numresolution: libc::c_int,
    pub cblockw_init: libc::c_int,
    pub cblockh_init: libc::c_int,
    pub mode: libc::c_int,
    pub irreversible: libc::c_int,
    pub roi_compno: libc::c_int,
    pub roi_shift: libc::c_int,
    pub res_spec: libc::c_int,
    pub prcw_init: [libc::c_int; 33],
    pub prch_init: [libc::c_int; 33],
    pub infile: [libc::c_char; 4096],
    pub outfile: [libc::c_char; 4096],
    pub index_on: libc::c_int,
    pub index: [libc::c_char; 4096],
    pub image_offset_x0: libc::c_int,
    pub image_offset_y0: libc::c_int,
    pub subsampling_dx: libc::c_int,
    pub subsampling_dy: libc::c_int,
    pub decod_format: libc::c_int,
    pub cod_format: libc::c_int,
    pub jpwl_epc_on: OPJ_BOOL,
    pub jpwl_hprot_MH: libc::c_int,
    pub jpwl_hprot_TPH_tileno: [libc::c_int; 16],
    pub jpwl_hprot_TPH: [libc::c_int; 16],
    pub jpwl_pprot_tileno: [libc::c_int; 16],
    pub jpwl_pprot_packno: [libc::c_int; 16],
    pub jpwl_pprot: [libc::c_int; 16],
    pub jpwl_sens_size: libc::c_int,
    pub jpwl_sens_addr: libc::c_int,
    pub jpwl_sens_range: libc::c_int,
    pub jpwl_sens_MH: libc::c_int,
    pub jpwl_sens_TPH_tileno: [libc::c_int; 16],
    pub jpwl_sens_TPH: [libc::c_int; 16],
    pub cp_cinema: OPJ_CINEMA_MODE,
    pub max_comp_size: libc::c_int,
    pub cp_rsiz: OPJ_RSIZ_CAPABILITIES,
    pub tp_on: libc::c_char,
    pub tp_flag: libc::c_char,
    pub tcp_mct: libc::c_char,
    pub jpip_on: OPJ_BOOL,
    pub mct_data: *mut libc::c_void,
    pub max_cs_size: libc::c_int,
    pub rsiz: OPJ_UINT16,
}
pub type opj_cparameters_t = opj_cparameters;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_dparameters {
    pub cp_reduce: OPJ_UINT32,
    pub cp_layer: OPJ_UINT32,
    pub infile: [libc::c_char; 4096],
    pub outfile: [libc::c_char; 4096],
    pub decod_format: libc::c_int,
    pub cod_format: libc::c_int,
    pub DA_x0: OPJ_UINT32,
    pub DA_x1: OPJ_UINT32,
    pub DA_y0: OPJ_UINT32,
    pub DA_y1: OPJ_UINT32,
    pub m_verbose: OPJ_BOOL,
    pub tile_index: OPJ_UINT32,
    pub nb_tile_to_decode: OPJ_UINT32,
    pub jpwl_correct: OPJ_BOOL,
    pub jpwl_exp_comps: libc::c_int,
    pub jpwl_max_tiles: libc::c_int,
    pub flags: libc::c_uint,
}
pub type opj_dparameters_t = opj_dparameters;
pub type opj_stream_read_fn
    =
    Option<unsafe extern "C" fn(_: *mut libc::c_void, _: OPJ_SIZE_T,
                                _: *mut libc::c_void) -> OPJ_SIZE_T>;
pub type opj_stream_write_fn
    =
    Option<unsafe extern "C" fn(_: *mut libc::c_void, _: OPJ_SIZE_T,
                                _: *mut libc::c_void) -> OPJ_SIZE_T>;
pub type opj_stream_skip_fn
    =
    Option<unsafe extern "C" fn(_: OPJ_OFF_T, _: *mut libc::c_void)
               -> OPJ_OFF_T>;
pub type opj_stream_seek_fn
    =
    Option<unsafe extern "C" fn(_: OPJ_OFF_T, _: *mut libc::c_void)
               -> OPJ_BOOL>;
pub type opj_stream_free_user_data_fn
    =
    Option<unsafe extern "C" fn(_: *mut libc::c_void) -> ()>;
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
pub struct opj_tccp_info {
    pub compno: OPJ_UINT32,
    pub csty: OPJ_UINT32,
    pub numresolutions: OPJ_UINT32,
    pub cblkw: OPJ_UINT32,
    pub cblkh: OPJ_UINT32,
    pub cblksty: OPJ_UINT32,
    pub qmfbid: OPJ_UINT32,
    pub qntsty: OPJ_UINT32,
    pub stepsizes_mant: [OPJ_UINT32; 97],
    pub stepsizes_expn: [OPJ_UINT32; 97],
    pub numgbits: OPJ_UINT32,
    pub roishift: OPJ_INT32,
    pub prcw: [OPJ_UINT32; 33],
    pub prch: [OPJ_UINT32; 33],
}
pub type opj_tccp_info_t = opj_tccp_info;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_tile_v2_info {
    pub tileno: libc::c_int,
    pub csty: OPJ_UINT32,
    pub prg: OPJ_PROG_ORDER,
    pub numlayers: OPJ_UINT32,
    pub mct: OPJ_UINT32,
    pub tccp_info: *mut opj_tccp_info_t,
}
pub type opj_tile_info_v2_t = opj_tile_v2_info;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_codestream_info_v2 {
    pub tx0: OPJ_UINT32,
    pub ty0: OPJ_UINT32,
    pub tdx: OPJ_UINT32,
    pub tdy: OPJ_UINT32,
    pub tw: OPJ_UINT32,
    pub th: OPJ_UINT32,
    pub nbcomps: OPJ_UINT32,
    pub m_default_tile_info: opj_tile_info_v2_t,
    pub tile_info: *mut opj_tile_info_v2_t,
}
pub type opj_codestream_info_v2_t = opj_codestream_info_v2;
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
pub type opj_procedure = Option<unsafe extern "C" fn() -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_procedure_list {
    pub m_nb_procedures: OPJ_UINT32,
    pub m_nb_max_procedures: OPJ_UINT32,
    pub m_procedures: *mut opj_procedure,
}
pub type opj_procedure_list_t = opj_procedure_list;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_stream_private {
    pub m_user_data: *mut libc::c_void,
    pub m_free_user_data_fn: opj_stream_free_user_data_fn,
    pub m_user_data_length: OPJ_UINT64,
    pub m_read_fn: opj_stream_read_fn,
    pub m_write_fn: opj_stream_write_fn,
    pub m_skip_fn: opj_stream_skip_fn,
    pub m_seek_fn: opj_stream_seek_fn,
    pub m_stored_data: *mut OPJ_BYTE,
    pub m_current_data: *mut OPJ_BYTE,
    pub m_opj_skip: Option<unsafe extern "C" fn(_: *mut opj_stream_private,
                                                _: OPJ_OFF_T,
                                                _: *mut opj_event_mgr)
                               -> OPJ_OFF_T>,
    pub m_opj_seek: Option<unsafe extern "C" fn(_: *mut opj_stream_private,
                                                _: OPJ_OFF_T,
                                                _: *mut opj_event_mgr)
                               -> OPJ_BOOL>,
    pub m_bytes_in_buffer: OPJ_SIZE_T,
    pub m_byte_offset: OPJ_OFF_T,
    pub m_buffer_size: OPJ_SIZE_T,
    pub m_status: OPJ_UINT32,
}
pub type opj_stream_private_t = opj_stream_private;
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
pub type opj_cp_t = opj_cp;
#[derive(Copy, Clone, BitfieldStruct)]
#[repr(C)]
pub struct opj_j2k_dec {
    pub m_state: OPJ_UINT32,
    pub m_default_tcp: *mut opj_tcp_t,
    pub m_header_data: *mut OPJ_BYTE,
    pub m_header_data_size: OPJ_UINT32,
    pub m_sot_length: OPJ_UINT32,
    pub m_start_tile_x: OPJ_UINT32,
    pub m_start_tile_y: OPJ_UINT32,
    pub m_end_tile_x: OPJ_UINT32,
    pub m_end_tile_y: OPJ_UINT32,
    pub m_tile_ind_to_dec: OPJ_INT32,
    pub m_last_sot_read_pos: OPJ_OFF_T,
    pub m_last_tile_part: OPJ_BOOL,
    pub m_numcomps_to_decode: OPJ_UINT32,
    pub m_comps_indices_to_decode: *mut OPJ_UINT32,
    #[bitfield(name = "m_can_decode", ty = "OPJ_BITFIELD", bits = "0..=0")]
    #[bitfield(name = "m_discard_tiles", ty = "OPJ_BITFIELD", bits = "1..=1")]
    #[bitfield(name = "m_skip_data", ty = "OPJ_BITFIELD", bits = "2..=2")]
    #[bitfield(name = "m_nb_tile_parts_correction_checked", ty =
               "OPJ_BITFIELD", bits = "3..=3")]
    #[bitfield(name = "m_nb_tile_parts_correction", ty = "OPJ_BITFIELD", bits
               = "4..=4")]
    pub m_can_decode_m_discard_tiles_m_skip_data_m_nb_tile_parts_correction_checked_m_nb_tile_parts_correction: [u8; 1],
    #[bitfield(padding)]
    pub c2rust_padding: [u8; 7],
}
pub type opj_j2k_dec_t = opj_j2k_dec;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_j2k_enc {
    pub m_current_poc_tile_part_number: OPJ_UINT32,
    pub m_current_tile_part_number: OPJ_UINT32,
    pub m_TLM: OPJ_BOOL,
    pub m_Ttlmi_is_byte: OPJ_BOOL,
    pub m_tlm_start: OPJ_OFF_T,
    pub m_tlm_sot_offsets_buffer: *mut OPJ_BYTE,
    pub m_tlm_sot_offsets_current: *mut OPJ_BYTE,
    pub m_total_tile_parts: OPJ_UINT32,
    pub m_encoded_tile_data: *mut OPJ_BYTE,
    pub m_encoded_tile_size: OPJ_UINT32,
    pub m_header_tile_data: *mut OPJ_BYTE,
    pub m_header_tile_data_size: OPJ_UINT32,
    pub m_PLT: OPJ_BOOL,
    pub m_reserved_bytes_for_PLT: OPJ_UINT32,
    pub m_nb_comps: OPJ_UINT32,
}
pub type opj_j2k_enc_t = opj_j2k_enc;
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
pub struct opj_j2k {
    pub m_is_decoder: OPJ_BOOL,
    pub m_specific_param: C2RustUnnamed_1,
    pub m_private_image: *mut opj_image_t,
    pub m_output_image: *mut opj_image_t,
    pub m_cp: opj_cp_t,
    pub m_procedure_list: *mut opj_procedure_list_t,
    pub m_validation_list: *mut opj_procedure_list_t,
    pub cstr_index: *mut opj_codestream_index_t,
    pub m_current_tile_number: OPJ_UINT32,
    pub m_tcd: *mut opj_tcd,
    pub m_tp: *mut opj_thread_pool_t,
    pub ihdr_w: OPJ_UINT32,
    pub ihdr_h: OPJ_UINT32,
    pub dump_state: libc::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_1 {
    pub m_decoder: opj_j2k_dec_t,
    pub m_encoder: opj_j2k_enc_t,
}
pub type opj_j2k_t = opj_j2k;
pub type C2RustUnnamed_2 = libc::c_uint;
pub const JP2_STATE_UNKNOWN: C2RustUnnamed_2 = 2147483647;
pub const JP2_STATE_END_CODESTREAM: C2RustUnnamed_2 = 16;
pub const JP2_STATE_CODESTREAM: C2RustUnnamed_2 = 8;
pub const JP2_STATE_HEADER: C2RustUnnamed_2 = 4;
pub const JP2_STATE_FILE_TYPE: C2RustUnnamed_2 = 2;
pub const JP2_STATE_SIGNATURE: C2RustUnnamed_2 = 1;
pub const JP2_STATE_NONE: C2RustUnnamed_2 = 0;
pub type C2RustUnnamed_3 = libc::c_uint;
pub const JP2_IMG_STATE_UNKNOWN: C2RustUnnamed_3 = 2147483647;
pub const JP2_IMG_STATE_NONE: C2RustUnnamed_3 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_jp2_cdef_info {
    pub cn: OPJ_UINT16,
    pub typ: OPJ_UINT16,
    pub asoc: OPJ_UINT16,
}
pub type opj_jp2_cdef_info_t = opj_jp2_cdef_info;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_jp2_cdef {
    pub info: *mut opj_jp2_cdef_info_t,
    pub n: OPJ_UINT16,
}
pub type opj_jp2_cdef_t = opj_jp2_cdef;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_jp2_cmap_comp {
    pub cmp: OPJ_UINT16,
    pub mtyp: OPJ_BYTE,
    pub pcol: OPJ_BYTE,
}
pub type opj_jp2_cmap_comp_t = opj_jp2_cmap_comp;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_jp2_pclr {
    pub entries: *mut OPJ_UINT32,
    pub channel_sign: *mut OPJ_BYTE,
    pub channel_size: *mut OPJ_BYTE,
    pub cmap: *mut opj_jp2_cmap_comp_t,
    pub nr_entries: OPJ_UINT16,
    pub nr_channels: OPJ_BYTE,
}
pub type opj_jp2_pclr_t = opj_jp2_pclr;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_jp2_color {
    pub icc_profile_buf: *mut OPJ_BYTE,
    pub icc_profile_len: OPJ_UINT32,
    pub jp2_cdef: *mut opj_jp2_cdef_t,
    pub jp2_pclr: *mut opj_jp2_pclr_t,
    pub jp2_has_colr: OPJ_BYTE,
}
pub type opj_jp2_color_t = opj_jp2_color;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_jp2_comps {
    pub depth: OPJ_UINT32,
    pub sgnd: OPJ_UINT32,
    pub bpcc: OPJ_UINT32,
}
pub type opj_jp2_comps_t = opj_jp2_comps;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_jp2 {
    pub j2k: *mut opj_j2k_t,
    pub m_validation_list: *mut opj_procedure_list,
    pub m_procedure_list: *mut opj_procedure_list,
    pub w: OPJ_UINT32,
    pub h: OPJ_UINT32,
    pub numcomps: OPJ_UINT32,
    pub bpc: OPJ_UINT32,
    pub C: OPJ_UINT32,
    pub UnkC: OPJ_UINT32,
    pub IPR: OPJ_UINT32,
    pub meth: OPJ_UINT32,
    pub approx: OPJ_UINT32,
    pub enumcs: OPJ_UINT32,
    pub precedence: OPJ_UINT32,
    pub brand: OPJ_UINT32,
    pub minversion: OPJ_UINT32,
    pub numcl: OPJ_UINT32,
    pub cl: *mut OPJ_UINT32,
    pub comps: *mut opj_jp2_comps_t,
    pub j2k_codestream_offset: OPJ_OFF_T,
    pub jpip_iptr_offset: OPJ_OFF_T,
    pub jpip_on: OPJ_BOOL,
    pub jp2_state: OPJ_UINT32,
    pub jp2_img_state: OPJ_UINT32,
    pub color: opj_jp2_color_t,
    pub ignore_pclr_cmap_cdef: OPJ_BOOL,
    pub has_jp2h: OPJ_BYTE,
    pub has_ihdr: OPJ_BYTE,
}
pub type opj_jp2_t = opj_jp2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_jp2_box {
    pub length: OPJ_UINT32,
    pub type_0: OPJ_UINT32,
    pub init_pos: OPJ_INT32,
}
pub type opj_jp2_box_t = opj_jp2_box;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_jp2_header_handler {
    pub id: OPJ_UINT32,
    pub handler: Option<unsafe extern "C" fn(_: *mut opj_jp2_t,
                                             _: *mut OPJ_BYTE, _: OPJ_UINT32,
                                             _: *mut opj_event_mgr_t)
                            -> OPJ_BOOL>,
}
pub type opj_jp2_header_handler_t = opj_jp2_header_handler;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct opj_jp2_img_header_writer_handler {
    pub handler: Option<unsafe extern "C" fn(_: *mut opj_jp2_t,
                                             _: *mut OPJ_UINT32)
                            -> *mut OPJ_BYTE>,
    pub m_data: *mut OPJ_BYTE,
    pub m_size: OPJ_UINT32,
}
pub type opj_jp2_img_header_writer_handler_t
    =
    opj_jp2_img_header_writer_handler;
static mut jp2_header: [opj_jp2_header_handler_t; 3] =
    unsafe {
        [{
             let mut init =
                 opj_jp2_header_handler{id:
                                            0x6a502020 as libc::c_int as
                                                OPJ_UINT32,
                                        handler:
                                            Some(opj_jp2_read_jp as
                                                     unsafe extern "C" fn(_:
                                                                              *mut opj_jp2_t,
                                                                          _:
                                                                              *mut OPJ_BYTE,
                                                                          _:
                                                                              OPJ_UINT32,
                                                                          _:
                                                                              *mut opj_event_mgr_t)
                                                         -> OPJ_BOOL),};
             init
         },
         {
             let mut init =
                 opj_jp2_header_handler{id:
                                            0x66747970 as libc::c_int as
                                                OPJ_UINT32,
                                        handler:
                                            Some(opj_jp2_read_ftyp as
                                                     unsafe extern "C" fn(_:
                                                                              *mut opj_jp2_t,
                                                                          _:
                                                                              *mut OPJ_BYTE,
                                                                          _:
                                                                              OPJ_UINT32,
                                                                          _:
                                                                              *mut opj_event_mgr_t)
                                                         -> OPJ_BOOL),};
             init
         },
         {
             let mut init =
                 opj_jp2_header_handler{id:
                                            0x6a703268 as libc::c_int as
                                                OPJ_UINT32,
                                        handler:
                                            Some(opj_jp2_read_jp2h as
                                                     unsafe extern "C" fn(_:
                                                                              *mut opj_jp2_t,
                                                                          _:
                                                                              *mut OPJ_BYTE,
                                                                          _:
                                                                              OPJ_UINT32,
                                                                          _:
                                                                              *mut opj_event_mgr_t)
                                                         -> OPJ_BOOL),};
             init
         }]
    };
static mut jp2_img_header: [opj_jp2_header_handler_t; 6] =
    unsafe {
        [{
             let mut init =
                 opj_jp2_header_handler{id:
                                            0x69686472 as libc::c_int as
                                                OPJ_UINT32,
                                        handler:
                                            Some(opj_jp2_read_ihdr as
                                                     unsafe extern "C" fn(_:
                                                                              *mut opj_jp2_t,
                                                                          _:
                                                                              *mut OPJ_BYTE,
                                                                          _:
                                                                              OPJ_UINT32,
                                                                          _:
                                                                              *mut opj_event_mgr_t)
                                                         -> OPJ_BOOL),};
             init
         },
         {
             let mut init =
                 opj_jp2_header_handler{id:
                                            0x636f6c72 as libc::c_int as
                                                OPJ_UINT32,
                                        handler:
                                            Some(opj_jp2_read_colr as
                                                     unsafe extern "C" fn(_:
                                                                              *mut opj_jp2_t,
                                                                          _:
                                                                              *mut OPJ_BYTE,
                                                                          _:
                                                                              OPJ_UINT32,
                                                                          _:
                                                                              *mut opj_event_mgr_t)
                                                         -> OPJ_BOOL),};
             init
         },
         {
             let mut init =
                 opj_jp2_header_handler{id:
                                            0x62706363 as libc::c_int as
                                                OPJ_UINT32,
                                        handler:
                                            Some(opj_jp2_read_bpcc as
                                                     unsafe extern "C" fn(_:
                                                                              *mut opj_jp2_t,
                                                                          _:
                                                                              *mut OPJ_BYTE,
                                                                          _:
                                                                              OPJ_UINT32,
                                                                          _:
                                                                              *mut opj_event_mgr_t)
                                                         -> OPJ_BOOL),};
             init
         },
         {
             let mut init =
                 opj_jp2_header_handler{id:
                                            0x70636c72 as libc::c_int as
                                                OPJ_UINT32,
                                        handler:
                                            Some(opj_jp2_read_pclr as
                                                     unsafe extern "C" fn(_:
                                                                              *mut opj_jp2_t,
                                                                          _:
                                                                              *mut OPJ_BYTE,
                                                                          _:
                                                                              OPJ_UINT32,
                                                                          _:
                                                                              *mut opj_event_mgr_t)
                                                         -> OPJ_BOOL),};
             init
         },
         {
             let mut init =
                 opj_jp2_header_handler{id:
                                            0x636d6170 as libc::c_int as
                                                OPJ_UINT32,
                                        handler:
                                            Some(opj_jp2_read_cmap as
                                                     unsafe extern "C" fn(_:
                                                                              *mut opj_jp2_t,
                                                                          _:
                                                                              *mut OPJ_BYTE,
                                                                          _:
                                                                              OPJ_UINT32,
                                                                          _:
                                                                              *mut opj_event_mgr_t)
                                                         -> OPJ_BOOL),};
             init
         },
         {
             let mut init =
                 opj_jp2_header_handler{id:
                                            0x63646566 as libc::c_int as
                                                OPJ_UINT32,
                                        handler:
                                            Some(opj_jp2_read_cdef as
                                                     unsafe extern "C" fn(_:
                                                                              *mut opj_jp2_t,
                                                                          _:
                                                                              *mut OPJ_BYTE,
                                                                          _:
                                                                              OPJ_UINT32,
                                                                          _:
                                                                              *mut opj_event_mgr_t)
                                                         -> OPJ_BOOL),};
             init
         }]
    };
/* *
 * Reads a box header. The box is the way data is packed inside a jpeg2000 file structure.
 *
 * @param   cio                     the input stream to read data from.
 * @param   box                     the box structure to fill.
 * @param   p_number_bytes_read     pointer to an int that will store the number of bytes read from the stream (shoul usually be 2).
 * @param   p_manager               user event manager.
 *
 * @return  true if the box is recognized, false otherwise
*/
/* ----------------------------------------------------------------------- */
unsafe extern "C" fn opj_jp2_read_boxhdr(mut box_0: *mut opj_jp2_box_t,
                                         mut p_number_bytes_read:
                                             *mut OPJ_UINT32,
                                         mut cio: *mut opj_stream_private_t,
                                         mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    /* read header from file */
    let mut l_data_header: [OPJ_BYTE; 8] = [0; 8];
    /* preconditions */
    if !cio.is_null() {
    } else {
        __assert_fail(b"cio != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      482 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 103],
                                                &[libc::c_char; 103]>(b"OPJ_BOOL opj_jp2_read_boxhdr(opj_jp2_box_t *, OPJ_UINT32 *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !box_0.is_null() {
    } else {
        __assert_fail(b"box != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      483 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 103],
                                                &[libc::c_char; 103]>(b"OPJ_BOOL opj_jp2_read_boxhdr(opj_jp2_box_t *, OPJ_UINT32 *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_number_bytes_read.is_null() {
    } else {
        __assert_fail(b"p_number_bytes_read != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      484 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 103],
                                                &[libc::c_char; 103]>(b"OPJ_BOOL opj_jp2_read_boxhdr(opj_jp2_box_t *, OPJ_UINT32 *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      485 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 103],
                                                &[libc::c_char; 103]>(b"OPJ_BOOL opj_jp2_read_boxhdr(opj_jp2_box_t *, OPJ_UINT32 *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    *p_number_bytes_read =
        opj_stream_read_data(cio, l_data_header.as_mut_ptr(),
                             8 as libc::c_int as OPJ_SIZE_T, p_manager) as
            OPJ_UINT32;
    if *p_number_bytes_read != 8 as libc::c_int as libc::c_uint {
        return 0 as libc::c_int
    }
    /* process read data */
    opj_read_bytes_LE(l_data_header.as_mut_ptr(), &mut (*box_0).length,
                      4 as libc::c_int as OPJ_UINT32);
    opj_read_bytes_LE(l_data_header.as_mut_ptr().offset(4 as libc::c_int as
                                                            isize),
                      &mut (*box_0).type_0, 4 as libc::c_int as OPJ_UINT32);
    if (*box_0).length == 0 as libc::c_int as libc::c_uint {
        /* last box */
        let bleft = opj_stream_get_number_byte_left(cio);
        if bleft >
               (0xffffffff as libc::c_uint).wrapping_sub(8 as libc::c_uint) as
                   OPJ_OFF_T {
            opj_event_msg(p_manager, 1 as libc::c_int,
                          b"Cannot handle box sizes higher than 2^32\n\x00" as
                              *const u8 as *const libc::c_char);
            return 0 as libc::c_int
        }
        (*box_0).length =
            (bleft as OPJ_UINT32).wrapping_add(8 as libc::c_uint);
        if (*box_0).length as OPJ_OFF_T ==
               bleft + 8 as libc::c_int as libc::c_long {
        } else {
            __assert_fail(b"(OPJ_OFF_T)box->length == bleft + 8\x00" as
                              *const u8 as *const libc::c_char,
                          b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as
                              *const u8 as *const libc::c_char,
                          505 as libc::c_int as libc::c_uint,
                          (*::std::mem::transmute::<&[u8; 103],
                                                    &[libc::c_char; 103]>(b"OPJ_BOOL opj_jp2_read_boxhdr(opj_jp2_box_t *, OPJ_UINT32 *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
        }
        return 1 as libc::c_int
    }
    /* do we have a "special very large box ?" */
    /* read then the XLBox */
    if (*box_0).length == 1 as libc::c_int as libc::c_uint {
        let mut l_xl_part_size: OPJ_UINT32 = 0;
        let mut l_nb_bytes_read =
            opj_stream_read_data(cio, l_data_header.as_mut_ptr(),
                                 8 as libc::c_int as OPJ_SIZE_T, p_manager) as
                OPJ_UINT32;
        if l_nb_bytes_read != 8 as libc::c_int as libc::c_uint {
            if l_nb_bytes_read > 0 as libc::c_int as libc::c_uint {
                *p_number_bytes_read =
                    (*p_number_bytes_read as
                         libc::c_uint).wrapping_add(l_nb_bytes_read) as
                        OPJ_UINT32 as OPJ_UINT32
            }
            return 0 as libc::c_int
        }
        *p_number_bytes_read = 16 as libc::c_int as OPJ_UINT32;
        opj_read_bytes_LE(l_data_header.as_mut_ptr(), &mut l_xl_part_size,
                          4 as libc::c_int as OPJ_UINT32);
        if l_xl_part_size != 0 as libc::c_int as libc::c_uint {
            opj_event_msg(p_manager, 1 as libc::c_int,
                          b"Cannot handle box sizes higher than 2^32\n\x00" as
                              *const u8 as *const libc::c_char);
            return 0 as libc::c_int
        }
        opj_read_bytes_LE(l_data_header.as_mut_ptr().offset(4 as libc::c_int
                                                                as isize),
                          &mut (*box_0).length,
                          4 as libc::c_int as OPJ_UINT32);
    }
    return 1 as libc::c_int;
}
/* * @name Local static functions */
/*@{*/
/*static void jp2_write_url(opj_cio_t *cio, char *Idx_file);*/
/* *
 * Reads a IHDR box - Image Header box
 *
 * @param   p_image_header_data         pointer to actual data (already read from file)
 * @param   jp2                         the jpeg2000 file codec.
 * @param   p_image_header_size         the size of the image header
 * @param   p_manager                   the user event manager.
 *
 * @return  true if the image header is valid, false else.
 */
unsafe extern "C" fn opj_jp2_read_ihdr(mut jp2: *mut opj_jp2_t,
                                       mut p_image_header_data: *mut OPJ_BYTE,
                                       mut p_image_header_size: OPJ_UINT32,
                                       mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    /* preconditions */
    if !p_image_header_data.is_null() {
    } else {
        __assert_fail(b"p_image_header_data != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      567 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_ihdr(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr()); /* HEIGHT */
    } /* WIDTH */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      568 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_ihdr(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr()); /* NC */
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      569 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_ihdr(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !(*jp2).comps.is_null() {
        opj_event_msg(p_manager, 2 as libc::c_int,
                      b"Ignoring ihdr box. First ihdr box already read\n\x00"
                          as *const u8 as *const libc::c_char);
        return 1 as libc::c_int
    }
    if p_image_header_size != 14 as libc::c_int as libc::c_uint {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Bad image header box (bad size)\n\x00" as *const u8 as
                          *const libc::c_char);
        return 0 as libc::c_int
    }
    opj_read_bytes_LE(p_image_header_data, &mut (*jp2).h,
                      4 as libc::c_int as OPJ_UINT32);
    p_image_header_data =
        p_image_header_data.offset(4 as libc::c_int as isize);
    opj_read_bytes_LE(p_image_header_data, &mut (*jp2).w,
                      4 as libc::c_int as OPJ_UINT32);
    p_image_header_data =
        p_image_header_data.offset(4 as libc::c_int as isize);
    opj_read_bytes_LE(p_image_header_data, &mut (*jp2).numcomps,
                      2 as libc::c_int as OPJ_UINT32);
    p_image_header_data =
        p_image_header_data.offset(2 as libc::c_int as isize);
    if (*jp2).h < 1 as libc::c_int as libc::c_uint ||
           (*jp2).w < 1 as libc::c_int as libc::c_uint ||
           (*jp2).numcomps < 1 as libc::c_int as libc::c_uint {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Wrong values for: w(%d) h(%d) numcomps(%d) (ihdr)\n\x00"
                          as *const u8 as *const libc::c_char, (*jp2).w,
                      (*jp2).h, (*jp2).numcomps);
        return 0 as libc::c_int
    }
    if (*jp2).numcomps.wrapping_sub(1 as libc::c_uint) >=
           16384 as libc::c_uint {
        /* unsigned underflow is well defined: 1U <= jp2->numcomps <= 16384U */
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Invalid number of components (ihdr)\n\x00" as
                          *const u8 as *const libc::c_char);
        return 0 as libc::c_int
    }
    /* allocate memory for components */
    (*jp2).comps =
        opj_calloc((*jp2).numcomps as size_t,
                   ::std::mem::size_of::<opj_jp2_comps_t>() as libc::c_ulong)
            as *mut opj_jp2_comps_t; /* BPC */
    if (*jp2).comps.is_null() {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Not enough memory to handle image header (ihdr)\n\x00"
                          as *const u8 as *const libc::c_char); /* C */
        return 0 as libc::c_int
    }
    opj_read_bytes_LE(p_image_header_data, &mut (*jp2).bpc,
                      1 as libc::c_int as OPJ_UINT32);
    p_image_header_data = p_image_header_data.offset(1);
    opj_read_bytes_LE(p_image_header_data, &mut (*jp2).C,
                      1 as libc::c_int as OPJ_UINT32);
    p_image_header_data = p_image_header_data.offset(1);
    /* Should be equal to 7 cf. chapter about image header box of the norm */
    if (*jp2).C != 7 as libc::c_int as libc::c_uint {
        opj_event_msg(p_manager, 4 as libc::c_int,
                      b"JP2 IHDR box: compression type indicate that the file is not a conforming JP2 file (%d) \n\x00"
                          as *const u8 as *const libc::c_char,
                      (*jp2).C); /* UnkC */
    } /* IPR */
    opj_read_bytes_LE(p_image_header_data, &mut (*jp2).UnkC,
                      1 as libc::c_int as OPJ_UINT32);
    p_image_header_data = p_image_header_data.offset(1);
    opj_read_bytes_LE(p_image_header_data, &mut (*jp2).IPR,
                      1 as libc::c_int as OPJ_UINT32);
    p_image_header_data = p_image_header_data.offset(1);
    (*(*jp2).j2k).m_cp.set_allow_different_bit_depth_sign(((*jp2).bpc ==
                                                               255 as
                                                                   libc::c_int
                                                                   as
                                                                   libc::c_uint)
                                                              as libc::c_int
                                                              as
                                                              OPJ_BITFIELD);
    (*(*jp2).j2k).ihdr_w = (*jp2).w;
    (*(*jp2).j2k).ihdr_h = (*jp2).h;
    (*jp2).has_ihdr = 1 as libc::c_int as OPJ_BYTE;
    return 1 as libc::c_int;
}
/* *
 * Writes the Image Header box - Image Header box.
 *
 * @param jp2                   jpeg2000 file codec.
 * @param p_nb_bytes_written    pointer to store the nb of bytes written by the function.
 *
 * @return  the data being copied.
*/
unsafe extern "C" fn opj_jp2_write_ihdr(mut jp2: *mut opj_jp2_t,
                                        mut p_nb_bytes_written:
                                            *mut OPJ_UINT32)
 -> *mut OPJ_BYTE {
    let mut l_ihdr_data = 0 as *mut OPJ_BYTE;
    let mut l_current_ihdr_ptr = 0 as *mut OPJ_BYTE;
    /* preconditions */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      643 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 56],
                                                &[libc::c_char; 56]>(b"OPJ_BYTE *opj_jp2_write_ihdr(opj_jp2_t *, OPJ_UINT32 *)\x00")).as_ptr());
    }
    if !p_nb_bytes_written.is_null() {
    } else {
        __assert_fail(b"p_nb_bytes_written != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      644 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 56],
                                                &[libc::c_char; 56]>(b"OPJ_BYTE *opj_jp2_write_ihdr(opj_jp2_t *, OPJ_UINT32 *)\x00")).as_ptr());
    }
    /* default image header is 22 bytes wide */
    l_ihdr_data =
        opj_calloc(1 as libc::c_int as size_t, 22 as libc::c_int as size_t) as
            *mut OPJ_BYTE; /* write box size */
    if l_ihdr_data.is_null() { return 0 as *mut OPJ_BYTE } /* IHDR */
    l_current_ihdr_ptr = l_ihdr_data; /* HEIGHT */
    opj_write_bytes_LE(l_current_ihdr_ptr, 22 as libc::c_int as OPJ_UINT32,
                       4 as libc::c_int as OPJ_UINT32); /* WIDTH */
    l_current_ihdr_ptr =
        l_current_ihdr_ptr.offset(4 as libc::c_int as isize); /* NC */
    opj_write_bytes_LE(l_current_ihdr_ptr,
                       0x69686472 as libc::c_int as OPJ_UINT32,
                       4 as libc::c_int as OPJ_UINT32); /* BPC */
    l_current_ihdr_ptr =
        l_current_ihdr_ptr.offset(4 as libc::c_int as
                                      isize); /* C : Always 7 */
    opj_write_bytes_LE(l_current_ihdr_ptr, (*jp2).h,
                       4 as libc::c_int as
                           OPJ_UINT32); /* UnkC, colorspace unknown */
    l_current_ihdr_ptr =
        l_current_ihdr_ptr.offset(4 as libc::c_int as
                                      isize); /* IPR, no intellectual property */
    opj_write_bytes_LE(l_current_ihdr_ptr, (*jp2).w,
                       4 as libc::c_int as OPJ_UINT32);
    l_current_ihdr_ptr = l_current_ihdr_ptr.offset(4 as libc::c_int as isize);
    opj_write_bytes_LE(l_current_ihdr_ptr, (*jp2).numcomps,
                       2 as libc::c_int as OPJ_UINT32);
    l_current_ihdr_ptr = l_current_ihdr_ptr.offset(2 as libc::c_int as isize);
    opj_write_bytes_LE(l_current_ihdr_ptr, (*jp2).bpc,
                       1 as libc::c_int as OPJ_UINT32);
    l_current_ihdr_ptr = l_current_ihdr_ptr.offset(1);
    opj_write_bytes_LE(l_current_ihdr_ptr, (*jp2).C,
                       1 as libc::c_int as OPJ_UINT32);
    l_current_ihdr_ptr = l_current_ihdr_ptr.offset(1);
    opj_write_bytes_LE(l_current_ihdr_ptr, (*jp2).UnkC,
                       1 as libc::c_int as OPJ_UINT32);
    l_current_ihdr_ptr = l_current_ihdr_ptr.offset(1);
    opj_write_bytes_LE(l_current_ihdr_ptr, (*jp2).IPR,
                       1 as libc::c_int as OPJ_UINT32);
    l_current_ihdr_ptr = l_current_ihdr_ptr.offset(1);
    *p_nb_bytes_written = 22 as libc::c_int as OPJ_UINT32;
    return l_ihdr_data;
}
/* *
 * Writes the Bit per Component box.
 *
 * @param   jp2                     jpeg2000 file codec.
 * @param   p_nb_bytes_written      pointer to store the nb of bytes written by the function.
 *
 * @return  the data being copied.
*/
unsafe extern "C" fn opj_jp2_write_bpcc(mut jp2: *mut opj_jp2_t,
                                        mut p_nb_bytes_written:
                                            *mut OPJ_UINT32)
 -> *mut OPJ_BYTE {
    let mut i: OPJ_UINT32 = 0;
    /* room for 8 bytes for box and 1 byte for each component */
    let mut l_bpcc_size: OPJ_UINT32 = 0;
    let mut l_bpcc_data = 0 as *mut OPJ_BYTE;
    let mut l_current_bpcc_ptr = 0 as *mut OPJ_BYTE;
    /* preconditions */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      698 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 56],
                                                &[libc::c_char; 56]>(b"OPJ_BYTE *opj_jp2_write_bpcc(opj_jp2_t *, OPJ_UINT32 *)\x00")).as_ptr()); /* write box size */
    } /* BPCC */
    if !p_nb_bytes_written.is_null() {
    } else {
        __assert_fail(b"p_nb_bytes_written != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      699 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 56],
                                                &[libc::c_char; 56]>(b"OPJ_BYTE *opj_jp2_write_bpcc(opj_jp2_t *, OPJ_UINT32 *)\x00")).as_ptr()); /* write each component information */
    }
    l_bpcc_size =
        (8 as libc::c_int as libc::c_uint).wrapping_add((*jp2).numcomps);
    l_bpcc_data =
        opj_calloc(1 as libc::c_int as size_t, l_bpcc_size as size_t) as
            *mut OPJ_BYTE;
    if l_bpcc_data.is_null() { return 0 as *mut OPJ_BYTE }
    l_current_bpcc_ptr = l_bpcc_data;
    opj_write_bytes_LE(l_current_bpcc_ptr, l_bpcc_size,
                       4 as libc::c_int as OPJ_UINT32);
    l_current_bpcc_ptr = l_current_bpcc_ptr.offset(4 as libc::c_int as isize);
    opj_write_bytes_LE(l_current_bpcc_ptr,
                       0x62706363 as libc::c_int as OPJ_UINT32,
                       4 as libc::c_int as OPJ_UINT32);
    l_current_bpcc_ptr = l_current_bpcc_ptr.offset(4 as libc::c_int as isize);
    i = 0 as libc::c_int as OPJ_UINT32;
    while i < (*jp2).numcomps {
        opj_write_bytes_LE(l_current_bpcc_ptr,
                           (*(*jp2).comps.offset(i as isize)).bpcc,
                           1 as libc::c_int as OPJ_UINT32);
        l_current_bpcc_ptr = l_current_bpcc_ptr.offset(1);
        i = i.wrapping_add(1)
    }
    *p_nb_bytes_written = l_bpcc_size;
    return l_bpcc_data;
}
/* *
 * Reads a Bit per Component box.
 *
 * @param   p_bpc_header_data           pointer to actual data (already read from file)
 * @param   jp2                         the jpeg2000 file codec.
 * @param   p_bpc_header_size           the size of the bpc header
 * @param   p_manager                   the user event manager.
 *
 * @return  true if the bpc header is valid, false else.
 */
unsafe extern "C" fn opj_jp2_read_bpcc(mut jp2: *mut opj_jp2_t,
                                       mut p_bpc_header_data: *mut OPJ_BYTE,
                                       mut p_bpc_header_size: OPJ_UINT32,
                                       mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut i: OPJ_UINT32 = 0;
    /* preconditions */
    if !p_bpc_header_data.is_null() {
    } else {
        __assert_fail(b"p_bpc_header_data != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      736 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_bpcc(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      737 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_bpcc(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      738 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_bpcc(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if (*jp2).bpc != 255 as libc::c_int as libc::c_uint {
        opj_event_msg(p_manager, 2 as libc::c_int,
                      b"A BPCC header box is available although BPC given by the IHDR box (%d) indicate components bit depth is constant\n\x00"
                          as *const u8 as *const libc::c_char, (*jp2).bpc);
    }
    /* and length is relevant */
    if p_bpc_header_size != (*jp2).numcomps {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Bad BPCC header box (bad size)\n\x00" as *const u8 as
                          *const libc::c_char);
        return 0 as libc::c_int
    }
    /* read info for each component */
    i = 0 as libc::c_int as OPJ_UINT32; /* read each BPCC component */
    while i < (*jp2).numcomps {
        opj_read_bytes_LE(p_bpc_header_data,
                          &mut (*(*jp2).comps.offset(i as isize)).bpcc,
                          1 as libc::c_int as OPJ_UINT32);
        p_bpc_header_data = p_bpc_header_data.offset(1);
        i = i.wrapping_add(1)
    }
    return 1 as libc::c_int;
}
/* *
 * Writes the Channel Definition box.
 *
 * @param jp2                   jpeg2000 file codec.
 * @param p_nb_bytes_written    pointer to store the nb of bytes written by the function.
 *
 * @return  the data being copied.
 */
unsafe extern "C" fn opj_jp2_write_cdef(mut jp2: *mut opj_jp2_t,
                                        mut p_nb_bytes_written:
                                            *mut OPJ_UINT32)
 -> *mut OPJ_BYTE {
    /* room for 8 bytes for box, 2 for n */
    let mut l_cdef_size = 10 as libc::c_int as OPJ_UINT32;
    let mut l_cdef_data = 0 as *mut OPJ_BYTE;
    let mut l_current_cdef_ptr = 0 as *mut OPJ_BYTE;
    let mut l_value: OPJ_UINT32 = 0;
    let mut i: OPJ_UINT16 = 0;
    /* preconditions */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      772 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 56],
                                                &[libc::c_char; 56]>(b"OPJ_BYTE *opj_jp2_write_cdef(opj_jp2_t *, OPJ_UINT32 *)\x00")).as_ptr()); /* write box size */
    } /* BPCC */
    if !p_nb_bytes_written.is_null() {
    } else {
        __assert_fail(b"p_nb_bytes_written != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      773 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 56],
                                                &[libc::c_char; 56]>(b"OPJ_BYTE *opj_jp2_write_cdef(opj_jp2_t *, OPJ_UINT32 *)\x00")).as_ptr()); /* N */
    } /* Cni */
    if !(*jp2).color.jp2_cdef.is_null() {
    } else {
        __assert_fail(b"jp2->color.jp2_cdef != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      774 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 56],
                                                &[libc::c_char; 56]>(b"OPJ_BYTE *opj_jp2_write_cdef(opj_jp2_t *, OPJ_UINT32 *)\x00")).as_ptr()); /* Typi */
    } /* Asoci */
    if !(*(*jp2).color.jp2_cdef).info.is_null() {
    } else {
        __assert_fail(b"jp2->color.jp2_cdef->info != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      775 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 56],
                                                &[libc::c_char; 56]>(b"OPJ_BYTE *opj_jp2_write_cdef(opj_jp2_t *, OPJ_UINT32 *)\x00")).as_ptr());
    }
    if (*(*jp2).color.jp2_cdef).n as libc::c_uint > 0 as libc::c_uint {
    } else {
        __assert_fail(b"jp2->color.jp2_cdef->n > 0U\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      776 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 56],
                                                &[libc::c_char; 56]>(b"OPJ_BYTE *opj_jp2_write_cdef(opj_jp2_t *, OPJ_UINT32 *)\x00")).as_ptr());
    }
    l_cdef_size =
        (l_cdef_size as
             libc::c_uint).wrapping_add((6 as
                                             libc::c_uint).wrapping_mul((*(*jp2).color.jp2_cdef).n
                                                                            as
                                                                            libc::c_uint))
            as OPJ_UINT32 as OPJ_UINT32;
    l_cdef_data = opj_malloc(l_cdef_size as size_t) as *mut OPJ_BYTE;
    if l_cdef_data.is_null() { return 0 as *mut OPJ_BYTE }
    l_current_cdef_ptr = l_cdef_data;
    opj_write_bytes_LE(l_current_cdef_ptr, l_cdef_size,
                       4 as libc::c_int as OPJ_UINT32);
    l_current_cdef_ptr = l_current_cdef_ptr.offset(4 as libc::c_int as isize);
    opj_write_bytes_LE(l_current_cdef_ptr,
                       0x63646566 as libc::c_int as OPJ_UINT32,
                       4 as libc::c_int as OPJ_UINT32);
    l_current_cdef_ptr = l_current_cdef_ptr.offset(4 as libc::c_int as isize);
    l_value = (*(*jp2).color.jp2_cdef).n as OPJ_UINT32;
    opj_write_bytes_LE(l_current_cdef_ptr, l_value,
                       2 as libc::c_int as OPJ_UINT32);
    l_current_cdef_ptr = l_current_cdef_ptr.offset(2 as libc::c_int as isize);
    i = 0 as libc::c_uint as OPJ_UINT16;
    while (i as libc::c_int) < (*(*jp2).color.jp2_cdef).n as libc::c_int {
        l_value =
            (*(*(*jp2).color.jp2_cdef).info.offset(i as isize)).cn as
                OPJ_UINT32;
        opj_write_bytes_LE(l_current_cdef_ptr, l_value,
                           2 as libc::c_int as OPJ_UINT32);
        l_current_cdef_ptr =
            l_current_cdef_ptr.offset(2 as libc::c_int as isize);
        l_value =
            (*(*(*jp2).color.jp2_cdef).info.offset(i as isize)).typ as
                OPJ_UINT32;
        opj_write_bytes_LE(l_current_cdef_ptr, l_value,
                           2 as libc::c_int as OPJ_UINT32);
        l_current_cdef_ptr =
            l_current_cdef_ptr.offset(2 as libc::c_int as isize);
        l_value =
            (*(*(*jp2).color.jp2_cdef).info.offset(i as isize)).asoc as
                OPJ_UINT32;
        opj_write_bytes_LE(l_current_cdef_ptr, l_value,
                           2 as libc::c_int as OPJ_UINT32);
        l_current_cdef_ptr =
            l_current_cdef_ptr.offset(2 as libc::c_int as isize);
        i = i.wrapping_add(1)
    }
    *p_nb_bytes_written = l_cdef_size;
    return l_cdef_data;
}
/* *
 * Writes the Colour Specification box.
 *
 * @param jp2                   jpeg2000 file codec.
 * @param p_nb_bytes_written    pointer to store the nb of bytes written by the function.
 *
 * @return  the data being copied.
*/
unsafe extern "C" fn opj_jp2_write_colr(mut jp2: *mut opj_jp2_t,
                                        mut p_nb_bytes_written:
                                            *mut OPJ_UINT32)
 -> *mut OPJ_BYTE {
    /* room for 8 bytes for box 3 for common data and variable upon profile*/
    let mut l_colr_size = 11 as libc::c_int as OPJ_UINT32;
    let mut l_colr_data = 0 as *mut OPJ_BYTE;
    let mut l_current_colr_ptr = 0 as *mut OPJ_BYTE;
    /* preconditions */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      822 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 56],
                                                &[libc::c_char; 56]>(b"OPJ_BYTE *opj_jp2_write_colr(opj_jp2_t *, OPJ_UINT32 *)\x00")).as_ptr()); /* EnumCS */
    } /* ICC profile */
    if !p_nb_bytes_written.is_null() {
    } else {
        __assert_fail(b"p_nb_bytes_written != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      823 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 56],
                                                &[libc::c_char; 56]>(b"OPJ_BYTE *opj_jp2_write_colr(opj_jp2_t *, OPJ_UINT32 *)\x00")).as_ptr()); /* write box size */
    } /* BPCC */
    if (*jp2).meth == 1 as libc::c_int as libc::c_uint ||
           (*jp2).meth == 2 as libc::c_int as libc::c_uint {
    } else {
        __assert_fail(b"jp2->meth == 1 || jp2->meth == 2\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      824 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 56],
                                                &[libc::c_char; 56]>(b"OPJ_BYTE *opj_jp2_write_colr(opj_jp2_t *, OPJ_UINT32 *)\x00")).as_ptr()); /* METH */
    } /* PRECEDENCE */
    match (*jp2).meth {
        1 => {
            l_colr_size =
                (l_colr_size as
                     libc::c_uint).wrapping_add(4 as libc::c_int as
                                                    libc::c_uint) as
                    OPJ_UINT32 as OPJ_UINT32
        }
        2 => {
            if (*jp2).color.icc_profile_len != 0 {
            } else {
                __assert_fail(b"jp2->color.icc_profile_len\x00" as *const u8
                                  as *const libc::c_char,
                              b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as
                                  *const u8 as *const libc::c_char,
                              831 as libc::c_int as libc::c_uint,
                              (*::std::mem::transmute::<&[u8; 56],
                                                        &[libc::c_char; 56]>(b"OPJ_BYTE *opj_jp2_write_colr(opj_jp2_t *, OPJ_UINT32 *)\x00")).as_ptr()); /* APPROX */
            } /* EnumCS */
            l_colr_size =
                (l_colr_size as
                     libc::c_uint).wrapping_add((*jp2).color.icc_profile_len)
                    as OPJ_UINT32 as OPJ_UINT32
        }
        _ => { return 0 as *mut OPJ_BYTE }
    }
    l_colr_data =
        opj_calloc(1 as libc::c_int as size_t, l_colr_size as size_t) as
            *mut OPJ_BYTE;
    if l_colr_data.is_null() { return 0 as *mut OPJ_BYTE }
    l_current_colr_ptr = l_colr_data;
    opj_write_bytes_LE(l_current_colr_ptr, l_colr_size,
                       4 as libc::c_int as OPJ_UINT32);
    l_current_colr_ptr = l_current_colr_ptr.offset(4 as libc::c_int as isize);
    opj_write_bytes_LE(l_current_colr_ptr,
                       0x636f6c72 as libc::c_int as OPJ_UINT32,
                       4 as libc::c_int as OPJ_UINT32);
    l_current_colr_ptr = l_current_colr_ptr.offset(4 as libc::c_int as isize);
    opj_write_bytes_LE(l_current_colr_ptr, (*jp2).meth,
                       1 as libc::c_int as OPJ_UINT32);
    l_current_colr_ptr = l_current_colr_ptr.offset(1);
    opj_write_bytes_LE(l_current_colr_ptr, (*jp2).precedence,
                       1 as libc::c_int as OPJ_UINT32);
    l_current_colr_ptr = l_current_colr_ptr.offset(1);
    opj_write_bytes_LE(l_current_colr_ptr, (*jp2).approx,
                       1 as libc::c_int as OPJ_UINT32);
    l_current_colr_ptr = l_current_colr_ptr.offset(1);
    if (*jp2).meth == 1 as libc::c_int as libc::c_uint {
        /* Meth value is restricted to 1 or 2 (Table I.9 of part 1) */
        opj_write_bytes_LE(l_current_colr_ptr, (*jp2).enumcs,
                           4 as libc::c_int as OPJ_UINT32);
    } else if (*jp2).meth == 2 as libc::c_int as libc::c_uint {
        /* ICC profile */
        let mut i: OPJ_UINT32 = 0;
        i = 0 as libc::c_int as OPJ_UINT32;
        while i < (*jp2).color.icc_profile_len {
            opj_write_bytes_LE(l_current_colr_ptr,
                               *(*jp2).color.icc_profile_buf.offset(i as
                                                                        isize)
                                   as OPJ_UINT32,
                               1 as libc::c_int as OPJ_UINT32);
            l_current_colr_ptr = l_current_colr_ptr.offset(1);
            i = i.wrapping_add(1)
        }
    }
    *p_nb_bytes_written = l_colr_size;
    return l_colr_data;
}
unsafe extern "C" fn opj_jp2_free_pclr(mut color: *mut opj_jp2_color_t) {
    opj_free((*(*color).jp2_pclr).channel_sign as *mut libc::c_void);
    opj_free((*(*color).jp2_pclr).channel_size as *mut libc::c_void);
    opj_free((*(*color).jp2_pclr).entries as *mut libc::c_void);
    if !(*(*color).jp2_pclr).cmap.is_null() {
        opj_free((*(*color).jp2_pclr).cmap as *mut libc::c_void);
    }
    opj_free((*color).jp2_pclr as *mut libc::c_void);
    (*color).jp2_pclr = 0 as *mut opj_jp2_pclr_t;
}
unsafe extern "C" fn opj_jp2_check_color(mut image: *mut opj_image_t,
                                         mut color: *mut opj_jp2_color_t,
                                         mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut i: OPJ_UINT16 = 0;
    /* testcase 4149.pdf.SIGSEGV.cf7.3501 */
    if !(*color).jp2_cdef.is_null() {
        let mut info =
            (*(*color).jp2_cdef).info; /* FIXME image->numcomps == jp2->numcomps before color is applied ??? */
        let mut n = (*(*color).jp2_cdef).n;
        let mut nr_channels = (*image).numcomps;
        /* cdef applies to cmap channels if any */
        if !(*color).jp2_pclr.is_null() &&
               !(*(*color).jp2_pclr).cmap.is_null() {
            nr_channels = (*(*color).jp2_pclr).nr_channels as OPJ_UINT32
        }
        i = 0 as libc::c_int as OPJ_UINT16;
        while (i as libc::c_int) < n as libc::c_int {
            if (*info.offset(i as isize)).cn as libc::c_uint >= nr_channels {
                opj_event_msg(p_manager, 1 as libc::c_int,
                              b"Invalid component index %d (>= %d).\n\x00" as
                                  *const u8 as *const libc::c_char,
                              (*info.offset(i as isize)).cn as libc::c_int,
                              nr_channels);
                return 0 as libc::c_int
            }
            if !((*info.offset(i as isize)).asoc as libc::c_uint ==
                     65535 as libc::c_uint) {
                if (*info.offset(i as isize)).asoc as libc::c_int >
                       0 as libc::c_int &&
                       ((*info.offset(i as isize)).asoc as libc::c_int -
                            1 as libc::c_int) as OPJ_UINT32 >= nr_channels {
                    opj_event_msg(p_manager, 1 as libc::c_int,
                                  b"Invalid component index %d (>= %d).\n\x00"
                                      as *const u8 as *const libc::c_char,
                                  (*info.offset(i as isize)).asoc as
                                      libc::c_int - 1 as libc::c_int,
                                  nr_channels);
                    return 0 as libc::c_int
                }
            }
            i = i.wrapping_add(1)
        }
        /* issue 397 */
        /* ISO 15444-1 states that if cdef is present, it shall contain a complete list of channel definitions. */
        while nr_channels > 0 as libc::c_int as libc::c_uint {
            i = 0 as libc::c_int as OPJ_UINT16;
            while (i as libc::c_int) < n as libc::c_int {
                if (*info.offset(i as isize)).cn as OPJ_UINT32 ==
                       nr_channels.wrapping_sub(1 as libc::c_uint) {
                    break ;
                }
                i = i.wrapping_add(1)
            }
            if i as libc::c_int == n as libc::c_int {
                opj_event_msg(p_manager, 1 as libc::c_int,
                              b"Incomplete channel definitions.\n\x00" as
                                  *const u8 as *const libc::c_char);
                return 0 as libc::c_int
            }
            nr_channels = nr_channels.wrapping_sub(1)
        }
    }
    /* testcases 451.pdf.SIGSEGV.f4c.3723, 451.pdf.SIGSEGV.5b5.3723 and
       66ea31acbb0f23a2bbc91f64d69a03f5_signal_sigsegv_13937c0_7030_5725.pdf */
    if !(*color).jp2_pclr.is_null() && !(*(*color).jp2_pclr).cmap.is_null() {
        let mut nr_channels_0 =
            (*(*color).jp2_pclr).nr_channels as OPJ_UINT16;
        let mut cmap = (*(*color).jp2_pclr).cmap;
        let mut pcol_usage = 0 as *mut OPJ_BOOL;
        let mut is_sane = 1 as libc::c_int;
        /* verify that all original components match an existing one */
        i = 0 as libc::c_int as OPJ_UINT16;
        while (i as libc::c_int) < nr_channels_0 as libc::c_int {
            if (*cmap.offset(i as isize)).cmp as libc::c_uint >=
                   (*image).numcomps {
                opj_event_msg(p_manager, 1 as libc::c_int,
                              b"Invalid component index %d (>= %d).\n\x00" as
                                  *const u8 as *const libc::c_char,
                              (*cmap.offset(i as isize)).cmp as libc::c_int,
                              (*image).numcomps);
                is_sane = 0 as libc::c_int
            }
            i = i.wrapping_add(1)
        }
        pcol_usage =
            opj_calloc(nr_channels_0 as size_t,
                       ::std::mem::size_of::<OPJ_BOOL>() as libc::c_ulong) as
                *mut OPJ_BOOL;
        if pcol_usage.is_null() {
            opj_event_msg(p_manager, 1 as libc::c_int,
                          b"Unexpected OOM.\n\x00" as *const u8 as
                              *const libc::c_char);
            return 0 as libc::c_int
        }
        /* verify that no component is targeted more than once */
        i = 0 as libc::c_int as OPJ_UINT16;
        while (i as libc::c_int) < nr_channels_0 as libc::c_int {
            let mut mtyp = (*cmap.offset(i as isize)).mtyp;
            let mut pcol = (*cmap.offset(i as isize)).pcol;
            /* See ISO 15444-1 Table I.14 – MTYPi field values */
            if mtyp as libc::c_int != 0 as libc::c_int &&
                   mtyp as libc::c_int != 1 as libc::c_int {
                opj_event_msg(p_manager, 1 as libc::c_int,
                              b"Invalid value for cmap[%d].mtyp = %d.\n\x00"
                                  as *const u8 as *const libc::c_char,
                              i as libc::c_int, mtyp as libc::c_int);
                is_sane = 0 as libc::c_int
            } else if pcol as libc::c_int >= nr_channels_0 as libc::c_int {
                opj_event_msg(p_manager, 1 as libc::c_int,
                              b"Invalid component/palette index for direct mapping %d.\n\x00"
                                  as *const u8 as *const libc::c_char,
                              pcol as libc::c_int);
                is_sane = 0 as libc::c_int
            } else if *pcol_usage.offset(pcol as isize) != 0 &&
                          mtyp as libc::c_int == 1 as libc::c_int {
                opj_event_msg(p_manager, 1 as libc::c_int,
                              b"Component %d is mapped twice.\n\x00" as
                                  *const u8 as *const libc::c_char,
                              pcol as libc::c_int);
                is_sane = 0 as libc::c_int
            } else if mtyp as libc::c_int == 0 as libc::c_int &&
                          pcol as libc::c_int != 0 as libc::c_int {
                /* I.5.3.5 PCOL: If the value of the MTYP field for this channel is 0, then
                 * the value of this field shall be 0. */
                opj_event_msg(p_manager, 1 as libc::c_int,
                              b"Direct use at #%d however pcol=%d.\n\x00" as
                                  *const u8 as *const libc::c_char,
                              i as libc::c_int, pcol as libc::c_int);
                is_sane = 0 as libc::c_int
            } else if mtyp as libc::c_int == 1 as libc::c_int &&
                          pcol as libc::c_int != i as libc::c_int {
                /* OpenJPEG implementation limitation. See assert(i == pcol); */
                /* in opj_jp2_apply_pclr() */
                opj_event_msg(p_manager, 1 as libc::c_int,
                              b"Implementation limitation: for palette mapping, pcol[%d] should be equal to %d, but is equal to %d.\n\x00"
                                  as *const u8 as *const libc::c_char,
                              i as libc::c_int, i as libc::c_int,
                              pcol as libc::c_int);
                is_sane = 0 as libc::c_int
            } else { *pcol_usage.offset(pcol as isize) = 1 as libc::c_int }
            i = i.wrapping_add(1)
        }
        /* verify that all components are targeted at least once */
        i = 0 as libc::c_int as OPJ_UINT16;
        while (i as libc::c_int) < nr_channels_0 as libc::c_int {
            if *pcol_usage.offset(i as isize) == 0 &&
                   (*cmap.offset(i as isize)).mtyp as libc::c_int !=
                       0 as libc::c_int {
                opj_event_msg(p_manager, 1 as libc::c_int,
                              b"Component %d doesn\'t have a mapping.\n\x00"
                                  as *const u8 as *const libc::c_char,
                              i as libc::c_int);
                is_sane = 0 as libc::c_int
            }
            i = i.wrapping_add(1)
        }
        /* Issue 235/447 weird cmap */
        if 1 as libc::c_int != 0 && is_sane != 0 &&
               (*image).numcomps == 1 as libc::c_uint {
            i = 0 as libc::c_int as OPJ_UINT16;
            while (i as libc::c_int) < nr_channels_0 as libc::c_int {
                if *pcol_usage.offset(i as isize) == 0 {
                    is_sane = 0 as libc::c_uint as OPJ_BOOL;
                    opj_event_msg(p_manager, 2 as libc::c_int,
                                  b"Component mapping seems wrong. Trying to correct.\n\x00"
                                      as *const u8 as *const libc::c_char);
                    break ;
                } else { i = i.wrapping_add(1) }
            }
            if is_sane == 0 {
                is_sane = 1 as libc::c_int;
                i = 0 as libc::c_int as OPJ_UINT16;
                while (i as libc::c_int) < nr_channels_0 as libc::c_int {
                    (*cmap.offset(i as isize)).mtyp =
                        1 as libc::c_uint as OPJ_BYTE;
                    (*cmap.offset(i as isize)).pcol = i as OPJ_BYTE;
                    i = i.wrapping_add(1)
                }
            }
        }
        opj_free(pcol_usage as *mut libc::c_void);
        if is_sane == 0 { return 0 as libc::c_int }
    }
    return 1 as libc::c_int;
}
/* *
Apply collected palette data
@param image Image.
@param color Collector for profile, cdef and pclr data.
@param p_manager the user event manager.
@return true in case of success
*/
/* file9.jp2 */
unsafe extern "C" fn opj_jp2_apply_pclr(mut image: *mut opj_image_t,
                                        mut color: *mut opj_jp2_color_t,
                                        mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut old_comps = 0 as *mut opj_image_comp_t;
    let mut new_comps = 0 as *mut opj_image_comp_t;
    let mut channel_size = 0 as *mut OPJ_BYTE;
    let mut channel_sign = 0 as *mut OPJ_BYTE;
    let mut entries = 0 as *mut OPJ_UINT32;
    let mut cmap = 0 as *mut opj_jp2_cmap_comp_t;
    let mut src = 0 as *mut OPJ_INT32;
    let mut dst = 0 as *mut OPJ_INT32;
    let mut j: OPJ_UINT32 = 0;
    let mut max: OPJ_UINT32 = 0;
    let mut i: OPJ_UINT16 = 0;
    let mut nr_channels: OPJ_UINT16 = 0;
    let mut cmp: OPJ_UINT16 = 0;
    let mut pcol: OPJ_UINT16 = 0;
    let mut k: OPJ_INT32 = 0;
    let mut top_k: OPJ_INT32 = 0;
    channel_size = (*(*color).jp2_pclr).channel_size;
    channel_sign = (*(*color).jp2_pclr).channel_sign;
    entries = (*(*color).jp2_pclr).entries;
    cmap = (*(*color).jp2_pclr).cmap;
    nr_channels = (*(*color).jp2_pclr).nr_channels as OPJ_UINT16;
    i = 0 as libc::c_int as OPJ_UINT16;
    while (i as libc::c_int) < nr_channels as libc::c_int {
        /* Palette mapping: */
        cmp = (*cmap.offset(i as isize)).cmp;
        if (*(*image).comps.offset(cmp as isize)).data.is_null() {
            opj_event_msg(p_manager, 1 as libc::c_int,
                          b"image->comps[%d].data == NULL in opj_jp2_apply_pclr().\n\x00"
                              as *const u8 as *const libc::c_char,
                          i as libc::c_int);
            return 0 as libc::c_int
        }
        i = i.wrapping_add(1)
    }
    old_comps = (*image).comps;
    new_comps =
        opj_malloc((nr_channels as
                        libc::c_ulong).wrapping_mul(::std::mem::size_of::<opj_image_comp_t>()
                                                        as libc::c_ulong)) as
            *mut opj_image_comp_t;
    if new_comps.is_null() {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Memory allocation failure in opj_jp2_apply_pclr().\n\x00"
                          as *const u8 as *const libc::c_char);
        return 0 as libc::c_int
    }
    i = 0 as libc::c_int as OPJ_UINT16;
    while (i as libc::c_int) < nr_channels as libc::c_int {
        pcol = (*cmap.offset(i as isize)).pcol as OPJ_UINT16;
        cmp = (*cmap.offset(i as isize)).cmp;
        /* Direct use */
        if (*cmap.offset(i as isize)).mtyp as libc::c_int == 0 as libc::c_int
           {
            if pcol as libc::c_int == 0 as libc::c_int {
            } else {
                __assert_fail(b"pcol == 0\x00" as *const u8 as
                                  *const libc::c_char,
                              b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as
                                  *const u8 as *const libc::c_char,
                              1079 as libc::c_int as libc::c_uint,
                              (*::std::mem::transmute::<&[u8; 81],
                                                        &[libc::c_char; 81]>(b"OPJ_BOOL opj_jp2_apply_pclr(opj_image_t *, opj_jp2_color_t *, opj_event_mgr_t *)\x00")).as_ptr());
            }
            *new_comps.offset(i as isize) = *old_comps.offset(cmp as isize)
        } else {
            if i as libc::c_int == pcol as libc::c_int {
            } else {
                __assert_fail(b"i == pcol\x00" as *const u8 as
                                  *const libc::c_char,
                              b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as
                                  *const u8 as *const libc::c_char,
                              1082 as libc::c_int as libc::c_uint,
                              (*::std::mem::transmute::<&[u8; 81],
                                                        &[libc::c_char; 81]>(b"OPJ_BOOL opj_jp2_apply_pclr(opj_image_t *, opj_jp2_color_t *, opj_event_mgr_t *)\x00")).as_ptr());
            }
            *new_comps.offset(pcol as isize) = *old_comps.offset(cmp as isize)
        }
        /* Palette mapping: */
        let ref mut fresh0 = (*new_comps.offset(i as isize)).data;
        *fresh0 =
            opj_image_data_alloc((::std::mem::size_of::<OPJ_INT32>() as
                                      libc::c_ulong).wrapping_mul((*old_comps.offset(cmp
                                                                                         as
                                                                                         isize)).w
                                                                      as
                                                                      libc::c_ulong).wrapping_mul((*old_comps.offset(cmp
                                                                                                                         as
                                                                                                                         isize)).h
                                                                                                      as
                                                                                                      libc::c_ulong))
                as *mut OPJ_INT32;
        if (*new_comps.offset(i as isize)).data.is_null() {
            while i as libc::c_int > 0 as libc::c_int {
                i = i.wrapping_sub(1);
                opj_image_data_free((*new_comps.offset(i as isize)).data as
                                        *mut libc::c_void);
            }
            opj_free(new_comps as *mut libc::c_void);
            opj_event_msg(p_manager, 1 as libc::c_int,
                          b"Memory allocation failure in opj_jp2_apply_pclr().\n\x00"
                              as *const u8 as *const libc::c_char);
            return 0 as libc::c_int
        }
        (*new_comps.offset(i as isize)).prec =
            *channel_size.offset(i as isize) as OPJ_UINT32;
        (*new_comps.offset(i as isize)).sgnd =
            *channel_sign.offset(i as isize) as OPJ_UINT32;
        i = i.wrapping_add(1)
    }
    top_k = (*(*color).jp2_pclr).nr_entries as libc::c_int - 1 as libc::c_int;
    i = 0 as libc::c_int as OPJ_UINT16;
    while (i as libc::c_int) < nr_channels as libc::c_int {
        /* Palette mapping: */
        cmp = (*cmap.offset(i as isize)).cmp; /* verified above */
        pcol = (*cmap.offset(i as isize)).pcol as OPJ_UINT16;
        src = (*old_comps.offset(cmp as isize)).data;
        if !src.is_null() {
        } else {
            __assert_fail(b"src\x00" as *const u8 as *const libc::c_char,
                          b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as
                              *const u8 as *const libc::c_char,
                          1110 as libc::c_int as libc::c_uint,
                          (*::std::mem::transmute::<&[u8; 81],
                                                    &[libc::c_char; 81]>(b"OPJ_BOOL opj_jp2_apply_pclr(opj_image_t *, opj_jp2_color_t *, opj_event_mgr_t *)\x00")).as_ptr());
        }
        max =
            (*new_comps.offset(pcol as
                                   isize)).w.wrapping_mul((*new_comps.offset(pcol
                                                                                 as
                                                                                 isize)).h);
        /* Direct use: */
        if (*cmap.offset(i as isize)).mtyp as libc::c_int == 0 as libc::c_int
           {
            dst = (*new_comps.offset(i as isize)).data;
            if !dst.is_null() {
            } else {
                __assert_fail(b"dst\x00" as *const u8 as *const libc::c_char,
                              b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as
                                  *const u8 as *const libc::c_char,
                              1116 as libc::c_int as libc::c_uint,
                              (*::std::mem::transmute::<&[u8; 81],
                                                        &[libc::c_char; 81]>(b"OPJ_BOOL opj_jp2_apply_pclr(opj_image_t *, opj_jp2_color_t *, opj_event_mgr_t *)\x00")).as_ptr());
            }
            j = 0 as libc::c_int as OPJ_UINT32;
            while j < max {
                *dst.offset(j as isize) = *src.offset(j as isize);
                j = j.wrapping_add(1)
            }
        } else {
            if i as libc::c_int == pcol as libc::c_int {
            } else {
                __assert_fail(b"i == pcol\x00" as *const u8 as
                                  *const libc::c_char,
                              b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as
                                  *const u8 as *const libc::c_char,
                              1121 as libc::c_int as libc::c_uint,
                              (*::std::mem::transmute::<&[u8; 81],
                                                        &[libc::c_char; 81]>(b"OPJ_BOOL opj_jp2_apply_pclr(opj_image_t *, opj_jp2_color_t *, opj_event_mgr_t *)\x00")).as_ptr());
            }
            dst = (*new_comps.offset(pcol as isize)).data;
            if !dst.is_null() {
            } else {
                __assert_fail(b"dst\x00" as *const u8 as *const libc::c_char,
                              b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as
                                  *const u8 as *const libc::c_char,
                              1123 as libc::c_int as libc::c_uint,
                              (*::std::mem::transmute::<&[u8; 81],
                                                        &[libc::c_char; 81]>(b"OPJ_BOOL opj_jp2_apply_pclr(opj_image_t *, opj_jp2_color_t *, opj_event_mgr_t *)\x00")).as_ptr());
            }
            j = 0 as libc::c_int as OPJ_UINT32;
            while j < max {
                /* The index */
                k = *src.offset(j as isize);
                if k < 0 as libc::c_int {
                    k = 0 as libc::c_int
                } else if k > top_k { k = top_k }
                /* The colour */
                *dst.offset(j as isize) =
                    *entries.offset((k * nr_channels as libc::c_int +
                                         pcol as libc::c_int) as isize) as
                        OPJ_INT32;
                j = j.wrapping_add(1)
            }
        }
        i = i.wrapping_add(1)
    }
    max = (*image).numcomps;
    j = 0 as libc::c_int as OPJ_UINT32;
    while j < max {
        if !(*old_comps.offset(j as isize)).data.is_null() {
            opj_image_data_free((*old_comps.offset(j as isize)).data as
                                    *mut libc::c_void);
        }
        j = j.wrapping_add(1)
    }
    opj_free(old_comps as *mut libc::c_void);
    (*image).comps = new_comps;
    (*image).numcomps = nr_channels as OPJ_UINT32;
    return 1 as libc::c_int;
}
/* *
 * Collect palette data
 *
 * @param jp2 JP2 handle
 * @param p_pclr_header_data    FIXME DOC
 * @param p_pclr_header_size    FIXME DOC
 * @param p_manager
 *
 * @return Returns true if successful, returns false otherwise
*/
/* apply_pclr() */
unsafe extern "C" fn opj_jp2_read_pclr(mut jp2: *mut opj_jp2_t,
                                       mut p_pclr_header_data: *mut OPJ_BYTE,
                                       mut p_pclr_header_size: OPJ_UINT32,
                                       mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut jp2_pclr = 0 as *mut opj_jp2_pclr_t;
    let mut channel_size = 0 as *mut OPJ_BYTE;
    let mut channel_sign = 0 as *mut OPJ_BYTE;
    let mut entries = 0 as *mut OPJ_UINT32;
    let mut nr_entries: OPJ_UINT16 = 0;
    let mut nr_channels: OPJ_UINT16 = 0;
    let mut i: OPJ_UINT16 = 0;
    let mut j: OPJ_UINT16 = 0;
    let mut l_value: OPJ_UINT32 = 0;
    let mut orig_header_data = p_pclr_header_data;
    /* preconditions */
    if !p_pclr_header_data.is_null() {
    } else {
        __assert_fail(b"p_pclr_header_data != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1167 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_pclr(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr()); /* NE */
    } /* NPC */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1168 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_pclr(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr()); /* Bi */
    } /* Cji */
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1169 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_pclr(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !(*jp2).color.jp2_pclr.is_null() { return 0 as libc::c_int }
    if p_pclr_header_size < 3 as libc::c_int as libc::c_uint {
        return 0 as libc::c_int
    }
    opj_read_bytes_LE(p_pclr_header_data, &mut l_value,
                      2 as libc::c_int as OPJ_UINT32);
    p_pclr_header_data = p_pclr_header_data.offset(2 as libc::c_int as isize);
    nr_entries = l_value as OPJ_UINT16;
    if nr_entries as libc::c_uint == 0 as libc::c_uint ||
           nr_entries as libc::c_uint > 1024 as libc::c_uint {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Invalid PCLR box. Reports %d entries\n\x00" as
                          *const u8 as *const libc::c_char,
                      nr_entries as libc::c_int);
        return 0 as libc::c_int
    }
    opj_read_bytes_LE(p_pclr_header_data, &mut l_value,
                      1 as libc::c_int as OPJ_UINT32);
    p_pclr_header_data = p_pclr_header_data.offset(1);
    nr_channels = l_value as OPJ_UINT16;
    if nr_channels as libc::c_uint == 0 as libc::c_uint {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Invalid PCLR box. Reports 0 palette columns\n\x00" as
                          *const u8 as *const libc::c_char);
        return 0 as libc::c_int
    }
    if p_pclr_header_size <
           (3 as libc::c_int as
                libc::c_uint).wrapping_add(nr_channels as OPJ_UINT32) {
        return 0 as libc::c_int
    }
    entries =
        opj_malloc((::std::mem::size_of::<OPJ_UINT32>() as
                        libc::c_ulong).wrapping_mul(nr_channels as
                                                        libc::c_ulong).wrapping_mul(nr_entries
                                                                                        as
                                                                                        libc::c_ulong))
            as *mut OPJ_UINT32;
    if entries.is_null() { return 0 as libc::c_int }
    channel_size = opj_malloc(nr_channels as size_t) as *mut OPJ_BYTE;
    if channel_size.is_null() {
        opj_free(entries as *mut libc::c_void);
        return 0 as libc::c_int
    }
    channel_sign = opj_malloc(nr_channels as size_t) as *mut OPJ_BYTE;
    if channel_sign.is_null() {
        opj_free(entries as *mut libc::c_void);
        opj_free(channel_size as *mut libc::c_void);
        return 0 as libc::c_int
    }
    jp2_pclr =
        opj_malloc(::std::mem::size_of::<opj_jp2_pclr_t>() as libc::c_ulong)
            as *mut opj_jp2_pclr_t;
    if jp2_pclr.is_null() {
        opj_free(entries as *mut libc::c_void);
        opj_free(channel_size as *mut libc::c_void);
        opj_free(channel_sign as *mut libc::c_void);
        return 0 as libc::c_int
    }
    (*jp2_pclr).channel_sign = channel_sign;
    (*jp2_pclr).channel_size = channel_size;
    (*jp2_pclr).entries = entries;
    (*jp2_pclr).nr_entries = nr_entries;
    (*jp2_pclr).nr_channels = l_value as OPJ_BYTE;
    (*jp2_pclr).cmap = 0 as *mut opj_jp2_cmap_comp_t;
    (*jp2).color.jp2_pclr = jp2_pclr;
    i = 0 as libc::c_int as OPJ_UINT16;
    while (i as libc::c_int) < nr_channels as libc::c_int {
        opj_read_bytes_LE(p_pclr_header_data, &mut l_value,
                          1 as libc::c_int as OPJ_UINT32);
        p_pclr_header_data = p_pclr_header_data.offset(1);
        *channel_size.offset(i as isize) =
            (l_value &
                 0x7f as libc::c_int as
                     libc::c_uint).wrapping_add(1 as libc::c_int as
                                                    libc::c_uint) as OPJ_BYTE;
        *channel_sign.offset(i as isize) =
            if l_value & 0x80 as libc::c_int as libc::c_uint != 0 {
                1 as libc::c_int
            } else { 0 as libc::c_int } as OPJ_BYTE;
        i = i.wrapping_add(1)
    }
    j = 0 as libc::c_int as OPJ_UINT16;
    while (j as libc::c_int) < nr_entries as libc::c_int {
        i = 0 as libc::c_int as OPJ_UINT16;
        while (i as libc::c_int) < nr_channels as libc::c_int {
            let mut bytes_to_read =
                (*channel_size.offset(i as isize) as libc::c_int +
                     7 as libc::c_int >> 3 as libc::c_int) as OPJ_UINT32;
            if bytes_to_read as libc::c_ulong >
                   ::std::mem::size_of::<OPJ_UINT32>() as libc::c_ulong {
                bytes_to_read =
                    ::std::mem::size_of::<OPJ_UINT32>() as libc::c_ulong as
                        OPJ_UINT32
            }
            if (p_pclr_header_size as ptrdiff_t) <
                   p_pclr_header_data.wrapping_offset_from(orig_header_data)
                       as libc::c_long + bytes_to_read as ptrdiff_t {
                return 0 as libc::c_int
            }
            opj_read_bytes_LE(p_pclr_header_data, &mut l_value,
                              bytes_to_read);
            p_pclr_header_data =
                p_pclr_header_data.offset(bytes_to_read as isize);
            *entries = l_value;
            entries = entries.offset(1);
            i = i.wrapping_add(1)
        }
        j = j.wrapping_add(1)
    }
    return 1 as libc::c_int;
}
/* *
 * Collect component mapping data
 *
 * @param jp2                 JP2 handle
 * @param p_cmap_header_data  FIXME DOC
 * @param p_cmap_header_size  FIXME DOC
 * @param p_manager           FIXME DOC
 *
 * @return Returns true if successful, returns false otherwise
*/
unsafe extern "C" fn opj_jp2_read_cmap(mut jp2: *mut opj_jp2_t,
                                       mut p_cmap_header_data: *mut OPJ_BYTE,
                                       mut p_cmap_header_size: OPJ_UINT32,
                                       mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut cmap = 0 as *mut opj_jp2_cmap_comp_t;
    let mut i: OPJ_BYTE = 0;
    let mut nr_channels: OPJ_BYTE = 0;
    let mut l_value: OPJ_UINT32 = 0;
    /* preconditions */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1277 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_cmap(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_cmap_header_data.is_null() {
    } else {
        __assert_fail(b"p_cmap_header_data != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1278 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_cmap(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1279 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_cmap(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    /* Need nr_channels: */
    if (*jp2).color.jp2_pclr.is_null() {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Need to read a PCLR box before the CMAP box.\n\x00" as
                          *const u8 as *const libc::c_char);
        return 0 as libc::c_int
    }
    /* Part 1, I.5.3.5: 'There shall be at most one Component Mapping box
     * inside a JP2 Header box' :
    */
    if !(*(*jp2).color.jp2_pclr).cmap.is_null() {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Only one CMAP box is allowed.\n\x00" as *const u8 as
                          *const libc::c_char); /* CMP^i */
        return 0 as libc::c_int
    } /* MTYP^i */
    nr_channels = (*(*jp2).color.jp2_pclr).nr_channels; /* PCOL^i */
    if p_cmap_header_size <
           (nr_channels as
                OPJ_UINT32).wrapping_mul(4 as libc::c_int as libc::c_uint) {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Insufficient data for CMAP box.\n\x00" as *const u8 as
                          *const libc::c_char);
        return 0 as libc::c_int
    }
    cmap =
        opj_malloc((nr_channels as
                        libc::c_ulong).wrapping_mul(::std::mem::size_of::<opj_jp2_cmap_comp_t>()
                                                        as libc::c_ulong)) as
            *mut opj_jp2_cmap_comp_t;
    if cmap.is_null() { return 0 as libc::c_int }
    i = 0 as libc::c_int as OPJ_BYTE;
    while (i as libc::c_int) < nr_channels as libc::c_int {
        opj_read_bytes_LE(p_cmap_header_data, &mut l_value,
                          2 as libc::c_int as OPJ_UINT32);
        p_cmap_header_data =
            p_cmap_header_data.offset(2 as libc::c_int as isize);
        (*cmap.offset(i as isize)).cmp = l_value as OPJ_UINT16;
        opj_read_bytes_LE(p_cmap_header_data, &mut l_value,
                          1 as libc::c_int as OPJ_UINT32);
        p_cmap_header_data = p_cmap_header_data.offset(1);
        (*cmap.offset(i as isize)).mtyp = l_value as OPJ_BYTE;
        opj_read_bytes_LE(p_cmap_header_data, &mut l_value,
                          1 as libc::c_int as OPJ_UINT32);
        p_cmap_header_data = p_cmap_header_data.offset(1);
        (*cmap.offset(i as isize)).pcol = l_value as OPJ_BYTE;
        i = i.wrapping_add(1)
    }
    (*(*jp2).color.jp2_pclr).cmap = cmap;
    return 1 as libc::c_int;
}
unsafe extern "C" fn opj_jp2_apply_cdef(mut image: *mut opj_image_t,
                                        mut color: *mut opj_jp2_color_t,
                                        mut manager: *mut opj_event_mgr_t) {
    let mut info = 0 as *mut opj_jp2_cdef_info_t;
    let mut i: OPJ_UINT16 = 0;
    let mut n: OPJ_UINT16 = 0;
    let mut cn: OPJ_UINT16 = 0;
    let mut asoc: OPJ_UINT16 = 0;
    let mut acn: OPJ_UINT16 = 0;
    info = (*(*color).jp2_cdef).info;
    n = (*(*color).jp2_cdef).n;
    i = 0 as libc::c_int as OPJ_UINT16;
    while (i as libc::c_int) < n as libc::c_int {
        /* WATCH: acn = asoc - 1 ! */
        asoc = (*info.offset(i as isize)).asoc;
        cn = (*info.offset(i as isize)).cn;
        if cn as libc::c_uint >= (*image).numcomps {
            opj_event_msg(manager, 2 as libc::c_int,
                          b"opj_jp2_apply_cdef: cn=%d, numcomps=%d\n\x00" as
                              *const u8 as *const libc::c_char,
                          cn as libc::c_int, (*image).numcomps);
        } else if asoc as libc::c_int == 0 as libc::c_int ||
                      asoc as libc::c_int == 65535 as libc::c_int {
            (*(*image).comps.offset(cn as isize)).alpha =
                (*info.offset(i as isize)).typ
        } else {
            acn = (asoc as libc::c_int - 1 as libc::c_int) as OPJ_UINT16;
            if acn as libc::c_uint >= (*image).numcomps {
                opj_event_msg(manager, 2 as libc::c_int,
                              b"opj_jp2_apply_cdef: acn=%d, numcomps=%d\n\x00"
                                  as *const u8 as *const libc::c_char,
                              acn as libc::c_int, (*image).numcomps);
            } else {
                /* Swap only if color channel */
                if cn as libc::c_int != acn as libc::c_int &&
                       (*info.offset(i as isize)).typ as libc::c_int ==
                           0 as libc::c_int {
                    let mut saved =
                        opj_image_comp_t{dx: 0,
                                         dy: 0,
                                         w: 0,
                                         h: 0,
                                         x0: 0,
                                         y0: 0,
                                         prec: 0,
                                         bpp: 0,
                                         sgnd: 0,
                                         resno_decoded: 0,
                                         factor: 0,
                                         data: 0 as *mut OPJ_INT32,
                                         alpha: 0,};
                    let mut j: OPJ_UINT16 = 0;
                    memcpy(&mut saved as *mut opj_image_comp_t as
                               *mut libc::c_void,
                           &mut *(*image).comps.offset(cn as isize) as
                               *mut opj_image_comp_t as *const libc::c_void,
                           ::std::mem::size_of::<opj_image_comp_t>() as
                               libc::c_ulong);
                    memcpy(&mut *(*image).comps.offset(cn as isize) as
                               *mut opj_image_comp_t as *mut libc::c_void,
                           &mut *(*image).comps.offset(acn as isize) as
                               *mut opj_image_comp_t as *const libc::c_void,
                           ::std::mem::size_of::<opj_image_comp_t>() as
                               libc::c_ulong);
                    memcpy(&mut *(*image).comps.offset(acn as isize) as
                               *mut opj_image_comp_t as *mut libc::c_void,
                           &mut saved as *mut opj_image_comp_t as
                               *const libc::c_void,
                           ::std::mem::size_of::<opj_image_comp_t>() as
                               libc::c_ulong);
                    /* Swap channels in following channel definitions, don't bother with j <= i that are already processed */
                    j =
                        (i as libc::c_uint).wrapping_add(1 as libc::c_uint) as
                            OPJ_UINT16;
                    while (j as libc::c_int) < n as libc::c_int {
                        if (*info.offset(j as isize)).cn as libc::c_int ==
                               cn as libc::c_int {
                            (*info.offset(j as isize)).cn = acn
                        } else if (*info.offset(j as isize)).cn as libc::c_int
                                      == acn as libc::c_int {
                            (*info.offset(j as isize)).cn = cn
                        }
                        j = j.wrapping_add(1)
                        /* asoc is related to color index. Do not update. */
                    }
                }
                (*(*image).comps.offset(cn as isize)).alpha =
                    (*info.offset(i as isize)).typ
            }
        }
        i = i.wrapping_add(1)
    }
    if !(*(*color).jp2_cdef).info.is_null() {
        opj_free((*(*color).jp2_cdef).info as *mut libc::c_void);
    }
    opj_free((*color).jp2_cdef as *mut libc::c_void);
    (*color).jp2_cdef = 0 as *mut opj_jp2_cdef_t;
}
/* jp2_apply_cdef() */
unsafe extern "C" fn opj_jp2_read_cdef(mut jp2: *mut opj_jp2_t,
                                       mut p_cdef_header_data: *mut OPJ_BYTE,
                                       mut p_cdef_header_size: OPJ_UINT32,
                                       mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut cdef_info = 0 as *mut opj_jp2_cdef_info_t;
    let mut i: OPJ_UINT16 = 0;
    let mut l_value: OPJ_UINT32 = 0;
    /* preconditions */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1403 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_cdef(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_cdef_header_data.is_null() {
    } else {
        __assert_fail(b"p_cdef_header_data != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1404 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_cdef(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1405 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_cdef(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    /* Part 1, I.5.3.6: 'The shall be at most one Channel Definition box
     * inside a JP2 Header box.'*/
    if !(*jp2).color.jp2_cdef.is_null() { return 0 as libc::c_int } /* N */
    if p_cdef_header_size < 2 as libc::c_int as libc::c_uint {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Insufficient data for CDEF box.\n\x00" as *const u8 as
                          *const libc::c_char);
        return 0 as libc::c_int
    }
    opj_read_bytes_LE(p_cdef_header_data, &mut l_value,
                      2 as libc::c_int as OPJ_UINT32);
    p_cdef_header_data = p_cdef_header_data.offset(2 as libc::c_int as isize);
    if l_value as OPJ_UINT16 as libc::c_int == 0 as libc::c_int {
        /* szukw000: FIXME */
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Number of channel description is equal to zero in CDEF box.\n\x00"
                          as *const u8 as *const libc::c_char); /* Cn^i */
        return 0 as libc::c_int
    } /* Typ^i */
    if p_cdef_header_size <
           (2 as libc::c_int as
                libc::c_uint).wrapping_add((l_value as OPJ_UINT16 as
                                                OPJ_UINT32).wrapping_mul(6 as
                                                                             libc::c_int
                                                                             as
                                                                             libc::c_uint))
       {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Insufficient data for CDEF box.\n\x00" as *const u8 as
                          *const libc::c_char); /* Asoc^i */
        return 0 as libc::c_int
    }
    cdef_info =
        opj_malloc((l_value as
                        libc::c_ulong).wrapping_mul(::std::mem::size_of::<opj_jp2_cdef_info_t>()
                                                        as libc::c_ulong)) as
            *mut opj_jp2_cdef_info_t;
    if cdef_info.is_null() { return 0 as libc::c_int }
    (*jp2).color.jp2_cdef =
        opj_malloc(::std::mem::size_of::<opj_jp2_cdef_t>() as libc::c_ulong)
            as *mut opj_jp2_cdef_t;
    if (*jp2).color.jp2_cdef.is_null() {
        opj_free(cdef_info as *mut libc::c_void);
        return 0 as libc::c_int
    }
    (*(*jp2).color.jp2_cdef).info = cdef_info;
    (*(*jp2).color.jp2_cdef).n = l_value as OPJ_UINT16;
    i = 0 as libc::c_int as OPJ_UINT16;
    while (i as libc::c_int) < (*(*jp2).color.jp2_cdef).n as libc::c_int {
        opj_read_bytes_LE(p_cdef_header_data, &mut l_value,
                          2 as libc::c_int as OPJ_UINT32);
        p_cdef_header_data =
            p_cdef_header_data.offset(2 as libc::c_int as isize);
        (*cdef_info.offset(i as isize)).cn = l_value as OPJ_UINT16;
        opj_read_bytes_LE(p_cdef_header_data, &mut l_value,
                          2 as libc::c_int as OPJ_UINT32);
        p_cdef_header_data =
            p_cdef_header_data.offset(2 as libc::c_int as isize);
        (*cdef_info.offset(i as isize)).typ = l_value as OPJ_UINT16;
        opj_read_bytes_LE(p_cdef_header_data, &mut l_value,
                          2 as libc::c_int as OPJ_UINT32);
        p_cdef_header_data =
            p_cdef_header_data.offset(2 as libc::c_int as isize);
        (*cdef_info.offset(i as isize)).asoc = l_value as OPJ_UINT16;
        i = i.wrapping_add(1)
    }
    return 1 as libc::c_int;
}
/* *
 * Reads the Color Specification box.
 *
 * @param   p_colr_header_data          pointer to actual data (already read from file)
 * @param   jp2                         the jpeg2000 file codec.
 * @param   p_colr_header_size          the size of the color header
 * @param   p_manager                   the user event manager.
 *
 * @return  true if the bpc header is valid, false else.
*/
unsafe extern "C" fn opj_jp2_read_colr(mut jp2: *mut opj_jp2_t,
                                       mut p_colr_header_data: *mut OPJ_BYTE,
                                       mut p_colr_header_size: OPJ_UINT32,
                                       mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut l_value: OPJ_UINT32 = 0;
    /* preconditions */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1473 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_colr(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_colr_header_data.is_null() {
    } else {
        __assert_fail(b"p_colr_header_data != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1474 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_colr(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1475 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_colr(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if p_colr_header_size < 3 as libc::c_int as libc::c_uint {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Bad COLR header box (bad size)\n\x00" as *const u8 as
                          *const libc::c_char);
        return 0 as libc::c_int
    }
    /* Part 1, I.5.3.3 : 'A conforming JP2 reader shall ignore all Colour
     * Specification boxes after the first.'
    */
    if (*jp2).color.jp2_has_colr != 0 {
        opj_event_msg(p_manager, 4 as libc::c_int,
                      b"A conforming JP2 reader shall ignore all Colour Specification boxes after the first, so we ignore this one.\n\x00"
                          as *const u8 as *const libc::c_char); /* METH */
        p_colr_header_data =
            p_colr_header_data.offset(p_colr_header_size as
                                          isize); /* PRECEDENCE */
        return 1 as libc::c_int
    } /* APPROX */
    opj_read_bytes_LE(p_colr_header_data, &mut (*jp2).meth,
                      1 as libc::c_int as OPJ_UINT32);
    p_colr_header_data = p_colr_header_data.offset(1);
    opj_read_bytes_LE(p_colr_header_data, &mut (*jp2).precedence,
                      1 as libc::c_int as OPJ_UINT32);
    p_colr_header_data = p_colr_header_data.offset(1);
    opj_read_bytes_LE(p_colr_header_data, &mut (*jp2).approx,
                      1 as libc::c_int as OPJ_UINT32);
    p_colr_header_data = p_colr_header_data.offset(1);
    if (*jp2).meth == 1 as libc::c_int as libc::c_uint {
        if p_colr_header_size < 7 as libc::c_int as libc::c_uint {
            opj_event_msg(p_manager, 1 as libc::c_int,
                          b"Bad COLR header box (bad size: %d)\n\x00" as
                              *const u8 as *const libc::c_char,
                          p_colr_header_size);
            return 0 as libc::c_int
        }
        if p_colr_header_size > 7 as libc::c_int as libc::c_uint &&
               (*jp2).enumcs != 14 as libc::c_int as libc::c_uint {
            /* handled below for CIELab) */
            /* testcase Altona_Technical_v20_x4.pdf */
            opj_event_msg(p_manager, 2 as libc::c_int,
                          b"Bad COLR header box (bad size: %d)\n\x00" as
                              *const u8 as *const libc::c_char,
                          p_colr_header_size); /* EnumCS */
        }
        opj_read_bytes_LE(p_colr_header_data, &mut (*jp2).enumcs,
                          4 as libc::c_int as OPJ_UINT32);
        p_colr_header_data =
            p_colr_header_data.offset(4 as libc::c_int as isize);
        if (*jp2).enumcs == 14 as libc::c_int as libc::c_uint {
            /* CIELab */
            let mut cielab = 0 as *mut OPJ_UINT32; /* enumcs */
            let mut rl: OPJ_UINT32 = 0;
            let mut ol: OPJ_UINT32 = 0;
            let mut ra: OPJ_UINT32 = 0;
            let mut oa: OPJ_UINT32 = 0;
            let mut rb: OPJ_UINT32 = 0;
            let mut ob: OPJ_UINT32 = 0;
            let mut il: OPJ_UINT32 = 0;
            cielab =
                opj_malloc((9 as libc::c_int as
                                libc::c_ulong).wrapping_mul(::std::mem::size_of::<OPJ_UINT32>()
                                                                as
                                                                libc::c_ulong))
                    as *mut OPJ_UINT32;
            if cielab.is_null() {
                opj_event_msg(p_manager, 1 as libc::c_int,
                              b"Not enough memory for cielab\n\x00" as
                                  *const u8 as *const libc::c_char);
                return 0 as libc::c_int
            }
            *cielab.offset(0 as libc::c_int as isize) =
                14 as libc::c_int as OPJ_UINT32;
            /* default values */
            ob = 0 as libc::c_int as OPJ_UINT32; /* D50 */
            oa = ob; /* DEF */
            ol = oa;
            rb = ol;
            ra = rb;
            rl = ra;
            il = 0x443530 as libc::c_int as OPJ_UINT32;
            *cielab.offset(1 as libc::c_int as isize) =
                0x44454600 as libc::c_int as OPJ_UINT32;
            if p_colr_header_size == 35 as libc::c_int as libc::c_uint {
                opj_read_bytes_LE(p_colr_header_data, &mut rl,
                                  4 as libc::c_int as OPJ_UINT32);
                p_colr_header_data =
                    p_colr_header_data.offset(4 as libc::c_int as isize);
                opj_read_bytes_LE(p_colr_header_data, &mut ol,
                                  4 as libc::c_int as OPJ_UINT32);
                p_colr_header_data =
                    p_colr_header_data.offset(4 as libc::c_int as isize);
                opj_read_bytes_LE(p_colr_header_data, &mut ra,
                                  4 as libc::c_int as OPJ_UINT32);
                p_colr_header_data =
                    p_colr_header_data.offset(4 as libc::c_int as isize);
                opj_read_bytes_LE(p_colr_header_data, &mut oa,
                                  4 as libc::c_int as OPJ_UINT32);
                p_colr_header_data =
                    p_colr_header_data.offset(4 as libc::c_int as isize);
                opj_read_bytes_LE(p_colr_header_data, &mut rb,
                                  4 as libc::c_int as OPJ_UINT32);
                p_colr_header_data =
                    p_colr_header_data.offset(4 as libc::c_int as isize);
                opj_read_bytes_LE(p_colr_header_data, &mut ob,
                                  4 as libc::c_int as OPJ_UINT32);
                p_colr_header_data =
                    p_colr_header_data.offset(4 as libc::c_int as isize);
                opj_read_bytes_LE(p_colr_header_data, &mut il,
                                  4 as libc::c_int as OPJ_UINT32);
                p_colr_header_data =
                    p_colr_header_data.offset(4 as libc::c_int as isize);
                *cielab.offset(1 as libc::c_int as isize) =
                    0 as libc::c_int as OPJ_UINT32
            } else if p_colr_header_size != 7 as libc::c_int as libc::c_uint {
                opj_event_msg(p_manager, 2 as libc::c_int,
                              b"Bad COLR header box (CIELab, bad size: %d)\n\x00"
                                  as *const u8 as *const libc::c_char,
                              p_colr_header_size);
            }
            *cielab.offset(2 as libc::c_int as isize) = rl;
            *cielab.offset(4 as libc::c_int as isize) = ra;
            *cielab.offset(6 as libc::c_int as isize) = rb;
            *cielab.offset(3 as libc::c_int as isize) = ol;
            *cielab.offset(5 as libc::c_int as isize) = oa;
            *cielab.offset(7 as libc::c_int as isize) = ob;
            *cielab.offset(8 as libc::c_int as isize) = il;
            (*jp2).color.icc_profile_buf = cielab as *mut OPJ_BYTE;
            (*jp2).color.icc_profile_len = 0 as libc::c_int as OPJ_UINT32
        }
        (*jp2).color.jp2_has_colr = 1 as libc::c_int as OPJ_BYTE
    } else if (*jp2).meth == 2 as libc::c_int as libc::c_uint {
        /* ICC profile */
        let mut it_icc_value = 0 as libc::c_int; /* icc values */
        let mut icc_len = p_colr_header_size as OPJ_INT32 - 3 as libc::c_int;
        (*jp2).color.icc_profile_len = icc_len as OPJ_UINT32;
        (*jp2).color.icc_profile_buf =
            opj_calloc(1 as libc::c_int as size_t, icc_len as size_t) as
                *mut OPJ_BYTE;
        if (*jp2).color.icc_profile_buf.is_null() {
            (*jp2).color.icc_profile_len = 0 as libc::c_int as OPJ_UINT32;
            return 0 as libc::c_int
        }
        it_icc_value = 0 as libc::c_int;
        while it_icc_value < icc_len {
            opj_read_bytes_LE(p_colr_header_data, &mut l_value,
                              1 as libc::c_int as OPJ_UINT32);
            p_colr_header_data = p_colr_header_data.offset(1);
            *(*jp2).color.icc_profile_buf.offset(it_icc_value as isize) =
                l_value as OPJ_BYTE;
            it_icc_value += 1
        }
        (*jp2).color.jp2_has_colr = 1 as libc::c_int as OPJ_BYTE
    } else if (*jp2).meth > 2 as libc::c_int as libc::c_uint {
        /*  ISO/IEC 15444-1:2004 (E), Table I.9 Legal METH values:
        conforming JP2 reader shall ignore the entire Colour Specification box.*/
        opj_event_msg(p_manager, 4 as libc::c_int,
                      b"COLR BOX meth value is not a regular value (%d), so we will ignore the entire Colour Specification box. \n\x00"
                          as *const u8 as *const libc::c_char, (*jp2).meth);
    }
    return 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn opj_jp2_decode(mut jp2: *mut opj_jp2_t,
                                        mut p_stream:
                                            *mut opj_stream_private_t,
                                        mut p_image: *mut opj_image_t,
                                        mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    if p_image.is_null() { return 0 as libc::c_int }
    /* J2K decoding */
    if opj_j2k_decode((*jp2).j2k, p_stream, p_image, p_manager) == 0 {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Failed to decode the codestream in the JP2 file\n\x00"
                          as *const u8 as *const libc::c_char);
        return 0 as libc::c_int
    }
    if (*(*jp2).j2k).m_specific_param.m_decoder.m_numcomps_to_decode != 0 {
        /* Bypass all JP2 component transforms */
        return 1 as libc::c_int
    }
    if (*jp2).ignore_pclr_cmap_cdef == 0 {
        if opj_jp2_check_color(p_image, &mut (*jp2).color, p_manager) == 0 {
            return 0 as libc::c_int
        }
        /* Set Image Color Space */
        if (*jp2).enumcs == 16 as libc::c_int as libc::c_uint {
            (*p_image).color_space = OPJ_CLRSPC_SRGB
        } else if (*jp2).enumcs == 17 as libc::c_int as libc::c_uint {
            (*p_image).color_space = OPJ_CLRSPC_GRAY
        } else if (*jp2).enumcs == 18 as libc::c_int as libc::c_uint {
            (*p_image).color_space = OPJ_CLRSPC_SYCC
        } else if (*jp2).enumcs == 24 as libc::c_int as libc::c_uint {
            (*p_image).color_space = OPJ_CLRSPC_EYCC
        } else if (*jp2).enumcs == 12 as libc::c_int as libc::c_uint {
            (*p_image).color_space = OPJ_CLRSPC_CMYK
        } else { (*p_image).color_space = OPJ_CLRSPC_UNKNOWN }
        if !(*jp2).color.jp2_pclr.is_null() {
            /* Part 1, I.5.3.4: Either both or none : */
            if (*(*jp2).color.jp2_pclr).cmap.is_null() {
                opj_jp2_free_pclr(&mut (*jp2).color);
            } else if opj_jp2_apply_pclr(p_image, &mut (*jp2).color,
                                         p_manager) == 0 {
                return 0 as libc::c_int
            }
        }
        /* Apply the color space if needed */
        if !(*jp2).color.jp2_cdef.is_null() {
            opj_jp2_apply_cdef(p_image, &mut (*jp2).color, p_manager);
        }
        if !(*jp2).color.icc_profile_buf.is_null() {
            (*p_image).icc_profile_buf = (*jp2).color.icc_profile_buf;
            (*p_image).icc_profile_len = (*jp2).color.icc_profile_len;
            (*jp2).color.icc_profile_buf = 0 as *mut OPJ_BYTE
        }
    }
    return 1 as libc::c_int;
}
/* *
 * Writes the Jpeg2000 file Header box - JP2 Header box (warning, this is a super box).
 *
 * @param  jp2      the jpeg2000 file codec.
 * @param  stream      the stream to write data to.
 * @param  p_manager  user event manager.
 *
 * @return true if writing was successful.
 */
unsafe extern "C" fn opj_jp2_write_jp2h(mut jp2: *mut opj_jp2_t,
                                        mut stream: *mut opj_stream_private_t,
                                        mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut l_writers: [opj_jp2_img_header_writer_handler_t; 4] =
        [opj_jp2_img_header_writer_handler_t{handler: None,
                                             m_data: 0 as *mut OPJ_BYTE,
                                             m_size: 0,}; 4];
    let mut l_current_writer = 0 as *mut opj_jp2_img_header_writer_handler_t;
    let mut i: OPJ_INT32 = 0;
    let mut l_nb_pass: OPJ_INT32 = 0;
    /* size of data for super box*/
    let mut l_jp2h_size = 8 as libc::c_int as OPJ_UINT32;
    let mut l_result = 1 as libc::c_int;
    /* to store the data of the super box */
    let mut l_jp2h_data: [OPJ_BYTE; 8] = [0; 8];
    /* preconditions */
    if !stream.is_null() {
    } else {
        __assert_fail(b"stream != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1681 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 84],
                                                &[libc::c_char; 84]>(b"OPJ_BOOL opj_jp2_write_jp2h(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1682 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 84],
                                                &[libc::c_char; 84]>(b"OPJ_BOOL opj_jp2_write_jp2h(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1683 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 84],
                                                &[libc::c_char; 84]>(b"OPJ_BOOL opj_jp2_write_jp2h(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    memset(l_writers.as_mut_ptr() as *mut libc::c_void, 0 as libc::c_int,
           ::std::mem::size_of::<[opj_jp2_img_header_writer_handler_t; 4]>()
               as libc::c_ulong);
    if (*jp2).bpc == 255 as libc::c_int as libc::c_uint {
        l_nb_pass = 3 as libc::c_int;
        l_writers[0 as libc::c_int as usize].handler =
            Some(opj_jp2_write_ihdr as
                     unsafe extern "C" fn(_: *mut opj_jp2_t,
                                          _: *mut OPJ_UINT32)
                         -> *mut OPJ_BYTE);
        l_writers[1 as libc::c_int as usize].handler =
            Some(opj_jp2_write_bpcc as
                     unsafe extern "C" fn(_: *mut opj_jp2_t,
                                          _: *mut OPJ_UINT32)
                         -> *mut OPJ_BYTE);
        l_writers[2 as libc::c_int as usize].handler =
            Some(opj_jp2_write_colr as
                     unsafe extern "C" fn(_: *mut opj_jp2_t,
                                          _: *mut OPJ_UINT32)
                         -> *mut OPJ_BYTE)
    } else {
        l_nb_pass = 2 as libc::c_int;
        l_writers[0 as libc::c_int as usize].handler =
            Some(opj_jp2_write_ihdr as
                     unsafe extern "C" fn(_: *mut opj_jp2_t,
                                          _: *mut OPJ_UINT32)
                         -> *mut OPJ_BYTE);
        l_writers[1 as libc::c_int as usize].handler =
            Some(opj_jp2_write_colr as
                     unsafe extern "C" fn(_: *mut opj_jp2_t,
                                          _: *mut OPJ_UINT32)
                         -> *mut OPJ_BYTE)
    }
    if !(*jp2).color.jp2_cdef.is_null() {
        l_writers[l_nb_pass as usize].handler =
            Some(opj_jp2_write_cdef as
                     unsafe extern "C" fn(_: *mut opj_jp2_t,
                                          _: *mut OPJ_UINT32)
                         -> *mut OPJ_BYTE);
        l_nb_pass += 1
    }
    /* write box header */
    /* write JP2H type */
    opj_write_bytes_LE(l_jp2h_data.as_mut_ptr().offset(4 as libc::c_int as
                                                           isize),
                       0x6a703268 as libc::c_int as OPJ_UINT32,
                       4 as libc::c_int as OPJ_UINT32);
    l_current_writer = l_writers.as_mut_ptr();
    i = 0 as libc::c_int;
    while i < l_nb_pass {
        (*l_current_writer).m_data =
            (*l_current_writer).handler.expect("non-null function pointer")(jp2,
                                                                            &mut (*l_current_writer).m_size);
        if (*l_current_writer).m_data.is_null() {
            opj_event_msg(p_manager, 1 as libc::c_int,
                          b"Not enough memory to hold JP2 Header data\n\x00"
                              as *const u8 as *const libc::c_char);
            l_result = 0 as libc::c_int;
            break ;
        } else {
            l_jp2h_size =
                (l_jp2h_size as
                     libc::c_uint).wrapping_add((*l_current_writer).m_size) as
                    OPJ_UINT32 as OPJ_UINT32;
            l_current_writer = l_current_writer.offset(1);
            i += 1
        }
    }
    if l_result == 0 {
        l_current_writer = l_writers.as_mut_ptr();
        i = 0 as libc::c_int;
        while i < l_nb_pass {
            if !(*l_current_writer).m_data.is_null() {
                opj_free((*l_current_writer).m_data as *mut libc::c_void);
            }
            l_current_writer = l_current_writer.offset(1);
            i += 1
        }
        return 0 as libc::c_int
    }
    /* write super box size */
    opj_write_bytes_LE(l_jp2h_data.as_mut_ptr(), l_jp2h_size,
                       4 as libc::c_int as OPJ_UINT32);
    /* write super box data on stream */
    if opj_stream_write_data(stream, l_jp2h_data.as_mut_ptr(),
                             8 as libc::c_int as OPJ_SIZE_T, p_manager) !=
           8 as libc::c_int as libc::c_ulong {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Stream error while writing JP2 Header box\n\x00" as
                          *const u8 as *const libc::c_char);
        l_result = 0 as libc::c_int
    }
    if l_result != 0 {
        l_current_writer = l_writers.as_mut_ptr();
        i = 0 as libc::c_int;
        while i < l_nb_pass {
            if opj_stream_write_data(stream, (*l_current_writer).m_data,
                                     (*l_current_writer).m_size as OPJ_SIZE_T,
                                     p_manager) !=
                   (*l_current_writer).m_size as libc::c_ulong {
                opj_event_msg(p_manager, 1 as libc::c_int,
                              b"Stream error while writing JP2 Header box\n\x00"
                                  as *const u8 as *const libc::c_char);
                l_result = 0 as libc::c_int;
                break ;
            } else { l_current_writer = l_current_writer.offset(1); i += 1 }
        }
    }
    l_current_writer = l_writers.as_mut_ptr();
    /* cleanup */
    i = 0 as libc::c_int;
    while i < l_nb_pass {
        if !(*l_current_writer).m_data.is_null() {
            opj_free((*l_current_writer).m_data as *mut libc::c_void);
        }
        l_current_writer = l_current_writer.offset(1);
        i += 1
    }
    return l_result;
}
/* *
 * Writes a FTYP box - File type box
 *
 * @param   cio         the stream to write data to.
 * @param   jp2         the jpeg2000 file codec.
 * @param   p_manager   the user event manager.
 *
 * @return  true if writing was successful.
 */
unsafe extern "C" fn opj_jp2_write_ftyp(mut jp2: *mut opj_jp2_t,
                                        mut cio: *mut opj_stream_private_t,
                                        mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut i: OPJ_UINT32 = 0;
    let mut l_ftyp_size: OPJ_UINT32 = 0;
    let mut l_ftyp_data = 0 as *mut OPJ_BYTE;
    let mut l_current_data_ptr = 0 as *mut OPJ_BYTE;
    let mut l_result: OPJ_BOOL = 0;
    /* preconditions */
    if !cio.is_null() {
    } else {
        __assert_fail(b"cio != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1781 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 84],
                                                &[libc::c_char; 84]>(b"OPJ_BOOL opj_jp2_write_ftyp(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr()); /* box size */
    } /* FTYP */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1782 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 84],
                                                &[libc::c_char; 84]>(b"OPJ_BOOL opj_jp2_write_ftyp(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr()); /* BR */
    } /* MinV */
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1783 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 84],
                                                &[libc::c_char; 84]>(b"OPJ_BOOL opj_jp2_write_ftyp(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    l_ftyp_size =
        (16 as libc::c_int as
             libc::c_uint).wrapping_add((4 as libc::c_int as
                                             libc::c_uint).wrapping_mul((*jp2).numcl));
    l_ftyp_data =
        opj_calloc(1 as libc::c_int as size_t, l_ftyp_size as size_t) as
            *mut OPJ_BYTE;
    if l_ftyp_data.is_null() {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Not enough memory to handle ftyp data\n\x00" as
                          *const u8 as *const libc::c_char);
        return 0 as libc::c_int
    }
    l_current_data_ptr = l_ftyp_data;
    opj_write_bytes_LE(l_current_data_ptr, l_ftyp_size,
                       4 as libc::c_int as OPJ_UINT32);
    l_current_data_ptr = l_current_data_ptr.offset(4 as libc::c_int as isize);
    opj_write_bytes_LE(l_current_data_ptr,
                       0x66747970 as libc::c_int as OPJ_UINT32,
                       4 as libc::c_int as OPJ_UINT32);
    l_current_data_ptr = l_current_data_ptr.offset(4 as libc::c_int as isize);
    opj_write_bytes_LE(l_current_data_ptr, (*jp2).brand,
                       4 as libc::c_int as OPJ_UINT32);
    l_current_data_ptr = l_current_data_ptr.offset(4 as libc::c_int as isize);
    opj_write_bytes_LE(l_current_data_ptr, (*jp2).minversion,
                       4 as libc::c_int as OPJ_UINT32);
    l_current_data_ptr = l_current_data_ptr.offset(4 as libc::c_int as isize);
    i = 0 as libc::c_int as OPJ_UINT32;
    while i < (*jp2).numcl {
        opj_write_bytes_LE(l_current_data_ptr, *(*jp2).cl.offset(i as isize),
                           4 as libc::c_int as OPJ_UINT32);
        i = i.wrapping_add(1)
        /* CL */
    }
    l_result =
        (opj_stream_write_data(cio, l_ftyp_data, l_ftyp_size as OPJ_SIZE_T,
                               p_manager) == l_ftyp_size as libc::c_ulong) as
            libc::c_int;
    if l_result == 0 {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Error while writing ftyp data to stream\n\x00" as
                          *const u8 as *const libc::c_char);
    }
    opj_free(l_ftyp_data as *mut libc::c_void);
    return l_result;
}
/* *
 * Writes the Jpeg2000 codestream Header box - JP2C Header box. This function must be called AFTER the coding has been done.
 *
 * @param   cio         the stream to write data to.
 * @param   jp2         the jpeg2000 file codec.
 * @param   p_manager   user event manager.
 *
 * @return true if writing was successful.
*/
unsafe extern "C" fn opj_jp2_write_jp2c(mut jp2: *mut opj_jp2_t,
                                        mut cio: *mut opj_stream_private_t,
                                        mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut j2k_codestream_exit: OPJ_OFF_T = 0;
    let mut l_data_header: [OPJ_BYTE; 8] = [0; 8];
    /* preconditions */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1831 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 84],
                                                &[libc::c_char; 84]>(b"OPJ_BOOL opj_jp2_write_jp2c(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr()); /* size of codestream */
    } /* JP2C */
    if !cio.is_null() {
    } else {
        __assert_fail(b"cio != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1832 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 84],
                                                &[libc::c_char; 84]>(b"OPJ_BOOL opj_jp2_write_jp2c(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1833 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 84],
                                                &[libc::c_char; 84]>(b"OPJ_BOOL opj_jp2_write_jp2c(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if opj_stream_has_seek(cio) != 0 {
    } else {
        __assert_fail(b"opj_stream_has_seek(cio)\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1834 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 84],
                                                &[libc::c_char; 84]>(b"OPJ_BOOL opj_jp2_write_jp2c(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    j2k_codestream_exit = opj_stream_tell(cio);
    opj_write_bytes_LE(l_data_header.as_mut_ptr(),
                       (j2k_codestream_exit - (*jp2).j2k_codestream_offset) as
                           OPJ_UINT32, 4 as libc::c_int as OPJ_UINT32);
    opj_write_bytes_LE(l_data_header.as_mut_ptr().offset(4 as libc::c_int as
                                                             isize),
                       0x6a703263 as libc::c_int as OPJ_UINT32,
                       4 as libc::c_int as OPJ_UINT32);
    if opj_stream_seek(cio, (*jp2).j2k_codestream_offset, p_manager) == 0 {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Failed to seek in the stream.\n\x00" as *const u8 as
                          *const libc::c_char);
        return 0 as libc::c_int
    }
    if opj_stream_write_data(cio, l_data_header.as_mut_ptr(),
                             8 as libc::c_int as OPJ_SIZE_T, p_manager) !=
           8 as libc::c_int as libc::c_ulong {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Failed to seek in the stream.\n\x00" as *const u8 as
                          *const libc::c_char);
        return 0 as libc::c_int
    }
    if opj_stream_seek(cio, j2k_codestream_exit, p_manager) == 0 {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Failed to seek in the stream.\n\x00" as *const u8 as
                          *const libc::c_char);
        return 0 as libc::c_int
    }
    return 1 as libc::c_int;
}
/* *
 * Writes a jpeg2000 file signature box.
 *
 * @param cio the stream to write data to.
 * @param   jp2         the jpeg2000 file codec.
 * @param p_manager the user event manager.
 *
 * @return true if writing was successful.
 */
unsafe extern "C" fn opj_jp2_write_jp(mut jp2: *mut opj_jp2_t,
                                      mut cio: *mut opj_stream_private_t,
                                      mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    /* 12 bytes will be read */
    let mut l_signature_data: [OPJ_BYTE; 12] = [0; 12];
    /* preconditions */
    if !cio.is_null() {
    } else {
        __assert_fail(b"cio != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1869 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 82],
                                                &[libc::c_char; 82]>(b"OPJ_BOOL opj_jp2_write_jp(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1870 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 82],
                                                &[libc::c_char; 82]>(b"OPJ_BOOL opj_jp2_write_jp(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      1871 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 82],
                                                &[libc::c_char; 82]>(b"OPJ_BOOL opj_jp2_write_jp(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    /* write box length */
    opj_write_bytes_LE(l_signature_data.as_mut_ptr(),
                       12 as libc::c_int as OPJ_UINT32,
                       4 as libc::c_int as OPJ_UINT32);
    /* writes box type */
    opj_write_bytes_LE(l_signature_data.as_mut_ptr().offset(4 as libc::c_int
                                                                as isize),
                       0x6a502020 as libc::c_int as OPJ_UINT32,
                       4 as libc::c_int as OPJ_UINT32);
    /* writes magic number*/
    opj_write_bytes_LE(l_signature_data.as_mut_ptr().offset(8 as libc::c_int
                                                                as isize),
                       0xd0a870a as libc::c_int as OPJ_UINT32,
                       4 as libc::c_int as OPJ_UINT32);
    if opj_stream_write_data(cio, l_signature_data.as_mut_ptr(),
                             12 as libc::c_int as OPJ_SIZE_T, p_manager) !=
           12 as libc::c_int as libc::c_ulong {
        return 0 as libc::c_int
    }
    return 1 as libc::c_int;
}
/* ----------------------------------------------------------------------- */
/* JP2 decoder interface                                             */
/* ----------------------------------------------------------------------- */
#[no_mangle]
pub unsafe extern "C" fn opj_jp2_setup_decoder(mut jp2: *mut opj_jp2_t,
                                               mut parameters:
                                                   *mut opj_dparameters_t) {
    /* setup the J2K codec */
    opj_j2k_setup_decoder((*jp2).j2k, parameters);
    /* further JP2 initializations go here */
    (*jp2).color.jp2_has_colr = 0 as libc::c_int as OPJ_BYTE;
    (*jp2).ignore_pclr_cmap_cdef =
        ((*parameters).flags & 0x1 as libc::c_int as libc::c_uint) as
            OPJ_BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn opj_jp2_decoder_set_strict_mode(mut jp2:
                                                             *mut opj_jp2_t,
                                                         mut strict:
                                                             OPJ_BOOL) {
    opj_j2k_decoder_set_strict_mode((*jp2).j2k, strict);
}
#[no_mangle]
pub unsafe extern "C" fn opj_jp2_set_threads(mut jp2: *mut opj_jp2_t,
                                             mut num_threads: OPJ_UINT32)
 -> OPJ_BOOL {
    return opj_j2k_set_threads((*jp2).j2k, num_threads);
}
/* ----------------------------------------------------------------------- */
/* JP2 encoder interface                                             */
/* ----------------------------------------------------------------------- */
#[no_mangle]
pub unsafe extern "C" fn opj_jp2_setup_encoder(mut jp2: *mut opj_jp2_t,
                                               mut parameters:
                                                   *mut opj_cparameters_t,
                                               mut image: *mut opj_image_t,
                                               mut p_manager:
                                                   *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut i: OPJ_UINT32 = 0;
    let mut depth_0: OPJ_UINT32 = 0;
    let mut sign: OPJ_UINT32 = 0;
    let mut alpha_count: OPJ_UINT32 = 0;
    let mut color_channels = 0 as libc::c_uint;
    let mut alpha_channel = 0 as libc::c_uint;
    if jp2.is_null() || parameters.is_null() || image.is_null() {
        return 0 as libc::c_int
    }
    /* setup the J2K codec */
    /* ------------------- */
    /* Check if number of components respects standard */
    if (*image).numcomps < 1 as libc::c_int as libc::c_uint ||
           (*image).numcomps > 16384 as libc::c_int as libc::c_uint {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Invalid number of components specified while setting up JP2 encoder\n\x00"
                          as *const u8 as *const libc::c_char);
        return 0 as libc::c_int
    }
    if opj_j2k_setup_encoder((*jp2).j2k, parameters, image, p_manager) ==
           0 as libc::c_int {
        return 0 as libc::c_int
    }
    /* setup the JP2 codec */
    /* ------------------- */
    /* Profile box */
    (*jp2).brand = 0x6a703220 as libc::c_int as OPJ_UINT32; /* BR */
    (*jp2).minversion = 0 as libc::c_int as OPJ_UINT32; /* MinV */
    (*jp2).numcl = 1 as libc::c_int as OPJ_UINT32; /* CL0 : JP2 */
    (*jp2).cl =
        opj_malloc(((*jp2).numcl as
                        libc::c_ulong).wrapping_mul(::std::mem::size_of::<OPJ_UINT32>()
                                                        as libc::c_ulong)) as
            *mut OPJ_UINT32;
    if (*jp2).cl.is_null() {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Not enough memory when setup the JP2 encoder\n\x00" as
                          *const u8 as *const libc::c_char);
        return 0 as libc::c_int
    }
    *(*jp2).cl.offset(0 as libc::c_int as isize) =
        0x6a703220 as libc::c_int as OPJ_UINT32;
    /* Image Header box */
    (*jp2).numcomps = (*image).numcomps; /* NC */
    (*jp2).comps =
        opj_malloc(((*jp2).numcomps as
                        libc::c_ulong).wrapping_mul(::std::mem::size_of::<opj_jp2_comps_t>()
                                                        as libc::c_ulong)) as
            *mut opj_jp2_comps_t;
    if (*jp2).comps.is_null() {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Not enough memory when setup the JP2 encoder\n\x00" as
                          *const u8 as *const libc::c_char);
        /* Memory of jp2->cl will be freed by opj_jp2_destroy */
        return 0 as libc::c_int
    } /* HEIGHT */
    (*jp2).h = (*image).y1.wrapping_sub((*image).y0); /* WIDTH */
    (*jp2).w = (*image).x1.wrapping_sub((*image).x0);
    /* BPC */
    depth_0 =
        (*(*image).comps.offset(0 as libc::c_int as
                                    isize)).prec.wrapping_sub(1 as libc::c_int
                                                                  as
                                                                  libc::c_uint); /* C : Always 7 */
    sign =
        (*(*image).comps.offset(0 as libc::c_int as
                                    isize)).sgnd; /* UnkC, colorspace specified in colr box */
    (*jp2).bpc =
        depth_0.wrapping_add(sign <<
                                 7 as
                                     libc::c_int); /* IPR, no intellectual property */
    i = 1 as libc::c_int as OPJ_UINT32;
    while i < (*image).numcomps {
        let mut depth =
            (*(*image).comps.offset(i as
                                        isize)).prec.wrapping_sub(1 as
                                                                      libc::c_int
                                                                      as
                                                                      libc::c_uint);
        sign = (*(*image).comps.offset(i as isize)).sgnd;
        if depth_0 != depth { (*jp2).bpc = 255 as libc::c_int as OPJ_UINT32 }
        i = i.wrapping_add(1)
    }
    (*jp2).C = 7 as libc::c_int as OPJ_UINT32;
    (*jp2).UnkC = 0 as libc::c_int as OPJ_UINT32;
    (*jp2).IPR = 0 as libc::c_int as OPJ_UINT32;
    /* BitsPerComponent box */
    i = 0 as libc::c_int as OPJ_UINT32;
    while i < (*image).numcomps {
        (*(*jp2).comps.offset(i as isize)).bpcc =
            (*(*image).comps.offset(i as
                                        isize)).prec.wrapping_sub(1 as
                                                                      libc::c_int
                                                                      as
                                                                      libc::c_uint).wrapping_add((*(*image).comps.offset(i
                                                                                                                             as
                                                                                                                             isize)).sgnd
                                                                                                     <<
                                                                                                     7
                                                                                                         as
                                                                                                         libc::c_int);
        i = i.wrapping_add(1)
    }
    /* Colour Specification box */
    if (*image).icc_profile_len != 0 {
        (*jp2).meth = 2 as libc::c_int as OPJ_UINT32;
        (*jp2).enumcs = 0 as libc::c_int as OPJ_UINT32
    } else {
        (*jp2).meth = 1 as libc::c_int as OPJ_UINT32;
        if (*image).color_space as libc::c_int == 1 as libc::c_int {
            (*jp2).enumcs = 16 as libc::c_int as OPJ_UINT32
            /* sRGB as defined by IEC 61966-2-1 */
        } else if (*image).color_space as libc::c_int == 2 as libc::c_int {
            (*jp2).enumcs = 17 as libc::c_int as OPJ_UINT32
            /* greyscale */
        } else if (*image).color_space as libc::c_int == 3 as libc::c_int {
            (*jp2).enumcs = 18 as libc::c_int as OPJ_UINT32
            /* YUV */
        }
    }
    /* Channel Definition box */
    /* FIXME not provided by parameters */
    /* We try to do what we can... */
    alpha_count = 0 as libc::c_uint;
    i = 0 as libc::c_int as OPJ_UINT32;
    while i < (*image).numcomps {
        if (*(*image).comps.offset(i as isize)).alpha as libc::c_int !=
               0 as libc::c_int {
            alpha_count = alpha_count.wrapping_add(1);
            alpha_channel = i
        }
        i = i.wrapping_add(1)
    }
    if alpha_count == 1 as libc::c_uint {
        /* no way to deal with more than 1 alpha channel */
        match (*jp2).enumcs {
            16 | 18 => { color_channels = 3 as libc::c_int as OPJ_UINT32 }
            17 => { color_channels = 1 as libc::c_int as OPJ_UINT32 }
            _ => { alpha_count = 0 as libc::c_uint }
        }
        if alpha_count == 0 as libc::c_uint {
            opj_event_msg(p_manager, 2 as libc::c_int,
                          b"Alpha channel specified but unknown enumcs. No cdef box will be created.\n\x00"
                              as *const u8 as *const libc::c_char);
        } else if (*image).numcomps <
                      color_channels.wrapping_add(1 as libc::c_int as
                                                      libc::c_uint) {
            opj_event_msg(p_manager, 2 as libc::c_int,
                          b"Alpha channel specified but not enough image components for an automatic cdef box creation.\n\x00"
                              as *const u8 as *const libc::c_char);
            alpha_count = 0 as libc::c_uint
        } else if alpha_channel < color_channels {
            opj_event_msg(p_manager, 2 as libc::c_int,
                          b"Alpha channel position conflicts with color channel. No cdef box will be created.\n\x00"
                              as *const u8 as *const libc::c_char);
            alpha_count = 0 as libc::c_uint
        }
    } else if alpha_count > 1 as libc::c_int as libc::c_uint {
        opj_event_msg(p_manager, 2 as libc::c_int,
                      b"Multiple alpha channels specified. No cdef box will be created.\n\x00"
                          as *const u8 as *const libc::c_char);
    }
    if alpha_count == 1 as libc::c_uint {
        /* if here, we know what we can do */
        (*jp2).color.jp2_cdef =
            opj_malloc(::std::mem::size_of::<opj_jp2_cdef_t>() as
                           libc::c_ulong) as *mut opj_jp2_cdef_t;
        if (*jp2).color.jp2_cdef.is_null() {
            opj_event_msg(p_manager, 1 as libc::c_int,
                          b"Not enough memory to setup the JP2 encoder\n\x00"
                              as *const u8 as *const libc::c_char);
            return 0 as libc::c_int
        }
        /* no memset needed, all values will be overwritten except if jp2->color.jp2_cdef->info allocation fails, */
        /* in which case jp2->color.jp2_cdef->info will be NULL => valid for destruction */
        (*(*jp2).color.jp2_cdef).info =
            opj_malloc(((*image).numcomps as
                            libc::c_ulong).wrapping_mul(::std::mem::size_of::<opj_jp2_cdef_info_t>()
                                                            as libc::c_ulong))
                as *mut opj_jp2_cdef_info_t;
        if (*(*jp2).color.jp2_cdef).info.is_null() {
            /* memory will be freed by opj_jp2_destroy */
            opj_event_msg(p_manager, 1 as libc::c_int,
                          b"Not enough memory to setup the JP2 encoder\n\x00"
                              as *const u8 as
                              *const libc::c_char); /* cast is valid : image->numcomps [1,16384] */
            return 0 as libc::c_int
        } /* cast is valid : image->numcomps [1,16384] */
        (*(*jp2).color.jp2_cdef).n = (*image).numcomps as OPJ_UINT16;
        i = 0 as libc::c_uint;
        while i < color_channels {
            (*(*(*jp2).color.jp2_cdef).info.offset(i as isize)).cn =
                i as OPJ_UINT16;
            (*(*(*jp2).color.jp2_cdef).info.offset(i as isize)).typ =
                0 as libc::c_uint as OPJ_UINT16;
            (*(*(*jp2).color.jp2_cdef).info.offset(i as isize)).asoc =
                i.wrapping_add(1 as libc::c_uint) as OPJ_UINT16;
            i = i.wrapping_add(1)
            /* No overflow + cast is valid : image->numcomps [1,16384] */
        }
        while i < (*image).numcomps {
            if (*(*image).comps.offset(i as isize)).alpha as libc::c_int !=
                   0 as libc::c_int {
                /* we'll be here exactly once */
                (*(*(*jp2).color.jp2_cdef).info.offset(i as isize)).cn =
                    i as
                        OPJ_UINT16; /* cast is valid : image->numcomps [1,16384] */
                /* Apply alpha channel to the whole image */
                (*(*(*jp2).color.jp2_cdef).info.offset(i as isize)).typ =
                    1 as libc::c_uint as OPJ_UINT16; /* Opacity channel */
                (*(*(*jp2).color.jp2_cdef).info.offset(i as isize)).asoc =
                    0 as libc::c_uint as OPJ_UINT16
            } else {
                /* Unknown channel */
                (*(*(*jp2).color.jp2_cdef).info.offset(i as isize)).cn =
                    i as
                        OPJ_UINT16; /* cast is valid : image->numcomps [1,16384] */
                (*(*(*jp2).color.jp2_cdef).info.offset(i as isize)).typ =
                    65535 as libc::c_uint as OPJ_UINT16; /* PRECEDENCE */
                (*(*(*jp2).color.jp2_cdef).info.offset(i as isize)).asoc =
                    65535 as libc::c_uint as OPJ_UINT16
            } /* APPROX */
            i = i.wrapping_add(1)
        }
    }
    (*jp2).precedence = 0 as libc::c_int as OPJ_UINT32;
    (*jp2).approx = 0 as libc::c_int as OPJ_UINT32;
    (*jp2).jpip_on = (*parameters).jpip_on;
    return 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn opj_jp2_encode(mut jp2: *mut opj_jp2_t,
                                        mut stream: *mut opj_stream_private_t,
                                        mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    return opj_j2k_encode((*jp2).j2k, stream, p_manager);
}
#[no_mangle]
pub unsafe extern "C" fn opj_jp2_end_decompress(mut jp2: *mut opj_jp2_t,
                                                mut cio:
                                                    *mut opj_stream_private_t,
                                                mut p_manager:
                                                    *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    /* preconditions */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2118 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 88],
                                                &[libc::c_char; 88]>(b"OPJ_BOOL opj_jp2_end_decompress(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !cio.is_null() {
    } else {
        __assert_fail(b"cio != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2119 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 88],
                                                &[libc::c_char; 88]>(b"OPJ_BOOL opj_jp2_end_decompress(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2120 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 88],
                                                &[libc::c_char; 88]>(b"OPJ_BOOL opj_jp2_end_decompress(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    /* customization of the end encoding */
    if opj_jp2_setup_end_header_reading(jp2, p_manager) == 0 {
        return 0 as libc::c_int
    }
    /* write header */
    if opj_jp2_exec(jp2, (*jp2).m_procedure_list, cio, p_manager) == 0 {
        return 0 as libc::c_int
    }
    return opj_j2k_end_decompress((*jp2).j2k, cio, p_manager);
}
#[no_mangle]
pub unsafe extern "C" fn opj_jp2_end_compress(mut jp2: *mut opj_jp2_t,
                                              mut cio:
                                                  *mut opj_stream_private_t,
                                              mut p_manager:
                                                  *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    /* preconditions */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2141 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 86],
                                                &[libc::c_char; 86]>(b"OPJ_BOOL opj_jp2_end_compress(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !cio.is_null() {
    } else {
        __assert_fail(b"cio != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2142 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 86],
                                                &[libc::c_char; 86]>(b"OPJ_BOOL opj_jp2_end_compress(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2143 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 86],
                                                &[libc::c_char; 86]>(b"OPJ_BOOL opj_jp2_end_compress(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    /* customization of the end encoding */
    if opj_jp2_setup_end_header_writing(jp2, p_manager) == 0 {
        return 0 as libc::c_int
    }
    if opj_j2k_end_compress((*jp2).j2k, cio, p_manager) == 0 {
        return 0 as libc::c_int
    }
    /* write header */
    return opj_jp2_exec(jp2, (*jp2).m_procedure_list, cio, p_manager);
}
/*@}*/
/*@}*/
/* *
 * Sets up the procedures to do on writing header after the codestream.
 * Developers wanting to extend the library can add their own writing procedures.
 */
unsafe extern "C" fn opj_jp2_setup_end_header_writing(mut jp2: *mut opj_jp2_t,
                                                      mut p_manager:
                                                          *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    /* preconditions */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2162 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 74],
                                                &[libc::c_char; 74]>(b"OPJ_BOOL opj_jp2_setup_end_header_writing(opj_jp2_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2163 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 74],
                                                &[libc::c_char; 74]>(b"OPJ_BOOL opj_jp2_setup_end_header_writing(opj_jp2_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if opj_procedure_list_add_procedure((*jp2).m_procedure_list,
                                        ::std::mem::transmute::<Option<unsafe extern "C" fn(_:
                                                                                                *mut opj_jp2_t,
                                                                                            _:
                                                                                                *mut opj_stream_private_t,
                                                                                            _:
                                                                                                *mut opj_event_mgr_t)
                                                                           ->
                                                                               OPJ_BOOL>,
                                                                opj_procedure>(Some(opj_jp2_write_jp2c
                                                                                        as
                                                                                        unsafe extern "C" fn(_:
                                                                                                                 *mut opj_jp2_t,
                                                                                                             _:
                                                                                                                 *mut opj_stream_private_t,
                                                                                                             _:
                                                                                                                 *mut opj_event_mgr_t)
                                                                                            ->
                                                                                                OPJ_BOOL)),
                                        p_manager) == 0 {
        return 0 as libc::c_int
    }
    /* DEVELOPER CORNER, add your custom procedures */
    return 1 as libc::c_int;
}
/* *
 * Sets up the procedures to do on reading header after the codestream.
 * Developers wanting to extend the library can add their own writing procedures.
 */
unsafe extern "C" fn opj_jp2_setup_end_header_reading(mut jp2: *mut opj_jp2_t,
                                                      mut p_manager:
                                                          *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    /* preconditions */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2197 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 74],
                                                &[libc::c_char; 74]>(b"OPJ_BOOL opj_jp2_setup_end_header_reading(opj_jp2_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2198 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 74],
                                                &[libc::c_char; 74]>(b"OPJ_BOOL opj_jp2_setup_end_header_reading(opj_jp2_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if opj_procedure_list_add_procedure((*jp2).m_procedure_list,
                                        ::std::mem::transmute::<Option<unsafe extern "C" fn(_:
                                                                                                *mut opj_jp2_t,
                                                                                            _:
                                                                                                *mut opj_stream_private_t,
                                                                                            _:
                                                                                                *mut opj_event_mgr_t)
                                                                           ->
                                                                               OPJ_BOOL>,
                                                                opj_procedure>(Some(opj_jp2_read_header_procedure
                                                                                        as
                                                                                        unsafe extern "C" fn(_:
                                                                                                                 *mut opj_jp2_t,
                                                                                                             _:
                                                                                                                 *mut opj_stream_private_t,
                                                                                                             _:
                                                                                                                 *mut opj_event_mgr_t)
                                                                                            ->
                                                                                                OPJ_BOOL)),
                                        p_manager) == 0 {
        return 0 as libc::c_int
    }
    /* DEVELOPER CORNER, add your custom procedures */
    return 1 as libc::c_int;
}
unsafe extern "C" fn opj_jp2_default_validation(mut jp2: *mut opj_jp2_t,
                                                mut cio:
                                                    *mut opj_stream_private_t,
                                                mut p_manager:
                                                    *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut l_is_valid = 1 as libc::c_int;
    let mut i: OPJ_UINT32 = 0;
    /* preconditions */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2218 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 92],
                                                &[libc::c_char; 92]>(b"OPJ_BOOL opj_jp2_default_validation(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !cio.is_null() {
    } else {
        __assert_fail(b"cio != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2219 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 92],
                                                &[libc::c_char; 92]>(b"OPJ_BOOL opj_jp2_default_validation(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2220 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 92],
                                                &[libc::c_char; 92]>(b"OPJ_BOOL opj_jp2_default_validation(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    /* JPEG2000 codec validation */
    /* STATE checking */
    /* make sure the state is at 0 */
    l_is_valid &=
        ((*jp2).jp2_state == JP2_STATE_NONE as libc::c_int as libc::c_uint) as
            libc::c_int;
    /* make sure not reading a jp2h ???? WEIRD */
    l_is_valid &=
        ((*jp2).jp2_img_state ==
             JP2_IMG_STATE_NONE as libc::c_int as libc::c_uint) as
            libc::c_int;
    /* POINTER validation */
    /* make sure a j2k codec is present */
    l_is_valid &= ((*jp2).j2k != 0 as *mut opj_j2k_t) as libc::c_int;
    /* make sure a procedure list is present */
    l_is_valid &=
        ((*jp2).m_procedure_list != 0 as *mut opj_procedure_list) as
            libc::c_int;
    /* make sure a validation list is present */
    l_is_valid &=
        ((*jp2).m_validation_list != 0 as *mut opj_procedure_list) as
            libc::c_int;
    /* PARAMETER VALIDATION */
    /* number of components */
    l_is_valid &=
        ((*jp2).numcl > 0 as libc::c_int as libc::c_uint) as libc::c_int;
    /* width */
    l_is_valid &=
        ((*jp2).h > 0 as libc::c_int as libc::c_uint) as libc::c_int;
    /* height */
    l_is_valid &=
        ((*jp2).w > 0 as libc::c_int as libc::c_uint) as libc::c_int;
    /* precision */
    i = 0 as libc::c_int as OPJ_UINT32;
    while i < (*jp2).numcomps {
        l_is_valid &=
            (((*(*jp2).comps.offset(i as isize)).bpcc & 0x7f as libc::c_uint)
                 < 38 as libc::c_uint) as libc::c_int;
        i = i.wrapping_add(1)
        /* 0 is valid, ignore sign for check */
    }
    /* METH */
    l_is_valid &=
        ((*jp2).meth > 0 as libc::c_int as libc::c_uint &&
             (*jp2).meth < 3 as libc::c_int as libc::c_uint) as libc::c_int;
    /* stream validation */
    /* back and forth is needed */
    l_is_valid &= opj_stream_has_seek(cio);
    return l_is_valid;
}
/* *
 * Reads a jpeg2000 file header structure.
 *
 * @param jp2 the jpeg2000 file header structure.
 * @param stream the stream to read data from.
 * @param p_manager the user event manager.
 *
 * @return true if the box is valid.
 */
unsafe extern "C" fn opj_jp2_read_header_procedure(mut jp2: *mut opj_jp2_t,
                                                   mut stream:
                                                       *mut opj_stream_private_t,
                                                   mut p_manager:
                                                       *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut box_0 = opj_jp2_box_t{length: 0, type_0: 0, init_pos: 0,};
    let mut l_nb_bytes_read: OPJ_UINT32 = 0;
    let mut l_current_handler = 0 as *const opj_jp2_header_handler_t;
    let mut l_current_handler_misplaced =
        0 as *const opj_jp2_header_handler_t;
    let mut l_last_data_size = 1024 as libc::c_int as OPJ_UINT32;
    let mut l_current_data_size: OPJ_UINT32 = 0;
    let mut l_current_data = 0 as *mut OPJ_BYTE;
    /* preconditions */
    if !stream.is_null() {
    } else {
        __assert_fail(b"stream != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2280 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 95],
                                                &[libc::c_char; 95]>(b"OPJ_BOOL opj_jp2_read_header_procedure(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2281 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 95],
                                                &[libc::c_char; 95]>(b"OPJ_BOOL opj_jp2_read_header_procedure(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2282 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 95],
                                                &[libc::c_char; 95]>(b"OPJ_BOOL opj_jp2_read_header_procedure(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    l_current_data =
        opj_calloc(1 as libc::c_int as size_t, l_last_data_size as size_t) as
            *mut OPJ_BYTE;
    if l_current_data.is_null() {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Not enough memory to handle jpeg2000 file header\n\x00"
                          as *const u8 as *const libc::c_char);
        return 0 as libc::c_int
    }
    while opj_jp2_read_boxhdr(&mut box_0, &mut l_nb_bytes_read, stream,
                              p_manager) != 0 {
        /* is it the codestream box ? */
        if box_0.type_0 == 0x6a703263 as libc::c_int as libc::c_uint {
            if (*jp2).jp2_state &
                   JP2_STATE_HEADER as libc::c_int as libc::c_uint != 0 {
                (*jp2).jp2_state |=
                    JP2_STATE_CODESTREAM as libc::c_int as libc::c_uint;
                opj_free(l_current_data as *mut libc::c_void);
                return 1 as libc::c_int
            } else {
                opj_event_msg(p_manager, 1 as libc::c_int,
                              b"bad placed jpeg codestream\n\x00" as *const u8
                                  as *const libc::c_char);
                opj_free(l_current_data as *mut libc::c_void);
                return 0 as libc::c_int
            }
        } else {
            if box_0.length == 0 as libc::c_int as libc::c_uint {
                opj_event_msg(p_manager, 1 as libc::c_int,
                              b"Cannot handle box of undefined sizes\n\x00" as
                                  *const u8 as *const libc::c_char);
                opj_free(l_current_data as *mut libc::c_void);
                return 0 as libc::c_int
            } else {
                /* testcase 1851.pdf.SIGSEGV.ce9.948 */
                if box_0.length < l_nb_bytes_read {
                    opj_event_msg(p_manager, 1 as libc::c_int,
                                  b"invalid box size %d (%x)\n\x00" as
                                      *const u8 as *const libc::c_char,
                                  box_0.length, box_0.type_0);
                    opj_free(l_current_data as *mut libc::c_void);
                    return 0 as libc::c_int
                }
            }
        }
        l_current_handler = opj_jp2_find_handler(box_0.type_0);
        l_current_handler_misplaced = opj_jp2_img_find_handler(box_0.type_0);
        l_current_data_size = box_0.length.wrapping_sub(l_nb_bytes_read);
        if !l_current_handler.is_null() ||
               !l_current_handler_misplaced.is_null() {
            if l_current_handler.is_null() {
                opj_event_msg(p_manager, 2 as libc::c_int,
                              b"Found a misplaced \'%c%c%c%c\' box outside jp2h box\n\x00"
                                  as *const u8 as *const libc::c_char,
                              (box_0.type_0 >> 24 as libc::c_int) as OPJ_BYTE
                                  as libc::c_int,
                              (box_0.type_0 >> 16 as libc::c_int) as OPJ_BYTE
                                  as libc::c_int,
                              (box_0.type_0 >> 8 as libc::c_int) as OPJ_BYTE
                                  as libc::c_int,
                              (box_0.type_0 >> 0 as libc::c_int) as OPJ_BYTE
                                  as libc::c_int);
                if (*jp2).jp2_state &
                       JP2_STATE_HEADER as libc::c_int as libc::c_uint != 0 {
                    /* read anyway, we already have jp2h */
                    l_current_handler = l_current_handler_misplaced
                } else {
                    opj_event_msg(p_manager, 2 as libc::c_int,
                                  b"JPEG2000 Header box not read yet, \'%c%c%c%c\' box will be ignored\n\x00"
                                      as *const u8 as *const libc::c_char,
                                  (box_0.type_0 >> 24 as libc::c_int) as
                                      OPJ_BYTE as libc::c_int,
                                  (box_0.type_0 >> 16 as libc::c_int) as
                                      OPJ_BYTE as libc::c_int,
                                  (box_0.type_0 >> 8 as libc::c_int) as
                                      OPJ_BYTE as libc::c_int,
                                  (box_0.type_0 >> 0 as libc::c_int) as
                                      OPJ_BYTE as libc::c_int);
                    (*jp2).jp2_state |=
                        JP2_STATE_UNKNOWN as libc::c_int as libc::c_uint;
                    if opj_stream_skip(stream,
                                       l_current_data_size as OPJ_OFF_T,
                                       p_manager) !=
                           l_current_data_size as libc::c_long {
                        opj_event_msg(p_manager, 1 as libc::c_int,
                                      b"Problem with skipping JPEG2000 box, stream error\n\x00"
                                          as *const u8 as
                                          *const libc::c_char);
                        opj_free(l_current_data as *mut libc::c_void);
                        return 0 as libc::c_int
                    }
                    continue ;
                }
            }
            if l_current_data_size as OPJ_OFF_T >
                   opj_stream_get_number_byte_left(stream) {
                /* do not even try to malloc if we can't read */
                opj_event_msg(p_manager, 1 as libc::c_int,
                              b"Invalid box size %d for box \'%c%c%c%c\'. Need %d bytes, %d bytes remaining \n\x00"
                                  as *const u8 as *const libc::c_char,
                              box_0.length,
                              (box_0.type_0 >> 24 as libc::c_int) as OPJ_BYTE
                                  as libc::c_int,
                              (box_0.type_0 >> 16 as libc::c_int) as OPJ_BYTE
                                  as libc::c_int,
                              (box_0.type_0 >> 8 as libc::c_int) as OPJ_BYTE
                                  as libc::c_int,
                              (box_0.type_0 >> 0 as libc::c_int) as OPJ_BYTE
                                  as libc::c_int, l_current_data_size,
                              opj_stream_get_number_byte_left(stream) as
                                  OPJ_UINT32);
                opj_free(l_current_data as *mut libc::c_void);
                return 0 as libc::c_int
            }
            if l_current_data_size > l_last_data_size {
                let mut new_current_data =
                    opj_realloc(l_current_data as *mut libc::c_void,
                                l_current_data_size as size_t) as
                        *mut OPJ_BYTE;
                if new_current_data.is_null() {
                    opj_free(l_current_data as *mut libc::c_void);
                    opj_event_msg(p_manager, 1 as libc::c_int,
                                  b"Not enough memory to handle jpeg2000 box\n\x00"
                                      as *const u8 as *const libc::c_char);
                    return 0 as libc::c_int
                }
                l_current_data = new_current_data;
                l_last_data_size = l_current_data_size
            }
            l_nb_bytes_read =
                opj_stream_read_data(stream, l_current_data,
                                     l_current_data_size as OPJ_SIZE_T,
                                     p_manager) as OPJ_UINT32;
            if l_nb_bytes_read != l_current_data_size {
                opj_event_msg(p_manager, 1 as libc::c_int,
                              b"Problem with reading JPEG2000 box, stream error\n\x00"
                                  as *const u8 as *const libc::c_char);
                opj_free(l_current_data as *mut libc::c_void);
                return 0 as libc::c_int
            }
            if (*l_current_handler).handler.expect("non-null function pointer")(jp2,
                                                                                l_current_data,
                                                                                l_current_data_size,
                                                                                p_manager)
                   == 0 {
                opj_free(l_current_data as *mut libc::c_void);
                return 0 as libc::c_int
            }
        } else {
            if (*jp2).jp2_state &
                   JP2_STATE_SIGNATURE as libc::c_int as libc::c_uint == 0 {
                opj_event_msg(p_manager, 1 as libc::c_int,
                              b"Malformed JP2 file format: first box must be JPEG 2000 signature box\n\x00"
                                  as *const u8 as *const libc::c_char);
                opj_free(l_current_data as *mut libc::c_void);
                return 0 as libc::c_int
            }
            if (*jp2).jp2_state &
                   JP2_STATE_FILE_TYPE as libc::c_int as libc::c_uint == 0 {
                opj_event_msg(p_manager, 1 as libc::c_int,
                              b"Malformed JP2 file format: second box must be file type box\n\x00"
                                  as *const u8 as *const libc::c_char);
                opj_free(l_current_data as *mut libc::c_void);
                return 0 as libc::c_int
            }
            (*jp2).jp2_state |=
                JP2_STATE_UNKNOWN as libc::c_int as libc::c_uint;
            if opj_stream_skip(stream, l_current_data_size as OPJ_OFF_T,
                               p_manager) !=
                   l_current_data_size as libc::c_long {
                if (*jp2).jp2_state &
                       JP2_STATE_CODESTREAM as libc::c_int as libc::c_uint !=
                       0 {
                    /* If we already read the codestream, do not error out */
                    /* Needed for data/input/nonregression/issue254.jp2 */
                    opj_event_msg(p_manager, 2 as libc::c_int,
                                  b"Problem with skipping JPEG2000 box, stream error\n\x00"
                                      as *const u8 as *const libc::c_char);
                    opj_free(l_current_data as *mut libc::c_void);
                    return 1 as libc::c_int
                } else {
                    opj_event_msg(p_manager, 1 as libc::c_int,
                                  b"Problem with skipping JPEG2000 box, stream error\n\x00"
                                      as *const u8 as *const libc::c_char);
                    opj_free(l_current_data as *mut libc::c_void);
                    return 0 as libc::c_int
                }
            }
        }
    }
    opj_free(l_current_data as *mut libc::c_void);
    return 1 as libc::c_int;
}
/* *
 * Executes the given procedures on the given codec.
 *
 * @param   p_procedure_list    the list of procedures to execute
 * @param   jp2                 the jpeg2000 file codec to execute the procedures on.
 * @param   stream                  the stream to execute the procedures on.
 * @param   p_manager           the user manager.
 *
 * @return  true                if all the procedures were successfully executed.
 */
/* *
 * Executes the given procedures on the given codec.
 *
 * @param   p_procedure_list    the list of procedures to execute
 * @param   jp2                 the jpeg2000 file codec to execute the procedures on.
 * @param   stream                  the stream to execute the procedures on.
 * @param   p_manager           the user manager.
 *
 * @return  true                if all the procedures were successfully executed.
 */
unsafe extern "C" fn opj_jp2_exec(mut jp2: *mut opj_jp2_t,
                                  mut p_procedure_list:
                                      *mut opj_procedure_list_t,
                                  mut stream: *mut opj_stream_private_t,
                                  mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut l_procedure =
        0 as
            *mut Option<unsafe extern "C" fn(_: *mut opj_jp2_t,
                                             _: *mut opj_stream_private_t,
                                             _: *mut opj_event_mgr_t)
                            -> OPJ_BOOL>;
    let mut l_result = 1 as libc::c_int;
    let mut l_nb_proc: OPJ_UINT32 = 0;
    let mut i: OPJ_UINT32 = 0;
    /* preconditions */
    if !p_procedure_list.is_null() {
    } else {
        __assert_fail(b"p_procedure_list != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2444 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 102],
                                                &[libc::c_char; 102]>(b"OPJ_BOOL opj_jp2_exec(opj_jp2_t *, opj_procedure_list_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2445 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 102],
                                                &[libc::c_char; 102]>(b"OPJ_BOOL opj_jp2_exec(opj_jp2_t *, opj_procedure_list_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !stream.is_null() {
    } else {
        __assert_fail(b"stream != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2446 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 102],
                                                &[libc::c_char; 102]>(b"OPJ_BOOL opj_jp2_exec(opj_jp2_t *, opj_procedure_list_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2447 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 102],
                                                &[libc::c_char; 102]>(b"OPJ_BOOL opj_jp2_exec(opj_jp2_t *, opj_procedure_list_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    l_nb_proc = opj_procedure_list_get_nb_procedures(p_procedure_list);
    l_procedure =
        opj_procedure_list_get_first_procedure(p_procedure_list) as
            *mut Option<unsafe extern "C" fn(_: *mut opj_jp2_t,
                                             _: *mut opj_stream_private_t,
                                             _: *mut opj_event_mgr_t)
                            -> OPJ_BOOL>;
    i = 0 as libc::c_int as OPJ_UINT32;
    while i < l_nb_proc {
        l_result =
            (l_result != 0 &&
                 (*l_procedure).expect("non-null function pointer")(jp2,
                                                                    stream,
                                                                    p_manager)
                     != 0) as libc::c_int;
        l_procedure = l_procedure.offset(1);
        i = i.wrapping_add(1)
    }
    /* and clear the procedure list at the end. */
    opj_procedure_list_clear(p_procedure_list);
    return l_result;
}
#[no_mangle]
pub unsafe extern "C" fn opj_jp2_start_compress(mut jp2: *mut opj_jp2_t,
                                                mut stream:
                                                    *mut opj_stream_private_t,
                                                mut p_image: *mut opj_image_t,
                                                mut p_manager:
                                                    *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    /* preconditions */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2470 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 103],
                                                &[libc::c_char; 103]>(b"OPJ_BOOL opj_jp2_start_compress(opj_jp2_t *, opj_stream_private_t *, opj_image_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !stream.is_null() {
    } else {
        __assert_fail(b"stream != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2471 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 103],
                                                &[libc::c_char; 103]>(b"OPJ_BOOL opj_jp2_start_compress(opj_jp2_t *, opj_stream_private_t *, opj_image_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2472 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 103],
                                                &[libc::c_char; 103]>(b"OPJ_BOOL opj_jp2_start_compress(opj_jp2_t *, opj_stream_private_t *, opj_image_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    /* customization of the validation */
    if opj_jp2_setup_encoding_validation(jp2, p_manager) == 0 {
        return 0 as libc::c_int
    }
    /* validation of the parameters codec */
    if opj_jp2_exec(jp2, (*jp2).m_validation_list, stream, p_manager) == 0 {
        return 0 as libc::c_int
    }
    /* customization of the encoding */
    if opj_jp2_setup_header_writing(jp2, p_manager) == 0 {
        return 0 as libc::c_int
    }
    /* write header */
    if opj_jp2_exec(jp2, (*jp2).m_procedure_list, stream, p_manager) == 0 {
        return 0 as libc::c_int
    }
    return opj_j2k_start_compress((*jp2).j2k, stream, p_image, p_manager);
}
/* *
 * Finds the execution function related to the given box id.
 *
 * @param   p_id    the id of the handler to fetch.
 *
 * @return  the given handler or NULL if it could not be found.
 */
unsafe extern "C" fn opj_jp2_find_handler(mut p_id: OPJ_UINT32)
 -> *const opj_jp2_header_handler_t {
    let mut i: OPJ_UINT32 = 0;
    let mut l_handler_size =
        (::std::mem::size_of::<[opj_jp2_header_handler_t; 3]>() as
             libc::c_ulong).wrapping_div(::std::mem::size_of::<opj_jp2_header_handler_t>()
                                             as libc::c_ulong) as OPJ_UINT32;
    i = 0 as libc::c_int as OPJ_UINT32;
    while i < l_handler_size {
        if jp2_header[i as usize].id == p_id {
            return &*jp2_header.as_ptr().offset(i as isize) as
                       *const opj_jp2_header_handler_t
        }
        i = i.wrapping_add(1)
    }
    return 0 as *const opj_jp2_header_handler_t;
}
/* *
 * Finds the image execution function related to the given box id.
 *
 * @param   p_id    the id of the handler to fetch.
 *
 * @return  the given handler or NULL if it could not be found.
 */
/* *
 * Finds the image execution function related to the given box id.
 *
 * @param   p_id    the id of the handler to fetch.
 *
 * @return  the given handler or 00 if it could not be found.
 */
unsafe extern "C" fn opj_jp2_img_find_handler(mut p_id: OPJ_UINT32)
 -> *const opj_jp2_header_handler_t {
    let mut i: OPJ_UINT32 = 0;
    let mut l_handler_size =
        (::std::mem::size_of::<[opj_jp2_header_handler_t; 6]>() as
             libc::c_ulong).wrapping_div(::std::mem::size_of::<opj_jp2_header_handler_t>()
                                             as libc::c_ulong) as OPJ_UINT32;
    i = 0 as libc::c_int as OPJ_UINT32;
    while i < l_handler_size {
        if jp2_img_header[i as usize].id == p_id {
            return &*jp2_img_header.as_ptr().offset(i as isize) as
                       *const opj_jp2_header_handler_t
        }
        i = i.wrapping_add(1)
    }
    return 0 as *const opj_jp2_header_handler_t;
}
/* USE_JPIP */
/* *
 * Reads a jpeg2000 file signature box.
 *
 * @param   p_header_data   the data contained in the signature box.
 * @param   jp2             the jpeg2000 file codec.
 * @param   p_header_size   the size of the data contained in the signature box.
 * @param   p_manager       the user event manager.
 *
 * @return true if the file signature box is valid.
 */
/* *
 * Reads a jpeg2000 file signature box.
 *
 * @param   p_header_data   the data contained in the signature box.
 * @param   jp2             the jpeg2000 file codec.
 * @param   p_header_size   the size of the data contained in the signature box.
 * @param   p_manager       the user event manager.
 *
 * @return true if the file signature box is valid.
 */
unsafe extern "C" fn opj_jp2_read_jp(mut jp2: *mut opj_jp2_t,
                                     mut p_header_data: *mut OPJ_BYTE,
                                     mut p_header_size: OPJ_UINT32,
                                     mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut l_magic_number: OPJ_UINT32 = 0;
    /* preconditions */
    if !p_header_data.is_null() {
    } else {
        __assert_fail(b"p_header_data != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2551 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 81],
                                                &[libc::c_char; 81]>(b"OPJ_BOOL opj_jp2_read_jp(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2552 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 81],
                                                &[libc::c_char; 81]>(b"OPJ_BOOL opj_jp2_read_jp(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2553 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 81],
                                                &[libc::c_char; 81]>(b"OPJ_BOOL opj_jp2_read_jp(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if (*jp2).jp2_state != JP2_STATE_NONE as libc::c_int as libc::c_uint {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"The signature box must be the first box in the file.\n\x00"
                          as *const u8 as *const libc::c_char);
        return 0 as libc::c_int
    }
    /* assure length of data is correct (4 -> magic number) */
    if p_header_size != 4 as libc::c_int as libc::c_uint {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Error with JP signature Box size\n\x00" as *const u8
                          as *const libc::c_char);
        return 0 as libc::c_int
    }
    /* rearrange data */
    opj_read_bytes_LE(p_header_data, &mut l_magic_number,
                      4 as libc::c_int as OPJ_UINT32);
    if l_magic_number != 0xd0a870a as libc::c_int as libc::c_uint {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Error with JP Signature : bad magic number\n\x00" as
                          *const u8 as *const libc::c_char);
        return 0 as libc::c_int
    }
    (*jp2).jp2_state |= JP2_STATE_SIGNATURE as libc::c_int as libc::c_uint;
    return 1 as libc::c_int;
}
/* *
 * Reads a a FTYP box - File type box
 *
 * @param   p_header_data   the data contained in the FTYP box.
 * @param   jp2             the jpeg2000 file codec.
 * @param   p_header_size   the size of the data contained in the FTYP box.
 * @param   p_manager       the user event manager.
 *
 * @return true if the FTYP box is valid.
 */
/* *
 * Reads a a FTYP box - File type box
 *
 * @param   p_header_data   the data contained in the FTYP box.
 * @param   jp2             the jpeg2000 file codec.
 * @param   p_header_size   the size of the data contained in the FTYP box.
 * @param   p_manager       the user event manager.
 *
 * @return true if the FTYP box is valid.
 */
unsafe extern "C" fn opj_jp2_read_ftyp(mut jp2: *mut opj_jp2_t,
                                       mut p_header_data: *mut OPJ_BYTE,
                                       mut p_header_size: OPJ_UINT32,
                                       mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut i: OPJ_UINT32 = 0;
    let mut l_remaining_bytes: OPJ_UINT32 = 0;
    /* preconditions */
    if !p_header_data.is_null() {
    } else {
        __assert_fail(b"p_header_data != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2599 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_ftyp(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2600 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_ftyp(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2601 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_ftyp(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if (*jp2).jp2_state != JP2_STATE_SIGNATURE as libc::c_int as libc::c_uint
       {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"The ftyp box must be the second box in the file.\n\x00"
                          as *const u8 as *const libc::c_char);
        return 0 as libc::c_int
    }
    /* assure length of data is correct */
    if p_header_size < 8 as libc::c_int as libc::c_uint {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Error with FTYP signature Box size\n\x00" as *const u8
                          as *const libc::c_char); /* BR */
        return 0 as libc::c_int
    } /* MinV */
    opj_read_bytes_LE(p_header_data, &mut (*jp2).brand,
                      4 as libc::c_int as OPJ_UINT32);
    p_header_data = p_header_data.offset(4 as libc::c_int as isize);
    opj_read_bytes_LE(p_header_data, &mut (*jp2).minversion,
                      4 as libc::c_int as OPJ_UINT32);
    p_header_data = p_header_data.offset(4 as libc::c_int as isize);
    l_remaining_bytes =
        p_header_size.wrapping_sub(8 as libc::c_int as libc::c_uint);
    /* the number of remaining bytes should be a multiple of 4 */
    if l_remaining_bytes & 0x3 as libc::c_int as libc::c_uint !=
           0 as libc::c_int as libc::c_uint {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Error with FTYP signature Box size\n\x00" as *const u8
                          as *const libc::c_char);
        return 0 as libc::c_int
    }
    /* div by 4 */
    (*jp2).numcl = l_remaining_bytes >> 2 as libc::c_int; /* CLi */
    if (*jp2).numcl != 0 {
        (*jp2).cl =
            opj_calloc((*jp2).numcl as size_t,
                       ::std::mem::size_of::<OPJ_UINT32>() as libc::c_ulong)
                as *mut OPJ_UINT32;
        if (*jp2).cl.is_null() {
            opj_event_msg(p_manager, 1 as libc::c_int,
                          b"Not enough memory with FTYP Box\n\x00" as
                              *const u8 as *const libc::c_char);
            return 0 as libc::c_int
        }
    }
    i = 0 as libc::c_int as OPJ_UINT32;
    while i < (*jp2).numcl {
        opj_read_bytes_LE(p_header_data, &mut *(*jp2).cl.offset(i as isize),
                          4 as libc::c_int as OPJ_UINT32);
        p_header_data = p_header_data.offset(4 as libc::c_int as isize);
        i = i.wrapping_add(1)
    }
    (*jp2).jp2_state |= JP2_STATE_FILE_TYPE as libc::c_int as libc::c_uint;
    return 1 as libc::c_int;
}
unsafe extern "C" fn opj_jp2_skip_jp2c(mut jp2: *mut opj_jp2_t,
                                       mut stream: *mut opj_stream_private_t,
                                       mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    /* preconditions */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2654 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_skip_jp2c(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !stream.is_null() {
    } else {
        __assert_fail(b"stream != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2655 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_skip_jp2c(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2656 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_skip_jp2c(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    (*jp2).j2k_codestream_offset = opj_stream_tell(stream);
    if opj_stream_skip(stream, 8 as libc::c_int as OPJ_OFF_T, p_manager) !=
           8 as libc::c_int as libc::c_long {
        return 0 as libc::c_int
    }
    return 1 as libc::c_int;
}
unsafe extern "C" fn opj_jpip_skip_iptr(mut jp2: *mut opj_jp2_t,
                                        mut stream: *mut opj_stream_private_t,
                                        mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    /* preconditions */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2672 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 84],
                                                &[libc::c_char; 84]>(b"OPJ_BOOL opj_jpip_skip_iptr(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !stream.is_null() {
    } else {
        __assert_fail(b"stream != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2673 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 84],
                                                &[libc::c_char; 84]>(b"OPJ_BOOL opj_jpip_skip_iptr(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2674 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 84],
                                                &[libc::c_char; 84]>(b"OPJ_BOOL opj_jpip_skip_iptr(opj_jp2_t *, opj_stream_private_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    (*jp2).jpip_iptr_offset = opj_stream_tell(stream);
    if opj_stream_skip(stream, 24 as libc::c_int as OPJ_OFF_T, p_manager) !=
           24 as libc::c_int as libc::c_long {
        return 0 as libc::c_int
    }
    return 1 as libc::c_int;
}
/* *
 * Reads the Jpeg2000 file Header box - JP2 Header box (warning, this is a super box).
 *
 * @param   p_header_data   the data contained in the file header box.
 * @param   jp2             the jpeg2000 file codec.
 * @param   p_header_size   the size of the data contained in the file header box.
 * @param   p_manager       the user event manager.
 *
 * @return true if the JP2 Header box was successfully recognized.
*/
/* *
 * Reads the Jpeg2000 file Header box - JP2 Header box (warning, this is a super box).
 *
 * @param   p_header_data   the data contained in the file header box.
 * @param   jp2             the jpeg2000 file codec.
 * @param   p_header_size   the size of the data contained in the file header box.
 * @param   p_manager       the user event manager.
 *
 * @return true if the JP2 Header box was successfully recognized.
*/
unsafe extern "C" fn opj_jp2_read_jp2h(mut jp2: *mut opj_jp2_t,
                                       mut p_header_data: *mut OPJ_BYTE,
                                       mut p_header_size: OPJ_UINT32,
                                       mut p_manager: *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut l_box_size = 0 as libc::c_int as OPJ_UINT32;
    let mut l_current_data_size = 0 as libc::c_int as OPJ_UINT32;
    let mut box_0 = opj_jp2_box_t{length: 0, type_0: 0, init_pos: 0,};
    let mut l_current_handler = 0 as *const opj_jp2_header_handler_t;
    let mut l_has_ihdr = 0 as libc::c_int;
    /* preconditions */
    if !p_header_data.is_null() {
    } else {
        __assert_fail(b"p_header_data != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2707 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_jp2h(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2708 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_jp2h(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2709 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 83],
                                                &[libc::c_char; 83]>(b"OPJ_BOOL opj_jp2_read_jp2h(opj_jp2_t *, OPJ_BYTE *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    /* make sure the box is well placed */
    if (*jp2).jp2_state & JP2_STATE_FILE_TYPE as libc::c_int as libc::c_uint
           != JP2_STATE_FILE_TYPE as libc::c_int as libc::c_uint {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"The  box must be the first box in the file.\n\x00" as
                          *const u8 as *const libc::c_char);
        return 0 as libc::c_int
    }
    (*jp2).jp2_img_state = JP2_IMG_STATE_NONE as libc::c_int as OPJ_UINT32;
    /* iterate while remaining data */
    while p_header_size > 0 as libc::c_int as libc::c_uint {
        if opj_jp2_read_boxhdr_char(&mut box_0, p_header_data,
                                    &mut l_box_size, p_header_size, p_manager)
               == 0 {
            opj_event_msg(p_manager, 1 as libc::c_int,
                          b"Stream error while reading JP2 Header box\n\x00"
                              as *const u8 as *const libc::c_char);
            return 0 as libc::c_int
        }
        if box_0.length > p_header_size {
            opj_event_msg(p_manager, 1 as libc::c_int,
                          b"Stream error while reading JP2 Header box: box length is inconsistent.\n\x00"
                              as *const u8 as *const libc::c_char);
            return 0 as libc::c_int
        }
        l_current_handler = opj_jp2_img_find_handler(box_0.type_0);
        l_current_data_size = box_0.length.wrapping_sub(l_box_size);
        p_header_data = p_header_data.offset(l_box_size as isize);
        if !l_current_handler.is_null() {
            if (*l_current_handler).handler.expect("non-null function pointer")(jp2,
                                                                                p_header_data,
                                                                                l_current_data_size,
                                                                                p_manager)
                   == 0 {
                return 0 as libc::c_int
            }
        } else {
            (*jp2).jp2_img_state |=
                JP2_IMG_STATE_UNKNOWN as libc::c_int as libc::c_uint
        }
        if box_0.type_0 == 0x69686472 as libc::c_int as libc::c_uint {
            l_has_ihdr = 1 as libc::c_int
        }
        p_header_data = p_header_data.offset(l_current_data_size as isize);
        p_header_size =
            (p_header_size as libc::c_uint).wrapping_sub(box_0.length) as
                OPJ_UINT32 as OPJ_UINT32
    }
    if l_has_ihdr == 0 as libc::c_int {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Stream error while reading JP2 Header box: no \'ihdr\' box.\n\x00"
                          as *const u8 as *const libc::c_char);
        return 0 as libc::c_int
    }
    (*jp2).jp2_state |= JP2_STATE_HEADER as libc::c_int as libc::c_uint;
    (*jp2).has_jp2h = 1 as libc::c_int as OPJ_BYTE;
    return 1 as libc::c_int;
}
/* *
 * Reads a box header. The box is the way data is packed inside a jpeg2000 file structure. Data is read from a character string
 *
 * @param   box                     the box structure to fill.
 * @param   p_data                  the character string to read data from.
 * @param   p_number_bytes_read     pointer to an int that will store the number of bytes read from the stream (shoul usually be 2).
 * @param   p_box_max_size          the maximum number of bytes in the box.
 * @param   p_manager         FIXME DOC
 *
 * @return  true if the box is recognized, false otherwise
*/
unsafe extern "C" fn opj_jp2_read_boxhdr_char(mut box_0: *mut opj_jp2_box_t,
                                              mut p_data: *mut OPJ_BYTE,
                                              mut p_number_bytes_read:
                                                  *mut OPJ_UINT32,
                                              mut p_box_max_size: OPJ_UINT32,
                                              mut p_manager:
                                                  *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    let mut l_value: OPJ_UINT32 = 0;
    /* preconditions */
    if !p_data.is_null() {
    } else {
        __assert_fail(b"p_data != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2779 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 108],
                                                &[libc::c_char; 108]>(b"OPJ_BOOL opj_jp2_read_boxhdr_char(opj_jp2_box_t *, OPJ_BYTE *, OPJ_UINT32 *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !box_0.is_null() {
    } else {
        __assert_fail(b"box != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2780 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 108],
                                                &[libc::c_char; 108]>(b"OPJ_BOOL opj_jp2_read_boxhdr_char(opj_jp2_box_t *, OPJ_BYTE *, OPJ_UINT32 *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_number_bytes_read.is_null() {
    } else {
        __assert_fail(b"p_number_bytes_read != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2781 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 108],
                                                &[libc::c_char; 108]>(b"OPJ_BOOL opj_jp2_read_boxhdr_char(opj_jp2_box_t *, OPJ_BYTE *, OPJ_UINT32 *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2782 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 108],
                                                &[libc::c_char; 108]>(b"OPJ_BOOL opj_jp2_read_boxhdr_char(opj_jp2_box_t *, OPJ_BYTE *, OPJ_UINT32 *, OPJ_UINT32, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if p_box_max_size < 8 as libc::c_int as libc::c_uint {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Cannot handle box of less than 8 bytes\n\x00" as
                          *const u8 as *const libc::c_char);
        return 0 as libc::c_int
    }
    /* process read data */
    opj_read_bytes_LE(p_data, &mut l_value, 4 as libc::c_int as OPJ_UINT32);
    p_data = p_data.offset(4 as libc::c_int as isize);
    (*box_0).length = l_value;
    opj_read_bytes_LE(p_data, &mut l_value, 4 as libc::c_int as OPJ_UINT32);
    p_data = p_data.offset(4 as libc::c_int as isize);
    (*box_0).type_0 = l_value;
    *p_number_bytes_read = 8 as libc::c_int as OPJ_UINT32;
    /* do we have a "special very large box ?" */
    /* read then the XLBox */
    if (*box_0).length == 1 as libc::c_int as libc::c_uint {
        let mut l_xl_part_size: OPJ_UINT32 = 0;
        if p_box_max_size < 16 as libc::c_int as libc::c_uint {
            opj_event_msg(p_manager, 1 as libc::c_int,
                          b"Cannot handle XL box of less than 16 bytes\n\x00"
                              as *const u8 as *const libc::c_char);
            return 0 as libc::c_int
        }
        opj_read_bytes_LE(p_data, &mut l_xl_part_size,
                          4 as libc::c_int as OPJ_UINT32);
        p_data = p_data.offset(4 as libc::c_int as isize);
        *p_number_bytes_read =
            (*p_number_bytes_read as
                 libc::c_uint).wrapping_add(4 as libc::c_int as libc::c_uint)
                as OPJ_UINT32 as OPJ_UINT32;
        if l_xl_part_size != 0 as libc::c_int as libc::c_uint {
            opj_event_msg(p_manager, 1 as libc::c_int,
                          b"Cannot handle box sizes higher than 2^32\n\x00" as
                              *const u8 as *const libc::c_char);
            return 0 as libc::c_int
        }
        opj_read_bytes_LE(p_data, &mut l_value,
                          4 as libc::c_int as OPJ_UINT32);
        *p_number_bytes_read =
            (*p_number_bytes_read as
                 libc::c_uint).wrapping_add(4 as libc::c_int as libc::c_uint)
                as OPJ_UINT32 as OPJ_UINT32;
        (*box_0).length = l_value;
        if (*box_0).length == 0 as libc::c_int as libc::c_uint {
            opj_event_msg(p_manager, 1 as libc::c_int,
                          b"Cannot handle box of undefined sizes\n\x00" as
                              *const u8 as *const libc::c_char);
            return 0 as libc::c_int
        }
    } else if (*box_0).length == 0 as libc::c_int as libc::c_uint {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Cannot handle box of undefined sizes\n\x00" as
                          *const u8 as *const libc::c_char);
        return 0 as libc::c_int
    }
    if (*box_0).length < *p_number_bytes_read {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Box length is inconsistent.\n\x00" as *const u8 as
                          *const libc::c_char);
        return 0 as libc::c_int
    }
    return 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn opj_jp2_read_header(mut p_stream:
                                                 *mut opj_stream_private_t,
                                             mut jp2: *mut opj_jp2_t,
                                             mut p_image:
                                                 *mut *mut opj_image_t,
                                             mut p_manager:
                                                 *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    /* preconditions */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2847 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 101],
                                                &[libc::c_char; 101]>(b"OPJ_BOOL opj_jp2_read_header(opj_stream_private_t *, opj_jp2_t *, opj_image_t **, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_stream.is_null() {
    } else {
        __assert_fail(b"p_stream != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2848 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 101],
                                                &[libc::c_char; 101]>(b"OPJ_BOOL opj_jp2_read_header(opj_stream_private_t *, opj_jp2_t *, opj_image_t **, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2849 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 101],
                                                &[libc::c_char; 101]>(b"OPJ_BOOL opj_jp2_read_header(opj_stream_private_t *, opj_jp2_t *, opj_image_t **, opj_event_mgr_t *)\x00")).as_ptr());
    }
    /* customization of the validation */
    if opj_jp2_setup_decoding_validation(jp2, p_manager) == 0 {
        return 0 as libc::c_int
    }
    /* customization of the encoding */
    if opj_jp2_setup_header_reading(jp2, p_manager) == 0 {
        return 0 as libc::c_int
    }
    /* validation of the parameters codec */
    if opj_jp2_exec(jp2, (*jp2).m_validation_list, p_stream, p_manager) == 0 {
        return 0 as libc::c_int
    }
    /* read header */
    if opj_jp2_exec(jp2, (*jp2).m_procedure_list, p_stream, p_manager) == 0 {
        return 0 as libc::c_int
    }
    if (*jp2).has_jp2h as libc::c_int == 0 as libc::c_int {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"JP2H box missing. Required.\n\x00" as *const u8 as
                          *const libc::c_char);
        return 0 as libc::c_int
    }
    if (*jp2).has_ihdr as libc::c_int == 0 as libc::c_int {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"IHDR box_missing. Required.\n\x00" as *const u8 as
                          *const libc::c_char);
        return 0 as libc::c_int
    }
    return opj_j2k_read_header(p_stream, (*jp2).j2k, p_image, p_manager);
}
/* *
 * Sets up the validation ,i.e. adds the procedures to launch to make sure the codec parameters
 * are valid. Developers wanting to extend the library can add their own validation procedures.
 */
unsafe extern "C" fn opj_jp2_setup_encoding_validation(mut jp2:
                                                           *mut opj_jp2_t,
                                                       mut p_manager:
                                                           *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    /* preconditions */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2889 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 75],
                                                &[libc::c_char; 75]>(b"OPJ_BOOL opj_jp2_setup_encoding_validation(opj_jp2_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2890 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 75],
                                                &[libc::c_char; 75]>(b"OPJ_BOOL opj_jp2_setup_encoding_validation(opj_jp2_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if opj_procedure_list_add_procedure((*jp2).m_validation_list,
                                        ::std::mem::transmute::<Option<unsafe extern "C" fn(_:
                                                                                                *mut opj_jp2_t,
                                                                                            _:
                                                                                                *mut opj_stream_private_t,
                                                                                            _:
                                                                                                *mut opj_event_mgr_t)
                                                                           ->
                                                                               OPJ_BOOL>,
                                                                opj_procedure>(Some(opj_jp2_default_validation
                                                                                        as
                                                                                        unsafe extern "C" fn(_:
                                                                                                                 *mut opj_jp2_t,
                                                                                                             _:
                                                                                                                 *mut opj_stream_private_t,
                                                                                                             _:
                                                                                                                 *mut opj_event_mgr_t)
                                                                                            ->
                                                                                                OPJ_BOOL)),
                                        p_manager) == 0 {
        return 0 as libc::c_int
    }
    /* DEVELOPER CORNER, add your custom validation procedure */
    return 1 as libc::c_int;
}
/* *
 * Sets up the validation ,i.e. adds the procedures to launch to make sure the codec parameters
 * are valid. Developers wanting to extend the library can add their own validation procedures.
 */
unsafe extern "C" fn opj_jp2_setup_decoding_validation(mut jp2:
                                                           *mut opj_jp2_t,
                                                       mut p_manager:
                                                           *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    /* preconditions */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2905 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 75],
                                                &[libc::c_char; 75]>(b"OPJ_BOOL opj_jp2_setup_decoding_validation(opj_jp2_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2906 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 75],
                                                &[libc::c_char; 75]>(b"OPJ_BOOL opj_jp2_setup_decoding_validation(opj_jp2_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    /* DEVELOPER CORNER, add your custom validation procedure */
    return 1 as libc::c_int;
}
/* *
 * Sets up the procedures to do on writing header. Developers wanting to extend the library can add their own writing procedures.
 */
unsafe extern "C" fn opj_jp2_setup_header_writing(mut jp2: *mut opj_jp2_t,
                                                  mut p_manager:
                                                      *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    /* preconditions */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2920 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 70],
                                                &[libc::c_char; 70]>(b"OPJ_BOOL opj_jp2_setup_header_writing(opj_jp2_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2921 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 70],
                                                &[libc::c_char; 70]>(b"OPJ_BOOL opj_jp2_setup_header_writing(opj_jp2_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if opj_procedure_list_add_procedure((*jp2).m_procedure_list,
                                        ::std::mem::transmute::<Option<unsafe extern "C" fn(_:
                                                                                                *mut opj_jp2_t,
                                                                                            _:
                                                                                                *mut opj_stream_private_t,
                                                                                            _:
                                                                                                *mut opj_event_mgr_t)
                                                                           ->
                                                                               OPJ_BOOL>,
                                                                opj_procedure>(Some(opj_jp2_write_jp
                                                                                        as
                                                                                        unsafe extern "C" fn(_:
                                                                                                                 *mut opj_jp2_t,
                                                                                                             _:
                                                                                                                 *mut opj_stream_private_t,
                                                                                                             _:
                                                                                                                 *mut opj_event_mgr_t)
                                                                                            ->
                                                                                                OPJ_BOOL)),
                                        p_manager) == 0 {
        return 0 as libc::c_int
    }
    if opj_procedure_list_add_procedure((*jp2).m_procedure_list,
                                        ::std::mem::transmute::<Option<unsafe extern "C" fn(_:
                                                                                                *mut opj_jp2_t,
                                                                                            _:
                                                                                                *mut opj_stream_private_t,
                                                                                            _:
                                                                                                *mut opj_event_mgr_t)
                                                                           ->
                                                                               OPJ_BOOL>,
                                                                opj_procedure>(Some(opj_jp2_write_ftyp
                                                                                        as
                                                                                        unsafe extern "C" fn(_:
                                                                                                                 *mut opj_jp2_t,
                                                                                                             _:
                                                                                                                 *mut opj_stream_private_t,
                                                                                                             _:
                                                                                                                 *mut opj_event_mgr_t)
                                                                                            ->
                                                                                                OPJ_BOOL)),
                                        p_manager) == 0 {
        return 0 as libc::c_int
    }
    if opj_procedure_list_add_procedure((*jp2).m_procedure_list,
                                        ::std::mem::transmute::<Option<unsafe extern "C" fn(_:
                                                                                                *mut opj_jp2_t,
                                                                                            _:
                                                                                                *mut opj_stream_private_t,
                                                                                            _:
                                                                                                *mut opj_event_mgr_t)
                                                                           ->
                                                                               OPJ_BOOL>,
                                                                opj_procedure>(Some(opj_jp2_write_jp2h
                                                                                        as
                                                                                        unsafe extern "C" fn(_:
                                                                                                                 *mut opj_jp2_t,
                                                                                                             _:
                                                                                                                 *mut opj_stream_private_t,
                                                                                                             _:
                                                                                                                 *mut opj_event_mgr_t)
                                                                                            ->
                                                                                                OPJ_BOOL)),
                                        p_manager) == 0 {
        return 0 as libc::c_int
    }
    if (*jp2).jpip_on != 0 {
        if opj_procedure_list_add_procedure((*jp2).m_procedure_list,
                                            ::std::mem::transmute::<Option<unsafe extern "C" fn(_:
                                                                                                    *mut opj_jp2_t,
                                                                                                _:
                                                                                                    *mut opj_stream_private_t,
                                                                                                _:
                                                                                                    *mut opj_event_mgr_t)
                                                                               ->
                                                                                   OPJ_BOOL>,
                                                                    opj_procedure>(Some(opj_jpip_skip_iptr
                                                                                            as
                                                                                            unsafe extern "C" fn(_:
                                                                                                                     *mut opj_jp2_t,
                                                                                                                 _:
                                                                                                                     *mut opj_stream_private_t,
                                                                                                                 _:
                                                                                                                     *mut opj_event_mgr_t)
                                                                                                ->
                                                                                                    OPJ_BOOL)),
                                            p_manager) == 0 {
            return 0 as libc::c_int
        }
    }
    if opj_procedure_list_add_procedure((*jp2).m_procedure_list,
                                        ::std::mem::transmute::<Option<unsafe extern "C" fn(_:
                                                                                                *mut opj_jp2_t,
                                                                                            _:
                                                                                                *mut opj_stream_private_t,
                                                                                            _:
                                                                                                *mut opj_event_mgr_t)
                                                                           ->
                                                                               OPJ_BOOL>,
                                                                opj_procedure>(Some(opj_jp2_skip_jp2c
                                                                                        as
                                                                                        unsafe extern "C" fn(_:
                                                                                                                 *mut opj_jp2_t,
                                                                                                             _:
                                                                                                                 *mut opj_stream_private_t,
                                                                                                             _:
                                                                                                                 *mut opj_event_mgr_t)
                                                                                            ->
                                                                                                OPJ_BOOL)),
                                        p_manager) == 0 {
        return 0 as libc::c_int
    }
    /* DEVELOPER CORNER, insert your custom procedures */
    return 1 as libc::c_int;
}
/* *
 * Sets up the procedures to do on reading header.
 * Developers wanting to extend the library can add their own writing procedures.
 */
unsafe extern "C" fn opj_jp2_setup_header_reading(mut jp2: *mut opj_jp2_t,
                                                  mut p_manager:
                                                      *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    /* preconditions */
    if !jp2.is_null() {
    } else {
        __assert_fail(b"jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2955 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 70],
                                                &[libc::c_char; 70]>(b"OPJ_BOOL opj_jp2_setup_header_reading(opj_jp2_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if !p_manager.is_null() {
    } else {
        __assert_fail(b"p_manager != 00\x00" as *const u8 as
                          *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      2956 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 70],
                                                &[libc::c_char; 70]>(b"OPJ_BOOL opj_jp2_setup_header_reading(opj_jp2_t *, opj_event_mgr_t *)\x00")).as_ptr());
    }
    if opj_procedure_list_add_procedure((*jp2).m_procedure_list,
                                        ::std::mem::transmute::<Option<unsafe extern "C" fn(_:
                                                                                                *mut opj_jp2_t,
                                                                                            _:
                                                                                                *mut opj_stream_private_t,
                                                                                            _:
                                                                                                *mut opj_event_mgr_t)
                                                                           ->
                                                                               OPJ_BOOL>,
                                                                opj_procedure>(Some(opj_jp2_read_header_procedure
                                                                                        as
                                                                                        unsafe extern "C" fn(_:
                                                                                                                 *mut opj_jp2_t,
                                                                                                             _:
                                                                                                                 *mut opj_stream_private_t,
                                                                                                             _:
                                                                                                                 *mut opj_event_mgr_t)
                                                                                            ->
                                                                                                OPJ_BOOL)),
                                        p_manager) == 0 {
        return 0 as libc::c_int
    }
    /* DEVELOPER CORNER, add your custom procedures */
    return 1 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn opj_jp2_read_tile_header(mut p_jp2: *mut opj_jp2_t,
                                                  mut p_tile_index:
                                                      *mut OPJ_UINT32,
                                                  mut p_data_size:
                                                      *mut OPJ_UINT32,
                                                  mut p_tile_x0:
                                                      *mut OPJ_INT32,
                                                  mut p_tile_y0:
                                                      *mut OPJ_INT32,
                                                  mut p_tile_x1:
                                                      *mut OPJ_INT32,
                                                  mut p_tile_y1:
                                                      *mut OPJ_INT32,
                                                  mut p_nb_comps:
                                                      *mut OPJ_UINT32,
                                                  mut p_go_on: *mut OPJ_BOOL,
                                                  mut p_stream:
                                                      *mut opj_stream_private_t,
                                                  mut p_manager:
                                                      *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    return opj_j2k_read_tile_header((*p_jp2).j2k, p_tile_index, p_data_size,
                                    p_tile_x0, p_tile_y0, p_tile_x1,
                                    p_tile_y1, p_nb_comps, p_go_on, p_stream,
                                    p_manager);
}
#[no_mangle]
pub unsafe extern "C" fn opj_jp2_write_tile(mut p_jp2: *mut opj_jp2_t,
                                            mut p_tile_index: OPJ_UINT32,
                                            mut p_data: *mut OPJ_BYTE,
                                            mut p_data_size: OPJ_UINT32,
                                            mut p_stream:
                                                *mut opj_stream_private_t,
                                            mut p_manager:
                                                *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    return opj_j2k_write_tile((*p_jp2).j2k, p_tile_index, p_data, p_data_size,
                              p_stream, p_manager);
}
#[no_mangle]
pub unsafe extern "C" fn opj_jp2_decode_tile(mut p_jp2: *mut opj_jp2_t,
                                             mut p_tile_index: OPJ_UINT32,
                                             mut p_data: *mut OPJ_BYTE,
                                             mut p_data_size: OPJ_UINT32,
                                             mut p_stream:
                                                 *mut opj_stream_private_t,
                                             mut p_manager:
                                                 *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    return opj_j2k_decode_tile((*p_jp2).j2k, p_tile_index, p_data,
                               p_data_size, p_stream, p_manager);
}
#[no_mangle]
pub unsafe extern "C" fn opj_jp2_destroy(mut jp2: *mut opj_jp2_t) {
    if !jp2.is_null() {
        /* destroy the J2K codec */
        opj_j2k_destroy((*jp2).j2k);
        (*jp2).j2k = 0 as *mut opj_j2k_t;
        if !(*jp2).comps.is_null() {
            opj_free((*jp2).comps as *mut libc::c_void);
            (*jp2).comps = 0 as *mut opj_jp2_comps_t
        }
        if !(*jp2).cl.is_null() {
            opj_free((*jp2).cl as *mut libc::c_void);
            (*jp2).cl = 0 as *mut OPJ_UINT32
        }
        if !(*jp2).color.icc_profile_buf.is_null() {
            opj_free((*jp2).color.icc_profile_buf as *mut libc::c_void);
            (*jp2).color.icc_profile_buf = 0 as *mut OPJ_BYTE
        }
        if !(*jp2).color.jp2_cdef.is_null() {
            if !(*(*jp2).color.jp2_cdef).info.is_null() {
                opj_free((*(*jp2).color.jp2_cdef).info as *mut libc::c_void);
                (*(*jp2).color.jp2_cdef).info = 0 as *mut opj_jp2_cdef_info_t
            }
            opj_free((*jp2).color.jp2_cdef as *mut libc::c_void);
            (*jp2).color.jp2_cdef = 0 as *mut opj_jp2_cdef_t
        }
        if !(*jp2).color.jp2_pclr.is_null() {
            if !(*(*jp2).color.jp2_pclr).cmap.is_null() {
                opj_free((*(*jp2).color.jp2_pclr).cmap as *mut libc::c_void);
                (*(*jp2).color.jp2_pclr).cmap = 0 as *mut opj_jp2_cmap_comp_t
            }
            if !(*(*jp2).color.jp2_pclr).channel_sign.is_null() {
                opj_free((*(*jp2).color.jp2_pclr).channel_sign as
                             *mut libc::c_void);
                (*(*jp2).color.jp2_pclr).channel_sign = 0 as *mut OPJ_BYTE
            }
            if !(*(*jp2).color.jp2_pclr).channel_size.is_null() {
                opj_free((*(*jp2).color.jp2_pclr).channel_size as
                             *mut libc::c_void);
                (*(*jp2).color.jp2_pclr).channel_size = 0 as *mut OPJ_BYTE
            }
            if !(*(*jp2).color.jp2_pclr).entries.is_null() {
                opj_free((*(*jp2).color.jp2_pclr).entries as
                             *mut libc::c_void);
                (*(*jp2).color.jp2_pclr).entries = 0 as *mut OPJ_UINT32
            }
            opj_free((*jp2).color.jp2_pclr as *mut libc::c_void);
            (*jp2).color.jp2_pclr = 0 as *mut opj_jp2_pclr_t
        }
        if !(*jp2).m_validation_list.is_null() {
            opj_procedure_list_destroy((*jp2).m_validation_list);
            (*jp2).m_validation_list = 0 as *mut opj_procedure_list
        }
        if !(*jp2).m_procedure_list.is_null() {
            opj_procedure_list_destroy((*jp2).m_procedure_list);
            (*jp2).m_procedure_list = 0 as *mut opj_procedure_list
        }
        opj_free(jp2 as *mut libc::c_void);
    };
}
#[no_mangle]
pub unsafe extern "C" fn opj_jp2_set_decoded_components(mut p_jp2:
                                                            *mut opj_jp2_t,
                                                        mut numcomps:
                                                            OPJ_UINT32,
                                                        mut comps_indices:
                                                            *const OPJ_UINT32,
                                                        mut p_manager:
                                                            *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    return opj_j2k_set_decoded_components((*p_jp2).j2k, numcomps,
                                          comps_indices, p_manager);
}
#[no_mangle]
pub unsafe extern "C" fn opj_jp2_set_decode_area(mut p_jp2: *mut opj_jp2_t,
                                                 mut p_image:
                                                     *mut opj_image_t,
                                                 mut p_start_x: OPJ_INT32,
                                                 mut p_start_y: OPJ_INT32,
                                                 mut p_end_x: OPJ_INT32,
                                                 mut p_end_y: OPJ_INT32,
                                                 mut p_manager:
                                                     *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    return opj_j2k_set_decode_area((*p_jp2).j2k, p_image, p_start_x,
                                   p_start_y, p_end_x, p_end_y, p_manager);
}
#[no_mangle]
pub unsafe extern "C" fn opj_jp2_get_tile(mut p_jp2: *mut opj_jp2_t,
                                          mut p_stream:
                                              *mut opj_stream_private_t,
                                          mut p_image: *mut opj_image_t,
                                          mut p_manager: *mut opj_event_mgr_t,
                                          mut tile_index: OPJ_UINT32)
 -> OPJ_BOOL {
    if p_image.is_null() { return 0 as libc::c_int }
    opj_event_msg(p_manager, 2 as libc::c_int,
                  b"JP2 box which are after the codestream will not be read by this function.\n\x00"
                      as *const u8 as *const libc::c_char);
    if opj_j2k_get_tile((*p_jp2).j2k, p_stream, p_image, p_manager,
                        tile_index) == 0 {
        opj_event_msg(p_manager, 1 as libc::c_int,
                      b"Failed to decode the codestream in the JP2 file\n\x00"
                          as *const u8 as *const libc::c_char);
        return 0 as libc::c_int
    }
    if (*(*p_jp2).j2k).m_specific_param.m_decoder.m_numcomps_to_decode != 0 {
        /* Bypass all JP2 component transforms */
        return 1 as libc::c_int
    }
    if opj_jp2_check_color(p_image, &mut (*p_jp2).color, p_manager) == 0 {
        return 0 as libc::c_int
    }
    /* Set Image Color Space */
    if (*p_jp2).enumcs == 16 as libc::c_int as libc::c_uint {
        (*p_image).color_space = OPJ_CLRSPC_SRGB
    } else if (*p_jp2).enumcs == 17 as libc::c_int as libc::c_uint {
        (*p_image).color_space = OPJ_CLRSPC_GRAY
    } else if (*p_jp2).enumcs == 18 as libc::c_int as libc::c_uint {
        (*p_image).color_space = OPJ_CLRSPC_SYCC
    } else if (*p_jp2).enumcs == 24 as libc::c_int as libc::c_uint {
        (*p_image).color_space = OPJ_CLRSPC_EYCC
    } else if (*p_jp2).enumcs == 12 as libc::c_int as libc::c_uint {
        (*p_image).color_space = OPJ_CLRSPC_CMYK
    } else { (*p_image).color_space = OPJ_CLRSPC_UNKNOWN }
    if !(*p_jp2).color.jp2_pclr.is_null() {
        /* Part 1, I.5.3.4: Either both or none : */
        if (*(*p_jp2).color.jp2_pclr).cmap.is_null() {
            opj_jp2_free_pclr(&mut (*p_jp2).color);
        } else if opj_jp2_apply_pclr(p_image, &mut (*p_jp2).color, p_manager)
                      == 0 {
            return 0 as libc::c_int
        }
    }
    /* Apply the color space if needed */
    if !(*p_jp2).color.jp2_cdef.is_null() {
        opj_jp2_apply_cdef(p_image, &mut (*p_jp2).color, p_manager);
    }
    if !(*p_jp2).color.icc_profile_buf.is_null() {
        (*p_image).icc_profile_buf = (*p_jp2).color.icc_profile_buf;
        (*p_image).icc_profile_len = (*p_jp2).color.icc_profile_len;
        (*p_jp2).color.icc_profile_buf = 0 as *mut OPJ_BYTE
    }
    return 1 as libc::c_int;
}
/* ----------------------------------------------------------------------- */
/* JP2 encoder interface                                             */
/* ----------------------------------------------------------------------- */
#[no_mangle]
pub unsafe extern "C" fn opj_jp2_create(mut p_is_decoder: OPJ_BOOL)
 -> *mut opj_jp2_t {
    let mut jp2 =
        opj_calloc(1 as libc::c_int as size_t,
                   ::std::mem::size_of::<opj_jp2_t>() as libc::c_ulong) as
            *mut opj_jp2_t;
    if !jp2.is_null() {
        /* create the J2K codec */
        if p_is_decoder == 0 {
            (*jp2).j2k = opj_j2k_create_compress()
        } else { (*jp2).j2k = opj_j2k_create_decompress() }
        if (*jp2).j2k.is_null() {
            opj_jp2_destroy(jp2);
            return 0 as *mut opj_jp2_t
        }
        /* Color structure */
        (*jp2).color.icc_profile_buf = 0 as *mut OPJ_BYTE;
        (*jp2).color.icc_profile_len = 0 as libc::c_int as OPJ_UINT32;
        (*jp2).color.jp2_cdef = 0 as *mut opj_jp2_cdef_t;
        (*jp2).color.jp2_pclr = 0 as *mut opj_jp2_pclr_t;
        (*jp2).color.jp2_has_colr = 0 as libc::c_int as OPJ_BYTE;
        /* validation list creation */
        (*jp2).m_validation_list = opj_procedure_list_create();
        if (*jp2).m_validation_list.is_null() {
            opj_jp2_destroy(jp2);
            return 0 as *mut opj_jp2_t
        }
        /* execution list creation */
        (*jp2).m_procedure_list = opj_procedure_list_create();
        if (*jp2).m_procedure_list.is_null() {
            opj_jp2_destroy(jp2);
            return 0 as *mut opj_jp2_t
        }
    }
    return jp2;
}
#[no_mangle]
pub unsafe extern "C" fn jp2_dump(mut p_jp2: *mut opj_jp2_t,
                                  mut flag: OPJ_INT32,
                                  mut out_stream: *mut FILE) {
    /* preconditions */
    if !p_jp2.is_null() {
    } else {
        __assert_fail(b"p_jp2 != 00\x00" as *const u8 as *const libc::c_char,
                      b"/opt/openjpeg/src/lib/openjp2/jp2.c\x00" as *const u8
                          as *const libc::c_char,
                      3224 as libc::c_int as libc::c_uint,
                      (*::std::mem::transmute::<&[u8; 46],
                                                &[libc::c_char; 46]>(b"void jp2_dump(opj_jp2_t *, OPJ_INT32, FILE *)\x00")).as_ptr());
    }
    j2k_dump((*p_jp2).j2k, flag, out_stream);
}
#[no_mangle]
pub unsafe extern "C" fn jp2_get_cstr_index(mut p_jp2: *mut opj_jp2_t)
 -> *mut opj_codestream_index_t {
    return j2k_get_cstr_index((*p_jp2).j2k);
}
#[no_mangle]
pub unsafe extern "C" fn jp2_get_cstr_info(mut p_jp2: *mut opj_jp2_t)
 -> *mut opj_codestream_info_v2_t {
    return j2k_get_cstr_info((*p_jp2).j2k);
}
#[no_mangle]
pub unsafe extern "C" fn opj_jp2_set_decoded_resolution_factor(mut p_jp2:
                                                                   *mut opj_jp2_t,
                                                               mut res_factor:
                                                                   OPJ_UINT32,
                                                               mut p_manager:
                                                                   *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    return opj_j2k_set_decoded_resolution_factor((*p_jp2).j2k, res_factor,
                                                 p_manager);
}
/* ----------------------------------------------------------------------- */
#[no_mangle]
pub unsafe extern "C" fn opj_jp2_encoder_set_extra_options(mut p_jp2:
                                                               *mut opj_jp2_t,
                                                           mut p_options:
                                                               *const *const libc::c_char,
                                                           mut p_manager:
                                                               *mut opj_event_mgr_t)
 -> OPJ_BOOL {
    return opj_j2k_encoder_set_extra_options((*p_jp2).j2k, p_options,
                                             p_manager);
}
/* USE_JPIP */
/* JPIP specific */
/* ----------------------------------------------------------------------- */
