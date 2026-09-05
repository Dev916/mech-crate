/**
 * `Head.astro`'s bridge to the build's page-date map.
 *
 * Task 2 resolves every route's date from committed facts (`researched:`
 * frontmatter, else the source file's last git commit), and Task 4 wants the
 * same numbers for `datePublished`/`dateModified` and the `article:*` metas —
 * one date per page across the sitemap, the head and the structured data.
 *
 * The awkward part is *where* that map may be touched. `src/loaders/lib/dates.ts`
 * shells out to `git log`, and `make dev` runs the site in a container that
 * bind-mounts `apps/site` plus a read-only `docs/` and no `.git` at all. A
 * component that statically imports the dates module therefore cannot render
 * there. So:
 *
 *   - the import is **dynamic**, so nothing git-shaped is pulled into
 *     `Head.astro`'s module graph until the first render asks for it;
 *   - the failure is caught **only in dev**. "No repository" is a legitimate
 *     state for the dev container and an illegitimate one for a build, so under
 *     `astro build` the error is rethrown and the build stops. Swallowing it
 *     there is the one outcome nobody wants: a published page whose dates
 *     silently disappeared;
 *   - the outcome is **memoised for the process**, success or failure, so a
 *     110-page build spawns one `git log` and a git-less dev server spawns one
 *     failed one rather than 110.
 *
 * `appRoot` is passed explicitly rather than left to the dates module's own
 * default, which resolves `../../..` from `import.meta.url`. That is correct for
 * the file on disk and wrong for the Vite chunk this module is bundled into
 * during `astro build` (`dist/chunks/…` is a directory deeper), where it lands
 * on `site/apps` and dates nothing. `process.cwd()` is the Astro project root
 * for both `astro build` and `astro dev`, and a wrong answer now fails the build
 * instead of quietly degrading.
 *
 * `src/lib/__tests__/jsonld-real.test.ts` asserts the map really does resolve
 * for corpus and authored pages when git is present.
 */

/** Route pathname → ISO-8601 UTC timestamp. */
export type RouteDates = ReadonlyMap<string, string>;

let resolved: RouteDates | undefined;
let attempted = false;

/**
 * The build's route → date map, or `undefined` when a *dev* process cannot read
 * git history. Throws under `astro build`, where an unreadable map is a bug.
 */
export async function buildPageDates(): Promise<RouteDates | undefined> {
  if (attempted) return resolved;
  attempted = true;

  try {
    const { pageDateIndex } = await import('../loaders/lib/dates.ts');
    resolved = pageDateIndex({ appRoot: process.cwd() }).dates;
  } catch (error) {
    if (import.meta.env.PROD) {
      // A build that cannot date its pages must not publish them undated.
      attempted = false;
      throw error;
    }
    const detail = error instanceof Error ? error.message.split('\n')[0] : String(error);
    // Once per process, not once per page. Expected under `make dev`.
    console.warn(`[mechcrate:jsonld] page dates unavailable, omitting dateModified: ${detail}`);
    resolved = undefined;
  }

  return resolved;
}

/** Test seam: forget the memoised result so a later call re-attempts. */
export function resetPageDatesCache(): void {
  resolved = undefined;
  attempted = false;
}
