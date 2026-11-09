# The C-project build is the upstream-compatibility configuration: write
# the OpenJPEG product/version string into codestream comments so the
# test-suite dumps match the upstream baselines byte-for-byte.
cargo_build(NAME ${OPENJPEG_LIBRARY_NAME} FEATURES openjpeg-version-string)
