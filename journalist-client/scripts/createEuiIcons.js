import { typeToPathMap } from "@elastic/eui/lib/components/icon/icon_map.js";
import { writeFileSync } from "fs";
import path from "path";
import { fileURLToPath } from "url";

// Slightly modified version of: https://github.com/elastic/eui/issues/5463#issuecomment-1799538311
//
// Generates a static map of icons to work around the dynamic imports used in EUI by default
// which won't work with Vite.

const iconsMap = typeToPathMap;

function escapeName(name) {
  if (["package", "function"].includes(name)) {
    return `euiIcon${name}`;
  } else return name;
}

let code = `
/* eslint-disable import/no-duplicates */
// NOTE: This file exists because of a known issue with EUI icons dynamic import.
// https://github.com/elastic/eui/issues/5463#issuecomment-1107665339
//
// GENERATED by scripts/create-eui-icons.js
//
import { ICON_TYPES } from '@elastic/eui';
import { appendIconComponentCache } from '@elastic/eui/es/components/icon/icon';
`;

for (const [name, path] of Object.entries(iconsMap)) {
  code += `import { icon as ${escapeName(
    name,
  )} } from '@elastic/eui/es/components/icon/assets/${path}';\n`;
}

code += `
type IconComponentNameType = typeof ICON_TYPES[0];
type IconComponentCacheType = Record<IconComponentNameType, unknown>;
const cachedIcons: IconComponentCacheType = {
`;

for (const name of Object.keys(iconsMap)) {
  const escaped = escapeName(name);
  code += escaped === name ? `\n  ${name},` : `\n  ${name}: ${escaped},`;
}

code += `
};
appendIconComponentCache(cachedIcons);`;

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

writeFileSync(`${__dirname}/../src/euiIconsWorkAround.ts`, code.trim());
