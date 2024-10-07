import { ponder } from "@/generated";

ponder.on("BasePaint:setup", async ({ context }) => {
  const { Global } = context.db;

  await Global.create({
    id: 1,
    data: {
      startedAt: 0,
      epochDuration: 0,
    },
  });
});

ponder.on("BasePaint:Started", async ({ event, context }) => {
  const { Global } = context.db;
  const epochDuration = await context.client.readContract({
    abi: context.contracts.BasePaint.abi,
    address: context.contracts.BasePaint.address,
    functionName: "epochDuration",
  });

  await Global.update({
    id: 1,
    data: {
      startedAt: Number(event.block.timestamp),
      epochDuration: Number(epochDuration),
    },
  });
});

ponder.on("BasePaint:Painted", async ({ event, context }) => {
  const { Canvas, Brush, Contribution, Account, Usage, Stroke } = context.db;

  const day = Number(event.args.day);
  const pixelsContributed = Math.floor((event.args.pixels.length - 2) / 6);

  const brush = await Brush.findUnique({ id: Number(event.args.tokenId) });
  if (brush) {
    let streak = brush.streak;

    if (brush.lastUsedDay === Number(event.args.day) - 1) {
      streak += 1;
    }
    if (brush.lastUsedDay == null || brush.lastUsedDay < Number(event.args.day) - 1) {
      streak = 1;
    }

    await Brush.update({
      id: Number(event.args.tokenId),
      data: {
        lastUsedDay: Number(event.args.day),
        lastUsedTimestamp: Number(event.block.timestamp),
        streak,
      },
    });
  }

  const canvas = await Canvas.findUnique({ id: Number(event.args.day) });
  if (!canvas) {
    const previousCanvas = await Canvas.findUnique({ id: Number(event.args.day) - 2 });
    if (previousCanvas) {
      let cursor: string | undefined;
      do {
        const contributions = await Contribution.findMany({
          where: {
            canvasId: previousCanvas.id,
          },
          after: cursor,
          limit: 1000,
        });
        for (const contribution of contributions.items) {
          const earnings =
            (BigInt(contribution.pixelsCount) * previousCanvas.totalEarned) / BigInt(previousCanvas.pixelsCount);
          await Account.update({
            id: contribution.accountId,
            data: ({ current }) => ({
              totalEarned: current.totalEarned + earnings,
            }),
          });
        }
        cursor = contributions.pageInfo.endCursor ?? undefined;
      } while (cursor);
    }
  }

  const contributionId = `${event.args.day}_${event.args.author}`;
  const contribution = await Contribution.findUnique({ id: contributionId });
  await Contribution.upsert({
    id: contributionId,
    create: {
      canvasId: day,
      accountId: event.args.author,
      pixelsCount: pixelsContributed,
    },
    update: ({ current }) => ({
      pixelsCount: current.pixelsCount + pixelsContributed,
    }),
  });

  await Canvas.upsert({
    id: day,
    create: {
      totalMints: 0,
      totalBurns: 0,
      totalEarned: 0n,
      pixelsCount: pixelsContributed,
      totalArtists: 1,
    },
    update: ({ current }) => ({
      pixelsCount: current.pixelsCount + pixelsContributed,
      totalArtists: contribution === null ? current.totalArtists + 1 : current.totalArtists,
    }),
  });

  const usageId = `${event.args.day}_${event.args.tokenId}`;
  await Usage.upsert({
    id: usageId,
    create: {
      canvasId: day,
      brushId: Number(event.args.tokenId),
      pixelsCount: pixelsContributed,
    },
    update: ({ current }) => ({
      pixelsCount: current.pixelsCount + pixelsContributed,
    }),
  });

  await Account.update({
    id: event.args.author,
    data: ({ current }) => {
      let streak = current.streak;

      if (current.lastPaintedDay === day - 1) {
        streak += 1;
      }

      if (current.lastPaintedDay == null || current.lastPaintedDay < day - 1) {
        streak = 1;
      }

      return {
        streak,
        longestStreak: Math.max(current.longestStreak, streak),
        lastPaintedDay: day,
        totalPixels: current.totalPixels + pixelsContributed,
      };
    },
  });

  await Stroke.create({
    id: event.block.number * 100_000n + BigInt(event.log.logIndex),
    data: {
      canvasId: day,
      accountId: event.args.author,
      brushId: Number(event.args.tokenId),
      data: event.args.pixels,
      pixels: pixelsContributed,
      tx: event.transaction.hash,
      timestamp: Number(event.block.timestamp),
    },
  });
});

ponder.on("BasePaint:ArtistsEarned", async ({ event, context }) => {
  const { Canvas } = context.db;

  await Canvas.update({
    id: Number(event.args.day),
    data: ({ current }) => ({
      totalEarned: current.totalEarned + event.args.amount,
    }),
  });
});

ponder.on("BasePaint:ArtistWithdraw", async ({ event, context }) => {
  const { Withdrawal, Account } = context.db;

  await Account.update({
    id: event.args.author,
    data: ({ current }) => ({
      totalWithdrawn: current.totalWithdrawn + event.args.amount,
    }),
  });

  await Withdrawal.create({
    id: event.args.day + "_" + event.args.author,
    data: {
      canvasId: Number(event.args.day),
      accountId: event.args.author,
      amount: event.args.amount,
      timestamp: Number(event.block.timestamp),
    },
  });
});

ponder.on("BasePaint:TransferSingle", async ({ event, context }) => {
  const { Canvas } = context.db;

  if (BigInt(event.args.from) === 0n) {
    await Canvas.update({
      id: Number(event.args.id),
      data: ({ current }) => ({
        totalMints: current.totalMints + Number(event.args.value),
      }),
    });
  }
});

ponder.on("BasePaint:TransferBatch", async ({ event, context }) => {
  const { Canvas } = context.db;

  for (let i = 0; i < event.args.ids.length; i++) {
    if (BigInt(event.args.from) === 0n) {
      const id = event.args.ids[i];
      const value = event.args.values[i];
      await Canvas.update({
        id: Number(id),
        data: ({ current }) => ({
          totalMints: current.totalMints + Number(value),
        }),
      });
    }
  }
});

ponder.on("BasePaintMetadataRegistry:MetadataUpdated", async ({ event, context }) => {
  const { Canvas } = context.db;
  const canvas = await Canvas.findUnique({ id: Number(event.args.id) });

  if (canvas) {
    await Canvas.update({
      id: Number(event.args.id),
      data: {
        name: event.args.name,
        palette: event.args.palette.map((color) => "#" + color.toString(16).padStart(6, "0").toUpperCase()).join(","),
        size: Number(event.args.size),
        proposer: event.args.proposer,
      },
    });
  }
});
