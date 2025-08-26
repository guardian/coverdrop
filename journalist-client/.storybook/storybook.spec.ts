import { test, expect } from "@playwright/test";
import { readFileSync, existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

type StoriesManifest = {
  entries: {
    [key: string]: {
      type: string;
      id: string;
      name: string;
      title: string;
      importPath: string;
      tags: string[];
    };
  };
};

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const STORIES_MANIFEST_PATH = path.join(
  __dirname,
  "../storybook-static/",
  "index.json",
);

let storiesManifest: string;

if (existsSync(STORIES_MANIFEST_PATH)) {
  storiesManifest = readFileSync(STORIES_MANIFEST_PATH, {
    encoding: "utf-8",
  });
} else {
  throw new Error("Missing stories manifest. Run 'npm run storybook:build'");
}

const stories: StoriesManifest = JSON.parse(storiesManifest);

const nonSkippedStories = Object.values(stories.entries).filter(
  (story) => !story.tags.includes("skip"),
);

for (const story of nonSkippedStories) {
  test(story.id, async ({ page }) => {
    const base = new URL("/iframe.html", "http://localhost:6006");
    base.searchParams.set("id", story.id);
    base.searchParams.set("viewMode", "story");

    const url = base.toString();

    await page.goto(url);
    await page.waitForURL(url, { waitUntil: "domcontentloaded" });

    await expect(page).toHaveScreenshot(
      [".storybook", "screenshots", story.title, `${story.id}.png`],
      {
        threshold: 0.2,
        fullPage: true,
      },
    );
  });
}
