import fs from "node:fs";
import path from "node:path";

const assetsDir = path.resolve(process.cwd(), "dist/assets");

if (!fs.existsSync(assetsDir)) {
  console.error(`Bundle assets directory not found: ${assetsDir}`);
  console.error("Run `pnpm build` before running the bundle budget check.");
  process.exit(1);
}

const files = fs.readdirSync(assetsDir).map((name) => {
  const filePath = path.join(assetsDir, name);
  const stats = fs.statSync(filePath);
  return { name, bytes: stats.size };
});

const jsFiles = files.filter((file) => file.name.endsWith(".js"));
const cssFiles = files.filter((file) => file.name.endsWith(".css"));

function toBytes(kb) {
  return kb * 1024;
}

function fmt(bytes) {
  return `${(bytes / 1024).toFixed(2)} KB`;
}

function largestMatch(entries, pattern) {
  const matches = entries.filter((entry) => pattern.test(entry.name));
  if (matches.length === 0) return null;
  return matches.sort((a, b) => b.bytes - a.bytes)[0];
}

const budgets = [
  {
    label: "Largest entry JS chunk",
    file: largestMatch(jsFiles, /^index-.*\.js$/),
    max: toBytes(350),
    required: true,
  },
  {
    label: "Largest entry CSS chunk",
    file: largestMatch(cssFiles, /^index-.*\.css$/),
    max: toBytes(60),
    required: true,
  },
  {
    label: "Markdown vendor chunk",
    file: largestMatch(jsFiles, /^vendor-markdown-.*\.js$/),
    max: toBytes(500),
    required: false,
  },
  {
    label: "Charts vendor chunk",
    file: largestMatch(jsFiles, /^vendor-charts-.*\.js$/),
    max: toBytes(400),
    required: false,
  },
  {
    label: "Largest JS chunk overall",
    file: jsFiles.sort((a, b) => b.bytes - a.bytes)[0] ?? null,
    max: toBytes(500),
    required: true,
  },
];

const failures = [];

console.log("Bundle budget report:");
for (const budget of budgets) {
  if (!budget.file) {
    if (budget.required) {
      failures.push(`${budget.label}: required chunk not found`);
      console.log(`- ${budget.label}: missing (required)`);
    } else {
      console.log(`- ${budget.label}: n/a`);
    }
    continue;
  }

  const status = budget.file.bytes <= budget.max ? "PASS" : "FAIL";
  console.log(
    `- ${budget.label}: ${status} (${budget.file.name} = ${fmt(
      budget.file.bytes
    )}, budget ${fmt(budget.max)})`
  );

  if (status === "FAIL") {
    failures.push(
      `${budget.label} exceeded budget: ${budget.file.name} is ${fmt(
        budget.file.bytes
      )}, limit ${fmt(budget.max)}`
    );
  }
}

if (failures.length > 0) {
  console.error("\nBundle budget check failed:");
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log("\nBundle budget check passed.");
