enum Sides {
    One,
    Two,
}

enum ApplyTo {
    SingleMesh,
    AllMeshes,
}

struct Span {
    pub line: u32
}

struct CreateMeshBuilder {
    pub span: Span,
}