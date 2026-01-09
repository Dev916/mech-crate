import Redis from 'ioredis';

const redisUrl = import.meta.env.REDIS_URL || 'redis://localhost:6379';

/**
 * Redis client instance
 */
export const redis = new Redis(redisUrl, {
  maxRetriesPerRequest: 3,
  retryStrategy(times) {
    const delay = Math.min(times * 50, 2000);
    return delay;
  },
  lazyConnect: true,
});

// Error handling
redis.on('error', (err) => {
  console.error('[Redis] Connection error:', err.message);
});

redis.on('connect', () => {
  console.log('[Redis] Connected');
});

/**
 * Cache operations with type safety
 */
export const cache = {
  /**
   * Get cached value
   */
  async get<T>(key: string): Promise<T | null> {
    const value = await redis.get(key);
    if (!value) return null;
    try {
      return JSON.parse(value) as T;
    } catch {
      return value as unknown as T;
    }
  },

  /**
   * Set cached value with optional TTL (in seconds)
   */
  async set<T>(key: string, value: T, ttl?: number): Promise<void> {
    const serialized = typeof value === 'string' ? value : JSON.stringify(value);
    if (ttl) {
      await redis.setex(key, ttl, serialized);
    } else {
      await redis.set(key, serialized);
    }
  },

  /**
   * Delete cached value
   */
  async del(key: string): Promise<void> {
    await redis.del(key);
  },

  /**
   * Delete keys by pattern
   */
  async delPattern(pattern: string): Promise<void> {
    const keys = await redis.keys(pattern);
    if (keys.length > 0) {
      await redis.del(...keys);
    }
  },

  /**
   * Check if key exists
   */
  async exists(key: string): Promise<boolean> {
    const result = await redis.exists(key);
    return result === 1;
  },

  /**
   * Get or set with callback
   */
  async getOrSet<T>(key: string, ttl: number, fn: () => Promise<T>): Promise<T> {
    const cached = await this.get<T>(key);
    if (cached !== null) return cached;
    
    const value = await fn();
    await this.set(key, value, ttl);
    return value;
  },

  /**
   * Increment counter
   */
  async incr(key: string): Promise<number> {
    return redis.incr(key);
  },

  /**
   * Decrement counter
   */
  async decr(key: string): Promise<number> {
    return redis.decr(key);
  },
};

/**
 * Pub/Sub operations
 */
export const pubsub = {
  /**
   * Publish message to channel
   */
  async publish(channel: string, message: unknown): Promise<void> {
    await redis.publish(channel, JSON.stringify(message));
  },

  /**
   * Subscribe to channel
   */
  subscribe(channel: string, callback: (message: unknown) => void): () => void {
    const subscriber = redis.duplicate();
    subscriber.subscribe(channel);
    subscriber.on('message', (ch, msg) => {
      if (ch === channel) {
        try {
          callback(JSON.parse(msg));
        } catch {
          callback(msg);
        }
      }
    });
    return () => {
      subscriber.unsubscribe(channel);
      subscriber.disconnect();
    };
  },
};

/**
 * Check Redis connection
 */
export async function checkConnection(): Promise<boolean> {
  try {
    await redis.ping();
    return true;
  } catch {
    return false;
  }
}

/**
 * Close Redis connection
 */
export async function closeConnection(): Promise<void> {
  await redis.quit();
}
