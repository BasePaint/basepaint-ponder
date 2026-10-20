import { readFile } from "node:fs/promises";

async function healthcheck() {
  const port = process.env.PORT || 42069;
  // `ponder serve` only needs to be up; /ready reflects the indexer and would pull the API out of rotation.
  const isServer = (await readFile("/proc/1/cmdline", "utf8").catch(() => "")).includes("serve");
  const url = `http://localhost:${port}/${isServer ? "health" : "ready"}`;
  console.log(`Checking ${url}`);
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Healthcheck failed: ${response.status} ${await response.text()}`);
  }

  console.log("Healthcheck passed");
}

healthcheck().catch((error) => {
  console.error(error.message);
  process.exit(1);
});
