import postgres from 'postgres';

const connectionString = import.meta.env.DATABASE_URL || 'postgres://localhost:5432/{{SERVICE_SLUG}}';

/**
 * PostgreSQL client using postgres.js
 * Supports both tagged template literals and parameterized queries
 */
export const db = postgres(connectionString, {
  max: 10,
  idle_timeout: 20,
  connect_timeout: 10,
  prepare: true,
  transform: {
    undefined: null,
  },
});

/**
 * Execute a raw SQL query
 */
export async function query<T = unknown>(sql: string, params: unknown[] = []): Promise<T[]> {
  return db.unsafe(sql, params) as unknown as T[];
}

/**
 * Transaction helper
 */
export async function transaction<T>(
  fn: (sql: typeof db) => Promise<T>
): Promise<T> {
  return db.begin(async (tx) => fn(tx));
}

/**
 * Check database connection
 */
export async function checkConnection(): Promise<boolean> {
  try {
    await db`SELECT 1`;
    return true;
  } catch {
    return false;
  }
}

/**
 * Close database connection pool
 */
export async function closeConnection(): Promise<void> {
  await db.end();
}
