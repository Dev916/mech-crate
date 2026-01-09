import { pgTable, text, timestamp, uuid, boolean, jsonb } from 'drizzle-orm/pg-core';

/**
 * Users table
 */
export const users = pgTable('users', {
  id: uuid('id').defaultRandom().primaryKey(),
  email: text('email').notNull().unique(),
  name: text('name'),
  avatarUrl: text('avatar_url'),
  metadata: jsonb('metadata').$type<Record<string, unknown>>().default({}),
  isActive: boolean('is_active').default(true).notNull(),
  createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
  updatedAt: timestamp('updated_at', { withTimezone: true }).defaultNow().notNull(),
});

/**
 * Sessions table for authentication
 */
export const sessions = pgTable('sessions', {
  id: uuid('id').defaultRandom().primaryKey(),
  userId: uuid('user_id')
    .references(() => users.id, { onDelete: 'cascade' })
    .notNull(),
  token: text('token').notNull().unique(),
  expiresAt: timestamp('expires_at', { withTimezone: true }).notNull(),
  createdAt: timestamp('created_at', { withTimezone: true }).defaultNow().notNull(),
});

/**
 * Settings table for app configuration
 */
export const settings = pgTable('settings', {
  key: text('key').primaryKey(),
  value: jsonb('value').$type<unknown>(),
  description: text('description'),
  updatedAt: timestamp('updated_at', { withTimezone: true }).defaultNow().notNull(),
});

// Type exports
export type User = typeof users.$inferSelect;
export type NewUser = typeof users.$inferInsert;
export type Session = typeof sessions.$inferSelect;
export type NewSession = typeof sessions.$inferInsert;
export type Setting = typeof settings.$inferSelect;
