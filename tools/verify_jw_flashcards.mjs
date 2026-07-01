/**
 * Fetch Jack Westin textbook chapter content for flashcard verification.
 *
 * JW is a Nuxt SPA behind Cloudflare: direct HTTP is blocked and full render
 * is slow. Instead we drive a real browser and capture the JSON the app fetches
 * from its /api/mcat-books/ endpoint, falling back to rendered body text.
 *
 * Usage: node tools/verify_jw_flashcards.mjs <url> [search-term]
 */
import { chromium } from "@playwright/test";

const url = process.argv[2];
const search = (process.argv[3] ?? "").toLowerCase();

if (!url) {
  console.error("Usage: node tools/verify_jw_flashcards.mjs <url> [search-term]");
  process.exit(1);
}

function stripHtml(html) {
  return String(html)
    .replace(/<script[\s\S]*?<\/script>/gi, " ")
    .replace(/<style[\s\S]*?<\/style>/gi, " ")
    .replace(/<[^>]+>/g, " ")
    .replace(/&nbsp;/g, " ")
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&#39;/g, "'")
    .replace(/&quot;/g, '"')
    .replace(/\s+/g, " ")
    .trim();
}

function harvestStrings(node, acc) {
  if (node == null) return;
  if (typeof node === "string") {
    if (node.length > 40 && /[a-z]/i.test(node)) acc.push(node);
    return;
  }
  if (Array.isArray(node)) {
    for (const v of node) harvestStrings(v, acc);
    return;
  }
  if (typeof node === "object") {
    for (const v of Object.values(node)) harvestStrings(v, acc);
  }
}

const browser = await chromium.launch({ headless: true });
const context = await browser.newContext({
  userAgent:
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0 Safari/537.36",
});
const page = await context.newPage();

const apiPayloads = [];
page.on("response", async (resp) => {
  const u = resp.url();
  if (u.includes("/api/") && u.includes("mcat-books")) {
    try {
      const json = await resp.json();
      apiPayloads.push({ url: u, json });
    } catch {
      /* not json */
    }
  }
});

try {
  await page.goto(url, { waitUntil: "domcontentloaded", timeout: 60000 });
} catch (e) {
  // proceed with whatever loaded
}
await page.waitForTimeout(9000);

let bodyText = "";
try {
  bodyText = await page.evaluate(() => document.body.innerText);
} catch {
  /* ignore */
}
await browser.close();

// Prefer API-delivered content (raw chapter HTML/text) when present.
const apiStrings = [];
for (const p of apiPayloads) harvestStrings(p.json, apiStrings);
const apiText = stripHtml(apiStrings.join("\n"));

const combined = (apiText + "\n" + bodyText).trim();
const lines = combined
  .split(/\n|(?<=\.)\s+(?=[A-Z])/)
  .map((l) => l.trim())
  .filter((l) => l.length > 0);

const out = {
  url,
  apiHits: apiPayloads.map((p) => p.url),
  apiCharCount: apiText.length,
  bodyCharCount: bodyText.length,
};

if (search) {
  out.matches = lines.filter((l) => l.toLowerCase().includes(search)).slice(0, 40);
}
out.sample = lines.slice(0, 120);

console.log(JSON.stringify(out, null, 2));
