/**
 * The five architecture diagrams, and the prose that travels with them.
 *
 * This is the single source of truth for which diagrams exist. Three things are
 * checked against it by `__tests__/diagrams.test.ts`: the `.mmd` sources, the
 * committed SVG pairs under `rendered/`, and `rendered/manifest.json`. Adding a
 * diagram means adding all three plus an entry here — and the test says so.
 *
 * See `scripts/render-diagrams.mjs` for why the SVGs are pre-rendered and
 * committed rather than produced during `astro build`.
 */

export const DIAGRAM_NAMES = [
  'ecosystem-topology',
  'folder-contract',
  'compose-layering',
  'ai-loop',
  'recipe-install-flow',
] as const;

export type DiagramName = (typeof DIAGRAM_NAMES)[number];

export interface DiagramMeta {
  /** Short heading, used for the `<figure>` accessible name. */
  title: string;
  /** One-sentence caption rendered under the diagram. */
  caption: string;
}

export const DIAGRAMS: Record<DiagramName, DiagramMeta> = {
  'ecosystem-topology': {
    title: 'Ecosystem topology',
    caption:
      'One router per workstation. Every project attaches to the same devmesh-traefik network, Traefik reads Host() rules off container labels, and only the services you started are up.',
  },
  'folder-contract': {
    title: 'The folder contract',
    caption:
      'Every mx project has the same shape, so every project answers to the same verbs. mx owns the contract; you own apps/{service} and docker/system.',
  },
  'compose-layering': {
    title: 'Compose layering',
    caption:
      'A baseline compose file describes the service everywhere; a dev override adds the source mount, the HMR port and a relaxed healthcheck. make dev merges the two, make up uses the baseline alone.',
  },
  'ai-loop': {
    title: 'The AI loop',
    caption:
      'Agents read the corpus through the MCP rag_context tool. The research pipeline writes to it — but only through a pull request a human merges, so the corpus grows on purpose.',
  },
  'recipe-install-flow': {
    title: 'Recipe install flow',
    caption:
      'mx add reads a recipe manifest, expands its placeholders, scaffolds the framework app, lands the compose, dockerfile and env files, and writes the router labels. Then make dev.',
  },
};

/** Narrowing helper so callers (and the test) can validate a name at runtime. */
export function isDiagramName(value: string): value is DiagramName {
  return (DIAGRAM_NAMES as readonly string[]).includes(value);
}
