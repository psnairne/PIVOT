use crate::HGVSError;

pub fn is_c_hgvs(allele: &str) -> bool {
    allele.starts_with("c.")
}

pub fn is_n_hgvs(allele: &str) -> bool {
    allele.starts_with("n.")
}

pub(crate) fn get_transcript_and_allele(unvalidated_hgvs: &str) -> Result<(&str, &str), HGVSError> {
    let split_hgvs = unvalidated_hgvs.split(':').collect::<Vec<&str>>();
    let colon_count = split_hgvs.len() - 1;
    if colon_count != 1 {
        Err(HGVSError::HgvsFormatNotAccepted {
            hgvs: unvalidated_hgvs.to_string(),
            problem: "There must be exactly one colon in a HGVS string.".to_string(),
        })
    } else {
        let transcript = split_hgvs[0];
        let allele = split_hgvs[1];
        Ok((transcript, allele))
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::get_transcript_and_allele;
    use rstest::rstest;

    #[rstest]
    fn test_get_transcript_and_allele() {
        let unvalidated_hgvs = "NM_001173464.1:c.2860C>T";
        let (transcript, allele) = get_transcript_and_allele(unvalidated_hgvs).unwrap();
        assert_eq!(transcript, "NM_001173464.1");
        assert_eq!(allele, "c.2860C>T");
    }

    #[rstest]
    fn test_get_transcript_and_allele_no_colon_err() {
        let invalid_hgvs = "NM_001173464.1*c.2860C>T";
        assert!(get_transcript_and_allele(invalid_hgvs).is_err());
    }

    #[rstest]
    fn test_get_transcript_and_allele_multiple_colons_err() {
        let invalid_hgvs = "NM_001173464.1:c:2860C>T";
        assert!(get_transcript_and_allele(invalid_hgvs).is_err());
    }
}
