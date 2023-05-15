/*
[dependencies]
test-case = "3.1.0"
*/
#![feature(custom_test_frameworks)]
use test_case;
#[test_case::test_case(2)]
#[test_case::test_case(3)]
fn internal_tested_function1(value: i32) -> i32 {
    assert!(1 == 1);
    if value == 3 {
        0
    } else {
        value * 2
    }
}
