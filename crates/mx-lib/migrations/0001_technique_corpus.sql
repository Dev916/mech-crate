CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE IF NOT EXISTS technique_docs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    path TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'other',
    languages TEXT[] NOT NULL DEFAULT '{}',
    complexity TEXT NOT NULL DEFAULT 'intermediate',
    use_cases TEXT[] NOT NULL DEFAULT '{}',
    summary TEXT,
    sha256 TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS technique_chunks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    doc_id UUID NOT NULL REFERENCES technique_docs(id) ON DELETE CASCADE,
    heading_path TEXT NOT NULL DEFAULT '',
    content TEXT NOT NULL,
    embedding vector(1536),
    embedding_model TEXT NOT NULL,
    content_sha256 TEXT UNIQUE NOT NULL,
    category TEXT NOT NULL DEFAULT 'other',
    languages TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS technique_chunks_embedding_hnsw
    ON technique_chunks USING hnsw (embedding vector_cosine_ops);
CREATE INDEX IF NOT EXISTS technique_chunks_content_trgm
    ON technique_chunks USING gin (content gin_trgm_ops);
CREATE INDEX IF NOT EXISTS technique_chunks_category_idx ON technique_chunks (category);
