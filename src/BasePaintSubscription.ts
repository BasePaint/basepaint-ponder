import { ponder } from "ponder:registry";
import { Global } from "ponder:schema";
import { trackBalance } from "./utils";
import { isZeroAddress } from "./address";

ponder.on("BasePaintSubscription:TransferSingle", async ({ event, context }) => {
  // Track balance changes
  await trackBalance("0x75CF063a65d361527180805b244bC51c1deAb075", event, context);

  let delta = 0;

  if (isZeroAddress(event.args.from)) {
    delta = Number(event.args.value);
  } else if (isZeroAddress(event.args.to)) {
    delta = -Number(event.args.value);
  }

  if (delta === 0) {
    return;
  }

  const global = await context.db.find(Global, { id: 1 });
  if (global) {
    await context.db.update(Global, { id: 1 }).set({
      totalSubscriptions: (global.totalSubscriptions ?? 0) + delta,
    });
  }
});

ponder.on("BasePaintSubscription:TransferBatch", async ({ event, context }) => {
  // Track balance changes
  await trackBalance("0x75CF063a65d361527180805b244bC51c1deAb075", event, context);

  let delta = 0;

  if (isZeroAddress(event.args.from)) {
    delta = Number(event.args.values.reduce((a, b) => a + Number(b), 0));
  } else if (isZeroAddress(event.args.to)) {
    delta = -Number(event.args.values.reduce((a, b) => a + Number(b), 0));
  }

  if (delta === 0) {
    return;
  }

  const global = await context.db.find(Global, { id: 1 });
  if (global) {
    await context.db.update(Global, { id: 1 }).set({
      totalSubscriptions: (global.totalSubscriptions ?? 0) + delta,
    });
  }
});
