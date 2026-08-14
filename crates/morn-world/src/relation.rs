//! Relations between operational objects.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{ObjectId, RelationId, RelationTypeId, WorkspaceId};
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationType {
    pub id: RelationTypeId,
    pub name: String,
    pub workspace_id: WorkspaceId,
    pub from_type: String,
    pub to_type: String,
}

impl RelationType {
    pub fn new(
        name: impl Into<String>,
        workspace_id: WorkspaceId,
        from_type: impl Into<String>,
        to_type: impl Into<String>,
    ) -> Self {
        Self {
            id: RelationTypeId::generate_with("relt"),
            name: name.into(),
            workspace_id,
            from_type: from_type.into(),
            to_type: to_type.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relation {
    pub id: RelationId,
    pub relation_type_id: RelationTypeId,
    pub from_object: ObjectId,
    pub to_object: ObjectId,
    pub workspace_id: WorkspaceId,
    pub created_at: Timestamp,
}

impl Relation {
    pub fn new(
        relation_type_id: RelationTypeId,
        from_object: ObjectId,
        to_object: ObjectId,
        workspace_id: WorkspaceId,
    ) -> Self {
        Self {
            id: RelationId::generate_with("rel"),
            relation_type_id,
            from_object,
            to_object,
            workspace_id,
            created_at: Timestamp::now(),
        }
    }
}
