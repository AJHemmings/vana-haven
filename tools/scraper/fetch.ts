import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { API_BASE, BATCH_SIZE, CACHE_DIR, REQUEST_DELAY_MS, USER_AGENT } from "./config.ts";

export type FetchJson = (url: string) => Promise<unknown>;
export type SleepFn = (ms: number) => Promise<void>;
export type ThrottleState = { hasMadeRequest: boolean };

export type ScraperDeps = {
  fetchJson?: FetchJson;
  sleepFn?: SleepFn;
  cacheDir?: string;
  force?: boolean;
  throttleState?: ThrottleState;
};

const defaultFetchJson: FetchJson = async (url) => {
  const res = await fetch(url, { headers: { "User-Agent": USER_AGENT } });
  if (!res.ok) {
    throw new Error(`BG-Wiki API request failed: ${res.status} ${res.statusText} (${url})`);
  }
  return res.json();
};

const defaultSleep: SleepFn = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

// MediaWiki returns HTTP 200 with an { error: { code, info } } envelope for many
// failure types (bad title, rate limiting, maintenance mode) instead of a normal
// { query: ... } body. Applied to whatever fetchJson returns — real or test fake —
// so a bad title never gets silently misread as "zero results".
function assertNoMediaWikiError(data: unknown, url: string): void {
  if (data && typeof data === "object" && "error" in data) {
    const err = (data as { error: { code?: string; info?: string } }).error;
    throw new Error(`BG-Wiki API returned an error: ${err.code ?? "unknown"} — ${err.info ?? "no details"} (${url})`);
  }
}

function cachePathFor(cacheDir: string, key: string): string {
  const safe = Buffer.from(key).toString("hex");
  return join(cacheDir, `${safe}.json`);
}

function readCache<T>(cacheDir: string, key: string): T | null {
  const file = cachePathFor(cacheDir, key);
  if (!existsSync(file)) return null;
  try {
    const parsed = JSON.parse(readFileSync(file, "utf8"));
    if (!parsed || typeof parsed !== "object" || !("data" in parsed)) {
      console.warn(`Cache file is missing the expected "data" key, treating as a cache miss: ${file}`);
      return null;
    }
    return (parsed as { data: T }).data;
  } catch (err) {
    console.warn(`Cache file is corrupted and could not be parsed, treating as a cache miss: ${file}`, err);
    return null;
  }
}

function writeCache<T>(cacheDir: string, key: string, data: T): void {
  mkdirSync(cacheDir, { recursive: true });
  writeFileSync(cachePathFor(cacheDir, key), JSON.stringify({ fetchedAt: new Date().toISOString(), data }, null, 2));
}

function chunk<T>(items: T[], size: number): T[][] {
  const chunks: T[][] = [];
  for (let i = 0; i < items.length; i += size) {
    chunks.push(items.slice(i, i + size));
  }
  return chunks;
}

async function throttle(state: ThrottleState, sleepFn: SleepFn): Promise<void> {
  if (state.hasMadeRequest) {
    await sleepFn(REQUEST_DELAY_MS);
  }
  state.hasMadeRequest = true;
}

export async function fetchCategoryMembers(category: string, deps: ScraperDeps = {}): Promise<string[]> {
  const {
    fetchJson = defaultFetchJson,
    sleepFn = defaultSleep,
    cacheDir = CACHE_DIR,
    force = false,
    throttleState = { hasMadeRequest: false },
  } = deps;
  const cacheKey = `categorymembers:${category}`;

  if (!force) {
    const cached = readCache<string[]>(cacheDir, cacheKey);
    if (cached !== null) return cached;
  }

  await throttle(throttleState, sleepFn);

  const url = `${API_BASE}?action=query&list=categorymembers&cmtitle=${encodeURIComponent(
    "Category:" + category
  )}&cmlimit=500&format=json`;
  const data = await fetchJson(url);
  assertNoMediaWikiError(data, url);
  const typed = data as { query?: { categorymembers?: { title: string }[] } };
  const titles = (typed.query?.categorymembers ?? []).map((m) => m.title);

  writeCache(cacheDir, cacheKey, titles);
  return titles;
}

export async function fetchWikitextBatch(titles: string[], deps: ScraperDeps = {}): Promise<Map<string, string>> {
  const {
    fetchJson = defaultFetchJson,
    sleepFn = defaultSleep,
    cacheDir = CACHE_DIR,
    force = false,
    throttleState = { hasMadeRequest: false },
  } = deps;

  const result = new Map<string, string>();
  const uncached: string[] = [];

  for (const title of titles) {
    if (!force) {
      const cached = readCache<string>(cacheDir, `wikitext:${title}`);
      if (cached !== null) {
        result.set(title, cached);
        continue;
      }
    }
    uncached.push(title);
  }

  for (const batch of chunk(uncached, BATCH_SIZE)) {
    await throttle(throttleState, sleepFn);

    const url = `${API_BASE}?action=query&prop=revisions&rvprop=content&rvslots=main&titles=${encodeURIComponent(
      batch.join("|")
    )}&format=json&formatversion=2`;
    const data = await fetchJson(url);
    assertNoMediaWikiError(data, url);
    const typed = data as {
      query?: { pages?: { title: string; revisions?: { slots?: { main?: { content?: string } } }[] }[] };
    };

    for (const page of typed.query?.pages ?? []) {
      const content = page.revisions?.[0]?.slots?.main?.content ?? "";
      result.set(page.title, content);
      writeCache(cacheDir, `wikitext:${page.title}`, content);
    }
  }

  return result;
}
