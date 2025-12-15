#![allow(clippy::upper_case_acronyms)]

use crate::hgvs::HGVSError;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum ChromosomalSex {
    X,
    XX,
    XXX,
    XXY,
    XYY,
    XY,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlleleCount {
    Single,
    Double,
}

impl TryFrom<u8> for AlleleCount {
    type Error = HGVSError;

    fn try_from(allele_count: u8) -> Result<Self, Self::Error> {
        if allele_count == 1 {
            Ok(AlleleCount::Single)
        } else if allele_count == 2 {
            Ok(AlleleCount::Double)
        } else {
            Err(HGVSError::InvalidAlleleCount {
                found: allele_count,
            })
        }
    }
}

#[derive(Debug)]
pub enum GenomeAssembly {
    Hg38,
    Hg19,
}

impl Display for GenomeAssembly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            GenomeAssembly::Hg38 => "hg38".to_string(),
            GenomeAssembly::Hg19 => "hg19".to_string(),
        };
        write!(f, "{}", str)
    }
}
