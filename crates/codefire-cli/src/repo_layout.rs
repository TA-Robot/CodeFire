#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LayoutRequirement {
    Required,
    AutoCreate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LayoutDir {
    pub(crate) relative: &'static str,
    pub(crate) requirement: LayoutRequirement,
}

pub(crate) const REPO_DIRS: &[LayoutDir] = &[
    LayoutDir {
        relative: "objects",
        requirement: LayoutRequirement::Required,
    },
    LayoutDir {
        relative: "branches",
        requirement: LayoutRequirement::Required,
    },
    LayoutDir {
        relative: "opened",
        requirement: LayoutRequirement::Required,
    },
    LayoutDir {
        relative: "active",
        requirement: LayoutRequirement::Required,
    },
    LayoutDir {
        relative: "cache",
        requirement: LayoutRequirement::AutoCreate,
    },
    LayoutDir {
        relative: "locks",
        requirement: LayoutRequirement::AutoCreate,
    },
    LayoutDir {
        relative: "remotes",
        requirement: LayoutRequirement::AutoCreate,
    },
    LayoutDir {
        relative: "idempotency",
        requirement: LayoutRequirement::AutoCreate,
    },
];

pub(crate) fn object_subdir_requirement(subdir: &str) -> LayoutRequirement {
    match subdir {
        "artifact_refs" | "evidence" => LayoutRequirement::AutoCreate,
        _ => LayoutRequirement::Required,
    }
}
