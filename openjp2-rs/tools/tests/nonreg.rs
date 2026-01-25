/// Integration tests for non-regression tests
/// These tests verify compression and decompression functionality against known good baselines
use std::collections::BTreeSet;

use openjp2_tools::{
  args,
  cli::{run_compare_dump_files, run_dump},
  skip_without_test_data,
  testing::*,
};
#[cfg(feature = "parallel-tests")]
use rayon::prelude::*;

#[test]
fn test_nonreg() {
  skip_without_test_data!();

  let tests = parse_test_commands();
  let md5_refs = MD5References::load_md5_references();

  #[cfg(feature = "parallel-tests")]
  {
    tests.into_par_iter().for_each(|test| {
      test.run_nonreg(&md5_refs);
    });
  }

  #[cfg(not(feature = "parallel-tests"))]
  {
    for test in tests {
      test.run_nonreg(&md5_refs);
    }
  }
}

#[test]
fn test_nonreg_dump() {
  skip_without_test_data!();

  let input_nr_dir = get_input_nr_dir();
  let baseline_nr_dir = get_baseline_nr_dir().to_string_lossy().to_string();
  let temp_dir = get_temp_dir();

  // technically opj_dump should simply parse these one, since syntax is ok.
  const BLACKLIST_JPEG2000_TMP: &[&str] = &[
    "2539.pdf.SIGFPE.706.1712.jp2",
    "0290cb77c5df21828fa74cf2ab2c84d8.SIGFPE.d25.31.jp2",
    "26ccf3651020967f7778238ef5af08af.SIGFPE.d25.527.jp2",
    "4035.pdf.SIGSEGV.d8b.3375.jp2",
    "3635.pdf.asan.77.2930.jp2",
    "issue165.jp2",
    //edf_c2_1103421.jp2
    "edf_c2_1178956.jp2",
    "edf_c2_1000290.jp2",
    //edf_c2_1000691.jp2 // ok
    "edf_c2_1377017.jp2",
    "edf_c2_1002767.jp2",
    "edf_c2_10025.jp2",
    "edf_c2_1000234.jp2",
    "edf_c2_225881.jp2",
    "edf_c2_1000671.jp2",
    //edf_c2_1013627.jp2 // weird box, but kdu_jp2info ok
    "edf_c2_1015644.jp2",
    "edf_c2_101463.jp2",
    "edf_c2_1674177.jp2",
    "edf_c2_1673169.jp2",
    "issue418.jp2",
    "issue429.jp2",
    "issue427-null-image-size.jp2",
    "issue427-illegal-tile-offset.jp2",
    "issue495.jp2",
    "issue820.jp2",
  ];

  // Define a list of file which should be gracefully rejected:
  const BLACKLIST_JPEG2000: &[&str] = &[
    "broken1.jp2",
    "broken2.jp2",
    "broken3.jp2",
    "broken4.jp2",
    "edf_c2_20.jp2", //may look ok as per kdu_jp2info, but inspection it reveals that the transformation value is out of range
    "gdal_fuzzer_assert_in_opj_j2k_read_SQcd_SQcc.patch.jp2",
    "gdal_fuzzer_check_comp_dx_dy.jp2",
    "gdal_fuzzer_check_number_of_tiles.jp2",
    "gdal_fuzzer_unchecked_numresolutions.jp2",
    "mem-b2ace68c-1381.jp2",
    "1851.pdf.SIGSEGV.ce9.948.jp2",
    "1888.pdf.asan.35.988.jp2",
    "issue362-2863.jp2", //kdu_jp2info ok
    "issue362-2866.jp2",
    "issue362-2894.jp2",
    "issue400.jp2", //kdu_jp2info ok
    "issue364-38.jp2",
    "issue364-903.jp2",                                    //kdu_jp2info ok
    "issue393.jp2",                                        //kdu_jp2info ok
    "issue408.jp2",                                        //kdu_jp2info ok
    "issue420.jp2",                                        //kdu_jp2info ok
    "27ac957758a35d00d6765a0c86350d9c.SIGFPE.d25.537.jpc", //kdu_jp2info crash
    "3672da2f1f67bbecad27d7181b4e9d7c.SIGFPE.d25.805.jpc", //kdu_jp2info crash
    "issue475.jp2",                                        //kdu_jp2info ok
    "issue413.jp2",                                        //kdu_jp2info ok
    "issue823.jp2",                                        //kdu_jp2info ok
    "issue826.jp2", //inspection reveals that the transformation value is out of range
    "oss-fuzz2785.jp2", //inspection reveals that the transformation value is out of range
    "issue1438.j2k",
  ];
  let blacklist: BTreeSet<String> = BLACKLIST_JPEG2000
    .iter()
    .chain(BLACKLIST_JPEG2000_TMP.iter())
    .map(|s| s.to_string())
    .collect();

  let dump_extensions: BTreeSet<String> = ["j2k", "j2c", "jp2", "jpc", "jph", "jhc"]
    .iter()
    .map(|s| s.to_string())
    .collect();

  // Recursively process the input non-regression directory
  // adding all files with dump-able extensions to test_files set
  let mut test_files = BTreeSet::new();
  find_files_with_extensions(&input_nr_dir, &dump_extensions, &mut test_files);

  for path in test_files {
    let input = path.to_string_lossy().to_string();
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap();
    let filename_we = path.file_stem().and_then(|s| s.to_str()).unwrap();
    let output = temp_dir
      .join(format!("{}-dump.txt", filename))
      .to_string_lossy()
      .to_string();
    println!("NR-{}-dump", filename);

    let bad_jpeg2000 = blacklist.contains(filename);

    let result = run_dump(args!["-i", input, "-o", &output, "-v"]);
    if bad_jpeg2000 {
      assert!(
        result.is_err(),
        "Dumping blacklisted non-regression test file {} unexpectedly succeeded",
        filename
      );
      continue;
    }
    assert!(
      result.is_ok(),
      "Dumping non-regression test file {} failed: {:?}",
      filename,
      result.err()
    );

    println!("NR-{}-compare_dump2base", filename);
    run_compare_dump_files(args![
      "-b",
      format!("{baseline_nr_dir}/opj_v2_{filename_we}.txt",),
      "-t",
      &output,
    ])
    .expect("Comparing dump to baseline failed");
  }
}
