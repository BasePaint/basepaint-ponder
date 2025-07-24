import { ponder } from "ponder:registry";
import { Global, Canvas, Brush, Contribution, Account, Usage, Stroke, Withdrawal } from "ponder:schema";
import { eq } from "ponder";
import { trackBalance } from "./utils";

ponder.on("BasePaint:setup", async ({ context }) => {
  await context.db.insert(Global).values({
    id: 1,
    startedAt: 0,
    epochDuration: 0,
    totalArtists: 0,
    totalPixels: 0,
    totalEarnings: 0n,
    totalWithdrawals: 0n,
    totalMints: 0,
    totalBurns: 0,
    totalSubscriptions: 0,
  });
});

ponder.on("BasePaint:Started", async ({ event, context }) => {
  const epochDuration = await context.client.readContract({
    abi: context.contracts.BasePaint.abi,
    address: context.contracts.BasePaint.address,
    functionName: "epochDuration",
  });

  await context.db.update(Global, { id: 1 }).set({
    startedAt: Number(event.block.timestamp),
    epochDuration: Number(epochDuration),
  });
});

ponder.on("BasePaint:Painted", async ({ event, context }) => {
  const day = Number(event.args.day);
  const pixelsContributed = Math.floor((event.args.pixels.length - 2) / 6);

  const brush = await context.db.find(Brush, { id: Number(event.args.tokenId) });
  if (brush) {
    let streak = brush.streak ?? 0;

    if (brush.lastUsedDay === Number(event.args.day) - 1) {
      streak += 1;
    }
    if (brush.lastUsedDay == null || brush.lastUsedDay < Number(event.args.day) - 1) {
      streak = 1;
    }

    await context.db.update(Brush, { id: Number(event.args.tokenId) }).set({
      lastUsedDay: Number(event.args.day),
      lastUsedTimestamp: Number(event.block.timestamp),
      streak,
    });
  }

  const canvas = await context.db.find(Canvas, { id: Number(event.args.day) });
  if (!canvas) {
    const previousCanvas = await context.db.find(Canvas, { id: Number(event.args.day) - 2 });
    if (previousCanvas) {
      const contributions = await context.db.sql
        .select()
        .from(Contribution)
        .where(eq(Contribution.canvasId, previousCanvas.id));

      for (const contribution of contributions) {
        const earnings =
          (BigInt(contribution.pixelsCount ?? 0) * (previousCanvas.totalEarned ?? 0n)) /
          BigInt(previousCanvas.pixelsCount ?? 1);
        const account = await context.db.find(Account, { id: contribution.accountId });
        if (account) {
          await context.db.update(Account, { id: contribution.accountId }).set({
            totalEarned: (account.totalEarned ?? 0n) + earnings,
          });
        }
      }
    }
  }

  const contributionId = `${event.args.day}_${event.args.author}`;
  const contribution = await context.db.find(Contribution, { id: contributionId });
  await context.db
    .insert(Contribution)
    .values({
      id: contributionId,
      canvasId: day,
      accountId: event.args.author,
      pixelsCount: pixelsContributed,
    })
    .onConflictDoUpdate((row) => ({
      pixelsCount: (row.pixelsCount ?? 0) + pixelsContributed,
    }));

  await context.db
    .insert(Canvas)
    .values({
      id: day,
      totalMints: 0,
      totalBurns: 0,
      totalEarned: 0n,
      pixelsCount: pixelsContributed,
      totalArtists: 1,
    })
    .onConflictDoUpdate((row) => ({
      pixelsCount: (row.pixelsCount ?? 0) + pixelsContributed,
      totalArtists: contribution === null ? (row.totalArtists ?? 0) + 1 : row.totalArtists,
    }));

  const usageId = `${event.args.day}_${event.args.tokenId}`;
  await context.db
    .insert(Usage)
    .values({
      id: usageId,
      canvasId: day,
      brushId: Number(event.args.tokenId),
      pixelsCount: pixelsContributed,
    })
    .onConflictDoUpdate((row) => ({
      pixelsCount: (row.pixelsCount ?? 0) + pixelsContributed,
    }));

  const account = await context.db.find(Account, { id: event.args.author });
  if (account) {
    let streak = account.streak ?? 0;

    if (account.lastPaintedDay === day - 1) {
      streak += 1;
    }

    if (account.lastPaintedDay == null || account.lastPaintedDay < day - 1) {
      streak = 1;
    }

    const canvasIncrement = contribution === null ? 1 : 0;

    await context.db.update(Account, { id: event.args.author }).set({
      streak,
      longestStreak: Math.max(account.longestStreak ?? 0, streak),
      lastPaintedDay: day,
      totalPixels: (account.totalPixels ?? 0) + pixelsContributed,
      totalDaysPainted: account.totalDaysPainted + canvasIncrement,
    });
  }

  await context.db.insert(Stroke).values({
    id: event.block.number * 100_000n + BigInt(event.log.logIndex),
    canvasId: day,
    accountId: event.args.author,
    brushId: Number(event.args.tokenId),
    data: event.args.pixels,
    pixels: pixelsContributed,
    tx: event.transaction.hash,
    timestamp: Number(event.block.timestamp),
  });

  const global = await context.db.find(Global, { id: 1 });
  if (global) {
    await context.db.update(Global, { id: 1 }).set({
      totalPixels: (global.totalPixels ?? 0) + pixelsContributed,
      totalArtists: account && (account.totalPixels ?? 0) > 0 ? global.totalArtists : (global.totalArtists ?? 0) + 1,
    });
  }
});

