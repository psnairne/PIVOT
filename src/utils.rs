use regex::Regex;

pub(crate) fn is_hgnc_id(gene: &str) -> bool {
    let hgnc_id_regex = Regex::new(r"^HGNC:\d+$").unwrap();
    hgnc_id_regex.is_match(gene)
}

#[cfg(test)]
mod tests {
    use crate::utils::is_hgnc_id;
    use rstest::rstest;

    #[rstest]
    fn test_is_hgnc_id() {
        assert!(is_hgnc_id("HGNC:1234"));
        assert!(!is_hgnc_id("CLOCK"));
    }
}
