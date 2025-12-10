use crate::utils::is_hgnc_id;

#[derive(Clone)]
pub enum GeneQuery<'a> {
    Symbol(&'a str),
    HgncId(&'a str),
}

impl<'a> From<&'a str> for GeneQuery<'a> {
    fn from(gene: &'a str) -> Self {
        if is_hgnc_id(gene) {
            GeneQuery::HgncId(gene)
        } else {
            GeneQuery::Symbol(gene)
        }
    }
}

impl<'a> GeneQuery<'a> {
    pub fn inner(&self) -> &'a str {
        match self {
            GeneQuery::Symbol(s) => s,
            GeneQuery::HgncId(s) => s,
        }
    }
}
