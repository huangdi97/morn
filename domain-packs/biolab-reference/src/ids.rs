//! BioLab reference domain IDs (domain-owned, not Core).

use morn_kernel::ids::Id;

pub use morn_kernel::ids::Id as DomainId;

macro_rules! domain_id {
    ($tag:ident, $alias:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        pub struct $tag;
        pub type $alias = Id<$tag>;
    };
}

domain_id!(DatasetTag, DatasetId);
domain_id!(SampleTag, SampleId);
domain_id!(AnalysisRunTag, AnalysisRunId);
domain_id!(QCResultTag, QCResultId);
domain_id!(ScientificClaimTag, ScientificClaimId);
