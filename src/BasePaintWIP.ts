import { ponder } from "ponder:registry";
import { trackBalance } from "./utils";

ponder.on("BasePaintWIP:Transfer", async ({ event, context }) => {
  // Track balance changes
  await trackBalance("0xE6249eAfdC9C8a809fE28a5213120B1860f9a75f", event, context);
});
