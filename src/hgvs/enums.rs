#![allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub enum ChromosomalSex {
    X,
    XX,
    XY,
    XXY,
    XYY,
    XXX,
    Unknown,
}

#[derive(Debug)]
pub enum AlleleCount {
    Single,
    Double,
}