ponder.on("BasePaint:ArtistsEarned", async ({ event, context }) => {
  const canvas = await context.db.find(Canvas, { id: Number(event.args.day) });
  if (canvas) {
    await context.db.update(Canvas, { id: Number(event.args.day) }).set({
      totalEarned: (canvas.totalEarned ?? 0n) + event.args.amount,
    });
  }

  const global = await context.db.find(Global, { id: 1 });
  if (global) {
    await context.db.update(Global, { id: 1 }).set({
      totalEarnings: (global.totalEarnings ?? 0n) + event.args.amount,
    });
  }
});

ponder.on("BasePaint:ArtistWithdraw", async ({ event, context }) => {
  const account = await context.db.find(Account, { id: event.args.author });
  if (account) {
    await context.db.update(Account, { id: event.args.author }).set({
      totalWithdrawn: (account.totalWithdrawn ?? 0n) + event.args.amount,
    });
  }

  await context.db.insert(Withdrawal).values({
    id: event.args.day + "_" + event.args.author,
    canvasId: Number(event.args.day),
    accountId: event.args.author,
    amount: event.args.amount,
    timestamp: Number(event.block.timestamp),
  });

  const global = await context.db.find(Global, { id: 1 });
  if (global) {
    await context.db.update(Global, { id: 1 }).set({
      totalWithdrawals: (global.totalWithdrawals ?? 0n) + event.args.amount,
    });
  }
});

ponder.on("BasePaint:TransferSingle", async ({ event, context }) => {
  // Track balance changes
  await trackBalance("0xBa5e05cb26b78eDa3A2f8e3b3814726305dcAc83", event, context);

  const canvas = await context.db.find(Canvas, { id: Number(event.args.id) });
  if (BigInt(event.args.from) === 0n && canvas) {
    await context.db.update(Canvas, { id: Number(event.args.id) }).set({
      totalMints: (canvas.totalMints ?? 0) + Number(event.args.value),
    });

    const global = await context.db.find(Global, { id: 1 });
    if (global) {
      await context.db.update(Global, { id: 1 }).set({
        totalMints: (global.totalMints ?? 0) + Number(event.args.value),
      });
    }
  }
});

ponder.on("BasePaint:TransferBatch", async ({ event, context }) => {
  // Track balance changes
  await trackBalance("0xBa5e05cb26b78eDa3A2f8e3b3814726305dcAc83", event, context);

  for (let i = 0; i < event.args.ids.length; i++) {
    if (BigInt(event.args.from) === 0n) {
      const id = event.args.ids[i];
      const value = event.args.values[i];
      const canvas = await context.db.find(Canvas, { id: Number(id) });
      if (canvas) {
        await context.db.update(Canvas, { id: Number(id) }).set({
          totalMints: (canvas.totalMints ?? 0) + Number(value),
        });
      }

      const global = await context.db.find(Global, { id: 1 });
      if (global) {
        await context.db.update(Global, { id: 1 }).set({
          totalMints: (global.totalMints ?? 0) + Number(value),
        });
      }
    }
  }
});

ponder.on("BasePaintMetadataRegistry:MetadataUpdated", async ({ event, context }) => {
  const canvas = await context.db.find(Canvas, { id: Number(event.args.id) });

  if (canvas) {
    await context.db.update(Canvas, { id: Number(event.args.id) }).set({
      name: event.args.name,
      palette: event.args.palette.map((color) => "#" + color.toString(16).padStart(6, "0").toUpperCase()).join(","),
      size: Number(event.args.size),
      proposer: event.args.proposer,
    });
  }
});
