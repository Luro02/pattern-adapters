#![feature(pattern)]
#![no_main]

use libfuzzer_sys::fuzz_target;
use pattern_adapters::adapters::PatternExt;

mod utils;

fuzz_target!(|data: (&str, &str)| {
    let (haystack, needle) = data;

    if !haystack.is_empty() && !needle.is_empty() {
        utils::assert_integrity(haystack, needle);
    }
});
/*
  - conda-forge/linux-64::libopenblas==0.3.15=pthreads_h8fe5266_1
  - conda-forge/linux-64::libblas==3.9.0=9_openblas
  - conda-forge/linux-64::libcblas==3.9.0=9_openblas
  - conda-forge/linux-64::libgfortran-ng==7.5.0=h14aa051_19
  - defaults/linux-64::intel-openmp==2020.2=254
  - defaults/linux-64::cairo==1.14.12=h8948797_3
  - defaults/linux-64::libopencv==3.4.2=hb342d67_1
  - defaults/linux-64::mkl_random==1.1.1=py37h0573a6f_0
  - defaults/linux-64::numpy==1.17.0=py37h7e9f1db_0
  - defaults/linux-64::pycairo==1.19.1=py37h2a1e443_0
  - defaults/linux-64::opencv==3.4.2=py37h6fd60c2_1
  - defaults/linux-64::mkl_fft==1.2.0=py37h23d657b_0
  - defaults/linux-64::py-opencv==3.4.2=py37hb342d67_1
  - defaults/linux-64::harfbuzz==1.8.8=hffaf4a1_0
  - conda-forge/linux-64::liblapack==3.9.0=9_openblas
failed with initial frozen solve. Retrying with flexible solve.
Solving environment: failed with repodata from current_repodata.json, will retry with next repodata source.
Collecting package metadata (repodata.json): done
Solving environment: -
The environment is inconsistent, please check the package plan carefully
The following packages are causing the inconsistency:

  - conda-forge/linux-64::libopenblas==0.3.15=pthreads_h8fe5266_1
  - conda-forge/linux-64::libblas==3.9.0=9_openblas
  - conda-forge/linux-64::libcblas==3.9.0=9_openblas
  - conda-forge/linux-64::libgfortran-ng==7.5.0=h14aa051_19
  - defaults/linux-64::intel-openmp==2020.2=254
  - defaults/linux-64::mkl_random==1.1.1=py37h0573a6f_0
  - defaults/linux-64::numpy==1.17.0=py37h7e9f1db_0
  - defaults/linux-64::opencv==3.4.2=py37h6fd60c2_1
  - defaults/linux-64::mkl_fft==1.2.0=py37h23d657b_0
  - defaults/linux-64::py-opencv==3.4.2=py37hb342d67_1
  - conda-forge/linux-64::liblapack==3.9.0=9_openblas
*/
