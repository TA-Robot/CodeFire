use super::project::RemoteAuthority;
use super::publication::{AuthorityWrite, PublicationRequest};
use serde_json::{json, Value};

pub(crate) struct UploadPublicationIdentity<'a> {
    pub(crate) stable_project_id: &'a str,
    pub(crate) remote_branch: &'a str,
    pub(crate) local_branch: &'a str,
    pub(crate) head: &'a str,
    pub(crate) object_set_digest: &'a str,
}

pub(crate) fn upload_publication(
    actor: &str,
    idempotency_key: Option<String>,
    identity: &UploadPublicationIdentity<'_>,
) -> PublicationRequest {
    PublicationRequest {
        operation: "remote_upload".to_string(),
        actor: actor.to_string(),
        idempotency_key,
        object_roots: vec![identity.head.to_string()],
        semantic: json!({
            "stable_project_id": identity.stable_project_id,
            "remote_branch": identity.remote_branch,
            "local_branch": identity.local_branch,
            "head": identity.head,
            "object_set_digest": identity.object_set_digest,
        }),
    }
}

pub(crate) fn upload_result(branch: &str, head: &str) -> Value {
    json!({"branch": branch, "head": head})
}

pub(crate) struct UploadPostimages<'a> {
    pub(crate) remote_branch: &'a str,
    pub(crate) generation_before: Option<Value>,
    pub(crate) generation_after: Value,
    pub(crate) branch_before: Option<Value>,
    pub(crate) branch_after: Value,
}

pub(crate) fn upload_writes(postimages: UploadPostimages<'_>) -> Vec<AuthorityWrite> {
    vec![
        AuthorityWrite {
            authority: RemoteAuthority::Generation,
            before: postimages.generation_before,
            after: postimages.generation_after,
        },
        AuthorityWrite {
            authority: RemoteAuthority::Branch(postimages.remote_branch.to_string()),
            before: postimages.branch_before,
            after: postimages.branch_after,
        },
    ]
}

pub(crate) struct MergePublicationIdentity<'a> {
    pub(crate) source_project_id: &'a str,
    pub(crate) target_project_id: &'a str,
    pub(crate) source_branch: &'a str,
    pub(crate) target_branch: &'a str,
    pub(crate) source_head: &'a str,
    pub(crate) target_head_at_request: &'a str,
    pub(crate) source_object_set_digest: &'a str,
}

pub(crate) fn request_merge_publication(
    actor: &str,
    idempotency_key: Option<String>,
    identity: &MergePublicationIdentity<'_>,
) -> PublicationRequest {
    PublicationRequest {
        operation: "remote_request_merge".to_string(),
        actor: actor.to_string(),
        idempotency_key,
        object_roots: vec![identity.source_head.to_string()],
        semantic: json!({
            "source_project_id": identity.source_project_id,
            "target_project_id": identity.target_project_id,
            "source_branch": identity.source_branch,
            "target_branch": identity.target_branch,
            "source_head": identity.source_head,
            "target_head_at_request": identity.target_head_at_request,
            "source_object_set_digest": identity.source_object_set_digest,
        }),
    }
}

pub(crate) fn request_merge_result(mr_id: &str, source_head: &str, target_head: &str) -> Value {
    json!({"id": mr_id, "source_head": source_head, "target_head": target_head})
}

pub(crate) struct ReviewPublicationIdentity<'a> {
    pub(crate) stable_project_id: &'a str,
    pub(crate) merge_request_id: &'a str,
    pub(crate) reviewer: &'a str,
    pub(crate) decision: &'a str,
    pub(crate) comment: &'a str,
    pub(crate) subject_digest: &'a str,
    pub(crate) source_head: &'a str,
}

pub(crate) fn review_publication(
    actor: &str,
    idempotency_key: Option<String>,
    identity: &ReviewPublicationIdentity<'_>,
) -> PublicationRequest {
    PublicationRequest {
        operation: "remote_request_review".to_string(),
        actor: actor.to_string(),
        idempotency_key,
        object_roots: vec![identity.source_head.to_string()],
        semantic: json!({
            "stable_project_id": identity.stable_project_id,
            "merge_request_id": identity.merge_request_id,
            "reviewer": identity.reviewer,
            "decision": identity.decision,
            "comment": identity.comment,
            "subject_digest": identity.subject_digest,
        }),
    }
}

pub(crate) fn review_result(mr_id: &str, reviewer: &str, decision: &str) -> Value {
    json!({"id": mr_id, "reviewer": reviewer, "decision": decision})
}

pub(crate) struct ApplyPublicationIdentity<'a> {
    pub(crate) stable_project_id: &'a str,
    pub(crate) merge_request_id: &'a str,
    pub(crate) subject_digest: &'a str,
    pub(crate) target_branch: &'a str,
    pub(crate) source_head: &'a str,
    pub(crate) target_head_at_request: &'a str,
}

pub(crate) fn apply_publication(
    actor: &str,
    idempotency_key: Option<String>,
    identity: &ApplyPublicationIdentity<'_>,
) -> PublicationRequest {
    PublicationRequest {
        operation: "remote_request_apply".to_string(),
        actor: actor.to_string(),
        idempotency_key,
        object_roots: if identity.source_head == identity.target_head_at_request {
            vec![identity.source_head.to_string()]
        } else {
            vec![
                identity.source_head.to_string(),
                identity.target_head_at_request.to_string(),
            ]
        },
        semantic: json!({
            "stable_project_id": identity.stable_project_id,
            "merge_request_id": identity.merge_request_id,
            "subject_digest": identity.subject_digest,
            "target_branch": identity.target_branch,
            "source_head": identity.source_head,
            "target_head_at_request": identity.target_head_at_request,
        }),
    }
}

pub(crate) fn apply_result(mr_id: &str, target_branch: &str, head: &str) -> Value {
    json!({"id": mr_id, "target_branch": target_branch, "head": head})
}

pub(crate) fn review_write(mr_id: &str, before: Value, after: Value) -> AuthorityWrite {
    AuthorityWrite {
        authority: RemoteAuthority::MergeRequest(mr_id.to_string()),
        before: Some(before),
        after,
    }
}

pub(crate) struct ApplyPostimages<'a> {
    pub(crate) target_branch: &'a str,
    pub(crate) merge_request_id: &'a str,
    pub(crate) generation_before: Option<Value>,
    pub(crate) generation_after: Value,
    pub(crate) branch_before: Value,
    pub(crate) branch_after: Value,
    pub(crate) merge_request_before: Value,
    pub(crate) merge_request_after: Value,
}

pub(crate) fn apply_writes(postimages: ApplyPostimages<'_>) -> Vec<AuthorityWrite> {
    vec![
        AuthorityWrite {
            authority: RemoteAuthority::Generation,
            before: postimages.generation_before,
            after: postimages.generation_after,
        },
        AuthorityWrite {
            authority: RemoteAuthority::Branch(postimages.target_branch.to_string()),
            before: Some(postimages.branch_before),
            after: postimages.branch_after,
        },
        AuthorityWrite {
            authority: RemoteAuthority::MergeRequest(postimages.merge_request_id.to_string()),
            before: Some(postimages.merge_request_before),
            after: postimages.merge_request_after,
        },
    ]
}
