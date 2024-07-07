import { ponder } from "@/generated";
import { zeroAddress } from "viem";

ponder.on("BasePaintBrush:Transfer", async ({ event, context }) => {
  const { Brush, Account } = context.db;

  if (event.args.from === zeroAddress) {
    const strength = await context.client.readContract({
      abi: context.contracts.BasePaintBrush.abi,
      address: context.contracts.BasePaintBrush.address,
      functionName: "strengths",
      args: [BigInt(event.args.tokenId)],
    });

    await Brush.create({
      id: Number(event.args.tokenId),
      data: {
        ownerId: event.args.to,
        strength: Number(strength),
        streak: 0,
        mintedTimestamp: Number(event.block.timestamp),
      },
    });
  } else {
    await Brush.update({
      id: Number(event.args.tokenId),
      data: {
        ownerId: event.args.to,
      },
    });
  }

  await Account.upsert({
    id: event.args.to,
    create: {
      totalPixels: 0,
    },
    update: {},
  });
});

ponder.on("BasePaintBrushEvents:Deployed", async ({ event, context }) => {
  const { Brush } = context.db;
  const { BasePaintBrush } = context.contracts;

  // Reindex all brush strengths to get accurate data for brushes
  // before the BasePaintBrushEvents contract.
  let cursor: string | undefined | null;
  while (true) {
    const brushes = await Brush.findMany({ limit: 100, after: cursor ?? undefined });
    console.log("Brushes", brushes.items.length, { cursor });

    const strengths = await Promise.all(
      brushes.items.map((brush) =>
        context.client.readContract({
          abi: BasePaintBrush.abi,
          address: BasePaintBrush.address,
          functionName: "strengths",
          args: [BigInt(brush.id)],
        })
      )
    );

    for (const [index, brush] of brushes.items.entries()) {
      await Brush.update({
        id: brush.id,
        data: {
          strength: Number(strengths[index]),
        },
      });
    }

    if (!brushes.pageInfo.hasNextPage) {
      break;
    }

    cursor = brushes.pageInfo.endCursor;
  }

  // From now on, we will listen to StrengthChanged events
});

ponder.on("BasePaintBrushEvents:StrengthChanged", async ({ event, context }) => {
  const { Brush } = context.db;

  await Brush.update({
    id: Number(event.args.tokenId),
    data: {
      strength: Number(event.args.strength),
    },
  });
});
