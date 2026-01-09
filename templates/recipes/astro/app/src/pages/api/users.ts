import type { APIRoute } from 'astro';
import { db } from '@/lib/db';
import { users, type NewUser } from '@/lib/db/schema';
import { z } from 'zod';

/**
 * User creation schema
 */
const createUserSchema = z.object({
  email: z.string().email(),
  name: z.string().min(1).max(255).optional(),
});

/**
 * GET /api/users - List all users
 */
export const GET: APIRoute = async ({ url }) => {
  try {
    const page = parseInt(url.searchParams.get('page') || '1');
    const limit = Math.min(parseInt(url.searchParams.get('limit') || '20'), 100);
    const offset = (page - 1) * limit;

    const result = await db`
      SELECT id, email, name, avatar_url, is_active, created_at, updated_at
      FROM users
      WHERE is_active = true
      ORDER BY created_at DESC
      LIMIT ${limit}
      OFFSET ${offset}
    `;

    const countResult = await db`SELECT COUNT(*) as total FROM users WHERE is_active = true`;
    const total = Number(countResult[0]?.total || 0);

    return new Response(
      JSON.stringify({
        success: true,
        data: {
          items: result,
          pagination: {
            total,
            page,
            limit,
            hasMore: offset + result.length < total,
          },
        },
      }),
      {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }
    );
  } catch (error) {
    console.error('[API] Error fetching users:', error);
    return new Response(
      JSON.stringify({
        success: false,
        error: {
          code: 'INTERNAL_ERROR',
          message: 'Failed to fetch users',
        },
      }),
      {
        status: 500,
        headers: { 'Content-Type': 'application/json' },
      }
    );
  }
};

/**
 * POST /api/users - Create a new user
 */
export const POST: APIRoute = async ({ request }) => {
  try {
    const body = await request.json();
    const validation = createUserSchema.safeParse(body);

    if (!validation.success) {
      return new Response(
        JSON.stringify({
          success: false,
          error: {
            code: 'VALIDATION_ERROR',
            message: 'Invalid request body',
            details: validation.error.flatten(),
          },
        }),
        {
          status: 400,
          headers: { 'Content-Type': 'application/json' },
        }
      );
    }

    const { email, name } = validation.data;

    // Check if email already exists
    const existing = await db`SELECT id FROM users WHERE email = ${email}`;
    if (existing.length > 0) {
      return new Response(
        JSON.stringify({
          success: false,
          error: {
            code: 'EMAIL_EXISTS',
            message: 'A user with this email already exists',
          },
        }),
        {
          status: 409,
          headers: { 'Content-Type': 'application/json' },
        }
      );
    }

    const result = await db`
      INSERT INTO users (email, name)
      VALUES (${email}, ${name || null})
      RETURNING id, email, name, avatar_url, is_active, created_at, updated_at
    `;

    return new Response(
      JSON.stringify({
        success: true,
        data: result[0],
      }),
      {
        status: 201,
        headers: { 'Content-Type': 'application/json' },
      }
    );
  } catch (error) {
    console.error('[API] Error creating user:', error);
    return new Response(
      JSON.stringify({
        success: false,
        error: {
          code: 'INTERNAL_ERROR',
          message: 'Failed to create user',
        },
      }),
      {
        status: 500,
        headers: { 'Content-Type': 'application/json' },
      }
    );
  }
};
