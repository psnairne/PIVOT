use std::fmt::Display;

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
