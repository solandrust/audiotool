use rx::proptest::prelude::*;
use audiotool::bitdepth::*;

fn do_i16_f32_eq_test(i: i16, f: f32) {
    let f_actual = i16_to_f32(i);
    assert_eq!(f_actual, f);
    let i_actual = f32_to_i16(f);
    assert_eq!(i_actual, i);
}

fn do_i24_f32_eq_test(i: i32, f: f32) {
    let f_actual = i24_to_f32(i);
    assert_eq!(f_actual, f);
    let i_actual = f32_to_i24(f);
    assert_eq!(i_actual, i);
}

fn do_i16_i24_via_f32_eq_test(i1: i16, i2: i32) {
    let f = i16_to_f32(i1);
    let i24_actual = f32_to_i24(f);
    assert_eq!(i24_actual, i2);
    let f = i24_to_f32(i2);
    let i16_actual = f32_to_i16(f);
    assert_eq!(i16_actual, i1);
}

#[test]
fn eq_tests() {
    do_i16_f32_eq_test(i16::MAX, 1.0);
    do_i16_f32_eq_test(i16::MIN, -1.0);
    do_i24_f32_eq_test(I24_MAX, 1.0);
    do_i24_f32_eq_test(I24_MIN, -1.0);
    do_i16_i24_via_f32_eq_test(i16::MAX, I24_MAX);
    do_i16_i24_via_f32_eq_test(i16::MIN, I24_MIN);
    do_i16_i24_via_f32_eq_test(0, 0);
}

fn do_i16_to_f32_roundtrip(i1: i16) {
    let f = i16_to_f32(i1);
    let i2 = f32_to_i16(f);
    assert_eq!(i1, i2);
}

fn do_i24_to_f32_roundtrip(i1: i32) {
    let f = i24_to_f32(i1);
    let i2 = f32_to_i24(f);
    assert_eq!(i1, i2);
}

fn do_i16_to_i24_via_f32_roundtrip(i1: i16) {
    let f1 = i16_to_f32(i1);
    let i2 = f32_to_i24(f1);
    let f2 = i24_to_f32(i2);
    let i3 = f32_to_i16(f2);
    assert_eq!(i1, i3);
}

#[test]
fn i16_to_f32_roundtrips() {
    do_i16_to_f32_roundtrip(0);
    do_i16_to_f32_roundtrip(i16::MIN);
    do_i16_to_f32_roundtrip(i16::MAX);
}

#[test]
fn i24_to_f32_roundtrips() {
    do_i24_to_f32_roundtrip(0);
    do_i24_to_f32_roundtrip(I24_MIN);
    do_i24_to_f32_roundtrip(I24_MAX);
}

#[test]
fn i16_to_i24_via_f32_roundtrips() {
    do_i16_to_i24_via_f32_roundtrip(0);
    do_i16_to_i24_via_f32_roundtrip(i16::MIN);
    do_i16_to_i24_via_f32_roundtrip(i16::MAX);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256 * 256))]

    #[test]
    fn i16_to_f32_roundtrip(
        i1 in any::<i16>()
    ) {
        do_i16_to_f32_roundtrip(i1);
    }

    #[test]
    fn i24_to_f32_roundtrip(
        i1 in I24_MIN..=I24_MAX
    ) {
        do_i24_to_f32_roundtrip(i1);
    }

    #[test]
    fn i16_to_i24_via_f32_roundtrip(
        i1 in any::<i16>()
    ) {
        do_i16_to_i24_via_f32_roundtrip(i1);
    }

}
