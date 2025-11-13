use phenopackets::schema::v2::core::OntologyClass;

pub(crate) fn get_allele_term(allele_count: usize, is_x: bool) -> OntologyClass {
    if allele_count == 2 {
        OntologyClass {
            id: "GENO:0000136".to_string(),
            label: "homozygous".to_string(),
        }
    } else if is_x {
        OntologyClass {
            id: "GENO:0000134".to_string(),
            label: "hemizygous".to_string(),
        }
    } else {
        OntologyClass {
            id: "GENO:0000135".to_string(),
            label: "heterozygous".to_string(),
        }
    }
}

pub(crate) fn is_hgnc_id(gene: &str) -> bool {
    let split_string = gene.split(':').collect::<Vec<&str>>();
    split_string.first() == Some(&"HGNC")
}
