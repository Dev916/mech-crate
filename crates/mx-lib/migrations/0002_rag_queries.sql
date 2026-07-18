CREATE TABLE IF NOT EXISTS rag_queries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    query TEXT NOT NULL,
    tool TEXT NOT NULL,
    category TEXT,
    language TEXT,
    top_score DOUBLE PRECISION,
    mode TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS rag_queries_created_idx ON rag_queries (created_at);
