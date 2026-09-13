import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fetchCategoryMembers, fetchWikitextBatch } from "./fetch.ts";

function withTempCacheDir(fn: (cacheDir: string) => Promise<void> | void) {
  const cacheDir = mkdtempSync(join(tmpdir(), "vh-scraper-test-"));
  return Promise.resolve(fn(cacheDir)).finally(() => rmSync(cacheDir, { recursive: true, force: true }));
}

test("fetchCategoryMembers calls fetchJson and caches the result on first call", () =>
  withTempCacheDir(async (cacheDir) => {
    let callCount = 0;
    const fetchJson = async () => {
      callCount++;
      return { query: { categorymembers: [{ title: "Page A" }, { title: "Page B" }] } };
    };

    const titles = await fetchCategoryMembers("Some Category", { fetchJson, cacheDir, sleepFn: async () => {} });

    assert.deepEqual(titles, ["Page A", "Page B"]);
    assert.equal(callCount, 1);
  }));

test("fetchCategoryMembers returns cached titles without calling fetchJson on a repeat call", () =>
  withTempCacheDir(async (cacheDir) => {
    let callCount = 0;
    const fetchJson = async () => {
      callCount++;
      return { query: { categorymembers: [{ title: "Page A" }] } };
    };
    const deps = { fetchJson, cacheDir, sleepFn: async () => {} };

    await fetchCategoryMembers("Some Category", deps);
    const titles = await fetchCategoryMembers("Some Category", deps);

    assert.deepEqual(titles, ["Page A"]);
    assert.equal(callCount, 1);
  }));

test("fetchCategoryMembers bypasses the cache when force is true", () =>
  withTempCacheDir(async (cacheDir) => {
    let callCount = 0;
    const fetchJson = async () => {
      callCount++;
      return { query: { categorymembers: [{ title: "Page A" }] } };
    };
    const deps = { fetchJson, cacheDir, sleepFn: async () => {} };

    await fetchCategoryMembers("Some Category", deps);
    await fetchCategoryMembers("Some Category", { ...deps, force: true });

    assert.equal(callCount, 2);
  }));

test("fetchWikitextBatch returns cached content for already-cached titles without calling fetchJson", () =>
  withTempCacheDir(async (cacheDir) => {
    let callCount = 0;
    const fetchJson = async () => {
      callCount++;
      return { query: { pages: [{ title: "Page A", revisions: [{ slots: { main: { content: "wikitext A" } } }] }] } };
    };
    const deps = { fetchJson, cacheDir, sleepFn: async () => {} };

    await fetchWikitextBatch(["Page A"], deps);
    const result = await fetchWikitextBatch(["Page A"], deps);

    assert.equal(result.get("Page A"), "wikitext A");
    assert.equal(callCount, 1);
  }));

test("fetchWikitextBatch only fetches uncached titles, merging with cached ones", () =>
  withTempCacheDir(async (cacheDir) => {
    const requestedTitleSets: string[][] = [];
    const fetchJson = async (url: string) => {
      const titlesParam = new URL(url).searchParams.get("titles")!;
      const titles = titlesParam.split("|");
      requestedTitleSets.push(titles);
      return {
        query: {
          pages: titles.map((title) => ({ title, revisions: [{ slots: { main: { content: `wikitext for ${title}` } } }] })),
        },
      };
    };
    const deps = { fetchJson, cacheDir, sleepFn: async () => {} };

    await fetchWikitextBatch(["Page A"], deps);
    const result = await fetchWikitextBatch(["Page A", "Page B"], deps);

    assert.equal(result.get("Page A"), "wikitext for Page A");
    assert.equal(result.get("Page B"), "wikitext for Page B");
    assert.equal(requestedTitleSets.length, 2);
    assert.deepEqual(requestedTitleSets[1], ["Page B"]);
  }));

test("fetchWikitextBatch splits more than 50 titles into multiple requests of at most 50", () =>
  withTempCacheDir(async (cacheDir) => {
    const requestSizes: number[] = [];
    const fetchJson = async (url: string) => {
      const titles = new URL(url).searchParams.get("titles")!.split("|");
      requestSizes.push(titles.length);
      return {
        query: {
          pages: titles.map((title) => ({ title, revisions: [{ slots: { main: { content: "x" } } }] })),
        },
      };
    };
    const deps = { fetchJson, cacheDir, sleepFn: async () => {} };

    const titles = Array.from({ length: 60 }, (_, i) => `Page ${i}`);
    const result = await fetchWikitextBatch(titles, deps);

    assert.deepEqual(requestSizes, [50, 10]);
    assert.equal(result.size, 60);
  }));

test("does not sleep before the first real network request across a shared throttle state", () =>
  withTempCacheDir(async (cacheDir) => {
    const sleepCalls: number[] = [];
    const throttleState = { hasMadeRequest: false };
    const fetchJson = async () => ({ query: { categorymembers: [{ title: "Page A" }] } });

    await fetchCategoryMembers("Some Category", {
      fetchJson,
      cacheDir,
      throttleState,
      sleepFn: async (ms: number) => {
        sleepCalls.push(ms);
      },
    });

    assert.deepEqual(sleepCalls, []);
    assert.equal(throttleState.hasMadeRequest, true);
  }));

test("sleeps between two real network requests sharing the same throttle state", () =>
  withTempCacheDir(async (cacheDir) => {
    const sleepCalls: number[] = [];
    const throttleState = { hasMadeRequest: false };
    const fetchJson = async () => ({ query: { categorymembers: [{ title: "Page A" }] } });
    const deps = {
      fetchJson,
      cacheDir,
      throttleState,
      sleepFn: async (ms: number) => {
        sleepCalls.push(ms);
      },
    };

    await fetchCategoryMembers("Category One", deps);
    await fetchCategoryMembers("Category Two", deps);

    assert.equal(sleepCalls.length, 1);
  }));

test("a title containing an apostrophe round-trips through the cache correctly", () =>
  withTempCacheDir(async (cacheDir) => {
    let callCount = 0;
    const title = "Duelist's Attire Set";
    const fetchJson = async () => {
      callCount++;
      return {
        query: {
          pages: [{ title, revisions: [{ slots: { main: { content: "relic wikitext" } } }] }],
        },
      };
    };
    const deps = { fetchJson, cacheDir, sleepFn: async () => {} };

    const first = await fetchWikitextBatch([title], deps);
    const second = await fetchWikitextBatch([title], deps);

    assert.equal(first.get(title), "relic wikitext");
    assert.equal(second.get(title), "relic wikitext");
    assert.equal(callCount, 1);
  }));

test("fetchCategoryMembers surfaces a MediaWiki API error instead of silently returning an empty list", () =>
  withTempCacheDir(async (cacheDir) => {
    const fetchJson = async () => ({ error: { code: "invalidcategory", info: "The category name you entered was invalid." } });
    await assert.rejects(
      () => fetchCategoryMembers("Some Category", { fetchJson, cacheDir, sleepFn: async () => {} }),
      /invalidcategory/
    );
  }));
