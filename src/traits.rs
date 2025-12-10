use crate::error::HGVSError;
use crate::single_variant_response::SingleVariantResponse;
use crate::{GenomeAssembly, HgvsVariant};

pub trait HGVSData {
    fn get_full_validated_hgvs_data(
        &self,
        unvalidated_hgvs: &str,
    ) -> Result<SingleVariantResponse, HGVSError>;

    fn get_abbreviated_validated_hgvs_data(
        &self,
        unvalidated_hgvs: &str,
        genome_assembly: GenomeAssembly,
    ) -> Result<HgvsVariant, HGVSError> {
        let full_data = self.get_full_validated_hgvs_data(unvalidated_hgvs);
        full_data?.abbreviate_response(genome_assembly)
    }
}
