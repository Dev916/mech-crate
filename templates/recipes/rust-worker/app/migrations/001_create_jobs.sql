-- Jobs table for persistent job storage
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS jobs (
    id UUID PRIMARY KEY,
    job_type VARCHAR(255) NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}',
    priority INTEGER NOT NULL DEFAULT 5,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error TEXT,
    result JSONB,
    
    CONSTRAINT valid_priority CHECK (priority >= 0 AND priority <= 10),
    CONSTRAINT valid_status CHECK (status IN ('pending', 'running', 'completed', 'failed', 'retrying', 'cancelled'))
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
CREATE INDEX IF NOT EXISTS idx_jobs_status_priority ON jobs(status, priority DESC, created_at ASC);
CREATE INDEX IF NOT EXISTS idx_jobs_type_status ON jobs(job_type, status);
CREATE INDEX IF NOT EXISTS idx_jobs_created_at ON jobs(created_at);

-- Composite index for pending job lookup
CREATE INDEX IF NOT EXISTS idx_jobs_pending ON jobs(status, priority DESC, created_at ASC) 
WHERE status IN ('pending', 'retrying');

-- Job results table for storing large outputs
CREATE TABLE IF NOT EXISTS job_results (
    job_id UUID PRIMARY KEY REFERENCES jobs(id) ON DELETE CASCADE,
    output JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Job metrics table for analytics
CREATE TABLE IF NOT EXISTS job_metrics (
    id SERIAL PRIMARY KEY,
    job_id UUID NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    metric_name VARCHAR(100) NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_job_metrics_job_id ON job_metrics(job_id);
CREATE INDEX IF NOT EXISTS idx_job_metrics_name ON job_metrics(metric_name, recorded_at);
