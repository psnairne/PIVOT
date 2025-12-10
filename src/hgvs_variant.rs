use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HgvsVariant {
    /// Genome build, e.g., hg38
    assembly: String,
    /// Chromosome, e.g., "17"
    chr: String,
    /// Position on the chromosome
    position: u32,
    /// Reference allele
    ref_allele: String,
    /// Alternate allele
    alt_allele: String,
    /// Gene symbol, e.g., FBN1
    symbol: String,
    /// HUGO Gene Nomenclature Committee identifier, e.g., HGNC:3603
    hgnc_id: String,
    /// Transcript, e.g., NM_000138.5
    transcript: String,
    /// HGVS Nomenclature, e.g., c.8242G>T
    allele: String,
    /// Coding or Non-coding (RNA) gene HGVS nomenclature, e.g., NM_000138.5:c.8242G>T or NR_002196.1:n.601G>T
    transcript_hgvs: String,
    /// Genomic HGVS nomenclature, e.g., NC_000015.10:g.48411364C>A
    g_hgvs: String,
    /// Protein level HGVS, if available
    p_hgvs: Option<String>,
}

impl HgvsVariant {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        assembly: impl Into<String>,
        chr: impl Into<String>,
        position: u32,
        ref_allele: impl Into<String>,
        alt_allele: impl Into<String>,
        symbol: impl Into<String>,
        hgnc_id: impl Into<String>,
        transcript: impl Into<String>,
        allele: impl Into<String>,
        transcript_hgvs: impl Into<String>,
        g_hgvs: impl Into<String>,
        p_hgvs: Option<impl Into<String>>,
    ) -> Self {
        HgvsVariant {
            assembly: assembly.into(),
            chr: chr.into(),
            position,
            ref_allele: ref_allele.into(),
            alt_allele: alt_allele.into(),
            symbol: symbol.into(),
            hgnc_id: hgnc_id.into(),
            transcript: transcript.into(),
            allele: allele.into(),
            transcript_hgvs: transcript_hgvs.into(),
            g_hgvs: g_hgvs.into(),
            p_hgvs: p_hgvs.map(|s| s.into()),
        }
    }

    pub fn assembly(&self) -> &str {
        self.assembly.as_ref()
    }

    pub fn chr(&self) -> &str {
        self.chr.as_ref()
    }

    pub fn position(&self) -> u32 {
        self.position
    }

    pub fn ref_allele(&self) -> &str {
        self.ref_allele.as_ref()
    }

    pub fn alt_allele(&self) -> &str {
        self.alt_allele.as_ref()
    }

    pub fn hgnc_id(&self) -> &str {
        self.hgnc_id.as_ref()
    }

    pub fn gene_symbol(&self) -> &str {
        &self.symbol
    }

    pub fn transcript(&self) -> &str {
        self.transcript.as_ref()
    }

    pub fn allele(&self) -> &str {
        self.allele.as_ref()
    }

    pub fn transcript_hgvs(&self) -> &str {
        self.transcript_hgvs.as_ref()
    }

    pub fn g_hgvs(&self) -> &str {
        self.g_hgvs.as_ref()
    }

    pub fn p_hgvs(&self) -> Option<String> {
        self.p_hgvs.as_ref().map(|phgvs| phgvs.to_string())
    }

    pub fn is_x_chromosomal(&self) -> bool {
        self.chr.contains("X")
    }

    pub fn is_y_chromosomal(&self) -> bool {
        self.chr.contains("Y")
    }
}
