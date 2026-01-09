import { Container, getContainer } from '@cloudflare/containers';

const HEALTH_PATHS = new Set(['/_health', '/_healthz', '/_readyz']);
const DEFAULT_INSTANCE = '{{PROJECT_NAME}}-ssr';
const CONTAINER_PORT = 8080;

export class ContainerDO extends Container {
  defaultPort = CONTAINER_PORT;
  sleepAfter = '10m';
  envVars = {
    NODE_ENV: 'production',
    HOST: '0.0.0.0',
    PORT: `${CONTAINER_PORT}`
  };
}

interface Env {
  APP_CONTAINER: DurableObjectNamespace<ContainerDO>;
  /**
   * Optional override that lets us pin requests to a specific container instance ID.
   */
  CONTAINER_INSTANCE_ID?: string;
}

const worker = {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    const instanceId =
      url.searchParams.get('instance') ?? env.CONTAINER_INSTANCE_ID ?? DEFAULT_INSTANCE;
    const container = getContainer(env.APP_CONTAINER, instanceId);

    if (HEALTH_PATHS.has(url.pathname)) {
      const state = await container.getState();
      return Response.json(
        {
          status: state.status,
          lastChange: new Date(state.lastChange).toISOString(),
          exitCode: 'exitCode' in state ? state.exitCode : undefined
        },
        { headers: { 'cache-control': 'no-store' } }
      );
    }

    if (url.pathname === '/_container/restart') {
      if (request.method !== 'POST') {
        return new Response('Method Not Allowed', { status: 405 });
      }
      await container.destroy();
      return new Response('Restart requested', { status: 202 });
    }

    if (url.pathname === '/_container/start') {
      if (request.method !== 'POST') {
        return new Response('Method Not Allowed', { status: 405 });
      }

      await container.startAndWaitForPorts({ ports: [CONTAINER_PORT] });
      const state = await container.getState();
      return Response.json({ message: 'Container started', state, instanceId });
    }

    if (url.pathname === '/_container/status') {
      const state = await container.getState();
      return Response.json({ state, instanceId });
    }

    if (url.pathname === '/_container/stop') {
      if (request.method !== 'POST') {
        return new Response('Method Not Allowed', { status: 405 });
      }
      await container.stop();
      return new Response('Stop signal sent', { status: 202 });
    }

    return container.fetch(request);
  }
};

export default worker;
