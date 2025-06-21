async function healthcheck() {
  const port = process.env.PORT || 42069;
  const url = `http://localhost:${port}/ready`;
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
