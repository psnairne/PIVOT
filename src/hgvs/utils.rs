pub fn is_c_hgvs(allele: &str) -> bool {
    allele.starts_with("c.")
}

pub fn is_n_hgvs(allele: &str) -> bool {
    allele.starts_with("n.")
}

pub fn is_m_hgvs(allele: &str) -> bool {
    allele.starts_with("m.")
}
