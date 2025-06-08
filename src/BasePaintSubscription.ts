import { ponder } from "@/generated";
import { trackBalance } from "./utils";

ponder.on("BasePaintSubscription:TransferSingle", async ({ event, context }) => {
  await trackBalance(context.contracts.BasePaintSubscription.address, event, context);

  const { Global } = context.db;

  let delta = 0;

  if (BigInt(event.args.from) === 0n) {
    delta = Number(event.args.value);
  } else if (BigInt(event.args.to) === 0n) {
    delta = -Number(event.args.value);
  }

  if (delta === 0) {
    return;
  }

  await Global.update({
    id: 1,
    data: ({ current }) => ({
      totalSubscriptions: current.totalSubscriptions + delta,
    }),
  });
});

ponder.on("BasePaintSubscription:TransferBatch", async ({ event, context }) => {
  await trackBalance(context.contracts.BasePaintSubscription.address, event, context);

  const { Global } = context.db;

  let delta = 0;

  if (BigInt(event.args.from) === 0n) {
    delta = Number(event.args.values.reduce((a, b) => a + Number(b), 0));
  } else if (BigInt(event.args.to) === 0n) {
    delta = -Number(event.args.values.reduce((a, b) => a + Number(b), 0));
  }

  if (delta === 0) {
    return;
  }

  await Global.update({
    id: 1,
    data: ({ current }) => ({
      totalSubscriptions: current.totalSubscriptions + delta,
    }),
  });
});
